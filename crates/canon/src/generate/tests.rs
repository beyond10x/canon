//! What the acceptance fixture does not reach: each refusal, a witness chosen among alternative
//! paths, an action blocked only by a denial, a protocol that declares no artifact, and quoting.

use std::collections::BTreeMap;

use super::*;
use crate::model::parse;

const FIXTURE: &str = "p.yaml";

fn compiled(source: &str) -> Ir {
    crate::ir::compile(&parse(source).expect("parses")).expect("compiles")
}

fn generated(source: &str) -> Vec<Scenario> {
    generate(&compiled(source), FIXTURE, source).expect("generates")
}

fn refused(source: &str) -> (&'static str, String) {
    let refusal = generate(&compiled(source), FIXTURE, source).expect_err("refused");
    (refusal.code(), refusal.to_string())
}

/// The evidence kinds of a scenario's records, in order.
fn kinds(scenario: &Scenario) -> Vec<String> {
    let parsed = conform::parse(&scenario.text).expect("a generated scenario parses");
    match &parsed.steps[0].kind {
        conform::StepKind::Evaluate { inputs, .. } => inputs
            .evidence
            .iter()
            .map(|record| record["kind"].as_str().expect("a kind").to_owned())
            .collect(),
        conform::StepKind::Compile { .. } => panic!("a witness step evaluates"),
    }
}

/// `done` holds on `k1` and `k2` together or on `k3` alone: the witness is the one record of `k3`.
const ALTERNATIVES: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {a: {}}\n\
    evidence_kinds: {k1: {}, k2: {}, k3: {}}\n\
    claims: {c: {true_when: {any: [{all: [{evidence: {kind: k1}}, {evidence: {kind: k2}}]}, \
    {evidence: {kind: k3}}]}}}\n\
    actions: {act: {may_produce: [{evidence: k1}, {evidence: k2}, {evidence: k3}]}}\n\
    outcomes: {done: {requires: {claim: c}}}\n";

#[test]
fn the_witness_is_the_least_alternative() {
    let scenarios = generated(ALTERNATIVES);
    let files: Vec<&str> = scenarios.iter().map(|s| s.file.as_str()).collect();
    assert_eq!(files, ["outcome.done.legitimate.yaml"]);
    assert_eq!(scenarios[0].id, "p.outcome.done.legitimate");
    assert_eq!(kinds(&scenarios[0]), ["k3"]);
}

#[test]
fn generation_is_deterministic() {
    assert_eq!(generated(ALTERNATIVES), generated(ALTERNATIVES));
}

#[test]
fn an_action_blocked_only_by_a_denial_is_witnessed_by_the_denial() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims: {c: {true_when: {evidence: {kind: k}}}}\n\
        actions: {act: {requires: [{capability: cap}], may_produce: [{evidence: k}]}}\n\
        outcomes: {done: {requires: {claim: c}}}\n";
    let scenarios = generated(source);
    let action = &scenarios[0];
    assert_eq!(action.file, "action.act.blocked.yaml");
    assert!(kinds(action).is_empty());
    assert!(
        action.text.contains(
            "      authority:\n        - capability: \"cap\"\n          decision: \"denied\"\n"
        ),
        "{}",
        action.text
    );
}

#[test]
fn a_protocol_without_artifacts_lists_none() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        actions: {act: {}}\n\
        outcomes: {ended: {requires: {decision: stop}}}\n";
    let scenarios = generated(source);
    assert_eq!(scenarios.len(), 1);
    assert!(scenarios[0].text.contains("        artifacts: {}\n"));
    assert!(scenarios[0].text.contains("      evidence: []\n"));
    assert!(scenarios[0].text.contains("principal: \"canon-generate\""));
}

#[test]
fn an_identifier_a_file_name_cannot_hold_is_refused_naming_it() {
    assert_eq!(
        refused(
            "format: protocol/1\n\
             protocol: {id: p, revision: 1}\n\
             outcomes: {\"a:b\": {requires: {decision: stop}}}\n"
        ),
        (
            "illegal-file-name-character",
            "outcome `a:b` holds `:`, which its scenario's file name cannot hold".to_owned()
        )
    );
    for c in ['/', '\\', ':', '*', '?', '"', '<', '>', '|'] {
        let id = format!("a{c}b");
        let key = id.replace('\\', "\\\\").replace('"', "\\\"");
        let source = format!(
            "format: protocol/1\n\
             protocol: {{id: p, revision: 1}}\n\
             outcomes: {{\"{key}\": {{requires: {{decision: stop}}}}}}\n"
        );
        assert_eq!(
            refused(&source),
            (
                "illegal-file-name-character",
                format!(
                    "outcome `{}` holds `{}`, which its scenario's file name cannot hold",
                    crate::model::one_line(&id),
                    c.escape_default()
                )
            ),
            "{c:?}"
        );
    }
    // An identifier holds no control character, so the protocol refuses one first; the file name
    // rule refuses it too.
    assert!(is_illegal_in_file_name('\u{1}') && is_illegal_in_file_name('\u{7f}'));
    assert!(!is_illegal_in_file_name('.') && !is_illegal_in_file_name('é'));
}

#[test]
fn file_names_equal_up_to_case_or_normalisation_are_refused_naming_both() {
    for (first, second) in [("A", "a"), ("\u{e9}", "e\u{301}"), ("\u{c9}", "e\u{301}")] {
        let source = format!(
            "format: protocol/1\n\
             protocol: {{id: p, revision: 1}}\n\
             outcomes: {{\"{first}\": {{requires: {{decision: x}}}}, \
             \"{second}\": {{requires: {{decision: y}}}}}}\n"
        );
        let ir = compiled(&source);
        let ids: Vec<&str> = ir.outcomes.keys().map(|id| id.as_str()).collect();
        assert_eq!(
            refused(&source),
            (
                "file-name-collision",
                format!(
                    "outcome `{}` and outcome `{}` have scenario file names that name one file \
                     where letter case or Unicode normalisation is ignored",
                    ids[0], ids[1]
                )
            ),
            "{first:?} {second:?}"
        );
    }
}

#[test]
fn a_protocol_that_yields_no_scenario_is_refused() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 3}\n\
        actions: {act: {}}\n";
    assert_eq!(
        refused(source),
        (
            "nothing-to-witness",
            "protocol `p` revision 3 declares no outcome and blocks no action in any state, so \
             there is no scenario to write"
                .to_owned()
        )
    );
}

#[test]
fn an_unreachable_outcome_is_refused() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims: {c: {true_when: {all: [{evidence: {kind: k}}, {not: {evidence: {kind: k}}}]}}}\n\
        actions: {act: {may_produce: [{evidence: k}]}}\n\
        outcomes: {never: {requires: {claim: c}}}\n";
    assert_eq!(
        refused(source),
        (
            "unreachable-outcome",
            "outcome `never` is legitimate in no state, so no scenario witnesses it".to_owned()
        )
    );
}

#[test]
fn a_fixture_path_no_scenario_can_name_is_refused() {
    let ir = compiled(ALTERNATIVES);
    for fixture in ["", "/abs/p.yaml", "../p.yaml", "C:p.yaml"] {
        let refusal = generate(&ir, fixture, ALTERNATIVES).expect_err("refused");
        assert_eq!(refusal.code(), "fixture-not-confined", "{fixture:?}");
    }
}

#[test]
fn a_state_space_above_the_bound_is_refused_as_check_refuses_it() {
    let results: Vec<String> = (0..16)
        .map(|n| format!("{{evidence: {{kind: k, result: r{n}}}}}"))
        .collect();
    let source = format!(
        "format: protocol/1\n\
         protocol: {{id: p, revision: 1}}\n\
         artifacts: {{a: {{}}}}\n\
         evidence_kinds: {{k: {{}}}}\n\
         claims: {{c: {{true_when: {{any: [{}]}}}}}}\n\
         actions: {{act: {{may_produce: [{{evidence: k}}]}}}}\n\
         outcomes: {{done: {{requires: {{claim: c}}}}}}\n",
        results.join(", ")
    );
    let ir = compiled(&source);
    let expected = check::check(&ir, None).expect_err("check refuses it");
    assert_eq!(
        refused(&source),
        ("state-space-bound", expected.to_string())
    );
}

#[test]
fn quoting_reads_back_as_the_text() {
    let texts = [
        "plain",
        "with \"quotes\" and \\ backslash",
        "tab\tnewline\ncontrol\u{1}del\u{7f}",
        "nel\u{85}ls\u{2028}ps\u{2029}bom\u{feff}nonchar\u{fffe}\u{ffff}",
        "unicode é 𝄞",
        "yes",
        "- not a list",
        "# not a comment",
    ];
    for text in texts {
        let document = format!("value: {}\n", quote(text));
        let read: BTreeMap<String, String> =
            serde_yaml_ng::from_str(&document).unwrap_or_else(|e| panic!("{document:?}: {e}"));
        assert_eq!(read["value"], text, "{document:?}");
    }
}

/// Each pair the `file-name-collision` docs name as equal folds to one file name, and the Turkish
/// pair they name as not equal does not.
#[test]
fn fold_treats_as_equal_what_the_docs_say() {
    let file = |id: &str| format!("outcome.{id}.legitimate.yaml");
    let equal = [
        ("\u{e9}", "e\u{301}"),
        ("\u{3a3}", "\u{3c3}"),
        ("\u{3a3}", "\u{3c2}"),
        ("\u{3c3}", "\u{3c2}"),
        ("\u{17f}", "s"),
        ("\u{b5}", "\u{3bc}"),
        ("\u{212a}", "k"),
        ("\u{131}", "i"),
        ("\u{df}", "ss"),
        ("\u{1e9e}", "ss"),
        ("\u{df}", "\u{1e9e}"),
        ("Go", "gO"),
    ];
    for (a, b) in equal {
        assert_eq!(fold(&file(a)), fold(&file(b)), "{a:?} {b:?}");
    }
    assert_ne!(fold(&file("\u{130}")), fold(&file("i")));
}

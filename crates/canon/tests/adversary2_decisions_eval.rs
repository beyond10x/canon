//! Adversary pass 2 for story:decision-outcomes (wave 2026-10-04-w8), against the evaluator
//! library where pass 1 did not look: the order of the refusals the unit added relative to each
//! other and to the refusals that were there; `decided_by` under every order of several
//! principals; entries with other keys, missing keys and values that are not text; and
//! `--decisions` beside `--authority`.
//!
//! The contract read is the module docs of `eval` ("Refusals", "Explicit decisions") and of
//! `eval/decisions.rs` and `eval/outcomes.rs`.

use b10x_canon::eval::{self, Refusal, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Json};

/// `done` requires the claim `c`; `inconclusive` requires the decision `stop`; the action `act`
/// requires the capability `stop`, a name a decision also has.
const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
    evidence_kinds: {k: {}}\n\
    claims: {c: {true_when: {evidence: {kind: k}}}}\n\
    actions: {act: {requires: [{capability: stop}]}}\n\
    outcomes:\n\
    \x20\x20done: {requires: {claim: c}}\n\
    \x20\x20inconclusive: {requires: {decision: stop}}\n";

fn compiled() -> ir::Ir {
    ir::compile(&model::parse(PROTOCOL).expect("protocol parses")).expect("protocol compiles")
}

fn case(extra: &str) -> model::Case {
    eval::read_case(&format!(
        "format: canon-case/1\nid: C-1\nprotocol: p\nrevision: c2\nartifacts: {{a: {{revision: r1}}}}\n{extra}"
    ))
    .expect("case reads")
}

fn run(case: &model::Case, supplied: Supplied<'_>) -> Result<model::Decision, Refusal> {
    eval::evaluate_with(&compiled(), case, &[], supplied)
}

fn with_decisions(case: &model::Case, decisions: &str) -> Result<model::Decision, Refusal> {
    run(
        case,
        Supplied {
            decisions: Some(decisions),
            ..Supplied::default()
        },
    )
}

fn entry(decision: &str, outcome: &str, principal: &str, revision: &str) -> String {
    format!(
        "{{decision: {decision}, outcome: {outcome}, principal: '{principal}', case_revision: {revision}}}"
    )
}

fn list(entries: &[String]) -> String {
    format!("[{}]", entries.join(", "))
}

/// Every pair of the refusals this unit added, and each beside the termination refusals that were
/// there, gives the first one in the documented order: the decisions are read (malformed,
/// invalid-identifier, duplicate-identifier, entry by entry) before any section runs; in the
/// `outcomes` section the termination's `undeclared-outcome` comes first, then each decision entry
/// in the order given (`undeclared-outcome` before `undeclared-decision` within one entry), and
/// `illegitimate-termination` last.
#[test]
fn the_refusals_come_in_the_documented_order() {
    let good = entry("stop", "inconclusive", "p", "c2");
    let undeclared_outcome = entry("stop", "abandoned", "p", "c2");
    let undeclared_decision = entry("stop", "done", "p", "c2");
    let invalid = entry("stop", "inconclusive", "a b", "c2");
    let cases: [(&str, String, &str, &str); 9] = [
        // A duplicate is read before the termination is checked.
        (
            "termination: nowhere\n",
            list(&[good.clone(), good.clone()]),
            "duplicate-identifier",
            "`--decisions` gives decision `stop` for outcome `inconclusive` by `p` at case revision `c2` more than once",
        ),
        // An entry is malformed before an earlier one is checked as an identifier.
        (
            "",
            format!("[{invalid}, {{decision: stop}}]"),
            "malformed-input",
            "",
        ),
        // Entry by entry: a duplicate at entry 2 comes before an invalid identifier in entry 3,
        // and an invalid identifier before an undeclared outcome in any entry.
        (
            "",
            list(&[good.clone(), good.clone(), invalid.clone()]),
            "duplicate-identifier",
            "",
        ),
        (
            "",
            list(&[undeclared_outcome.clone(), invalid.clone()]),
            "invalid-identifier",
            "",
        ),
        // The termination's undeclared outcome comes before every decision's refusal.
        (
            "termination: nowhere\n",
            list(std::slice::from_ref(&undeclared_decision)),
            "undeclared-outcome",
            "case `C-1` terminated through outcome `nowhere`, which the protocol does not declare",
        ),
        // Entries in the order given.
        (
            "",
            list(&[undeclared_decision.clone(), undeclared_outcome.clone()]),
            "undeclared-decision",
            "decision `stop` is taken for outcome `done`, which does not require it",
        ),
        (
            "",
            list(&[undeclared_outcome.clone(), undeclared_decision.clone()]),
            "undeclared-outcome",
            "decision `stop` is taken for outcome `abandoned`, which the protocol does not declare",
        ),
        // A decision's refusal comes before a blocked termination's, even one through the
        // outcome the decision names.
        (
            "termination: done\n",
            list(&[entry("go", "inconclusive", "p", "c2")]),
            "undeclared-decision",
            "decision `go` is taken for outcome `inconclusive`, which does not require it",
        ),
        // A decision at a superseded revision is not refused; the termination through the
        // outcome it would decide is.
        (
            "termination: inconclusive\n",
            list(&[entry("stop", "inconclusive", "p", "c1")]),
            "illegitimate-termination",
            "",
        ),
    ];
    for (extra, decisions, code, message) in cases {
        let refusal = with_decisions(&case(extra), &decisions)
            .expect_err(&format!("{extra:?} with {decisions}"));
        assert_eq!(
            refusal.code(),
            code,
            "{extra:?} with {decisions}: {refusal}"
        );
        if !message.is_empty() {
            assert_eq!(refusal.to_string(), message, "{extra:?} with {decisions}");
        }
    }
}

/// The supplied inputs are read in the order authority, instant, decisions: a refusal of an
/// earlier one wins over a refusal of `--decisions`, and a decisions document given as
/// `--authority` (or the reverse) is malformed, not read as the other.
#[test]
fn authority_and_decisions_are_read_in_order_and_not_as_each_other() {
    let case = case("");
    let bad_decisions = "[{decision: stop}]";
    let refusal = run(
        &case,
        Supplied {
            authority: Some("[1]"),
            at: Some("never"),
            decisions: Some(bad_decisions),
        },
    )
    .expect_err("authority first");
    assert!(
        refusal.to_string().starts_with("`--authority`"),
        "{refusal}"
    );
    let refusal = run(
        &case,
        Supplied {
            at: Some("never"),
            decisions: Some(bad_decisions),
            ..Supplied::default()
        },
    )
    .expect_err("instant second");
    assert_eq!(refusal.code(), "invalid-instant", "{refusal}");

    let decisions = list(&[entry("stop", "inconclusive", "p", "c2")]);
    let authority = "[{capability: stop, decision: granted}]";
    let refusal = run(
        &case,
        Supplied {
            authority: Some(&decisions),
            ..Supplied::default()
        },
    )
    .expect_err("decisions as authority");
    assert_eq!(refusal.code(), "malformed-input", "{refusal}");
    assert!(
        refusal.to_string().starts_with("`--authority`"),
        "{refusal}"
    );
    let refusal = with_decisions(&case, authority).expect_err("authority as decisions");
    assert_eq!(refusal.code(), "malformed-input", "{refusal}");
    assert!(
        refusal.to_string().starts_with("`--decisions`"),
        "{refusal}"
    );
}

/// A decision is not an authority grant: an explicit decision named like a capability an action
/// requires leaves the action as it is without one, and the authority grant of that capability
/// leaves the decided outcome blocked.
#[test]
fn a_decision_is_not_an_authority_grant_and_a_grant_is_not_a_decision() {
    let case = case("");
    let decisions = list(&[entry("stop", "inconclusive", "p", "c2")]);
    let none = run(&case, Supplied::default()).expect("evaluates");
    let decided = with_decisions(&case, &decisions).expect("evaluates");
    assert_eq!(
        decided.actions, none.actions,
        "a decision changed an action"
    );
    assert_eq!(
        decided.actions.as_ref().expect("actions")["act"]["status"],
        "approval-required"
    );
    let granted = run(
        &case,
        Supplied {
            authority: Some("[{capability: stop, decision: granted}]"),
            ..Supplied::default()
        },
    )
    .expect("evaluates");
    assert_eq!(
        granted.actions.as_ref().expect("actions")["act"]["status"],
        "admissible"
    );
    assert_eq!(
        granted.outcomes.as_ref().expect("outcomes")["inconclusive"],
        serde_json::json!({
            "status": "blocked",
            "reasons": [{"decision": "stop", "present": false}],
        }),
        "an authority grant decided an outcome"
    );
}

/// Every order of several principals' decisions gives byte-identical output, recording every
/// principal sorted by Unicode code point, also when principals differ only past ASCII. Changed by
/// the coordinator's decision on adversary pass 2 finding F5: `decided_by` records every principal
/// (`principals`), not the least one.
#[test]
fn decided_by_is_the_same_for_every_order_of_the_principals() {
    let principals = ["zoë", "Zoe", "zoe", "ß", "lead-investigator"];
    let mut expected = principals.to_vec();
    expected.sort_by(|a, b| a.chars().cmp(b.chars()));
    assert_eq!(expected[0], "Zoe");
    let expected = Json::Array(
        expected
            .into_iter()
            .map(|p| Json::String(p.to_owned()))
            .collect(),
    );
    let mut entries: Vec<String> = principals
        .iter()
        .map(|principal| entry("stop", "inconclusive", principal, "c2"))
        .collect();
    entries.push(entry("stop", "inconclusive", "AAA", "c1"));
    let case = case("termination: inconclusive\n");
    let first = eval::render(&with_decisions(&case, &list(&entries)).expect("evaluates"));
    let mut seen = 0;
    permutations(&mut entries, 0, &mut |order| {
        let decision = with_decisions(&case, &list(order)).expect("evaluates");
        assert_eq!(
            decision.outcomes.as_ref().expect("outcomes")["inconclusive"]["decided_by"]["principals"],
            expected,
            "{order:?}"
        );
        assert_eq!(eval::render(&decision), first, "{order:?}");
        seen += 1;
    });
    assert_eq!(seen, 720);
}

fn permutations(items: &mut Vec<String>, from: usize, visit: &mut dyn FnMut(&[String])) {
    if from == items.len() {
        visit(items);
        return;
    }
    for index in from..items.len() {
        items.swap(from, index);
        permutations(items, from + 1, visit);
        items.swap(from, index);
    }
}

/// An entry with another key, a key missing, a key given twice, or a value that is not text
/// (number, boolean, null, list, map) is `malformed-input`, as the schema (`additionalProperties:
/// false`, all four required, each an identifier string) says; a value YAML reads as text
/// (`yes`, a date) is an identifier like any other.
#[test]
fn entries_the_schema_refuses_are_malformed_and_text_is_read_as_text() {
    let case = case("");
    for decisions in [
        "[{decision: stop, outcome: inconclusive, principal: p, case_revision: c2, at: x}]",
        "[{decision: stop, outcome: inconclusive, case_revision: c2}]",
        "[{decision: stop, outcome: inconclusive, principal: p}]",
        "[{outcome: inconclusive, principal: p, case_revision: c2}]",
        "[{decision: stop, decision: stop, outcome: inconclusive, principal: p, case_revision: c2}]",
        "[{decision: stop, outcome: inconclusive, principal: true, case_revision: c2}]",
        "[{decision: stop, outcome: inconclusive, principal: 7, case_revision: c2}]",
        "[{decision: stop, outcome: inconclusive, principal: p, case_revision: 2.5}]",
        "[{decision: stop, outcome: inconclusive, principal: ~, case_revision: c2}]",
        "[{decision: [stop], outcome: inconclusive, principal: p, case_revision: c2}]",
        "[{decision: stop, outcome: {a: b}, principal: p, case_revision: c2}]",
        "[{Decision: stop, outcome: inconclusive, principal: p, case_revision: c2}]",
        "[~]",
        "{decision: stop, outcome: inconclusive, principal: p, case_revision: c2}",
    ] {
        let refusal = with_decisions(&case, decisions).expect_err(decisions);
        assert_eq!(refusal.code(), "malformed-input", "{decisions}: {refusal}");
    }
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nrevision: 2026-10-04\nartifacts: {a: {revision: r1}}\n",
    )
    .expect("a date reads as text");
    let decision = with_decisions(
        &case,
        "[{decision: stop, outcome: inconclusive, principal: yes, case_revision: 2026-10-04}]",
    )
    .expect("evaluates");
    assert_eq!(
        decision.outcomes.expect("outcomes")["inconclusive"]["decided_by"]["principals"],
        serde_json::json!(["yes"])
    );
}

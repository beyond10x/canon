//! Unit tests of story:decision-condition: an outcome's explicit decision carries an optional
//! `when`, and counts only while it is `true` (CANON-OUTCOME-003). The tests cover the source
//! reader, validation, the IR and its read-back, the depth bound, evaluation and `canon diff`;
//! the CLI test `crates/canon-cli/tests/decision_condition.rs` and the scenario
//! `conformance/scenarios/decision-condition.yaml` hold the whole acceptance.

use super::{Supplied, evaluate_with, read_case, read_evidence, read_ir, render};
use crate::diff::diff;
use crate::ir::{self, Ir};
use crate::model::{self, ClaimId, ClaimTest, OutcomeId, OutcomeRequirement, Predicate, Truth};

const HEADER: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
    evidence_kinds: {k: {}}\n\
    claims:\n  refused: {true_when: {evidence: {kind: k, result: refused}}}\n  \
    other: {true_when: {evidence: {kind: k, result: other}}}\n\
    case: {inputs: {mode: {values: [fast, slow]}}}\n";

fn protocol(outcomes: &str) -> String {
    format!("{HEADER}outcomes:\n{outcomes}")
}

const WITH_WHEN: &str =
    "  withdrawn:\n    requires:\n      decision: withdraw\n      when: {claim: refused}\n";
const WITHOUT_WHEN: &str = "  withdrawn:\n    requires:\n      decision: withdraw\n";

fn compiled(outcomes: &str) -> Ir {
    ir::compile(&model::parse(&protocol(outcomes)).expect("parses")).expect("compiles")
}

fn parse_error(outcomes: &str) -> String {
    model::parse(&protocol(outcomes))
        .expect_err("refused")
        .to_string()
}

fn problems(outcomes: &str) -> Vec<String> {
    match ir::compile(&model::parse(&protocol(outcomes)).expect("parses")) {
        Ok(_) => Vec::new(),
        Err(problems) => problems
            .iter()
            .map(|problem| format!("{}: {problem}", problem.code()))
            .collect(),
    }
}

#[test]
fn a_decision_requirement_reads_its_condition_and_a_condition_without_a_decision_is_refused() {
    let protocol = model::parse(&protocol(WITH_WHEN)).expect("parses");
    let requires = &protocol
        .outcomes
        .iter()
        .next()
        .expect("one outcome")
        .1
        .requires;
    assert_eq!(
        requires.decision().map(|name| name.as_str()),
        Some("withdraw")
    );
    assert_eq!(
        requires.when(),
        Some(&Predicate::Claim(ClaimTest {
            claim: ClaimId::new("refused"),
            is: Truth::True
        }))
    );
    let protocol = model::parse(&protocol_without()).expect("parses");
    let requires = &protocol
        .outcomes
        .iter()
        .next()
        .expect("one outcome")
        .1
        .requires;
    assert_eq!(requires.when(), None, "no condition written, none read");

    for (written, expected) in [
        (
            "  o:\n    requires:\n      claim: refused\n      when: {claim: refused}\n",
            "`when` is written only beside `decision`",
        ),
        (
            "  o:\n    requires:\n      when: {claim: refused}\n",
            "`when` is written only beside `decision`",
        ),
        (
            "  o:\n    requires:\n      decision: d\n      when: {all: [{decision: d}]}\n",
            "unknown field `decision`",
        ),
    ] {
        let error = parse_error(written);
        assert!(error.contains(expected), "{written}: {error}");
    }
}

fn protocol_without() -> String {
    protocol(WITHOUT_WHEN)
}

#[test]
fn validation_resolves_the_condition_and_refuses_evidence_in_it() {
    assert_eq!(problems(WITH_WHEN), Vec::<String>::new());
    assert_eq!(
        problems("  o:\n    requires: {decision: d, when: {claim: missing}}\n"),
        ["undeclared-claim: outcome `o` references claim `missing`, which is not declared"]
    );
    assert_eq!(
        problems(
            "  o:\n    requires:\n      decision: d\n      when:\n        all:\n          - {claim: refused}\n          - {not: {evidence: {kind: k}}}\n          - {input: mode, in: [fast]}\n"
        ),
        [
            "evidence-in-decision-condition: outcome `o` tests evidence kind `k` at `when.all[1].not.evidence`; a decision's condition tests only claim values and case inputs"
        ]
    );
    assert_eq!(
        problems("  o:\n    requires: {decision: d, when: {first: {kind: k, result: refused}}}\n"),
        [
            "evidence-in-decision-condition: outcome `o` tests evidence kind `k` at `when.first`; a decision's condition tests only claim values and case inputs"
        ]
    );
}

#[test]
fn the_ir_writes_when_only_when_it_is_present_normalized_and_reads_it_back() {
    let without =
        ir::compile(&model::parse(&protocol_without()).expect("parses")).expect("compiles");
    let without_text = without.canonical_json();
    assert!(!without_text.contains("\"when\":"), "{without_text}");
    assert!(
        without_text.contains("      \"requires\": {\n        \"decision\": \"withdraw\"\n      }"),
        "{without_text}"
    );
    assert_eq!(read_ir(&without_text), Ok(without));

    let with = compiled(
        "  withdrawn:\n    requires:\n      decision: withdraw\n      when:\n        all: [{claim: other}, {claim: refused}, {claim: other}]\n",
    );
    let text = with.canonical_json();
    // Normalized as `requires` is: the members sorted, each once.
    assert!(
        text.contains(
            "        \"decision\": \"withdraw\",\n        \"when\": {\n          \"all\": [\n"
        ),
        "{text}"
    );
    assert_eq!(text.matches("\"id\": \"other\"").count(), 1, "{text}");
    assert_eq!(read_ir(&text), Ok(with));
}

fn case(revision: &str) -> model::Case {
    read_case(&format!(
        "format: canon-case/1\nid: C\nprotocol: p\nrevision: {revision}\nartifacts: {{a: {{revision: r1}}}}\ninputs: {{mode: fast}}\n"
    ))
    .expect("case reads")
}

const DECISION: &str =
    "- decision: withdraw\n  outcome: withdrawn\n  principal: lead\n  case_revision: c1\n";

fn record(result: &str, revision: &str) -> model::EvidenceRecord {
    read_evidence(&format!(
        "format: canon-evidence/1\nid: e1\nkind: k\nresult: {result}\nsubject: a\nsubject_revision: {revision}\n"
    ))
    .expect("evidence reads")
}

/// The `withdrawn` entry of the evaluation, and the whole text.
fn evaluated(
    ir: &Ir,
    evidence: &[model::EvidenceRecord],
    decisions: Option<&str>,
) -> (serde_json::Value, String) {
    let supplied = Supplied {
        decisions,
        ..Supplied::default()
    };
    let decision =
        evaluate_with(ir, &case("c1"), evidence, supplied).expect("the evaluation is decided");
    let text = render(&decision);
    let json: serde_json::Value = serde_json::from_str(&text).expect("canonical JSON");
    (json["outcomes"]["withdrawn"].clone(), text)
}

/// The table of CANON-OUTCOME-003, row by row, with and without a decision given.
#[test]
fn the_decision_counts_only_while_the_condition_is_true() {
    let ir = compiled(WITH_WHEN);
    let blocked_by = |value: &str| {
        serde_json::json!({
            "status": "blocked",
            "reasons": [{"claim": "refused", "value": value}],
        })
    };
    // UNKNOWN, whether a decision is given or not.
    for decisions in [None, Some(DECISION)] {
        let (entry, text) = evaluated(&ir, &[], decisions);
        assert_eq!(entry, blocked_by("unknown"));
        assert!(!text.contains("pending_decisions"), "{text}");
    }
    // FALSE: the claim's record has another result.
    for decisions in [None, Some(DECISION)] {
        let (entry, text) = evaluated(&ir, &[record("other", "r1")], decisions);
        assert_eq!(entry, blocked_by("false"));
        assert!(!text.contains("pending_decisions"), "{text}");
    }
    // TRUE with no decision: blocked naming the decision, and pending.
    let (entry, text) = evaluated(&ir, &[record("refused", "r1")], None);
    assert_eq!(
        entry,
        serde_json::json!({"status": "blocked", "reasons": [{"decision": "withdraw", "present": false}]})
    );
    assert!(text.contains("\"pending_decisions\""), "{text}");
    // TRUE with the decision at the case revision: legitimate, recording who decided.
    let (entry, _) = evaluated(&ir, &[record("refused", "r1")], Some(DECISION));
    assert_eq!(
        entry,
        serde_json::json!({
            "status": "legitimate",
            "decided_by": {"decision": "withdraw", "principals": ["lead"]},
        })
    );
    // The same decisions document: ignored while `when` is not TRUE, counted once it is.
    let stale = [record("refused", "r0")];
    assert_eq!(
        evaluated(&ir, &stale, Some(DECISION)).0,
        blocked_by("unknown")
    );
}

/// Written without `when`, the requirement evaluates as it always did.
#[test]
fn a_decision_without_a_condition_is_decided_as_before() {
    let ir = compiled(WITHOUT_WHEN);
    let (entry, _) = evaluated(&ir, &[], Some(DECISION));
    assert_eq!(entry["status"], "legitimate");
    let (entry, text) = evaluated(&ir, &[], None);
    assert_eq!(
        entry,
        serde_json::json!({"status": "blocked", "reasons": [{"decision": "withdraw", "present": false}]})
    );
    assert!(text.contains("\"pending_decisions\""), "{text}");
}

/// A decision for an undeclared name is refused whatever `when` holds: `undeclared-decision`
/// depends only on the name.
#[test]
fn an_undeclared_decision_is_refused_while_the_condition_is_not_true() {
    let ir = compiled(WITH_WHEN);
    let other = "- decision: other\n  outcome: withdrawn\n  principal: lead\n  case_revision: c1\n";
    let supplied = Supplied {
        decisions: Some(other),
        ..Supplied::default()
    };
    let refused = evaluate_with(&ir, &case("c1"), &[], supplied).expect_err("refused");
    assert_eq!(refused.code(), "undeclared-decision");
}

/// Terminating through the outcome while its condition is not true is refused, a decision given
/// or not; once the condition is true and the decision applies, it is not.
#[test]
fn terminating_through_a_conditioned_outcome_is_refused_while_it_is_blocked() {
    let ir = compiled(WITH_WHEN);
    let terminated = read_case(
        "format: canon-case/1\nid: C\nprotocol: p\nrevision: c1\nartifacts: {a: {revision: r1}}\ninputs: {mode: fast}\ntermination: withdrawn\n",
    )
    .expect("case reads");
    let supplied = || Supplied {
        decisions: Some(DECISION),
        ..Supplied::default()
    };
    let refused = evaluate_with(&ir, &terminated, &[], supplied()).expect_err("blocked");
    assert_eq!(refused.code(), "illegitimate-termination");
    let refused = evaluate_with(&ir, &terminated, &[record("refused", "r0")], supplied())
        .expect_err("blocked");
    assert_eq!(refused.code(), "illegitimate-termination");
    evaluate_with(&ir, &terminated, &[record("refused", "r1")], supplied())
        .expect("legitimate once the condition holds and the decision applies");
}

/// The depth bound reaches `when`, naming the outcome and the field.
#[test]
fn a_condition_nested_beyond_the_bound_is_refused_naming_the_outcome_and_when() {
    let mut ir = compiled(WITH_WHEN);
    let mut deep = Predicate::Claim(ClaimTest {
        claim: ClaimId::new("refused"),
        is: Truth::True,
    });
    for _ in 0..super::read::MAX_IR_DEPTH {
        deep = Predicate::Not(Box::new(deep));
    }
    let OutcomeRequirement::Decision(requirement) = &mut ir
        .outcomes
        .get_mut(&OutcomeId::new("withdrawn"))
        .expect("declared")
        .requires
    else {
        panic!("a decision requirement");
    };
    requirement.when = Some(deep);
    let refused = evaluate_with(&ir, &case("c1"), &[], Supplied::default()).expect_err("too deep");
    assert_eq!(refused.code(), "predicate-too-deep");
    assert_eq!(
        refused.to_string(),
        "outcome `withdrawn` when nests predicates deeper than 4096"
    );
}

/// Adding, removing or changing `when` is `BREAKING`, under the existing rule.
#[test]
fn diff_reports_a_changed_condition_as_a_changed_requirement() {
    let without = compiled(WITHOUT_WHEN);
    let with = compiled(WITH_WHEN);
    let other = compiled(
        "  withdrawn:\n    requires:\n      decision: withdraw\n      when: {claim: other}\n",
    );
    let line = "BREAKING: outcome `withdrawn` requirement changed\n";
    for (from, to) in [(&without, &with), (&with, &without), (&with, &other)] {
        let report = diff(from, to)
            .expect("two revisions of one protocol")
            .render();
        assert_eq!(
            report,
            format!("diff: protocol `p` revision 1 to 1\n{line}")
        );
    }
    assert_eq!(
        diff(&with, &with).expect("diffs").render(),
        "diff: protocol `p` revision 1 to 1\nNO SEMANTIC CHANGE: no declaration differs\n"
    );
}

/// A claim tested in `when` is read positively. An action's precondition reads `refused` in
/// reverse; with `when` also reading it, it is read in both directions, so strengthening it is
/// `BREAKING`. Without `when`, read only in reverse, the same change is a `RELAXATION`.
#[test]
fn diff_reads_a_claim_tested_in_a_condition_positively() {
    let reader = "actions:\n  act: {precondition: {claim: refused, is: false}}\n";
    let build = |outcomes: &str| {
        ir::compile(
            &model::parse(&format!("{HEADER}{reader}outcomes:\n{outcomes}")).expect("parses"),
        )
        .expect("compiles")
    };
    let strengthened = |from: &Ir| {
        let mut to = from.clone();
        to.claims
            .get_mut(&ClaimId::new("refused"))
            .expect("declared")
            .true_when = Predicate::All(vec![
            from.claims[&ClaimId::new("refused")].true_when.clone(),
            from.claims[&ClaimId::new("other")].true_when.clone(),
        ]);
        to
    };
    let both = build(WITH_WHEN);
    assert_eq!(
        diff(&both, &strengthened(&both)).expect("diffs").render(),
        "diff: protocol `p` revision 1 to 1\n\
         BREAKING: claim `refused` predicate strengthened, read in both directions\n"
    );
    let reverse = build(WITHOUT_WHEN);
    assert_eq!(
        diff(&reverse, &strengthened(&reverse))
            .expect("diffs")
            .render(),
        "diff: protocol `p` revision 1 to 1\n\
         RELAXATION: claim `refused` predicate strengthened, read in reverse\n"
    );
}

/// `canon mutate` walks past an outcome that requires a decision, its condition included: no
/// mutant is made at it, though `drop-conjunct` and `all-to-any` would match its `all`.
#[test]
fn mutate_makes_no_mutant_at_a_conditioned_decision() {
    let source = protocol(
        "  withdrawn:\n    requires:\n      decision: withdraw\n      when: {all: [{claim: refused}, {claim: other}]}\n",
    );
    let parsed = model::parse(&source).expect("parses");
    let sites: Vec<String> = crate::mutate::mutants(&parsed)
        .into_iter()
        .map(|mutant| format!("{} {}", mutant.operator, mutant.site))
        .collect();
    assert!(
        sites.iter().all(|site| !site.contains("outcome")),
        "{sites:?}"
    );
    assert!(
        sites.iter().any(|site| site.contains("change-result")),
        "the claims' own sites are still mutated: {sites:?}"
    );
}

/// `canon floor` classifies a condition added to a floor's decision as `BREAKING`, below the
/// floor, as every other change of a requirement.
#[test]
fn a_condition_added_to_a_floors_decision_breaches_the_floor() {
    let verdict =
        crate::floor::check(&compiled(WITHOUT_WHEN), &compiled(WITH_WHEN)).expect("checks");
    assert_eq!(
        verdict.to_string(),
        "floor: breached\nBREAKING: outcome `withdrawn` requirement changed\n"
    );
}

/// `canon generate` witnesses an outcome whose decision is conditioned with the decision taken and
/// the evidence that makes the condition true.
#[test]
fn generate_witnesses_a_conditioned_decision_with_the_evidence_its_condition_needs() {
    let source = format!(
        "{}actions: {{act: {{may_produce: [{{evidence: k}}]}}}}\n",
        protocol(WITH_WHEN)
    );
    let scenarios =
        crate::generate::generate(&compiled_source(&source), "p.yaml", &source).expect("generates");
    let witness = scenarios
        .iter()
        .find(|scenario| scenario.file == "outcome.withdrawn.legitimate.yaml")
        .unwrap_or_else(|| panic!("a witness for `withdrawn`: {scenarios:?}"));
    assert!(
        witness.text.contains("decision: \"withdraw\""),
        "{}",
        witness.text
    );
    assert!(witness.text.contains("kind: \"k\""), "{}", witness.text);
}

fn compiled_source(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// `canon check` names an outcome whose decision's condition reads evidence no action may
/// produce, as it names an outcome whose predicate does (E3 of the wave 9 review).
#[test]
fn check_names_a_decision_outcome_whose_condition_reads_unproduced_evidence() {
    let report = crate::check::check(&compiled(WITH_WHEN), None).expect("checks");
    let findings: Vec<String> = report.findings.iter().map(ToString::to_string).collect();
    assert!(
        findings.contains(
            &"unproduced-evidence: outcome `withdrawn` reads evidence kind `k`, which no action may produce"
                .to_owned()
        ),
        "{findings:?}"
    );
}

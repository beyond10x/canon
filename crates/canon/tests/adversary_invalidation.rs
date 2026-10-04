//! Adversary pass 1 for story:invalidation-rules (wave 2026-10-04-w11), against the evaluator's
//! per-claim invalidation stage and the `canon check` state space that models it.
//!
//! The contracts read are the module docs of `eval` ("A record excluded from a claim is excluded
//! from every claim built on it", invalidation "keeps a record from the claims an invalidation
//! rule names and every claim built on one of them"), of `check` ("[`check`] enumerates every
//! state of a protocol"; a rule dimension per upstream artifact and evidence dimension "an
//! evidence match of a claim that a rule on that artifact invalidates reads (the claims it names
//! and every claim built on them)"), the status page row ("`canon check` also checks every state
//! with an upstream artifact moved") and the story's outcome: "Evidence bound to a claim named by
//! a rule stops supporting it once the upstream artifact's revision ... differs".

use b10x_canon::check::{Finding, check};
use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model::{self, ClaimId, EvidenceExclusion, EvidenceRecord, ExclusionReason, Truth};

fn compiled(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// The case: `a` at `r1`, the upstream artifact `up` at `u1`.
fn case() -> model::Case {
    eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\n\
         artifacts: {a: {revision: r1}, up: {revision: u1}}\n",
    )
    .expect("case reads")
}

/// A record of kind `k` about `a` at its current revision, with `result` when given and the
/// upstream revisions `upstream` (YAML flow map) when given.
fn record(id: &str, kind: &str, result: Option<&str>, upstream: Option<&str>) -> EvidenceRecord {
    let result = result.map(|r| format!("result: {r}\n")).unwrap_or_default();
    let upstream = upstream
        .map(|u| format!("upstream_revisions: {u}\n"))
        .unwrap_or_default();
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\n{result}subject: a\n\
         subject_revision: r1\n{upstream}"
    ))
    .expect("evidence reads")
}

fn evaluated(ir: &ir::Ir, evidence: &[EvidenceRecord]) -> model::Decision {
    eval::evaluate(ir, &case(), evidence).expect("decides")
}

fn value(decision: &model::Decision, claim: &str) -> Truth {
    decision
        .claims
        .get(&ClaimId::new(claim))
        .expect("declared claim")
        .value
}

fn excluded(decision: &model::Decision, claim: &str) -> Vec<(String, ExclusionReason)> {
    decision
        .claims
        .get(&ClaimId::new(claim))
        .expect("declared claim")
        .excluded_evidence
        .iter()
        .map(|EvidenceExclusion { evidence, reason }| (evidence.as_str().to_owned(), *reason))
        .collect()
}

/// `named` and `other` read the same matches of kind `k`; the rule invalidates `named` alone.
/// `split` holds when `named` is `false` and `other` is `unknown`.
const SPLIT: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}, up: {}}\n\
    evidence_kinds: {k: {}}\n\
    claims:\n\
    \x20\x20named: {true_when: {evidence: {kind: k, result: pass}}}\n\
    \x20\x20other: {true_when: {evidence: {kind: k, result: pass}}}\n\
    actions:\n\
    \x20\x20observe: {may_produce: [{evidence: k}]}\n\
    outcomes:\n\
    \x20\x20split: {requires: {all: [{claim: named, is: false}, {claim: other, is: unknown}]}}\n\
    invalidation: {r: {upstream: up, invalidates: [named]}}\n";

/// The case invalidation exists for: a `pass` observed against the old dataset revision and a
/// `fail` observed against the current one, two records of one kind about one artifact. `named`
/// keeps only the current `fail` (`false`); `other` reads both (`unknown`); `split` is legitimate.
///
/// `canon check` gives every record of an evidence dimension the same moved flag, so it never
/// builds a state where one record of the dimension moved and another did not. Its six `k` states
/// give (`named`, `other`) only (U,U), (T,T), (U,T), (F,F), (U,F), (U,U): never (F,U), so it
/// reports `split` unreachable while `canon evaluate` finds it legitimate on an ordinary case.
#[test]
fn check_does_not_call_an_outcome_unreachable_that_evaluate_reaches() {
    let ir = compiled(SPLIT);
    let decision = evaluated(
        &ir,
        &[
            record("e-old", "k", Some("pass"), Some("{up: u0}")),
            record("e-new", "k", Some("fail"), Some("{up: u1}")),
        ],
    );
    assert_eq!(
        (value(&decision, "named"), value(&decision, "other")),
        (Truth::False, Truth::Unknown)
    );
    assert_eq!(
        decision.outcomes.as_ref().expect("outcomes")["split"]["status"],
        "legitimate"
    );

    let report = check(&ir, None).expect("checked");
    let unreachable: Vec<String> = report
        .findings
        .iter()
        .filter(|finding| matches!(finding, Finding::UnreachableOutcome { .. }))
        .map(ToString::to_string)
        .collect();
    assert_eq!(unreachable, Vec::<String>::new(), "{report}");
}

/// `named` is built on `helper`, which the rule does not name and which reads `k`. The rule names
/// `named`, so the moved record `e1` is listed under `named` as `invalidated`; yet `helper` keeps
/// it, and `named` is `true` only because of it. A record listed as excluded from a claim must not
/// decide that claim: without `e1` in the input at all, `named` is `unknown`.
///
/// For `revision_mismatch` and `expired` this holds by construction (they are set aside for every
/// claim). Here the decision says `named`'s support was invalidated while `named` stays `true` on
/// the very record it says was invalidated, which also contradicts the story's outcome ("stops
/// supporting it").
#[test]
fn a_record_listed_as_excluded_from_a_claim_does_not_decide_that_claim() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
         artifacts: {a: {}, up: {}}\n\
         evidence_kinds: {k: {}, m: {}}\n\
         claims:\n\
         \x20\x20helper: {true_when: {evidence: {kind: k}}}\n\
         \x20\x20named: {true_when: {all: [{claim: helper}, {evidence: {kind: m}}]}}\n\
         invalidation: {r: {upstream: up, invalidates: [named]}}\n",
    );
    let with = evaluated(&ir, &[record("e1", "k", None, Some("{up: u0}"))]);
    let without = evaluated(&ir, &[]);
    for (evidence, reason) in excluded(&with, "named") {
        assert_eq!(
            value(&with, "named"),
            value(&without, "named"),
            "`named` lists `{evidence}` as {reason:?} yet its value changes when `{evidence}` is \
             removed"
        );
    }
}

/// `built` is built on `named` (which the rule names) and has an evidence match of its own on `m`,
/// a kind `named` does not read. The evaluator keeps a moved `m` record from `built` but not from
/// `other`, so (`other` true, `built` unknown, `named` true) is a reachable combination: `m`
/// moved, `k` not. `canon check` reaches it only if it gives `m` a rule dimension, through
/// `invalidated_claims`; with flags taken from the named claims alone it reports `mixed`
/// unreachable.
#[test]
fn check_flags_a_dimension_only_a_claim_built_on_a_named_one_reads() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
         artifacts: {a: {}, up: {}}\n\
         evidence_kinds: {k: {}, m: {}}\n\
         claims:\n\
         \x20\x20named: {true_when: {evidence: {kind: k}}}\n\
         \x20\x20built: {true_when: {all: [{claim: named}, {evidence: {kind: m}}]}}\n\
         \x20\x20other: {true_when: {evidence: {kind: m}}}\n\
         actions:\n\
         \x20\x20observe: {may_produce: [{evidence: k}, {evidence: m}]}\n\
         outcomes:\n\
         \x20\x20mixed: {requires: {all: [{claim: named}, {claim: other}, \
         {claim: built, is: unknown}]}}\n\
         invalidation: {r: {upstream: up, invalidates: [named]}}\n",
    );
    let decision = evaluated(
        &ir,
        &[
            record("e-k", "k", None, Some("{up: u1}")),
            record("e-m", "m", None, Some("{up: u0}")),
        ],
    );
    assert_eq!(
        decision.outcomes.as_ref().expect("outcomes")["mixed"]["status"],
        "legitimate"
    );
    assert_eq!(
        check(&ir, None).expect("checked").to_string(),
        "checked: protocol `p` revision 1: 16 states, 0 properties, 0 findings\n"
    );
}

/// Two rules on different upstream artifacts both name `named`; a record moved on both is listed
/// once under `named` and under the claims built on it, through `not` and `is: unknown`, and not
/// under `other`, which reads the same kind.
#[test]
fn two_rules_naming_one_claim_list_the_record_once_through_every_level() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
         artifacts: {a: {}, up: {}, up2: {}}\n\
         evidence_kinds: {k: {}}\n\
         claims:\n\
         \x20\x20named: {true_when: {evidence: {kind: k}}}\n\
         \x20\x20d1: {true_when: {not: {claim: named}}}\n\
         \x20\x20d2: {true_when: {claim: d1, is: unknown}}\n\
         \x20\x20d3: {true_when: {not: {claim: d2}}}\n\
         \x20\x20other: {true_when: {evidence: {kind: k}}}\n\
         invalidation:\n\
         \x20\x20r1: {upstream: up, invalidates: [named]}\n\
         \x20\x20r2: {upstream: up2, invalidates: [named, d2]}\n",
    );
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\n\
         artifacts: {a: {revision: r1}, up: {revision: u1}, up2: {revision: v1}}\n",
    )
    .expect("case reads");
    let decision = eval::evaluate(
        &ir,
        &case,
        &[record("e1", "k", None, Some("{up: u0, up2: v0}"))],
    )
    .expect("decides");
    let once = vec![("e1".to_owned(), ExclusionReason::Invalidated)];
    for claim in ["named", "d1", "d2", "d3"] {
        assert_eq!(excluded(&decision, claim), once, "{claim}");
    }
    assert_eq!(excluded(&decision, "other"), Vec::new());
    assert_eq!(
        ["named", "d1", "d2", "d3", "other"].map(|claim| value(&decision, claim)),
        [
            Truth::Unknown,
            Truth::Unknown,
            Truth::True,
            Truth::False,
            Truth::True
        ]
    );
}

/// A record bound to another revision of its subject and moved upstream is listed once, as
/// `revision_mismatch`: binding runs first and invalidation sees only what it left.
#[test]
fn binding_wins_over_invalidation_and_lists_the_record_once() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
         artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {}}\n\
         claims: {named: {true_when: {evidence: {kind: k}}}}\n\
         invalidation: {r: {upstream: up, invalidates: [named]}}\n",
    );
    let stale = eval::read_evidence(
        "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r0\n\
         upstream_revisions: {up: u0}\n",
    )
    .expect("reads");
    let decision = evaluated(&ir, &[stale]);
    assert_eq!(
        excluded(&decision, "named"),
        vec![("e1".to_owned(), ExclusionReason::RevisionMismatch)]
    );
}

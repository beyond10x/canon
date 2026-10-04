//! Adversary pass 2 for story:subject-bound-evidence-match: the ELS `software.change/1` shape end
//! to end with the merge capability granted, reason order across mixed subject and no-subject
//! matches of one kind, and `canon-ir/1` canonical order across result and subject combinations.
//!
//! The ELS need the story names (ELS story:software-change-protocol): a passing `test_result`
//! about another artifact must not satisfy `tests.pass`, so with `--authority` granting
//! `repository.merge` the merge stays blocked and `accepted` is never legitimate, and a case that
//! terminates through `accepted` is refused as `illegitimate-termination`.

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Truth};
use serde_json::json;

/// `software.change/1` as ELS declares it, cut to what reaches `tests.pass`.
const SOFTWARE_CHANGE: &str = "format: protocol/1\nprotocol: {id: software.change, revision: 1}\n\
    artifacts: {intent: {}, plan: {}, implementation: {}, release: {}, deployment: {}}\n\
    evidence_kinds: {test_result: {}, code_review: {}, build_provenance: {}}\n\
    claims:\n  \
      tests.pass: {true_when: {evidence: {kind: test_result, result: pass, subject: implementation}}}\n  \
      implementation.verified: {true_when: {claim: tests.pass}}\n  \
      release.proven: {true_when: {all: [{claim: implementation.verified}, {evidence: {kind: build_provenance, subject: release}}]}}\n\
    obligations:\n  \
      verified: {discharged_when: {claim: implementation.verified}}\n\
    actions:\n  \
      tests.run: {}\n  \
      repository.merge: {precondition: {claim: implementation.verified}, requires: [{capability: repository.merge}]}\n  \
      release.promote: {precondition: {evidence: {kind: test_result, result: pass, subject: implementation}}, requires: [{capability: release.promote}]}\n\
    outcomes:\n  \
      accepted: {requires: {claim: implementation.verified}}\n  \
      shipped: {requires: {all: [{evidence: {kind: test_result, result: pass, subject: implementation}}, {claim: release.proven}]}}\n";

const GRANTED: &str = "- capability: repository.merge\n  decision: granted\n\
    - capability: release.promote\n  decision: granted\n";

fn compile(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("protocol parses")).expect("protocol compiles")
}

fn case(termination: Option<&str>) -> model::Case {
    let termination = termination
        .map(|outcome| format!("termination: {outcome}\n"))
        .unwrap_or_default();
    eval::read_case(&format!(
        "format: canon-case/1\nid: CHG-1842\nprotocol: software.change\n\
         artifacts:\n  intent: {{revision: i1}}\n  plan: {{revision: p1}}\n  \
         implementation: {{revision: R2}}\n  release: {{revision: v1}}\n  deployment: {{revision: d1}}\n\
         {termination}"
    ))
    .expect("case reads")
}

fn record(id: &str, kind: &str, subject: &str, revision: &str) -> model::EvidenceRecord {
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nresult: pass\n\
         subject: {subject}\nsubject_revision: {revision}\n"
    ))
    .expect("evidence reads")
}

fn decide(
    evidence: &[model::EvidenceRecord],
    termination: Option<&str>,
) -> Result<model::Decision, eval::Refusal> {
    let ir = compile(SOFTWARE_CHANGE);
    // Through the canonical text, as `canon evaluate --ir` reads it.
    let ir = eval::read_ir(&ir.canonical_json()).expect("the IR reads back");
    eval::evaluate_with(
        &ir,
        &case(termination),
        evidence,
        Supplied {
            authority: Some(GRANTED),
            ..Supplied::default()
        },
    )
}

fn value(decision: &model::Decision, claim: &str) -> Truth {
    decision
        .claims
        .iter()
        .find(|(id, _)| id.as_str() == claim)
        .map(|(_, entry)| entry.value)
        .unwrap_or_else(|| panic!("claim {claim}"))
}

/// A passing test result about the release, at the release's current revision, beside the
/// authority granting both capabilities: `tests.pass` stays UNKNOWN, the merge and the promotion
/// stay blocked (never admissible, never approval-required), `verified` stays open, and neither
/// outcome is legitimate. The control (a passing result about the implementation at R2) admits
/// both actions and legitimates `accepted`, so the case can fail.
#[test]
fn a_passing_test_result_about_the_release_admits_nothing_with_the_merge_granted() {
    let other = record("t-release", "test_result", "release", "v1");
    let decision = decide(std::slice::from_ref(&other), None).expect("decides");
    assert_eq!(value(&decision, "tests.pass"), Truth::Unknown);
    assert_eq!(value(&decision, "implementation.verified"), Truth::Unknown);
    assert_eq!(
        decision.obligations,
        Some(json!([{"id": "verified", "status": "open"}]))
    );
    assert_eq!(
        decision.actions,
        Some(json!({
            "release.promote": {"status": "blocked", "reasons": [
                {"evidence": "test_result", "present": false, "subject": "implementation"}]},
            "repository.merge": {"status": "blocked", "reasons": [
                {"claim": "implementation.verified", "value": "unknown"}]},
            "tests.run": {"status": "admissible"},
        }))
    );
    assert_eq!(
        decision.outcomes,
        Some(json!({
            "accepted": {"status": "blocked", "reasons": [
                {"claim": "implementation.verified", "value": "unknown"}]},
            "shipped": {"status": "blocked", "reasons": [
                {"claim": "release.proven", "value": "unknown"},
                {"evidence": "test_result", "present": false, "subject": "implementation"}]},
        }))
    );

    // Terminating through `accepted` on that evidence is refused.
    let refusal = decide(std::slice::from_ref(&other), Some("accepted"))
        .expect_err("accepted is not legitimate");
    assert_eq!(refusal.code(), "illegitimate-termination", "{refusal}");

    // The control: the implementation's own passing result at R2.
    let own = record("t-impl", "test_result", "implementation", "R2");
    let decision = decide(&[other, own], Some("accepted")).expect("decides");
    assert_eq!(value(&decision, "tests.pass"), Truth::True);
    let actions = decision.actions.expect("actions");
    assert_eq!(actions["repository.merge"], json!({"status": "admissible"}));
    assert_eq!(actions["release.promote"], json!({"status": "admissible"}));
    assert_eq!(
        decision.outcomes.expect("outcomes")["accepted"],
        json!({"status": "legitimate"})
    );
}

/// A passing result about the implementation at the superseded R1, beside one about the release at
/// its current revision: the R1 record is excluded (`revision_mismatch`) and listed under the
/// claims that read it, the release record is listed nowhere, and nothing is admitted.
#[test]
fn a_stale_own_result_and_a_current_foreign_one_still_admit_nothing() {
    let stale = record("t-impl-r1", "test_result", "implementation", "R1");
    let other = record("t-release", "test_result", "release", "v1");
    let decision = decide(&[stale, other], None).expect("decides");
    let listed = |claim: &str| -> Vec<(String, String)> {
        decision
            .claims
            .iter()
            .find(|(id, _)| id.as_str() == claim)
            .map(|(_, entry)| {
                entry
                    .excluded_evidence
                    .iter()
                    .map(|e| (e.evidence.as_str().to_owned(), e.reason.as_str().to_owned()))
                    .collect()
            })
            .unwrap_or_default()
    };
    let r1 = vec![("t-impl-r1".to_owned(), "revision_mismatch".to_owned())];
    assert_eq!(listed("tests.pass"), r1);
    assert_eq!(listed("implementation.verified"), r1);
    assert_eq!(listed("release.proven"), r1);
    assert_eq!(
        decision.actions.expect("actions")["repository.merge"]["status"],
        json!("blocked")
    );
    assert_eq!(
        decision.outcomes.expect("outcomes")["accepted"]["status"],
        json!("blocked")
    );
}

/// Mixed subject and no-subject matches of two kinds, in an outcome requirement and in an action
/// precondition, over one passing `k` record about `a`. Each reason is named once per kind and
/// subject, in kind order and then subject order, no subject first, and `present` counts only the
/// records the match reads: `{k}` and `{k, a}` read the record, `{k, b}` and `{l}` read nothing.
#[test]
fn reasons_of_one_kind_with_and_without_a_subject_come_in_kind_then_subject_order() {
    let requirement = "{all: [\
        {evidence: {kind: l, result: pass}}, \
        {evidence: {kind: k, result: pass, subject: b}}, \
        {not: {evidence: {kind: k, result: pass, subject: a}}}, \
        {evidence: {kind: k, result: fail}}, \
        {evidence: {kind: k, result: fail, subject: b}}]}";
    // An `any` none of whose members is `true` is decided by every member, `false` or `unknown`,
    // so all four kind and subject pairs are named.
    let every = "{any: [\
        {evidence: {kind: l, result: pass}}, \
        {evidence: {kind: k, result: pass, subject: b}}, \
        {evidence: {kind: k, result: fail, subject: a}}, \
        {evidence: {kind: k, result: fail}}, \
        {evidence: {kind: k, result: fail, subject: b}}]}";
    let ir = compile(&format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
         artifacts: {{a: {{}}, b: {{}}}}\nevidence_kinds: {{k: {{}}, l: {{}}}}\n\
         actions:\n  act: {{precondition: {requirement}}}\n  act_every: {{precondition: {every}}}\n\
         outcomes:\n  out: {{requires: {requirement}}}\n  out_every: {{requires: {every}}}\n"
    ));
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\n\
         artifacts: {a: {revision: a1}, b: {revision: b1}}\n",
    )
    .expect("case reads");
    let evidence = [eval::read_evidence(
        "format: canon-evidence/1\nid: e1\nkind: k\nresult: pass\nsubject: a\nsubject_revision: a1\n",
    )
    .expect("evidence reads")];
    let decision = eval::evaluate(&ir, &case, &evidence).expect("decides");
    let expected = json!([
        {"evidence": "k", "present": true},
        {"evidence": "k", "present": true, "subject": "a"},
        {"evidence": "k", "present": false, "subject": "b"},
        {"evidence": "l", "present": false},
    ]);
    for (section, entry) in [("outcomes", "out_every"), ("actions", "act_every")] {
        let found = match section {
            "outcomes" => &decision.outcomes,
            _ => &decision.actions,
        };
        assert_eq!(
            found.as_ref().expect(section)[entry]["reasons"],
            expected,
            "{entry}"
        );
    }
    // Only the `false` members of a `false` `all` decide it: `{k, fail}` and the `not` over
    // `{k, pass, a}` are `false`; the `unknown` members are not named. An outcome's reasons follow
    // the action rule (story:review-hardening-w7).
    let decided_by_false = json!([
        {"evidence": "k", "present": true},
        {"evidence": "k", "present": true, "subject": "a"},
    ]);
    assert_eq!(
        decision.actions.as_ref().expect("actions")["act"]["reasons"],
        decided_by_false,
        "action"
    );
    assert_eq!(
        decision.outcomes.as_ref().expect("outcomes")["out"]["reasons"],
        decided_by_false,
        "outcome"
    );
    // The rendered bytes put `subject` after `present` inside each reason.
    let rendered = eval::render(&decision);
    assert!(
        rendered.contains(
            "{\n          \"evidence\": \"k\",\n          \"present\": false,\n          \"subject\": \"b\"\n        }"
        ),
        "{rendered}"
    );
}

/// A deterministic xorshift, so the permutations below are the same on every run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = (self.next() % (i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }
}

/// Every combination of result (none, `fail`, `pass`) and subject (none, `a`, `b`) for two kinds,
/// each written once and some twice, in 200 seeded orders, inside an `any` and inside a `not`
/// over an `all`: every order compiles to the bytes of the first, the IR text reads back, the
/// members come in kind, then result, then subject order with each written once, and the decision
/// over a fixed evidence set is the same.
#[test]
fn every_order_of_result_and_subject_combinations_compiles_to_one_ir() {
    let mut members = Vec::new();
    for kind in ["l", "k"] {
        for result in [None, Some("pass"), Some("fail")] {
            for subject in [Some("b"), None, Some("a")] {
                let mut fields = vec![format!("kind: {kind}")];
                if let Some(result) = result {
                    fields.push(format!("result: {result}"));
                }
                if let Some(subject) = subject {
                    fields.push(format!("subject: {subject}"));
                }
                members.push(format!("{{evidence: {{{}}}}}", fields.join(", ")));
            }
        }
    }
    members.push(members[3].clone());
    members.push(members[10].clone());
    let source = |members: &[String]| {
        let list = members.join(", ");
        format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
             artifacts: {{a: {{}}, b: {{}}}}\nevidence_kinds: {{k: {{}}, l: {{}}}}\n\
             claims:\n  either: {{true_when: {{any: [{list}]}}}}\n  \
             neither: {{true_when: {{not: {{all: [{list}]}}}}}}\n"
        )
    };
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\n\
         artifacts: {a: {revision: a1}, b: {revision: b1}}\n",
    )
    .expect("case reads");
    let evidence = [
        eval::read_evidence(
            "format: canon-evidence/1\nid: e1\nkind: k\nresult: pass\nsubject: a\nsubject_revision: a1\n",
        )
        .expect("evidence reads"),
        eval::read_evidence(
            "format: canon-evidence/1\nid: e2\nkind: l\nresult: fail\nsubject: b\nsubject_revision: b1\n",
        )
        .expect("evidence reads"),
    ];
    let first = compile(&source(&members));
    let text = first.canonical_json();
    assert_eq!(eval::read_ir(&text), Ok(first.clone()));
    let decided = eval::render(&eval::evaluate(&first, &case, &evidence).expect("decides"));

    // The members of `either`, in the order the IR writes them, as (kind, result, subject).
    let ir::Claim { true_when, .. } = &first.claims[&model::ClaimId::new("either")];
    let model::Predicate::Any(written) = true_when else {
        panic!("either is an any: {text}");
    };
    let triples: Vec<(String, Option<String>, Option<String>)> = written
        .iter()
        .map(|member| match member {
            model::Predicate::Evidence(m) => (
                m.kind.as_str().to_owned(),
                m.result.clone(),
                m.subject.as_ref().map(|s| s.as_str().to_owned()),
            ),
            other => panic!("not an evidence match: {other:?}"),
        })
        .collect();
    let mut expected = triples.clone();
    expected.sort();
    expected.dedup();
    assert_eq!(triples, expected, "kind, result, subject order, each once");
    assert_eq!(triples.len(), 18);

    let mut rng = Rng(0x5eed_cafe_f00d_0001);
    for round in 0..200 {
        let mut shuffled = members.clone();
        rng.shuffle(&mut shuffled);
        let compiled = compile(&source(&shuffled));
        assert_eq!(compiled.canonical_json(), text, "round {round}");
        assert_eq!(
            eval::render(&eval::evaluate(&compiled, &case, &evidence).expect("decides")),
            decided,
            "round {round}"
        );
    }
}

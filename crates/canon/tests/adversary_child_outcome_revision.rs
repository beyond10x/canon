//! Adversary pass 1 on story:child-outcome-revision. Every test of the unit, CANON-COMPOSE-002 and
//! `canon check`'s states give each artifact of a case one and the same revision (or declare one
//! artifact), so nothing pinned which artifact's current revision a `revision: current` form
//! compares a record with. These cases give two artifacts two different revisions, and state the
//! IR and the diff of `revision: any` outside a claim (a precondition and an outcome requirement).

use b10x_canon::diff::diff;
use b10x_canon::eval;
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, ClaimId, EvidenceRecord, Truth};

const PROTOCOL: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {spec: {}, plan: {}}\n\
    evidence_kinds: {case_outcome: {}}\n\
    claims:\n\
    \x20\x20now: {true_when: {children: {in: [delivered], revision: current}}}\n\
    \x20\x20ever: {true_when: {children: {in: [delivered]}}}\n";

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

fn outcome(id: &str, child: &str, result: &str, subject: &str, revision: &str) -> EvidenceRecord {
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: case_outcome\nresult: {result}\n\
         subject: {subject}\nsubject_revision: {revision}\nchild_case: {child}\n"
    ))
    .expect("a record")
}

/// `spec` at `s2`, `plan` at `p1`: two artifacts, two different current revisions.
fn case() -> model::Case {
    eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\n\
         artifacts: {spec: {revision: s2}, plan: {revision: p1}}\n",
    )
    .expect("case reads")
}

/// A `revision: current` form reads a record only when its `subject_revision` is the current
/// revision of the record's own `subject`: not of another artifact, not of any artifact. The form
/// without the key reads every record. Neither lists a record in `excluded_evidence`.
#[test]
fn the_bound_form_compares_each_record_with_its_own_subject() {
    let ir = compiled(PROTOCOL);
    for (records, now, ever, why) in [
        (
            vec![outcome("o1", "C1", "delivered", "spec", "p1")],
            Truth::Unknown,
            Truth::True,
            "about spec at plan's current revision: not current for spec",
        ),
        (
            vec![outcome("o1", "C1", "delivered", "plan", "s2")],
            Truth::Unknown,
            Truth::True,
            "about plan at spec's current revision: not current for plan",
        ),
        (
            vec![
                outcome("o1", "C1", "delivered", "spec", "s2"),
                outcome("o2", "C2", "abandoned", "plan", "p1"),
            ],
            Truth::False,
            Truth::False,
            "each current for its own subject: both read",
        ),
        (
            vec![
                outcome("o1", "C1", "delivered", "spec", "s2"),
                outcome("o2", "C2", "abandoned", "plan", "s2"),
            ],
            Truth::True,
            Truth::False,
            "the plan record names spec's revision, not plan's",
        ),
        (
            vec![
                outcome("o1", "C1", "abandoned", "spec", "p1"),
                outcome("o2", "C2", "delivered", "plan", "p1"),
            ],
            Truth::True,
            Truth::False,
            "the spec record names plan's revision, not spec's",
        ),
    ] {
        let decision = eval::evaluate(&ir, &case(), &records).expect("decides");
        let claim = |id: &str| {
            decision
                .claims
                .get(&ClaimId::new(id))
                .expect("the claim")
                .clone()
        };
        assert_eq!(claim("now").value, now, "now: {why}");
        assert_eq!(claim("ever").value, ever, "ever: {why}");
        assert!(
            claim("now").excluded_evidence.is_empty(),
            "now lists no record: {why}"
        );
        assert!(
            claim("ever").excluded_evidence.is_empty(),
            "ever lists no record: {why}"
        );
    }
}

/// `revision: any` compiles to the bytes of no key in an action's precondition and an outcome's
/// requirement, as it does in a claim, and the diff takes it as no change there.
#[test]
fn revision_any_outside_a_claim_is_no_key() {
    let with = |key: &str| {
        format!(
            "{PROTOCOL}actions:\n\
             \x20\x20build: {{precondition: {{children: {{in: [delivered]{key}}}}}}}\n\
             outcomes:\n\
             \x20\x20built: {{requires: {{not: {{children: {{in: [delivered]{key}}}}}}}}}\n"
        )
    };
    let any = compiled(&with(", revision: any"));
    let none = compiled(&with(""));
    assert_eq!(any.canonical_json(), none.canonical_json());
    let later = compiled(&with(", revision: any").replacen("revision: 1", "revision: 2", 1));
    let changes: Vec<String> = diff(&none, &later)
        .expect("diffs")
        .changes
        .iter()
        .map(ToString::to_string)
        .collect();
    assert!(changes.is_empty(), "{changes:?}");
    let current = compiled(&with(", revision: current"));
    assert_ne!(current.canonical_json(), none.canonical_json());
}

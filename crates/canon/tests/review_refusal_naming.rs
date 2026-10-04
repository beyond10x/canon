//! Independent review of wave 2026-10-04-w7: every refusal names what it refuses.
//!
//! The evidence check refuses "each evidence record in the order given" (module docs of
//! `eval`), and most of its refusals name the record: `` evidence `e2` is given more than once ``,
//! `` evidence `e2` is of kind `z`, which the protocol does not declare ``, `` evidence `e2`
//! observed_at `…` is not an instant ``. Four do not: a record whose `format` is not
//! `canon-evidence/1` (`format is `x`, expected `canon-evidence/1``, the same words a case
//! snapshot's wrong format gives), and a record whose kind, subject or subject revision is not an
//! identifier. Among several records, nothing in those messages says which one was refused.

use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model;

fn ir() -> ir::Ir {
    ir::compile(
        &model::parse(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
             evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\n",
        )
        .expect("parses"),
    )
    .expect("compiles")
}

fn record(id: &str) -> String {
    format!("format: canon-evidence/1\nid: {id}\nkind: k\nsubject: a\nsubject_revision: r1\n")
}

/// The refusal of a set holding a good record `e1` and the record `e2` written as `bad`.
fn refusal(bad: String) -> eval::Refusal {
    let case = eval::read_case(
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n",
    )
    .expect("case reads");
    let evidence = [
        eval::read_evidence(&record("e1")).expect("e1 reads"),
        eval::read_evidence(&bad).expect("e2 reads as a record"),
    ];
    eval::evaluate(&ir(), &case, &evidence).expect_err("e2 is refused")
}

/// A required identifier left out of a document is refused as if it had been written as null:
/// every identifier deserializes through `present::required` (`model/ids.rs`), and serde reads a
/// missing field as `none`, so the message is `an explicit null is not allowed here; leave the
/// key out to take its default` — which tells the author to do what they did, and names no field.
/// The same text for every left-out identifier of the case snapshot, an evidence record and an
/// authority entry.
#[test]
fn a_left_out_required_identifier_is_refused_naming_the_field_not_as_an_explicit_null() {
    let mut wrong = Vec::new();
    let full_record = record("e1");
    let full_case = "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n";
    let mut refusals: Vec<(String, eval::Refusal)> = Vec::new();
    for field in ["id", "kind", "subject", "subject_revision"] {
        let text: String = full_record
            .lines()
            .filter(|line| !line.starts_with(&format!("{field}:")))
            .map(|line| format!("{line}\n"))
            .collect();
        refusals.push((
            format!("evidence without `{field}`"),
            eval::read_evidence(&text).expect_err("refused"),
        ));
    }
    for field in ["id", "protocol"] {
        let text: String = full_case
            .lines()
            .filter(|line| !line.starts_with(&format!("{field}:")))
            .map(|line| format!("{line}\n"))
            .collect();
        refusals.push((
            format!("case without `{field}`"),
            eval::read_case(&text).expect_err("refused"),
        ));
    }
    for (what, refusal) in refusals {
        let message = refusal.to_string();
        let field = what.split('`').nth(1).expect("field");
        if message.contains("explicit null") || !message.contains(field) {
            wrong.push(format!("{what}: error[{}]: {message}", refusal.code()));
        }
    }
    assert!(
        wrong.is_empty(),
        "a left-out field refused as an explicit null, or without naming the field:\n{}",
        wrong.join("\n")
    );
}

/// The `canon-authority/1` reader, new in wave 2026-10-04-w7, inherits the same message: an entry
/// that leaves out `capability` is refused as an explicit null, naming no field and no entry.
#[test]
fn an_authority_entry_without_a_capability_is_refused_naming_what_is_missing() {
    let case = eval::read_case(
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n",
    )
    .expect("case reads");
    let supplied = eval::Supplied {
        authority: Some("- {capability: x, decision: granted}\n- {decision: granted}\n"),
        ..eval::Supplied::default()
    };
    let refusal = eval::evaluate_with(&ir(), &case, &[], supplied).expect_err("refused");
    let message = refusal.to_string();
    assert_eq!(refusal.code(), "malformed-input", "{message}");
    assert!(
        !message.contains("explicit null") && message.contains("capability"),
        "an authority entry without `capability` is refused as: {message}"
    );
}

#[test]
fn each_evidence_refusal_names_the_record_it_refuses() {
    let e2 = record("e2");
    let mut unnamed = Vec::new();
    for (what, bad) in [
        ("format", e2.replace("canon-evidence/1", "canon-evidence/2")),
        ("kind", e2.replace("kind: k", "kind: 'k k'")),
        ("subject", e2.replace("subject: a", "subject: 'a a'")),
        (
            "subject revision",
            e2.replace("subject_revision: r1", "subject_revision: 'r 1'"),
        ),
        // Controls: these already name the record.
        ("duplicate id", record("e1")),
        ("undeclared kind", e2.replace("kind: k", "kind: z")),
    ] {
        let refused = refusal(bad);
        let names = if what == "duplicate id" {
            "`e1`"
        } else {
            "`e2`"
        };
        if !refused.to_string().contains(names) {
            unnamed.push(format!("{what}: error[{}]: {refused}", refused.code()));
        }
    }
    assert!(
        unnamed.is_empty(),
        "refusals of one record among several that do not name it:\n{}",
        unnamed.join("\n")
    );
}

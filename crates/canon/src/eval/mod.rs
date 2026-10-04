//! Deterministic evaluation of a case under a compiled protocol (design § 8, § 13).
//!
//! [`evaluate`] is a pure function of a compiled protocol ([`Ir`]), a `canon-case/1` case snapshot
//! and a set of `canon-evidence/1` records. It reads nothing else: no clock, network, filesystem,
//! credentials or implicit `latest`. Its result is a `canon-decision/1` [`Decision`] giving every
//! declared claim the value `true`, `false` or `unknown`, or a [`Refusal`] when the inputs do not
//! fit the protocol. [`render`] is the decision's one serialization: canonical JSON, the
//! serialization `canon-ir/1` uses.
//!
//! # Three-valued claims
//!
//! A claim's value is its `true_when` predicate's value over the evidence set:
//!
//! - An evidence match without a result is `true` when a record of the kind exists and `unknown`
//!   when none does.
//! - An evidence match with a result is `unknown` when no record of the kind exists, `false` when
//!   records of the kind exist and none has the result, `true` when every record of the kind has
//!   that result, and `unknown` when records of the kind disagree (some with the result, some
//!   without): conflicting evidence neither establishes the predicate nor by itself contradicts it.
//! - `all` is `false` when a member is `false`, otherwise `true` when every member is `true` (an
//!   empty `all` is `true`), otherwise `unknown`.
//! - `any` is `true` when a member is `true`, otherwise `false` when every member is `false` (an
//!   empty `any` is `false`), otherwise `unknown`.
//! - `not` swaps `true` and `false` and keeps `unknown`.
//! - A claim reference is three-valued: `{claim: c}` and `{claim: c, is: true}` are `c`'s own value,
//!   and `{claim: c, is: false}` is `not` of it, so a claim built on `c` is `unknown` exactly when
//!   the evidence leaves `c` `unknown` and is never `false` for want of evidence.
//!   `{claim: c, is: unknown}` asks whether `c` is undecided, and is itself decided: `true` when
//!   `c` is `unknown`, `false` otherwise.
//!
//! `unknown` and `false` are never collapsed, and only `true` satisfies a positive requirement.
//!
//! # Refusals
//!
//! Inputs are checked before any claim is evaluated, in this order, and the first problem found is
//! the refusal: the case snapshot's format, its identifiers (case id, protocol id, then each
//! artifact id and revision in the order written), its protocol against the compiled protocol's,
//! each artifact it lists (declared by the protocol, listed once), each artifact the protocol
//! declares (listed by the case); then each evidence record in the order given: its format, its
//! identifiers (id, kind, subject, subject revision), its id (used once in the set) and its kind
//! (declared by the protocol). A record's subject and subject revision are checked as identifiers
//! only; they do not affect evaluation.
//!
//! Then claims are evaluated. An IR whose claims test each other in a cycle — which `canon
//! compile` never produces, but a caller can build — is refused as `claim-cycle`, naming the claims
//! from the first one reached again: `claims test each other in a cycle: c -> d -> c`.
//!
//! # Depth
//!
//! [`read_ir`] refuses an IR whose arrays and objects nest deeper than [`MAX_IR_DEPTH`]
//! (4096) before parsing it. Reading and claim evaluation recurse along the nesting of the
//! predicates, so they run on a thread of their own with a stack sized for that bound
//! (`DEEP_STACK`): an IR at the bound reads and evaluates from any caller's thread.

mod claims;
mod json;
mod read;

use std::collections::BTreeSet;
use std::fmt;

use crate::ir::Ir;
use crate::model::{
    CASE_FORMAT, Case, ClaimDecision, DECISION_FORMAT, Decision, Declarations, EVIDENCE_FORMAT,
    EvidenceRecord, is_identifier, one_line,
};

pub use read::{
    MAX_IR_DEPTH, case_from_value, evidence_from_value, read_case, read_evidence, read_ir,
};

/// Why an evaluation was refused: a stable machine-readable code and a one-line message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    code: &'static str,
    message: String,
}

impl Refusal {
    fn new(code: &'static str, message: String) -> Self {
        Self { code, message }
    }

    /// A stable machine-readable code for the kind of refusal; a conformance scenario names it.
    pub fn code(&self) -> &'static str {
        self.code
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Refusal {}

/// Evaluates every claim of `ir` for `case` from `evidence`. Pure and deterministic: the same
/// inputs give the same decision, and the order of `evidence` does not matter.
pub fn evaluate(ir: &Ir, case: &Case, evidence: &[EvidenceRecord]) -> Result<Decision, Refusal> {
    check_case(ir, case)?;
    check_evidence(ir, evidence)?;
    let values = on_deep_stack(|| claims::values(ir, evidence))?;
    Ok(Decision {
        format: DECISION_FORMAT.to_owned(),
        case: case.id.clone(),
        protocol: ir.protocol.id.clone(),
        protocol_revision: ir.protocol.revision,
        claims: Declarations::new(
            values
                .into_iter()
                .map(|(id, value)| (id, ClaimDecision { value }))
                .collect(),
        ),
    })
}

/// The decision's canonical JSON, ending with a newline: object keys in code-point order, two-space
/// indentation, the string escaping `canon-ir/1` uses.
pub fn render(decision: &Decision) -> String {
    use json::Value;
    let claims = decision
        .claims
        .iter()
        .map(|(id, entry)| {
            (
                id.as_str().to_owned(),
                Value::object([("value", Value::string(entry.value.to_string()))]),
            )
        })
        .collect();
    Value::object([
        ("format", Value::string(decision.format.as_str())),
        ("case", Value::string(decision.case.as_str())),
        ("protocol", Value::string(decision.protocol.as_str())),
        (
            "protocol_revision",
            Value::Integer(decision.protocol_revision),
        ),
        ("claims", Value::Object(claims)),
    ])
    .render()
}

/// The stack the recursive steps of [`read_ir`] and [`evaluate`] run on. Their depth follows the
/// nesting of the protocol's predicates, which [`MAX_IR_DEPTH`] bounds; a debug build needs
/// up to about 2 KiB per level, so the bound needs about 8 MiB, more than a spawned thread's
/// default 2 MiB. 64 MiB leaves room; it is reserved, not touched, until used.
const DEEP_STACK: usize = 64 << 20;

/// Runs `work` on a thread with [`DEEP_STACK`], so how deep a protocol's predicates may nest does
/// not depend on the caller's stack. The thread reads nothing and the result is the same as on the
/// caller's thread. A panic in `work` is resumed on the caller's thread.
fn on_deep_stack<T: Send>(work: impl FnOnce() -> T + Send) -> T {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("canon-eval".to_owned())
            .stack_size(DEEP_STACK)
            .spawn_scoped(scope, work)
            .expect("the operating system starts a thread for evaluation");
        match worker.join() {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}

fn unsupported_format(found: &str, expected: &str) -> Refusal {
    Refusal::new(
        "unsupported-format",
        format!("format is `{}`, expected `{expected}`", one_line(found)),
    )
}

fn identifier(what: &str, id: &str) -> Result<(), Refusal> {
    if is_identifier(id) {
        return Ok(());
    }
    Err(Refusal::new(
        "invalid-identifier",
        format!(
            "{what} `{}` is empty or contains whitespace or a control character",
            one_line(id)
        ),
    ))
}

fn check_case(ir: &Ir, case: &Case) -> Result<(), Refusal> {
    if case.format != CASE_FORMAT {
        return Err(unsupported_format(&case.format, CASE_FORMAT));
    }
    identifier("case identifier", case.id.as_str())?;
    identifier("case protocol identifier", case.protocol.as_str())?;
    for (artifact, entry) in case.artifacts.iter() {
        identifier("case artifact identifier", artifact.as_str())?;
        identifier("artifact revision", entry.revision.as_str())?;
    }
    if case.protocol != ir.protocol.id {
        return Err(Refusal::new(
            "protocol-mismatch",
            format!(
                "case `{}` is governed by protocol `{}`, not by `{}`",
                one_line(case.id.as_str()),
                one_line(case.protocol.as_str()),
                one_line(ir.protocol.id.as_str())
            ),
        ));
    }
    let mut seen = BTreeSet::new();
    for artifact in case.artifacts.ids() {
        if !ir.artifacts.contains_key(artifact) {
            return Err(Refusal::new(
                "undeclared-artifact",
                format!(
                    "case `{}` lists artifact `{}`, which the protocol does not declare",
                    one_line(case.id.as_str()),
                    one_line(artifact.as_str())
                ),
            ));
        }
        if !seen.insert(artifact) {
            return Err(Refusal::new(
                "duplicate-identifier",
                format!(
                    "case `{}` lists artifact `{}` more than once",
                    one_line(case.id.as_str()),
                    one_line(artifact.as_str())
                ),
            ));
        }
    }
    if let Some(missing) = ir.artifacts.keys().find(|id| !seen.contains(id)) {
        return Err(Refusal::new(
            "missing-artifact",
            format!(
                "case `{}` does not give the current revision of artifact `{}`",
                one_line(case.id.as_str()),
                one_line(missing.as_str())
            ),
        ));
    }
    Ok(())
}

fn check_evidence(ir: &Ir, evidence: &[EvidenceRecord]) -> Result<(), Refusal> {
    let mut seen = BTreeSet::new();
    for record in evidence {
        if record.format != EVIDENCE_FORMAT {
            return Err(unsupported_format(&record.format, EVIDENCE_FORMAT));
        }
        identifier("evidence identifier", record.id.as_str())?;
        identifier("evidence kind identifier", record.kind.as_str())?;
        identifier("evidence subject identifier", record.subject.as_str())?;
        identifier(
            "evidence subject revision",
            record.subject_revision.as_str(),
        )?;
        if !seen.insert(&record.id) {
            return Err(Refusal::new(
                "duplicate-identifier",
                format!(
                    "evidence `{}` is given more than once",
                    one_line(record.id.as_str())
                ),
            ));
        }
        if !ir.evidence_kinds.contains_key(&record.kind) {
            return Err(Refusal::new(
                "undeclared-evidence-kind",
                format!(
                    "evidence `{}` is of kind `{}`, which the protocol does not declare",
                    one_line(record.id.as_str()),
                    one_line(record.kind.as_str())
                ),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Truth;

    const PROTOCOL: &str = "format: protocol/1\n\
        protocol: {id: p, revision: 3}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims:\n\
          \x20\x20base: {true_when: {evidence: {kind: k, result: pass}}}\n\
          \x20\x20is_true: {true_when: {claim: base}}\n\
          \x20\x20is_false: {true_when: {claim: base, is: false}}\n\
          \x20\x20is_unknown: {true_when: {claim: base, is: unknown}}\n\
          \x20\x20negated: {true_when: {not: {claim: base, is: unknown}}}\n";

    const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}, b: {revision: r1}}\n";

    fn ir() -> Ir {
        crate::ir::compile(&crate::model::parse(PROTOCOL).expect("parses")).expect("compiles")
    }

    fn record(id: &str, result: &str) -> String {
        format!(
            "format: canon-evidence/1\nid: {id}\nkind: k\nresult: {result}\nsubject: a\nsubject_revision: r1\n"
        )
    }

    fn decide(case: &str, evidence: &[String]) -> Result<Decision, Refusal> {
        let case = read_case(case)?;
        let evidence = evidence
            .iter()
            .map(|text| read_evidence(text))
            .collect::<Result<Vec<_>, _>>()?;
        evaluate(&ir(), &case, &evidence)
    }

    fn values(evidence: &[String]) -> Vec<(String, Truth)> {
        decide(CASE, evidence)
            .expect("decides")
            .claims
            .iter()
            .map(|(id, entry)| (id.as_str().to_owned(), entry.value))
            .collect()
    }

    #[test]
    fn a_claim_reference_is_three_valued_and_only_is_unknown_is_decided() {
        use Truth::{False as F, True as T, Unknown as U};
        let names = ["base", "is_false", "is_true", "is_unknown", "negated"];
        for (evidence, expected) in [
            (vec![], [U, U, U, T, F]),
            (vec![record("e1", "pass")], [T, F, T, F, T]),
            (vec![record("e1", "fail")], [F, T, F, F, T]),
            (
                vec![record("e1", "pass"), record("e2", "fail")],
                [U, U, U, T, F],
            ),
        ] {
            let found = values(&evidence);
            let expected: Vec<(String, Truth)> = names
                .iter()
                .map(|name| name.to_string())
                .zip(expected)
                .collect();
            assert_eq!(found, expected, "{evidence:?}");
        }
    }

    #[test]
    fn the_order_of_the_evidence_set_does_not_change_the_decision() {
        let forward = [record("e1", "pass"), record("e2", "fail")];
        let backward = [record("e2", "fail"), record("e1", "pass")];
        let render_of = |evidence: &[String]| render(&decide(CASE, evidence).expect("decides"));
        assert_eq!(render_of(&forward), render_of(&backward));
    }

    #[test]
    fn the_decision_renders_every_declared_claim_in_identifier_order() {
        let decision = decide(CASE, &[record("e1", "pass")]).expect("decides");
        assert_eq!(
            render(&decision),
            "{\n  \"case\": \"C-1\",\n  \"claims\": {\n    \"base\": {\n      \"value\": \"true\"\n    },\n    \"is_false\": {\n      \"value\": \"false\"\n    },\n    \"is_true\": {\n      \"value\": \"true\"\n    },\n    \"is_unknown\": {\n      \"value\": \"false\"\n    },\n    \"negated\": {\n      \"value\": \"true\"\n    }\n  },\n  \"format\": \"canon-decision/1\",\n  \"protocol\": \"p\",\n  \"protocol_revision\": 3\n}\n"
        );
    }

    #[test]
    fn inputs_that_do_not_fit_the_protocol_are_refused_with_a_code() {
        let pass = record("e1", "pass");
        for (case, evidence, code, message) in [
            (
                CASE.replace("canon-case/1", "canon-case/2"),
                vec![],
                "unsupported-format",
                "format is `canon-case/2`, expected `canon-case/1`",
            ),
            (
                CASE.replace("id: C-1", "id: 'C 1'"),
                vec![],
                "invalid-identifier",
                "case identifier `C 1` is empty or contains whitespace or a control character",
            ),
            (
                CASE.replace("b: {revision: r1}", "b: {revision: ''}"),
                vec![],
                "invalid-identifier",
                "artifact revision `` is empty or contains whitespace or a control character",
            ),
            (
                CASE.replace("protocol: p", "protocol: q"),
                vec![],
                "protocol-mismatch",
                "case `C-1` is governed by protocol `q`, not by `p`",
            ),
            (
                CASE.replace("b: {revision: r1}", "c: {revision: r1}"),
                vec![],
                "undeclared-artifact",
                "case `C-1` lists artifact `c`, which the protocol does not declare",
            ),
            (
                CASE.replace("b: {revision: r1}", "a: {revision: r2}"),
                vec![],
                "malformed-input",
                "case is not a canon-case/1 document: artifacts: duplicate entry with key \"a\"",
            ),
            (
                CASE.replace(", b: {revision: r1}", ""),
                vec![],
                "missing-artifact",
                "case `C-1` does not give the current revision of artifact `b`",
            ),
            (
                CASE.to_owned(),
                vec![pass.replace("canon-evidence/1", "evidence/1")],
                "unsupported-format",
                "format is `evidence/1`, expected `canon-evidence/1`",
            ),
            (
                CASE.to_owned(),
                vec![pass.replace("subject_revision: r1", "subject_revision: 'r 1'")],
                "invalid-identifier",
                "evidence subject revision `r 1` is empty",
            ),
            (
                CASE.to_owned(),
                vec![pass.clone(), pass.clone()],
                "duplicate-identifier",
                "evidence `e1` is given more than once",
            ),
            (
                CASE.to_owned(),
                vec![pass.replace("kind: k", "kind: z")],
                "undeclared-evidence-kind",
                "evidence `e1` is of kind `z`, which the protocol does not declare",
            ),
        ] {
            let refusal = decide(&case, &evidence).expect_err(code);
            assert_eq!(refusal.code(), code, "{refusal}");
            assert!(refusal.to_string().starts_with(message), "{refusal}");
        }
    }

    /// `evaluate` takes any IR a caller builds; claims that test each other in a cycle are
    /// refused, naming the cycle, instead of recursing until the stack overflows.
    #[test]
    fn an_ir_whose_claims_test_each_other_in_a_cycle_is_refused() {
        use crate::model::{ClaimId, ClaimTest, Predicate};
        let case = read_case(CASE).expect("case");
        // Each edit replaces a claim's predicate with a test of another claim.
        let refused = |edits: &[(&str, &str)]| {
            let mut cyclic = ir();
            for (claim, tested) in edits {
                cyclic
                    .claims
                    .get_mut(&ClaimId::new(*claim))
                    .expect("declared")
                    .true_when = Predicate::Claim(ClaimTest {
                    claim: ClaimId::new(*tested),
                    is: Truth::True,
                });
            }
            let refusal = evaluate(&cyclic, &case, &[]).expect_err("a cycle is refused");
            assert_eq!(refusal.code(), "claim-cycle", "{refusal}");
            refusal.to_string()
        };
        assert_eq!(
            refused(&[("base", "base")]),
            "claims test each other in a cycle: base -> base"
        );
        // `is_true` already tests `base`.
        assert_eq!(
            refused(&[("base", "is_true")]),
            "claims test each other in a cycle: base -> is_true -> base"
        );
        assert_eq!(
            refused(&[("base", "is_true"), ("is_true", "is_false")]),
            "claims test each other in a cycle: base -> is_true -> is_false -> base"
        );
        // A cycle that does not pass through the first claim evaluated is found as well.
        assert_eq!(
            refused(&[("negated", "is_unknown"), ("is_unknown", "negated")]),
            "claims test each other in a cycle: is_unknown -> negated -> is_unknown"
        );
    }

    /// YAML refuses a repeated key before evaluation; a caller that builds the case itself can
    /// still repeat an artifact, and the evaluator refuses that too.
    #[test]
    fn a_case_built_with_a_repeated_artifact_is_refused() {
        let mut case = read_case(CASE).expect("case reads");
        let first = case.artifacts.iter().next().expect("an artifact");
        let repeated = (first.0.clone(), first.1.clone());
        let mut entries: Vec<_> = case
            .artifacts
            .iter()
            .map(|(id, entry)| (id.clone(), entry.clone()))
            .collect();
        entries.push(repeated);
        case.artifacts = Declarations::new(entries);
        let refusal = evaluate(&ir(), &case, &[]).expect_err("refused");
        assert_eq!(refusal.code(), "duplicate-identifier");
        assert_eq!(
            refusal.to_string(),
            "case `C-1` lists artifact `a` more than once"
        );
    }

    #[test]
    fn a_subject_outside_the_case_does_not_change_the_value() {
        let elsewhere = record("e1", "pass")
            .replace("subject: a", "subject: nowhere")
            .replace("subject_revision: r1", "subject_revision: r9");
        assert_eq!(values(&[elsewhere]), values(&[record("e1", "pass")]));
    }
}

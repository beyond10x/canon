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
//! identifiers (id, kind, subject, subject revision), its `observed_at` when given (an instant,
//! else `invalid-instant`), its id (used once in the set) and its kind
//! (declared by the protocol). A record's subject and subject revision are checked as identifiers
//! only; they do not affect evaluation.
//!
//! Then the supplied inputs are read, in this order: the authority decisions (`--authority`), the
//! evaluation instant (`--at`) and the explicit decisions. The instant is read by `freshness.rs`
//! and refused as `invalid-instant` when it is not one. The others are not read yet: each one
//! given is refused as `unsupported-input`, naming it (`` `--authority` is not supported yet ``).
//!
//! Then the exclusion stages run, in pipeline order: revision binding, freshness, invalidation.
//! Each may refuse the inputs it reads: freshness refuses a `max_age` it cannot read
//! (`invalid-max-age`); the others refuse nothing yet.
//!
//! Then claims are evaluated. An IR whose claims test each other in a cycle — which `canon
//! compile` never produces, but a caller can build — is refused as `claim-cycle`, naming the claims
//! from the first one reached again: `claims test each other in a cycle: c -> d -> c`.
//!
//! Last, the `outcomes` section may refuse the case snapshot (a termination through an outcome the
//! protocol does not declare, story:outcomes); it refuses nothing yet.
//!
//! # Depth
//!
//! [`read_ir`] refuses an IR whose arrays and objects nest deeper than [`MAX_IR_DEPTH`]
//! (4096) before parsing it. Reading and claim evaluation recurse along the nesting of the
//! predicates, so they run on a thread of their own with a stack sized for that bound
//! (`DEEP_STACK`): an IR at the bound reads and evaluates from any caller's thread.
//!
//! # The pipeline
//!
//! [`evaluate_with`] runs the same steps for every evaluation, each in the file that owns its
//! concept; this module holds only the order:
//!
//! 1. check the case snapshot (`case.rs`) and the evidence set (`evidence.rs`);
//! 2. read the supplied inputs: authority decisions (`authority.rs`), the evaluation instant
//!    (`freshness.rs`) and explicit decisions (`decisions.rs`);
//! 3. run the exclusion stages in order: revision binding (`binding.rs`), freshness
//!    (`freshness.rs`), invalidation (`invalidation.rs`). Each sees only the evidence the stages
//!    before it left, and returns the records it excludes with its reason;
//! 4. evaluate every claim over the evidence left (`claims.rs`). Each claim's `excluded_evidence`
//!    lists, in evidence-id order, every excluded record of a kind the claim reaches: a kind an
//!    evidence match in its own predicate names, or one a claim it tests reaches, through any
//!    number of claim references. A record excluded from a claim is excluded from every claim
//!    built on it;
//! 5. the `obligations`, `actions` and `outcomes` sections (`obligations.rs`, `actions.rs`,
//!    `outcomes.rs`), then the explanation (`crate::explain`). Each section evaluates its
//!    predicates with the one evaluator claims use (`claims::predicate`): discharge predicates over
//!    the claim values, action preconditions and outcome requirements over the claim values and
//!    the evidence left after step 3.
//!
//! The freshness stage and the evaluation instant are built: evidence older than its kind's
//! `max_age` at the instant is excluded as `expired`. The other stages, sections and inputs of
//! steps 2, 3 and 5 are not built yet: each stage excludes nothing, each section is absent, and a
//! supplied input is refused as `unsupported-input`, naming it. So without an instant the decision
//! is the one three-valued claim evaluation gives, byte for byte.

mod actions;
mod authority;
mod binding;
mod case;
mod claims;
mod decision;
mod decisions;
mod evidence;
mod freshness;
mod invalidation;
mod json;
mod obligations;
mod outcomes;
mod read;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::ir::Ir;
use crate::model::{
    Case, ClaimDecision, ClaimId, DECISION_FORMAT, Decision, Declarations, EvidenceExclusion,
    EvidenceId, EvidenceRecord, Predicate, is_identifier, one_line,
};

pub use case::{case_from_value, read_case};
pub use decision::{render, render_sections};
pub use evidence::{evidence_from_value, read_evidence};
pub use read::{MAX_IR_DEPTH, read_ir};

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

/// What a caller supplies besides the protocol, the case snapshot and the evidence, each as the
/// text it was given, unparsed: the file that owns it reads it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Supplied<'a> {
    /// `canon-authority/1` decisions (`--authority`), read by `authority.rs`.
    pub authority: Option<&'a str>,
    /// The evaluation instant (`--at`), read by `freshness.rs`.
    pub at: Option<&'a str>,
    /// Explicit decisions (`canon-decisions/1`), read by `decisions.rs`.
    pub decisions: Option<&'a str>,
}

/// Evaluates every claim of `ir` for `case` from `evidence`, with nothing else supplied. Pure and
/// deterministic: the same inputs give the same decision, and the order of `evidence` does not
/// matter.
pub fn evaluate(ir: &Ir, case: &Case, evidence: &[EvidenceRecord]) -> Result<Decision, Refusal> {
    evaluate_with(ir, case, evidence, Supplied::default())
}

/// Evaluates `case` under `ir` from `evidence` and the `supplied` inputs, in the order the module
/// docs give. Pure and deterministic.
pub fn evaluate_with(
    ir: &Ir,
    case: &Case,
    evidence: &[EvidenceRecord],
    supplied: Supplied<'_>,
) -> Result<Decision, Refusal> {
    case::check(ir, case)?;
    evidence::check(ir, evidence)?;
    let authority = authority::read(supplied.authority)?;
    let at = freshness::instant(supplied.at)?;
    let decisions = decisions::read(supplied.decisions)?;
    on_deep_stack(|| {
        let mut applicable: Vec<&EvidenceRecord> = evidence.iter().collect();
        let mut excluded: Vec<EvidenceExclusion> = Vec::new();
        let stage = binding::exclude(ir, case, &applicable)?;
        set_aside(&mut applicable, &mut excluded, stage);
        let stage = freshness::exclude(ir, case, &applicable, at.as_ref())?;
        set_aside(&mut applicable, &mut excluded, stage);
        let stage = invalidation::exclude(ir, case, &applicable)?;
        set_aside(&mut applicable, &mut excluded, stage);

        let applicable: Vec<EvidenceRecord> = applicable.into_iter().cloned().collect();
        let values = claims::values(ir, &applicable)?;
        let obligations = obligations::section(ir, &values);
        let actions = actions::section(ir, &values, &applicable, authority.as_ref());
        let outcomes = outcomes::section(ir, case, &values, &applicable, decisions.as_ref())?;
        let claims = values
            .iter()
            .map(|(id, value)| {
                let entry = ClaimDecision {
                    value: *value,
                    excluded_evidence: excluded_for(ir, id, evidence, &excluded),
                };
                (id.clone(), entry)
            })
            .collect();
        let mut decision = Decision {
            format: DECISION_FORMAT.to_owned(),
            case: case.id.clone(),
            protocol: ir.protocol.id.clone(),
            protocol_revision: ir.protocol.revision,
            claims: Declarations::new(claims),
            obligations,
            actions,
            outcomes,
            explanation: None,
        };
        decision.explanation = crate::explain::explain(ir, evidence, &decision);
        Ok(decision)
    })
}

/// Moves the records one exclusion stage excluded out of `applicable` and into `excluded`. A stage
/// that names a record no longer applicable excludes nothing more.
fn set_aside(
    applicable: &mut Vec<&EvidenceRecord>,
    excluded: &mut Vec<EvidenceExclusion>,
    stage: Vec<EvidenceExclusion>,
) {
    for exclusion in stage {
        if let Some(at) = applicable
            .iter()
            .position(|record| record.id == exclusion.evidence)
        {
            applicable.remove(at);
            excluded.push(exclusion);
        }
    }
}

/// The excluded records listed under `claim`: those of a kind the claim reaches, in evidence-id
/// order. A claim reaches the kinds the evidence matches of its own predicate name, and every kind
/// a claim it tests reaches, through any number of claim references; each claim is visited once,
/// so a cycle a caller builds into an IR ends.
fn excluded_for(
    ir: &Ir,
    claim: &ClaimId,
    evidence: &[EvidenceRecord],
    excluded: &[EvidenceExclusion],
) -> Vec<EvidenceExclusion> {
    if excluded.is_empty() {
        return Vec::new();
    }
    let mut kinds = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut pending = vec![claim];
    while let Some(next) = pending.pop() {
        if !visited.insert(next) {
            continue;
        }
        let Some(declared) = ir.claims.get(next) else {
            continue;
        };
        declared.true_when.visit(&mut |node| match node {
            Predicate::Evidence(matching) => {
                kinds.insert(&matching.kind);
            }
            Predicate::Claim(test) => pending.push(&test.claim),
            _ => {}
        });
    }
    let kind_of: BTreeMap<&EvidenceId, _> = evidence
        .iter()
        .map(|record| (&record.id, &record.kind))
        .collect();
    let mut listed: Vec<EvidenceExclusion> = excluded
        .iter()
        .filter(|exclusion| {
            kind_of
                .get(&exclusion.evidence)
                .is_some_and(|kind| kinds.contains(kind))
        })
        .cloned()
        .collect();
    listed.sort();
    listed
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

/// A supplied input this evaluator does not read yet, named as the caller gave it.
fn unsupported_input(input: &str) -> Refusal {
    Refusal::new(
        "unsupported-input",
        format!("`{input}` is not supported yet"),
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

    /// One record of `kind` about artifact `a` at `r1`, built directly.
    fn bare(id: &str, kind: &str) -> EvidenceRecord {
        EvidenceRecord {
            format: crate::model::EVIDENCE_FORMAT.to_owned(),
            id: EvidenceId::new(id),
            kind: crate::model::EvidenceKindId::new(kind),
            result: None,
            subject: crate::model::ArtifactId::new("a"),
            subject_revision: crate::model::Revision::new("r1"),
            observed_at: None,
        }
    }

    fn exclusion(id: &str, reason: crate::model::ExclusionReason) -> EvidenceExclusion {
        EvidenceExclusion {
            evidence: EvidenceId::new(id),
            reason,
        }
    }

    fn ids(kept: &[&EvidenceRecord]) -> Vec<String> {
        kept.iter()
            .map(|record| record.id.as_str().to_owned())
            .collect()
    }

    /// Each record a stage names leaves the kept set and joins the excluded list once, in the
    /// order named; a record no longer kept (excluded by an earlier stage, or never given) is not
    /// excluded again.
    #[test]
    fn set_aside_moves_each_named_record_out_of_the_kept_set_once() {
        use crate::model::ExclusionReason::{Expired, Invalidated, RevisionMismatch};
        let records = [bare("e1", "k"), bare("e2", "k"), bare("e3", "k")];
        let mut kept: Vec<&EvidenceRecord> = records.iter().collect();
        let mut excluded = Vec::new();
        set_aside(
            &mut kept,
            &mut excluded,
            vec![
                exclusion("e2", RevisionMismatch),
                exclusion("gone", Expired),
            ],
        );
        assert_eq!(ids(&kept), ["e1", "e3"]);
        assert_eq!(excluded, [exclusion("e2", RevisionMismatch)]);
        set_aside(
            &mut kept,
            &mut excluded,
            vec![exclusion("e3", Invalidated), exclusion("e2", Expired)],
        );
        assert_eq!(ids(&kept), ["e1"]);
        assert_eq!(
            excluded,
            [
                exclusion("e2", RevisionMismatch),
                exclusion("e3", Invalidated)
            ]
        );
    }

    /// An excluded record is listed under every claim that reaches its kind, directly or through
    /// a chain of claim references, in evidence-id order, and under no other claim.
    #[test]
    fn excluded_for_lists_the_kinds_a_claim_reaches_in_evidence_id_order() {
        use crate::model::ExclusionReason::{Expired, RevisionMismatch};
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {k: {}, l: {}}\n\
                 claims:\n\
                 \x20\x20direct: {true_when: {evidence: {kind: k}}}\n\
                 \x20\x20one_level: {true_when: {not: {claim: direct}}}\n\
                 \x20\x20two_levels: {true_when: {any: [{claim: one_level, is: unknown}]}}\n\
                 \x20\x20other: {true_when: {evidence: {kind: l}}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let evidence = [bare("e1", "k"), bare("e2", "l"), bare("e3", "k")];
        // In the order the stages excluded them, which is not evidence-id order.
        let excluded = [
            exclusion("e3", RevisionMismatch),
            exclusion("e2", Expired),
            exclusion("e1", Expired),
        ];
        let listed = |claim: &str| -> Vec<String> {
            excluded_for(&ir, &ClaimId::new(claim), &evidence, &excluded)
                .iter()
                .map(|exclusion| exclusion.evidence.as_str().to_owned())
                .collect()
        };
        for claim in ["direct", "one_level", "two_levels"] {
            assert_eq!(listed(claim), ["e1", "e3"], "{claim}");
        }
        assert_eq!(listed("other"), ["e2"]);
        assert_eq!(
            excluded_for(&ir, &ClaimId::new("direct"), &evidence, &[]),
            Vec::new()
        );
    }
}

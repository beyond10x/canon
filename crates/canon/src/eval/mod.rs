//! Deterministic evaluation of a case under a compiled protocol (design § 8, § 13).
//!
//! [`evaluate_with`] is a pure function of a compiled protocol ([`Ir`]), a `canon-case/1` case
//! snapshot, a set of `canon-evidence/1` records and the inputs a caller supplies ([`Supplied`]):
//! `canon-authority/1` authority decisions (`--authority`), the evaluation instant (`--at`) and
//! `canon-decisions/1` explicit decisions (`--decisions`).
//! [`evaluate`] is the same with nothing supplied. It reads nothing else: no clock, network,
//! filesystem, credentials or implicit `latest`. Its result is a `canon-decision/1` [`Decision`]
//! giving every declared claim the value `true`, `false` or `unknown`, each declared obligation
//! `open` or `discharged`, each declared action `admissible`, `approval-required` or `blocked`,
//! and each declared outcome `legitimate` or `blocked` (the sections below), or a
//! [`Refusal`] when the inputs do not fit the protocol. [`render`] is the decision's one
//! serialization: canonical JSON, the serialization `canon-ir/1` uses.
//!
//! # Three-valued claims
//!
//! A claim's value is its `true_when` predicate's value over the evidence set:
//!
//! - An evidence match reads the records of its kind. One that names a `subject` reads only the
//!   records about that artifact: a record about another artifact, even at that artifact's
//!   current revision, is not of the match and neither establishes nor contradicts it
//!   (CANON-EVIDENCE-003). One that names no subject reads records about any artifact. One that
//!   demands independence (`independent_on: [D, ...]`) reads only the records that name a
//!   `producer` the case's `independence` map lists on every dimension `D`: any other record of
//!   its kind and subject is not of the match, whatever its result, so it neither establishes nor
//!   contradicts it (CANON-INDEPENDENCE-001, `independence.rs`). "Records of the kind" below are
//!   the records the match reads.
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
//! - An input test `{input: s, in: [...]}` is `true` when the case's value of the input `s` is one
//!   it lists and `false` otherwise. A case gives every input the protocol declares a value (the
//!   case check refuses one that does not), so an input test is never `unknown`: inputs are
//!   configuration fixed when the case opens, not evidence.
//! - An order form, `{first: {kind: K, result: R}}`, `{before: {first: K1, then: K2}}` or
//!   `{after: {first: K1, then: {kind: K2, result: R}}}`, reads the evidence history instead: every
//!   record of its kinds (about its `subject` when it names one) the evidence check accepted, the
//!   records the exclusion stages set aside included, ordered by `observed_at`. `first` is whether
//!   the first record of `K` has result `R`; `before` whether the first record of `K1` was observed
//!   strictly before the first of `K2`; `after` whether a record of `K2` with result `R` was
//!   observed strictly after the first record of `K1`. Each is `unknown` while the order it reads
//!   is not established, as with no record or a compared record without `observed_at`; the
//!   predicate's reference in `protocol/1` gives every rule (CANON-ORDER-001, CANON-ORDER-002). No
//!   record is listed in `excluded_evidence` because of an order form.
//!
//! `unknown` and `false` are never collapsed, and only `true` satisfies a positive requirement.
//!
//! # Refusals
//!
//! Inputs are checked before any claim is evaluated, in this order, and the first problem found is
//! the refusal: the compiled protocol's depth (a predicate nested more than [`MAX_IR_DEPTH`]
//! levels, itself or along the claims it tests, as `predicate-too-deep`, and claims that test each
//! other in a cycle as `claim-cycle`; see "Depth" below), the case snapshot's format, its identifiers (case id, protocol id, the case
//! revision when given, then each artifact id and revision in the order written), its protocol against the compiled protocol's,
//! each artifact it lists (declared by the protocol, listed once), each artifact the protocol
//! declares (listed by the case), each case input it gives, in the order written (its id and
//! value identifiers, then declared by the protocol, else `undeclared-input`, given once, else
//! `duplicate-identifier`, and its value declared by the input, else `undeclared-input-value`,
//! naming the input and the value), each input the protocol declares (given by the case, else
//! `missing-input`, naming it), each producer its `independence` map lists, in the order written
//! (an identifier, listed once, else `duplicate-identifier`, and each dimension listed for it an
//! identifier, listed once for it, else `duplicate-identifier`, and declared by the protocol, else
//! `undeclared-dimension`, naming the producer and the dimension); then each evidence record in
//! the order given: its format, its identifiers (id, kind, subject, subject revision, then each
//! upstream artifact and revision in `upstream_revisions`, then its `producer` when given), its
//! `observed_at` when given (an instant, a fraction of one to nine digits allowed, else
//! `invalid-instant`), its id
//! (used once in the set), its kind (declared by the protocol) and each upstream artifact
//! (declared by the protocol, else `undeclared-artifact`, naming the record and the artifact; named
//! once, else `duplicate-identifier`). A record's subject and subject revision are checked here as
//! identifiers only; the revision-binding stage reads them. Every refusal of a record names it
//! (`` evidence `e2` format is `canon-evidence/2`, expected `canon-evidence/1` ``, `` evidence
//! `e2` kind identifier `k k` is empty or … ``), and cites it ([`Refusal::evidence`]), here and
//! in the revision-binding stage below, so a caller that read the records from files can name the
//! file of each (`canon evaluate` does).
//!
//! A case snapshot or evidence record that is not a document of its format is refused when it is
//! read (`read_case`, `read_evidence`), before any of this, as `malformed-input`; a required key
//! left out is named (`` missing field `subject` ``), and an evidence record is named by its `id`
//! when it has one (`` evidence `e2` is not a canon-evidence/1 document: missing field `subject`
//! ``).
//!
//! Then the supplied inputs are read, in this order: the authority decisions (`--authority`), the
//! evaluation instant (`--at`) and the explicit decisions. The authority decisions are read by
//! `authority.rs` as `canon-authority/1`, refused as the section "Authority decisions" below
//! says. The instant is read by `freshness.rs` and refused as `invalid-instant` when it is not
//! one. The explicit decisions are read by `decisions.rs` as `canon-decisions/1`, refused as the
//! section "Explicit decisions" below says.
//!
//! Then the exclusion stages run, in pipeline order: revision binding, freshness, invalidation.
//! Each may refuse the inputs it reads. Revision binding checks each record in the order given: a
//! record whose subject the protocol does not declare is refused as `undeclared-artifact`, naming
//! the record and its subject (`` evidence `e1` is about artifact `x`, which the protocol does not
//! declare ``), and a record bound to a revision of its subject that is not the case snapshot's
//! current one is excluded as `revision_mismatch`. Freshness refuses a `max_age` it cannot read
//! (`invalid-max-age`) and excludes a record older than its kind's `max_age` at the evaluation
//! instant as `expired`. Invalidation refuses a rule whose upstream artifact the case snapshot does
//! not name, which only an IR a caller builds can hold, as `undeclared-artifact`, and invalidates,
//! for the claims each rule names and every claim built on one of them, the records that recorded
//! a revision of the rule's upstream artifact other than the case snapshot's current one
//! (`invalidation.rs`, CANON-INVALIDATION-001).
//!
//! Then claims are evaluated.
//!
//! Last, the `outcomes` section checks the case snapshot's `termination`: one naming an outcome
//! the protocol does not declare is refused as `undeclared-outcome`, naming the outcome; then an
//! explicit decision taken for an outcome the protocol does not declare is refused as
//! `undeclared-outcome`, and one taken for a declared outcome that does not require a decision of
//! its name as `undeclared-decision`, each naming the decision and the outcome; and a
//! `termination` naming a declared outcome that is blocked as `illegitimate-termination`, naming
//! the outcome and its status (CANON-OUTCOME-001).
//!
//! # Depth
//!
//! [`read_ir`] refuses an IR whose arrays and objects nest deeper than [`MAX_IR_DEPTH`]
//! (4096) before parsing it. Reading and claim evaluation recurse along the nesting of the
//! predicates, so they run on a thread of their own with a stack sized for that bound
//! (`DEEP_STACK`): an IR at the bound reads and evaluates from any caller's thread.
//!
//! Claim evaluation also recurses through each claim test into the predicate of the claim it
//! tests, so the bound applies to a claim's effective depth: a predicate is one level and each
//! `all`, `any` or `not` around it one more, and a claim test at level `n` reaches `n` plus the
//! effective depth of the claim it tests. [`evaluate_with`] takes any [`Ir`] a caller builds and
//! checks the bound first, before any step recurses (`depth.rs`), computing each claim's
//! effective depth once, after the claims it tests, without recursion. It refuses as
//! `predicate-too-deep` an obligation, action, outcome or claim whose own predicate nests more
//! than [`MAX_IR_DEPTH`] levels (`` claim `c` true_when nests predicates deeper than 4096 ``), and
//! a claim whose effective depth is more than [`MAX_IR_DEPTH`], naming the claims of the chain that
//! reaches it (`` claim `c1` true_when nests predicates deeper than 4096 through the claims it
//! tests: c1 -> c2 -> c3 ``). Claims that test each other in a cycle — which `canon compile` never
//! produces, but a caller can build — have no effective depth and are refused there as
//! `claim-cycle`, naming the claims from the first one reached again, in the order evaluation
//! reaches them: `claims test each other in a cycle: c -> d -> c`. Each level of the JSON text is
//! at least one predicate level, so [`read_ir`]'s bound holds every predicate of an IR it returns
//! within the bound; a chain of claims, each within it, can still exceed it, and `canon compile`
//! compiles such a protocol, so an IR [`read_ir`] returns can still be refused here. Dropping a
//! caller's IR is the caller's own recursion.
//!
//! # The pipeline
//!
//! [`evaluate_with`] runs the same steps for every evaluation, each in the file that owns its
//! concept; this module holds only the order:
//!
//! 1. check the compiled protocol's depth (`depth.rs`), the case snapshot (`case.rs`) and the
//!    evidence set (`evidence.rs`);
//! 2. read the supplied inputs: authority decisions (`authority.rs`), the evaluation instant
//!    (`freshness.rs`) and explicit decisions (`decisions.rs`);
//! 3. run the exclusion stages in order: revision binding (`binding.rs`), freshness
//!    (`freshness.rs`), invalidation (`invalidation.rs`). Each sees only the evidence the stages
//!    before it left. Revision binding and freshness return the records they exclude from every
//!    claim, with their reason; invalidation returns, for each claim, the records it keeps from
//!    that claim alone;
//! 4. evaluate every claim over the evidence left (`claims.rs`), and its order forms over every
//!    record the evidence check accepted (`order.rs`), each in its own context: no
//!    evidence match in its evaluation reads a record invalidated for it, including the matches of
//!    every claim it tests, which are evaluated for that use without the records invalidated for
//!    either; the value reported for a tested claim is its own. Each claim's
//!    `excluded_evidence` lists, in evidence-id order, every excluded record of a kind the claim
//!    reaches: a kind an evidence match in its own predicate names, or one a claim it tests
//!    reaches, through any number of claim references. A match that names a subject reaches its
//!    kind only for records about that artifact: a record about another artifact does not match
//!    it, so it is not listed under a claim that reaches its kind only through such a match. A
//!    record excluded from a claim is excluded from every claim built on it. A record invalidated
//!    for a claim is listed under it, with the reason `invalidated`, when an evidence match the
//!    claim reaches reads it, and under no claim it was not invalidated for. Any other record is
//!    listed under a claim, with the reason `not_independent`, when an evidence match the claim
//!    reaches would read it by kind and subject and none of those matches accepts its producer
//!    (`independence.rs`): a match that demands independence keeps it out, and no match demanding
//!    less reads it. Each record is listed once, with the first of these reasons that applies;
//! 5. the `obligations`, `actions` and `outcomes` sections (`obligations.rs`, `actions.rs`,
//!    `outcomes.rs`), then the explanation (`crate::explain`). Each section evaluates its
//!    predicates with the one evaluator claims use (`claims::predicate`): discharge and
//!    applicability predicates over the claim values, action preconditions and outcome
//!    requirements over the claim values and the evidence left after step 3 (their order forms
//!    over every record accepted), each with the case's input values, which every input test
//!    reads. The invalidation stage keeps a record from claims only, so
//!    these predicates' own evidence matches read it. An outcome that requires an explicit
//!    decision is decided by the explicit decisions instead.
//!
//! Revision binding excludes a record bound to another revision as `revision_mismatch`, and
//! freshness a record older than its kind's `max_age` at the evaluation instant as `expired`,
//! each listed as excluded under each claim that reaches its kind through a match that reads it, as
//! step 4 says. Invalidation keeps a record from the claims an invalidation rule names and every
//! claim built on one of them, throughout their evaluation, as `invalidated`, listed under each of
//! those claims that reaches a match reading it; every other claim, even one reading the same
//! kind, reads the record and does not list it. The `obligations`, `actions` and `outcomes`
//! sections are written as the next section says, and every decision carries an `explanation`
//! (`crate::explain`).
//!
//! # Sections
//!
//! A section is written only when the protocol declares something it reports on: a protocol that
//! declares no obligation has no `obligations` key, and likewise for actions and outcomes.
//!
//! - `obligations` is an array with one `{"id": <obligation>, "status": <status>}` entry per
//!   declared obligation, in identifier order. An obligation with an `applicable_when` predicate is
//!   `not_applicable` when it is `false` over the claim values and the case's inputs. Otherwise the
//!   status is `discharged` when the obligation's `discharged_when` predicate is `true` over the
//!   claim values, whatever its applicability; `unknown` and `false` both leave it `open`
//!   (CANON-OBLIGATION-001). An open obligation whose applicability is `unknown` has
//!   `"reasons"` naming what is unknown, written as an action's reasons are (CANON-INPUT-001).
//! - `actions` maps each declared action to its `status` and, unless it is admissible, the
//!   `reasons` that decide it. `blocked` when the precondition is not `true`: each claim test and
//!   evidence match that decides it is a reason, `{"claim": <id>, "value": <the claim's value>}`
//!   or `{"evidence": <kind>, "present": <whether a record the match reads applies>}`, with
//!   `"subject": <artifact>` added when the match names one (a record about another artifact
//!   is not read by it), then each input test that decides it, `{"input": <id>, "value": <the
//!   case's value>}`, then each order form that decides it, `{"after": <kind>, "then": <kind>,
//!   "result": <result>, "value": <its value>}`, `{"before": <kind>, "then": <kind>, "value":
//!   <its value>}` or `{"first": <kind>, "result": <result>, "value": <its value>}`, with
//!   `"subject"` added when it names one, or
//!   `{"requirement": "unsatisfiable"}` when none does. `blocked` too when the precondition is
//!   `true` and the authority decisions deny a capability the action requires, each a reason
//!   `{"capability": <id>, "decision": "denied"}`. `approval-required` when the precondition is
//!   `true`, nothing is denied and some required capability is not decided, each a reason
//!   `{"capability": <id>, "decision": "none"}`. `admissible` when the precondition is `true` and
//!   every required capability is granted (CANON-AUTHORITY-001).
//! - `outcomes` maps each declared outcome to `{"status": "legitimate"}` when its `requires`
//!   predicate is `true` over the claim values and the evidence left, and otherwise to
//!   `{"status": "blocked", "reasons": [...]}`. Its reasons follow the one rule an action
//!   precondition's do, and are written the same way: in an `all` that is `false`, only its
//!   `false` members decide it, with polarity carried through `not`.
//!   An outcome that requires an explicit decision (`requires: decision: <name>`) is
//!   `legitimate` only when an explicit decision of that name, for that outcome, was taken at the
//!   case snapshot's `revision`, and its entry then records who decided (design § 37):
//!   `{"status": "legitimate", "decided_by": {"decision": <name>, "principals": [...]}}`, every
//!   principal whose decision applied, sorted by Unicode code point; otherwise it is blocked with
//!   the one reason `{"decision": <name>, "present": false}` (CANON-OUTCOME-002). The case snapshot's
//!   `termination` is checked against it, as the refusals above say (CANON-OUTCOME-001).
//! - `explanation` is written for every decision: what the decision was computed from (design
//!   § 37), and why each claim that is not `true`, each open obligation, each action that is not
//!   admissible and each blocked outcome has its status, down to the evidence records that applied
//!   or were excluded. Its shape is given by `crate::explain` (CANON-EXPLAIN-001).
//!
//! # Authority decisions
//!
//! `--authority` names a `canon-authority/1` document: a YAML (or JSON) list of decisions, each a
//! capability and whether it is granted, `- {capability: finding.publish, decision: granted}` (or
//! `denied`). An empty list decides nothing. Canon grants nothing and resolves no identity: the
//! caller decides and passes the decisions in. Refused, in this order: text that is not such a
//! list, or an entry with another key, another decision or a missing key, as `malformed-input`,
//! naming the entry by its position, counting from 1, and a missing key by name (`` entry 2:
//! missing field `capability` ``); then, entry by entry, a capability that is not an identifier
//! as `invalid-identifier`, and a capability decided a second time as `duplicate-identifier`.
//!
//! # Explicit decisions
//!
//! `--decisions` names a `canon-decisions/1` document: a YAML (or JSON) list of decisions, each
//! naming the decision, the outcome it was taken for, the principal who took it and the case
//! revision it was taken at, `- {decision: explicitly_inconclusive, outcome: inconclusive,
//! principal: lead-investigator, case_revision: c2}`. An empty list decides nothing. Canon decides
//! nothing and resolves no identity: the caller passes in the decisions that were taken. A
//! decision applies only while its case revision is the case snapshot's `revision`: one taken at a
//! superseded revision, or given for a snapshot without a revision, applies to nothing. Refused,
//! in this order: text that is not such a list, or an entry with another key or a missing one, as
//! `malformed-input`, naming the entry by its position and a missing key by name (`` entry 1:
//! missing field `case_revision` ``); then, entry by entry, a decision, outcome, principal or case revision that
//! is not an identifier as `invalid-identifier`, and an entry that repeats an earlier one exactly as
//! `duplicate-identifier`, naming it; then, in the `outcomes` section and at any case revision, a
//! decision for an outcome the protocol does not declare as `undeclared-outcome`, and one for an
//! outcome that does not require a decision of its name as `undeclared-decision`.

mod actions;
mod authority;
mod binding;
mod case;
mod claims;
mod decision;
mod decisions;
mod depth;
mod evidence;
mod freshness;
mod independence;
mod invalidation;
mod json;
mod obligations;
mod order;
mod outcomes;
mod read;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::ir::Ir;
use crate::model::{
    Case, ClaimDecision, ClaimId, DECISION_FORMAT, Decision, Declarations, EvidenceExclusion,
    EvidenceId, EvidenceRecord, ExclusionReason, Predicate, is_identifier, one_line,
};

pub use case::{case_from_value, read_case};
pub use decision::{render, render_sections};
pub use evidence::{evidence_from_value, read_evidence};
pub use read::{MAX_IR_DEPTH, read_ir};

pub(crate) use authority::Authority;
pub(crate) use claims::{EvidenceSets, Inputs, reads};
pub(crate) use decisions::Decisions;
pub(crate) use depth::check as check_depth;
pub(crate) use independence::Independence;
pub(crate) use invalidation::{Invalidated, invalidated_claims};

/// Why an evaluation was refused: a stable machine-readable code, a one-line message, and the
/// evidence records it refuses, by id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    code: &'static str,
    message: String,
    evidence: Vec<EvidenceId>,
}

impl Refusal {
    fn new(code: &'static str, message: String) -> Self {
        Self {
            code,
            message,
            evidence: Vec::new(),
        }
    }

    /// The same refusal, citing the evidence record `id`.
    fn citing(mut self, id: &EvidenceId) -> Self {
        self.evidence.push(id.clone());
        self
    }

    /// A stable machine-readable code for the kind of refusal; a conformance scenario names it.
    pub fn code(&self) -> &'static str {
        self.code
    }

    /// The evidence records the refusal is about, by id, each named in the message: every refusal
    /// of a record cites it, so a caller that read the records from files can name each one's file
    /// (`canon evaluate` does). Empty for a refusal of anything else.
    pub fn evidence(&self) -> &[EvidenceId] {
        &self.evidence
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
    depth::check(ir)?;
    case::check(ir, case)?;
    evidence::check(ir, evidence)?;
    let authority = authority::read(supplied.authority)?;
    let at = freshness::instant(supplied.at)?;
    let decisions = decisions::read(supplied.decisions)?;
    on_deep_stack(|| {
        let case_inputs: Inputs = case
            .inputs
            .iter()
            .map(|(input, value)| (input.clone(), value.clone()))
            .collect();
        let mut applicable: Vec<&EvidenceRecord> = evidence.iter().collect();
        let mut excluded: Vec<EvidenceExclusion> = Vec::new();
        let stage = binding::exclude(ir, case, &applicable)?;
        set_aside(&mut applicable, &mut excluded, stage);
        let stage = freshness::exclude(ir, case, &applicable, at.as_ref())?;
        set_aside(&mut applicable, &mut excluded, stage);
        let invalidated = invalidation::exclude(ir, case, &applicable)?;

        let applicable: Vec<EvidenceRecord> = applicable.into_iter().cloned().collect();
        // Evidence matches read what the exclusion stages left; the order forms read every record
        // the evidence check accepted.
        let independence = Independence::of(case);
        let sets = EvidenceSets {
            left: &applicable,
            history: evidence,
            independence: Some(&independence),
        };
        let values = claims::values(ir, sets, &invalidated, &case_inputs)?;
        let obligations = obligations::section(ir, &values, &case_inputs);
        let actions = actions::section(ir, &values, &case_inputs, sets, authority.as_ref());
        let outcomes =
            outcomes::section(ir, case, &values, &case_inputs, sets, decisions.as_ref())?;
        let claims = values
            .iter()
            .map(|(id, value)| {
                let entry = ClaimDecision {
                    value: *value,
                    excluded_evidence: excluded_for(
                        ir,
                        id,
                        evidence,
                        &excluded,
                        &invalidated,
                        &independence,
                    ),
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
        let inputs = crate::explain::Inputs {
            authority: authority.as_ref(),
            decisions: decisions.as_ref(),
            at: supplied.at,
        };
        decision.explanation = crate::explain::explain(
            ir,
            case,
            evidence,
            &excluded,
            &invalidated,
            inputs,
            &decision,
        );
        Ok(decision)
    })
}

/// The reasons `predicate` is not `true` over the claim `values`, the case's `inputs` and the
/// `evidence` (what was left, and the history), as a blocked outcome states them (`outcomes.rs`):
/// the explanation gives an open obligation the same reasons. Called only for a predicate that is
/// not `true`.
pub(crate) fn unmet_reasons(
    predicate: &Predicate,
    values: &BTreeMap<ClaimId, crate::model::Truth>,
    inputs: &Inputs,
    evidence: EvidenceSets<'_>,
) -> Vec<crate::model::Json> {
    outcomes::reasons(predicate, values, inputs, evidence)
}

/// The reason each order form among `forms` gives, as a blocked action names it, with its value
/// over the `history` (`order.rs`): `before` forms first, each form once.
pub(crate) fn order_reasons<'a>(
    forms: impl IntoIterator<Item = &'a Predicate>,
    history: &[EvidenceRecord],
) -> Vec<crate::model::Json> {
    order::reasons(forms, history)
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

/// The excluded records listed under `claim`: those an evidence match the claim reaches would read
/// (of its kind and, when it names a subject, about that artifact), in evidence-id order. A claim
/// reaches the evidence matches of its own predicate, and every match a claim it tests reaches,
/// through any number of claim references; each claim is visited once, so a cycle a caller builds
/// into an IR ends. The records the exclusion stages set aside for every claim (`excluded`) are
/// listed with their reasons. A record the invalidation stage kept from this claim
/// (`invalidated`) is listed with the reason `invalidated` when an evidence match the claim reaches
/// reads it ([`invalidation::Invalidated::lists`]). Any other record is listed with the reason
/// `not_independent` when a match the claim reaches would read it by kind and subject and none of
/// those matches accepts its producer ([`independence::rejected`]). Each record is listed once.
fn excluded_for(
    ir: &Ir,
    claim: &ClaimId,
    evidence: &[EvidenceRecord],
    excluded: &[EvidenceExclusion],
    invalidated: &invalidation::Invalidated,
    independence: &Independence,
) -> Vec<EvidenceExclusion> {
    let kept = invalidated.for_claim(claim);
    let reached = invalidation::reached_matches(ir, claim);
    let demanding = reached
        .iter()
        .any(|matching| matching.independent_on.is_some());
    if excluded.is_empty() && kept.is_none_or(BTreeSet::is_empty) && !demanding {
        return Vec::new();
    }
    let mut matches = BTreeSet::new();
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
                matches.insert((&matching.kind, matching.subject.as_ref()));
            }
            Predicate::Claim(test) => pending.push(&test.claim),
            _ => {}
        });
    }
    let record_of: BTreeMap<&EvidenceId, &EvidenceRecord> =
        evidence.iter().map(|record| (&record.id, record)).collect();
    let read = |record: &EvidenceRecord| {
        matches
            .iter()
            .any(|(kind, subject)| claims::reads(kind, *subject, record))
    };
    let mut listed: Vec<EvidenceExclusion> = excluded
        .iter()
        .filter(|exclusion| {
            record_of
                .get(&exclusion.evidence)
                .is_some_and(|record| read(record))
        })
        .cloned()
        .chain(
            kept.into_iter()
                .flatten()
                .filter(|id| {
                    record_of
                        .get(id)
                        .is_some_and(|record| invalidated.lists(ir, claim, record))
                })
                .map(|id| EvidenceExclusion {
                    evidence: id.clone(),
                    reason: ExclusionReason::Invalidated,
                }),
        )
        .collect();
    let set_aside: BTreeSet<&EvidenceId> = excluded
        .iter()
        .map(|exclusion| &exclusion.evidence)
        .collect();
    listed.extend(
        evidence
            .iter()
            .filter(|record| {
                !set_aside.contains(&record.id)
                    && !invalidated.excludes(claim, &record.id)
                    && independence::rejected(&reached, record, independence)
            })
            .map(|record| EvidenceExclusion {
                evidence: record.id.clone(),
                reason: ExclusionReason::NotIndependent,
            }),
    );
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
        // Every section but the explanation, whose shape `crate::explain` tests.
        let listed = ["case", "claims", "format", "protocol", "protocol_revision"]
            .map(str::to_owned)
            .into();
        assert_eq!(
            render_sections(&decision, &listed),
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
                "evidence `e1` format is `evidence/1`, expected `canon-evidence/1`",
            ),
            (
                CASE.to_owned(),
                vec![pass.replace("subject_revision: r1", "subject_revision: 'r 1'")],
                "invalid-identifier",
                "evidence `e1` subject revision `r 1` is empty",
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

    /// A record about an artifact the protocol does not declare is refused, naming it; one bound
    /// to a revision of its subject that is not current is excluded, so it decides nothing and is
    /// listed under every claim that reaches its kind.
    #[test]
    fn evidence_applies_only_to_the_current_revision_of_a_declared_subject() {
        let elsewhere = record("e1", "pass").replace("subject: a", "subject: nowhere");
        let refusal = decide(CASE, &[elsewhere]).expect_err("refused");
        assert_eq!(refusal.code(), "undeclared-artifact", "{refusal}");
        assert_eq!(
            refusal.to_string(),
            "evidence `e1` is about artifact `nowhere`, which the protocol does not declare"
        );

        let stale = record("e1", "fail").replace("subject_revision: r1", "subject_revision: r0");
        assert_eq!(values(std::slice::from_ref(&stale)), values(&[]));
        let decision = decide(CASE, &[stale, record("e2", "pass")]).expect("decides");
        for (claim, entry) in decision.claims.iter() {
            assert_eq!(
                entry.excluded_evidence,
                [exclusion(
                    "e1",
                    crate::model::ExclusionReason::RevisionMismatch
                )],
                "{}",
                claim.as_str()
            );
        }
        assert_eq!(
            values(&[record("e2", "pass")]),
            decision
                .claims
                .iter()
                .map(|(id, entry)| (id.as_str().to_owned(), entry.value))
                .collect::<Vec<_>>()
        );
    }

    /// One record of `kind` about artifact `a` at `r1`, built directly.
    fn bare(id: &str, kind: &str) -> EvidenceRecord {
        EvidenceRecord {
            format: crate::model::EVIDENCE_FORMAT.to_owned(),
            producer: None,
            id: EvidenceId::new(id),
            kind: crate::model::EvidenceKindId::new(kind),
            result: None,
            subject: crate::model::ArtifactId::new("a"),
            subject_revision: crate::model::Revision::new("r1"),
            observed_at: None,
            upstream_revisions: crate::model::Declarations::default(),
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
            excluded_for(
                &ir,
                &ClaimId::new(claim),
                &evidence,
                &excluded,
                &invalidation::Invalidated::default(),
                &Independence::default(),
            )
            .iter()
            .map(|exclusion| exclusion.evidence.as_str().to_owned())
            .collect()
        };
        for claim in ["direct", "one_level", "two_levels"] {
            assert_eq!(listed(claim), ["e1", "e3"], "{claim}");
        }
        assert_eq!(listed("other"), ["e2"]);
        assert_eq!(
            excluded_for(
                &ir,
                &ClaimId::new("direct"),
                &evidence,
                &[],
                &invalidation::Invalidated::default(),
                &Independence::default(),
            ),
            Vec::new()
        );
    }

    /// A claim is evaluated in its own context: a record invalidated for it is read by no match in
    /// its evaluation, including the match of `helper`, which the rule does not name. So `named`
    /// is `unknown`, not `true` on `e1` through `helper`, and lists `e1`; so do `built` and
    /// `tests_only`, built on it, the second through claim tests alone. `helper` reports its own
    /// value, `true` on `e1`, and lists nothing. The explanation agrees with `excluded_evidence`.
    #[test]
    fn a_claim_is_evaluated_without_its_invalidated_records_in_every_claim_it_tests() {
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {}, m: {}}\n\
                 claims: {helper: {true_when: {evidence: {kind: k}}}, \
                 named: {true_when: {all: [{claim: helper}, {evidence: {kind: m}}]}}, \
                 built: {true_when: {all: [{claim: named}, {evidence: {kind: k}}]}}, \
                 tests_only: {true_when: {not: {claim: named}}}}\n\
                 invalidation: {r: {upstream: up, invalidates: [named]}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let case = read_case(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}, up: {revision: u1}}\n",
        )
        .expect("reads");
        let evidence = [
            read_evidence(
                "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n\
                 upstream_revisions: {up: u0}\n",
            )
            .expect("reads"),
            read_evidence(
                "format: canon-evidence/1\nid: e2\nkind: m\nsubject: a\nsubject_revision: r1\n",
            )
            .expect("reads"),
        ];
        let decision = evaluate(&ir, &case, &evidence).expect("decides");
        let entry = |claim: &str| {
            let entry = decision.claims.get(&ClaimId::new(claim)).expect("declared");
            let listed: Vec<&str> = entry
                .excluded_evidence
                .iter()
                .map(|exclusion| exclusion.evidence.as_str())
                .collect();
            (entry.value, listed)
        };
        assert_eq!(entry("helper"), (Truth::True, vec![]));
        assert_eq!(entry("named"), (Truth::Unknown, vec!["e1"]));
        assert_eq!(entry("built"), (Truth::Unknown, vec!["e1"]));
        assert_eq!(entry("tests_only"), (Truth::Unknown, vec!["e1"]));
        let explained = |claim: &str| -> Vec<String> {
            decision.explanation.as_ref().expect("explained")["claims"][claim]["because"]
                .as_array()
                .expect("because")
                .iter()
                .filter(|reason| reason["evidence"] == "e1")
                .map(ToString::to_string)
                .collect()
        };
        for claim in ["named", "built", "tests_only"] {
            assert_eq!(
                explained(claim),
                [r#"{"evidence":"e1","reason":"invalidated","status":"excluded"}"#],
                "{claim}"
            );
        }
        assert_eq!(
            explained("helper"),
            [r#"{"evidence":"e1","status":"applied"}"#]
        );
    }

    /// The order forms read the history: a record excluded as `revision_mismatch` or `expired`, or
    /// invalidated for the claim, is still the first record of its kind. It is listed under a claim
    /// only because an evidence match the claim reaches reads it, never because of an order form.
    /// The explanation names each order form with its value, as an action's reasons do.
    #[test]
    fn an_order_form_reads_every_record_the_exclusion_stages_set_aside() {
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {max_age: 1h}, m: {}}\n\
                 claims: {red_first: {true_when: {first: {kind: k, result: red}}}, \
                 k_before_m: {true_when: {before: {first: k, then: m, subject: a}}}, \
                 green: {true_when: {evidence: {kind: k, result: green}}}, \
                 fixed: {true_when: {all: [{first: {kind: k, result: red}}, \
                 {evidence: {kind: k, result: green}}]}}, \
                 both: {true_when: {all: [{claim: red_first}, {not: {claim: k_before_m, is: false}}]}}}\n\
                 invalidation: {r: {upstream: up, invalidates: [fixed]}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let case = read_case(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r2}, up: {revision: u2}}\n",
        )
        .expect("reads");
        let record = |id: &str, kind: &str, result: &str, revision: &str, at: &str, up: &str| {
            let result = if result.is_empty() {
                String::new()
            } else {
                format!("result: {result}\n")
            };
            read_evidence(&format!(
                "format: canon-evidence/1\nid: {id}\nkind: {kind}\n{result}subject: a\n\
                 subject_revision: {revision}\nobserved_at: {at}\nupstream_revisions: {{up: {up}}}\n"
            ))
            .expect("reads")
        };
        let decide = |evidence: &[EvidenceRecord]| {
            evaluate_with(
                &ir,
                &case,
                evidence,
                Supplied {
                    at: Some("2026-10-02T12:00:00Z"),
                    ..Supplied::default()
                },
            )
            .expect("decides")
        };
        let green = record("g", "k", "green", "r2", "2026-10-02T11:30:00Z", "u2");
        let diff = record("m1", "m", "", "r2", "2026-10-02T11:40:00Z", "u2");
        for (red, reason) in [
            (
                record("e", "k", "red", "r1", "2026-10-02T11:00:00Z", "u2"),
                crate::model::ExclusionReason::RevisionMismatch,
            ),
            (
                record("e", "k", "red", "r2", "2026-10-01T11:00:00Z", "u2"),
                crate::model::ExclusionReason::Expired,
            ),
        ] {
            let decision = decide(&[red, green.clone(), diff.clone()]);
            let entry = |claim: &str| decision.claims.get(&ClaimId::new(claim)).expect("declared");
            for claim in ["red_first", "k_before_m", "both", "green", "fixed"] {
                assert_eq!(entry(claim).value, Truth::True, "{claim} {reason:?}");
            }
            for claim in ["red_first", "k_before_m", "both"] {
                assert!(entry(claim).excluded_evidence.is_empty(), "{claim}");
            }
            for claim in ["green", "fixed"] {
                assert_eq!(entry(claim).excluded_evidence, [exclusion("e", reason)]);
            }
        }
        // Invalidated for `fixed`: its evidence match does not read the red record, its order form
        // does. `green` reads both records, which disagree.
        let moved = record("e", "k", "red", "r2", "2026-10-02T11:00:00Z", "u1");
        let decision = decide(&[moved, green.clone()]);
        let entry = |claim: &str| decision.claims.get(&ClaimId::new(claim)).expect("declared");
        assert_eq!(entry("fixed").value, Truth::True);
        assert_eq!(
            entry("fixed").excluded_evidence,
            [exclusion("e", crate::model::ExclusionReason::Invalidated)]
        );
        assert_eq!(entry("red_first").value, Truth::True);
        assert_eq!(entry("green").value, Truth::Unknown);
        assert_eq!(entry("k_before_m").value, Truth::Unknown, "no `m` record");
        assert_eq!(entry("both").value, Truth::Unknown);
        let explained = &decision.explanation.as_ref().expect("explained")["claims"];
        assert_eq!(
            explained["k_before_m"],
            serde_json::json!({"because": [
                {"before": "k", "subject": "a", "then": "m", "value": "unknown"},
            ], "value": "unknown"})
        );
        assert_eq!(
            explained["both"]["because"],
            serde_json::json!([
                {"claim": "k_before_m", "value": "unknown"},
                {"claim": "red_first", "value": "true"},
            ])
        );
    }
}

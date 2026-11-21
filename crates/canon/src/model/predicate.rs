//! The predicate language: a small, total, deterministic expression language (design § 39.2).
//!
//! A predicate is one of `all`, `any`, `not`, an evidence match by kind, optional result and
//! optional subject, a test on a claim value, a test on a case input's value, one of the four order
//! forms over the evidence history, the order form over revisions (`descended`), or the form over
//! the case's child cases. Written in source as a map with exactly one of the keys `all`, `any`,
//! `not`, `evidence`, `claim`, `input`, `first`, `before`, `after`, `since`, `descended` or
//! `children`; a claim test may add `is`, which defaults to `true`,
//! and an input test adds `in`, the values it is true for (`{input: severity, in: [high]}`).
//!
//! The order forms read the evidence history: every record of their kind (about their `subject`
//! when they name one) that the evidence check accepted, including the records the exclusion
//! stages keep from evidence matches (`revision_mismatch`, `expired`, `invalidated`). A record's
//! place in the order is its `observed_at`; a record without one has no known place. An evaluation
//! compares every instant at one resolution, the coarsest precision written among the records
//! with an `observed_at`: each is read with its finer digits dropped, and two records read at one
//! instant are not ordered. Instant below means the instant at the evaluation's resolution.
//! `{first: {kind: K, result: R}}` asks whether the first record of `K` has result `R`: `unknown`
//! with no record or with a record without `observed_at`; otherwise, of the records at the
//! earliest instant, `true` when every one has `R`, `false` when none has, `unknown` when they
//! disagree. `{before: {first: K1, then: K2}}` asks whether the first record of `K1` was observed
//! strictly before the first record of `K2`: `unknown` with no record of either, `false` with a
//! record of `K2` and none of `K1` (a later `K1` record comes after it), `unknown` with a record of
//! `K1` and none of `K2`, `unknown` when a record of either has no `observed_at`, and otherwise
//! `true`, `false` or `unknown` as the earliest instants of `K1` and `K2` are earlier, later or
//! equal. `{after: {first: K1, then: {kind: K2, result: R}}}` asks whether some record of `K2`
//! with result `R` was observed strictly after the first record of `K1`: `unknown` with no record
//! of `K1`, `false` with no record of `K2` with `R`, `unknown` when a record of `K1`, or of `K2`
//! with `R`, has no `observed_at`, and otherwise `true` when one of `K2` with `R` comes after the
//! earliest instant of `K1`, `unknown` when one only shares that instant (other than the first
//! record itself: a record is not later than itself), and `false` when none comes after it. A
//! record of `K2` with another result takes no part. `K1` and `K2` may be one kind. `after` reads
//! no descent: a record bound to any earlier revision counts, whether or not the current revision
//! descends from it; `descended` below is the form that reads it (story:revision-ordering).
//! `{since: {last: K1, then: {kind: K2, result: R}}}` asks whether a record of `K2` with result `R`,
//! at the subject and subject revision of the latest record of `K1`, was observed after that
//! record: `unknown` with no record of `K1`, `false` with no record of `K2` with `R`, `unknown`
//! when a record of `K1`, or of `K2` with `R`, has no `observed_at`; otherwise each latest record
//! of `K1` (one no record of `K1` is observed after) gives `true` when a record of `K2` with `R` at
//! its subject and subject revision, other than itself, is observed after it, `false` when every
//! such record is observed before it or there is none, and `unknown` otherwise, and the form is
//! `true` when every latest record gives `true`, `false` when every one gives `false`, and
//! `unknown` otherwise. A record at a revision descended from the latest record's does not count.
//!
//! `{descended: {first: {kind: K1, result: R1}, then: {kind: K2, result: R2}}}`
//! (story:revision-ordering) orders by revision, not by instant: whether an item that had result
//! `R1` in a record of `K1` on a revision the current revision of its subject descends from has
//! result `R2` in a record of `K2` on the current revision. A later record is one of `K2` with `R2`
//! that names an `item` and is about the case snapshot's current revision of its subject; an
//! earlier record for it is one of `K1` with `R1` with the same `item`, about the same subject, at
//! a revision the snapshot lists in that artifact's `descends_from`. `true` when a later record
//! has an earlier record; otherwise `unknown` with no later record, or when the snapshot gives no
//! `descends_from` for the subject of one; otherwise `false`. A record without an `item` takes no
//! part, and `observed_at` is not read.
//!
//! An evidence match may demand independence: `{evidence: {kind: K, result: R, independent_on:
//! [D, ...]}}` reads a record only when it names a `producer` and the case's `independence` map
//! lists that producer on every dimension `D` (dimensions the protocol declares under
//! `independence_dimensions`). A record of the match's kind and subject that fails the demand is
//! not of the match, whatever its result: it neither establishes nor contradicts it, and is
//! excluded from it as `not_independent` (CANON-INDEPENDENCE-001). The order forms do not demand
//! independence.
//!
//! `{children: {in: [O, ...]}}` (story:case-composition) asks whether every child case of the case
//! ended with one of the outcomes `O`. It reads every `case_outcome` record the evidence check
//! accepted, as the order forms read the history, one record per child case: its `child_case`
//! names the child, its `result` the child's outcome, `open` while the child runs. `unknown` with no
//! record; `false` when a record's outcome is neither listed nor `open`, a child that ended outside
//! the set; otherwise `unknown` while a child is `open`, and `true` when every child ended with a
//! listed outcome. A child's outcome is a fact about the child case, so a record about an earlier
//! revision of its subject, an expired one and an invalidated one still count.
//!
//! `{children: {in: [O, ...], revision: current}}` (story:child-outcome-revision) binds the form
//! to the current revision: it reads only the `case_outcome` records whose `subject_revision` is
//! the case snapshot's current revision of their `subject`, by the rules above. A record about an
//! earlier revision is not read: it neither establishes nor contradicts the form, and is not
//! listed in `excluded_evidence`; with no record read the form is `unknown`. So a parent whose
//! subject is revised after its children ended falls back to `unknown` until a child about the new
//! revision ends, and since one child has one record whatever its revision
//! (`duplicate-child-case`), that child is a new one, with a new id. `revision: any`, like no key,
//! reads every record. A terminal `case_outcome` record's `subject_revision` is the revision the
//! child's outcome was computed against: when the child protocol declares the subject artifact,
//! the child's own revision of it at its terminating evaluation; when it does not, the parent's
//! revision of the subject when the child was opened. Canon does not write the record and does not
//! check who did; it compares the record's `subject_revision` with the snapshot's current revision
//! of the record's `subject`, as it binds every evidence record to the revision it was observed on.
//! A runner that stamps a record with the revision at which the child was opened, whatever the
//! child did, never satisfies the form for a child that writes its own subject.

use std::fmt;

use serde::Deserialize;

use super::ids::{
    ArtifactId, ClaimId, EvidenceKindId, IndependenceDimensionId, InputId, InputValue,
};

/// A three-valued truth value. `Unknown` is not `False`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Truth {
    True,
    False,
    Unknown,
}

impl fmt::Display for Truth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Truth::True => "true",
            Truth::False => "false",
            Truth::Unknown => "unknown",
        })
    }
}

impl<'de> Deserialize<'de> for Truth {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Written {
            Bool(bool),
            Word(String),
        }
        match Written::deserialize(deserializer)? {
            Written::Bool(true) => Ok(Truth::True),
            Written::Bool(false) => Ok(Truth::False),
            Written::Word(word) => match word.as_str() {
                "true" => Ok(Truth::True),
                "false" => Ok(Truth::False),
                "unknown" => Ok(Truth::Unknown),
                other => Err(serde::de::Error::custom(format!(
                    "truth value must be true, false or unknown, found `{other}`"
                ))),
            },
        }
    }
}

/// A predicate over evidence and claim values.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "WrittenPredicate")]
pub enum Predicate {
    /// True when every member is true; true when empty.
    All(Vec<Predicate>),
    /// True when some member is true; false when empty.
    Any(Vec<Predicate>),
    Not(Box<Predicate>),
    Evidence(EvidenceMatch),
    /// True when the case's value of the input is one of the values listed, false otherwise.
    Input(InputTest),
    /// Whether the first record of a kind, in the evidence history, has a result.
    First(FirstResult),
    /// Whether the first record of one kind, in the evidence history, was observed strictly before
    /// the first record of another.
    Before(EvidenceOrder),
    /// Whether a record of a kind with a result, in the evidence history, was observed strictly
    /// after the first record of a kind.
    After(EvidenceAfter),
    /// Whether a record of a kind with a result, at the subject revision of the latest record of a
    /// kind in the evidence history, was observed after that record.
    Since(EvidenceSince),
    /// Whether an item that had a result on a revision the current revision of its subject
    /// descends from has a result on the current revision (story:revision-ordering).
    Descended(EvidenceDescended),
    /// Whether every child case of the case ended with one of the outcomes listed, over its
    /// `case_outcome` records (story:case-composition).
    Children(ChildOutcomes),
    Claim(ClaimTest),
}

/// An order form: whether the first record of `kind`, about `subject` when it names one, has
/// `result`: the records at the earliest instant at the evaluation's resolution. It reads every
/// record of the kind the evidence check accepted, excluded or not.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FirstResult {
    /// The evidence kind whose first record is read.
    #[serde(deserialize_with = "super::present::required")]
    pub kind: EvidenceKindId,
    /// The result that first record is to have.
    #[serde(deserialize_with = "super::present::required")]
    pub result: String,
    /// The declared artifact the records read are about; without it, records about any artifact.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub subject: Option<ArtifactId>,
}

/// An order form: whether the first record of kind `first` was observed strictly before the first
/// record of kind `then`, both about `subject` when it names one: the earliest instants of the two
/// kinds at the evaluation's resolution, an equal pair not ordered. It reads every record of the
/// two kinds the evidence check accepted, excluded or not.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceOrder {
    /// The evidence kind whose first record is to come first.
    #[serde(deserialize_with = "super::present::required")]
    pub first: EvidenceKindId,
    /// The evidence kind whose first record is to come after it; another kind than `first`.
    #[serde(deserialize_with = "super::present::required")]
    pub then: EvidenceKindId,
    /// The declared artifact the records of both kinds are about; without it, records about any
    /// artifact.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub subject: Option<ArtifactId>,
}

/// An order form: whether some record of `then.kind` with result `then.result` was observed
/// strictly after the first record of kind `first`, all about `subject` when it names one: after
/// the earliest instant of `first` at the evaluation's resolution. It reads every record of the
/// two kinds the evidence check accepted, excluded or not. `first` and `then.kind` may be one
/// kind: a record is not later than itself.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceAfter {
    /// The evidence kind whose first record the later one is to come after.
    #[serde(deserialize_with = "super::present::required")]
    pub first: EvidenceKindId,
    /// The record that is to come after it: its kind and its result.
    #[serde(deserialize_with = "super::present::required")]
    pub then: LaterRecord,
    /// The declared artifact the records of both kinds are about; without it, records about any
    /// artifact.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub subject: Option<ArtifactId>,
}

/// An order form: whether a record of `then.kind` with result `then.result`, about the subject and
/// subject revision of the latest record of kind `last`, was observed after that record, all about
/// `subject` when it names one. It reads every record of the two kinds the evidence check
/// accepted, excluded or not. `last` and `then.kind` may be one kind: a record is not later than
/// itself.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSince {
    /// The evidence kind whose latest record the later one is to come after, at its revision.
    #[serde(deserialize_with = "super::present::required")]
    pub last: EvidenceKindId,
    /// The record that is to come after it: its kind and its result.
    #[serde(deserialize_with = "super::present::required")]
    pub then: LaterRecord,
    /// The declared artifact the records of both kinds are about; without it, records about any
    /// artifact.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub subject: Option<ArtifactId>,
}

/// An order form over revisions (story:revision-ordering): whether an item that had `first.result`
/// in a record of `first.kind`, on a revision the case snapshot lists in its subject's
/// `descends_from`, has `then.result` in a record of `then.kind` about the current revision of
/// that subject, all about `subject` when it names one. It reads every record of the two kinds the
/// evidence check accepted, excluded or not, and pairs records only on equal `item`. `first` and
/// `then` may name one kind.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceDescended {
    /// The earlier record: its kind and its result.
    #[serde(deserialize_with = "super::present::required")]
    pub first: LaterRecord,
    /// The later record, about the current revision: its kind and its result.
    #[serde(deserialize_with = "super::present::required")]
    pub then: LaterRecord,
    /// The declared artifact the records of both kinds are about; without it, records about any
    /// artifact.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub subject: Option<ArtifactId>,
}

/// What an `after` form reads later than the first record of its `first` kind, and a `since` form
/// later than the latest record of its `last` kind: a record of `kind` with `result`. A
/// `descended` form names each of the two records it pairs by one.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaterRecord {
    /// The evidence kind of the later record.
    #[serde(deserialize_with = "super::present::required")]
    pub kind: EvidenceKindId,
    /// The result the later record is to have.
    #[serde(deserialize_with = "super::present::required")]
    pub result: String,
}

/// Matches evidence of one kind, optionally with one result, optionally only evidence about one
/// declared artifact (a match that names a subject reads only the records whose subject it is),
/// and optionally only evidence whose producer the case lists as independent on every dimension
/// `independent_on` names.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceMatch {
    #[serde(deserialize_with = "super::present::required")]
    pub kind: EvidenceKindId,
    #[serde(default, deserialize_with = "super::present::optional")]
    pub result: Option<String>,
    #[serde(default, deserialize_with = "super::present::optional")]
    pub subject: Option<ArtifactId>,
    /// The independence the match demands of a record's producer: declared dimensions, not empty,
    /// each once. A match without it reads records whoever produced them.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub independent_on: Option<Vec<IndependenceDimensionId>>,
}

/// The evidence kind whose records carry a child case's outcome (story:case-composition, Atlas ADR
/// 0081): one record per child case, its `child_case` the child's id and its `result` the child's
/// outcome. A protocol that writes a [`ChildOutcomes`] form declares it; a record of it names its
/// child and its outcome.
pub const CASE_OUTCOME: &str = "case_outcome";

/// The outcome a `case_outcome` record gives a child case that is still running.
pub const OPEN_OUTCOME: &str = "open";

/// The form over the case's child cases, written `{children: {in: [<outcome>, ...]}}`: whether
/// every child case ended with one of the outcomes listed. It reads every `case_outcome` record
/// the evidence check accepted, as the order forms read the history, or with `revision: current`
/// only those about the current revision of their subject, and with `subject` only those about
/// that artifact: `unknown` with no record read, `false` when a record's outcome is neither listed
/// nor `open` (a child that ended outside the set), otherwise `unknown` when a child is still
/// `open` and `true` when every child ended with a listed outcome. It counts only the records it
/// is given. Canon does not write them; the runner that opens a child case does, with two duties:
/// a record's `subject` is the `case_open` subject of the action that opened the child, and the
/// runner writes a record with the outcome `open` when it opens the child.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChildOutcomes {
    /// The outcomes a child case may have ended with, written `in`: not empty, each once, never
    /// `open`.
    #[serde(deserialize_with = "super::present::required")]
    pub r#in: Vec<String>,
    /// Which records the form reads (story:child-outcome-revision): absent or `any`, every
    /// `case_outcome` record; `current`, only those whose `subject_revision` is the case
    /// snapshot's current revision of their `subject`.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub revision: Option<ChildRevision>,
    /// The declared artifact the records are about (story:child-outcome-subject): absent, records
    /// about every artifact; named, only the `case_outcome` records whose `subject` it is. A record
    /// about another artifact is not read: it neither establishes nor contradicts the form.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub subject: Option<ArtifactId>,
}

/// Which `case_outcome` records a `children` form reads (story:child-outcome-revision). A terminal
/// record's `subject_revision` is the revision the child's outcome was computed against: when the
/// child protocol declares the subject artifact, the child's own revision of it at its terminating
/// evaluation; when it does not, the parent's revision of the subject when the child was opened.
/// Canon does not write the record: it compares that revision with the case snapshot's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChildRevision {
    /// Every record the evidence check accepted, about whatever revision of its subject: the form
    /// without the key.
    Any,
    /// Only the records whose `subject_revision` is the case snapshot's current revision of their
    /// `subject`. A record about another revision is not read: it neither establishes nor
    /// contradicts the form.
    Current,
}

impl ChildRevision {
    /// The value as `protocol/1` and `canon-ir/1` write it.
    pub fn as_str(self) -> &'static str {
        match self {
            ChildRevision::Any => "any",
            ChildRevision::Current => "current",
        }
    }
}

impl std::str::FromStr for ChildRevision {
    type Err = String;

    /// Reads the value as `canon-ir/1` writes it.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "any" => Ok(ChildRevision::Any),
            "current" => Ok(ChildRevision::Current),
            other => Err(format!(
                "a children form's revision is `any` or `current`, found `{other}`"
            )),
        }
    }
}

impl ChildOutcomes {
    /// Whether the form reads only the records about the current revision of their subject
    /// (`revision: current`).
    pub fn current_only(&self) -> bool {
        self.revision == Some(ChildRevision::Current)
    }
}

/// Tests whether a claim has a given value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimTest {
    pub claim: ClaimId,
    pub is: Truth,
}

/// Tests whether a case input has one of the values listed. A case gives every declared input a
/// value, so the test is never unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputTest {
    pub input: InputId,
    /// The values the test is true for, written `in`.
    pub r#in: Vec<InputValue>,
}

impl InputTest {
    /// The test's value for the case's value of its input: whether it is one of those listed.
    pub fn holds_for(&self, value: &InputValue) -> bool {
        self.r#in.contains(value)
    }
}

impl Predicate {
    /// Every claim this predicate references, in source order.
    pub fn claim_references(&self) -> Vec<&ClaimId> {
        let mut found = Vec::new();
        self.visit(&mut |predicate| {
            if let Predicate::Claim(test) = predicate {
                found.push(&test.claim);
            }
        });
        found
    }

    /// Calls `f` on this predicate and every predicate inside it, in source order.
    pub fn visit<'a>(&'a self, f: &mut impl FnMut(&'a Predicate)) {
        f(self);
        match self {
            Predicate::All(members) | Predicate::Any(members) => {
                for member in members {
                    member.visit(f);
                }
            }
            Predicate::Not(inner) => inner.visit(f),
            Predicate::Evidence(_)
            | Predicate::Claim(_)
            | Predicate::Input(_)
            | Predicate::First(_)
            | Predicate::Before(_)
            | Predicate::After(_)
            | Predicate::Since(_)
            | Predicate::Descended(_)
            | Predicate::Children(_) => {}
        }
    }

    /// Every input test inside this predicate, in source order.
    pub fn input_tests(&self) -> Vec<&InputTest> {
        let mut found = Vec::new();
        self.visit(&mut |predicate| {
            if let Predicate::Input(test) = predicate {
                found.push(test);
            }
        });
        found
    }

    /// Every form over the child cases (`children`) inside this predicate, in source order.
    pub fn children_forms(&self) -> Vec<&ChildOutcomes> {
        let mut found = Vec::new();
        self.visit(&mut |predicate| {
            if let Predicate::Children(form) = predicate {
                found.push(form);
            }
        });
        found
    }

    /// Every order form (`first`, `before`, `after`, `since` and `descended`) inside this
    /// predicate, in source order.
    pub fn order_forms(&self) -> Vec<&Predicate> {
        let mut found = Vec::new();
        self.visit(&mut |predicate| {
            if matches!(
                predicate,
                Predicate::First(_)
                    | Predicate::Before(_)
                    | Predicate::After(_)
                    | Predicate::Since(_)
                    | Predicate::Descended(_)
            ) {
                found.push(predicate);
            }
        });
        found
    }

    /// The evidence an order form reads, as `(kind, subject)` pairs: `first`'s kind, `before`'s two
    /// kinds, `first` then `then`, `after`'s two kinds, `first` then `then.kind`, or `since`'s two
    /// kinds, `last` then `then.kind`, or `descended`'s two kinds, `first.kind` then `then.kind`,
    /// each with the form's subject. Empty for any other form.
    pub fn order_reads(&self) -> Vec<(&EvidenceKindId, Option<&ArtifactId>)> {
        match self {
            Predicate::First(form) => vec![(&form.kind, form.subject.as_ref())],
            Predicate::Before(form) => vec![
                (&form.first, form.subject.as_ref()),
                (&form.then, form.subject.as_ref()),
            ],
            Predicate::After(form) => vec![
                (&form.first, form.subject.as_ref()),
                (&form.then.kind, form.subject.as_ref()),
            ],
            Predicate::Since(form) => vec![
                (&form.last, form.subject.as_ref()),
                (&form.then.kind, form.subject.as_ref()),
            ],
            Predicate::Descended(form) => vec![
                (&form.first.kind, form.subject.as_ref()),
                (&form.then.kind, form.subject.as_ref()),
            ],
            _ => Vec::new(),
        }
    }
}

/// A predicate as written: each form a key, at most one of them present. `requirement.rs` reads an
/// outcome's requirement through it too.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WrittenPredicate {
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) all: Option<Vec<Predicate>>,
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) any: Option<Vec<Predicate>>,
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) not: Option<Box<Predicate>>,
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) evidence: Option<EvidenceMatch>,
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) claim: Option<ClaimId>,
    #[serde(default, deserialize_with = "super::present::optional")]
    pub(crate) is: Option<Truth>,
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) input: Option<InputId>,
    // Not optional when its `input` is written: a null on it says it needs a value.
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) r#in: Option<Vec<InputValue>>,
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) first: Option<FirstResult>,
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) before: Option<EvidenceOrder>,
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) after: Option<EvidenceAfter>,
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) since: Option<EvidenceSince>,
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) descended: Option<EvidenceDescended>,
    #[serde(default, deserialize_with = "super::present::form")]
    pub(crate) children: Option<ChildOutcomes>,
}

impl TryFrom<WrittenPredicate> for Predicate {
    type Error = String;

    fn try_from(written: WrittenPredicate) -> Result<Self, Self::Error> {
        let WrittenPredicate {
            all,
            any,
            not,
            evidence,
            claim,
            is,
            input,
            r#in: values,
            first,
            before,
            after,
            since,
            descended,
            children,
        } = written;
        let forms = [
            all.is_some(),
            any.is_some(),
            not.is_some(),
            evidence.is_some(),
            claim.is_some(),
            input.is_some(),
            first.is_some(),
            before.is_some(),
            after.is_some(),
            since.is_some(),
            descended.is_some(),
            children.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        if forms != 1 {
            return Err(format!(
                "a predicate has exactly one of `all`, `any`, `not`, `evidence`, `claim`, `input`, `first`, `before`, `after`, `since`, `descended` or `children`, found {forms}"
            ));
        }
        if is.is_some() && claim.is_none() {
            return Err("`is` is allowed only beside `claim`".to_owned());
        }
        if values.is_some() && input.is_none() {
            return Err("`in` is allowed only beside `input`".to_owned());
        }
        if let Some(input) = input {
            let Some(values) = values else {
                return Err("an `input` test lists the values it holds for in `in`".to_owned());
            };
            return Ok(Predicate::Input(InputTest {
                input,
                r#in: values,
            }));
        }
        Ok(if let Some(members) = all {
            Predicate::All(members)
        } else if let Some(members) = any {
            Predicate::Any(members)
        } else if let Some(inner) = not {
            Predicate::Not(inner)
        } else if let Some(matching) = evidence {
            Predicate::Evidence(matching)
        } else if let Some(form) = first {
            Predicate::First(form)
        } else if let Some(form) = before {
            Predicate::Before(form)
        } else if let Some(form) = after {
            Predicate::After(form)
        } else if let Some(form) = since {
            Predicate::Since(form)
        } else if let Some(form) = descended {
            Predicate::Descended(form)
        } else if let Some(form) = children {
            Predicate::Children(form)
        } else {
            let claim = claim.expect("exactly one form is present");
            Predicate::Claim(ClaimTest {
                claim,
                is: is.unwrap_or(Truth::True),
            })
        })
    }
}

//! The predicate language: a small, total, deterministic expression language (design § 39.2).
//!
//! A predicate is one of `all`, `any`, `not`, an evidence match by kind, optional result and
//! optional subject, a test on a claim value, a test on a case input's value, or one of the three
//! order forms over the evidence history. Written in source as a map with exactly one of the keys
//! `all`, `any`, `not`, `evidence`, `claim`, `input`, `first`, `before` or `after`; a claim test
//! may add `is`, which defaults to `true`, and an input test adds `in`, the values it is true for
//! (`{input: severity, in: [high]}`).
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
//! record of `K2` with another result takes no part. `K1` and `K2` may be one kind. A known limit:
//! a record bound to any earlier revision counts, whether or not the current revision descends
//! from it, since Canon does not know how revisions descend (story:revision-ordering).
//!
//! An evidence match may demand independence: `{evidence: {kind: K, result: R, independent_on:
//! [D, ...]}}` reads a record only when it names a `producer` and the case's `independence` map
//! lists that producer on every dimension `D` (dimensions the protocol declares under
//! `independence_dimensions`). A record of the match's kind and subject that fails the demand is
//! not of the match, whatever its result: it neither establishes nor contradicts it, and is
//! excluded from it as `not_independent` (CANON-INDEPENDENCE-001). The order forms do not demand
//! independence.

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

/// What an `after` form reads later than the first record of its `first` kind: a record of `kind`
/// with `result`.
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
            | Predicate::After(_) => {}
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

    /// Every order form (`first`, `before` and `after`) inside this predicate, in source order.
    pub fn order_forms(&self) -> Vec<&Predicate> {
        let mut found = Vec::new();
        self.visit(&mut |predicate| {
            if matches!(
                predicate,
                Predicate::First(_) | Predicate::Before(_) | Predicate::After(_)
            ) {
                found.push(predicate);
            }
        });
        found
    }

    /// The evidence an order form reads, as `(kind, subject)` pairs: `first`'s kind, `before`'s two
    /// kinds, `first` then `then`, or `after`'s two kinds, `first` then `then.kind`, each with the
    /// form's subject. Empty for any other form.
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
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        if forms != 1 {
            return Err(format!(
                "a predicate has exactly one of `all`, `any`, `not`, `evidence`, `claim`, `input`, `first`, `before` or `after`, found {forms}"
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
        } else {
            let claim = claim.expect("exactly one form is present");
            Predicate::Claim(ClaimTest {
                claim,
                is: is.unwrap_or(Truth::True),
            })
        })
    }
}

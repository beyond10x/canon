//! The predicate language: a small, total, deterministic expression language (design § 39.2).
//!
//! A predicate is one of `all`, `any`, `not`, an evidence match by kind, optional result and
//! optional subject, or a test on a claim value. Written in source as a map with exactly one of the
//! keys `all`, `any`, `not`, `evidence` or `claim`; a claim test may add `is`, which defaults to
//! `true`.

use std::fmt;

use serde::Deserialize;

use super::ids::{ArtifactId, ClaimId, EvidenceKindId};

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
    Claim(ClaimTest),
}

/// Matches evidence of one kind, optionally with one result, and optionally only evidence about one
/// declared artifact: a match that names a subject reads only the records whose subject it is.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceMatch {
    #[serde(deserialize_with = "super::present::required")]
    pub kind: EvidenceKindId,
    #[serde(default, deserialize_with = "super::present::optional")]
    pub result: Option<String>,
    #[serde(default, deserialize_with = "super::present::optional")]
    pub subject: Option<ArtifactId>,
}

/// Tests whether a claim has a given value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimTest {
    pub claim: ClaimId,
    pub is: Truth,
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
            Predicate::Evidence(_) | Predicate::Claim(_) => {}
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
        } = written;
        let forms = [
            all.is_some(),
            any.is_some(),
            not.is_some(),
            evidence.is_some(),
            claim.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        if forms != 1 {
            return Err(format!(
                "a predicate has exactly one of `all`, `any`, `not`, `evidence` or `claim`, found {forms}"
            ));
        }
        if is.is_some() && claim.is_none() {
            return Err("`is` is allowed only beside `claim`".to_owned());
        }
        Ok(if let Some(members) = all {
            Predicate::All(members)
        } else if let Some(members) = any {
            Predicate::Any(members)
        } else if let Some(inner) = not {
            Predicate::Not(inner)
        } else if let Some(matching) = evidence {
            Predicate::Evidence(matching)
        } else {
            let claim = claim.expect("exactly one form is present");
            Predicate::Claim(ClaimTest {
                claim,
                is: is.unwrap_or(Truth::True),
            })
        })
    }
}

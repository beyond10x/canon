//! The invalidation exclusion stage (design § 4.1, CANON-INVALIDATION-001): evidence observed
//! against an earlier revision of an upstream artifact no longer supports the claims an
//! invalidation rule names.
//!
//! A rule names an upstream artifact and the claims a change of that artifact's revision
//! invalidates. A record that records a revision of the upstream artifact (its
//! `upstream_revisions`) other than the case snapshot's current revision of it is invalidated by
//! the rule. Exclusion is per claim, unlike the other two stages: the record is kept from each
//! claim the rule names and from every claim built on one of them, through any number of claim
//! references (a record excluded from a claim is excluded from every claim built on it), and from
//! no other claim. Such a claim is evaluated without the record throughout, including in the
//! claims it tests (`claims.rs`). A claim the rule does not name, and that is not built on one it
//! names, reads the record as before, even when it reads the same kind. A record that records no
//! revision of the upstream artifact is not invalidated by the rule.

use std::collections::{BTreeMap, BTreeSet};

use super::Refusal;
use super::claims::reads;
use crate::ir::Ir;
use crate::model::{Case, ClaimId, EvidenceId, EvidenceMatch, EvidenceRecord, Predicate, one_line};

/// What this stage kept from which claim: for each claim, the records invalidated for it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Invalidated {
    by_claim: BTreeMap<ClaimId, BTreeSet<EvidenceId>>,
}

impl Invalidated {
    /// The records invalidated for `claim`; none when the stage kept nothing from it.
    pub(crate) fn for_claim(&self, claim: &ClaimId) -> Option<&BTreeSet<EvidenceId>> {
        self.by_claim.get(claim)
    }

    /// Whether `record` is invalidated for `claim`.
    pub(crate) fn excludes(&self, claim: &ClaimId, record: &EvidenceId) -> bool {
        self.for_claim(claim)
            .is_some_and(|records| records.contains(record))
    }

    /// Whether `record` is listed as `invalidated` under `claim`: invalidated for it, and read by
    /// an evidence match the claim reaches, in its own predicate or in a claim it tests through any
    /// number of claim references. Every such match is evaluated in the claim's context, so none of
    /// them reads the record there.
    pub(crate) fn lists(&self, ir: &Ir, claim: &ClaimId, record: &EvidenceRecord) -> bool {
        self.excludes(claim, &record.id)
            && reached_matches(ir, claim)
                .iter()
                .any(|matching| reads(&matching.kind, matching.subject.as_ref(), record))
    }
}

/// Every evidence match `claim` reaches: those of its own predicate and of every claim it tests,
/// through any number of claim references, each claim visited once.
fn reached_matches<'a>(ir: &'a Ir, claim: &ClaimId) -> Vec<&'a EvidenceMatch> {
    let mut matches = Vec::new();
    let mut visited = BTreeSet::new();
    let mut pending = vec![claim.clone()];
    while let Some(next) = pending.pop() {
        if !visited.insert(next.clone()) {
            continue;
        }
        let Some(declared) = ir.claims.get(&next) else {
            continue;
        };
        declared.true_when.visit(&mut |node| match node {
            Predicate::Evidence(matching) => matches.push(matching),
            Predicate::Claim(test) => pending.push(test.claim.clone()),
            _ => {}
        });
    }
    matches
}

/// The records of `evidence` this stage keeps from each claim, with the reason `invalidated`.
/// `evidence` is what earlier stages left. A rule whose upstream artifact the case snapshot does
/// not name, which only an IR a caller builds can hold, is refused as `undeclared-artifact`.
pub(super) fn exclude(
    ir: &Ir,
    case: &Case,
    evidence: &[&EvidenceRecord],
) -> Result<Invalidated, Refusal> {
    let mut invalidated = Invalidated::default();
    for (id, rule) in &ir.invalidation {
        let Some(current) = case.artifacts.get(&rule.upstream) else {
            return Err(Refusal::new(
                "undeclared-artifact",
                format!(
                    "invalidation rule `{}` watches artifact `{}`, which the protocol does not declare",
                    one_line(id.as_str()),
                    one_line(rule.upstream.as_str())
                ),
            ));
        };
        let moved: Vec<&EvidenceId> = evidence
            .iter()
            .filter(|record| {
                record
                    .upstream_revisions
                    .get(&rule.upstream)
                    .is_some_and(|recorded| *recorded != current.revision)
            })
            .map(|record| &record.id)
            .collect();
        if moved.is_empty() {
            continue;
        }
        for claim in invalidated_claims(ir, &rule.invalidates) {
            invalidated
                .by_claim
                .entry(claim)
                .or_default()
                .extend(moved.iter().map(|id| (*id).clone()));
        }
    }
    Ok(invalidated)
}

/// The claims a rule naming `named` keeps an invalidated record from: `named`, and every declared
/// claim built on one of them through any number of claim references. `canon check` reads it too,
/// to know which evidence dimensions a rule can move.
pub(crate) fn invalidated_claims(ir: &Ir, named: &[ClaimId]) -> BTreeSet<ClaimId> {
    let mut found: BTreeSet<ClaimId> = named.iter().cloned().collect();
    loop {
        let more: Vec<ClaimId> = ir
            .claims
            .iter()
            .filter(|(id, claim)| {
                !found.contains(*id)
                    && claim
                        .true_when
                        .claim_references()
                        .into_iter()
                        .any(|tested| found.contains(tested))
            })
            .map(|(id, _)| id.clone())
            .collect();
        if more.is_empty() {
            return found;
        }
        found.extend(more);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ir() -> Ir {
        crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {}}\n\
                 claims: {named: {true_when: {evidence: {kind: k}}}, \
                 on: {true_when: {claim: named}}, \
                 on_on: {true_when: {not: {claim: on}}}, \
                 other: {true_when: {evidence: {kind: k}}}}\n\
                 invalidation: {r: {upstream: up, invalidates: [named]}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles")
    }

    fn case(up: &str) -> Case {
        super::super::read_case(&format!(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {{a: {{revision: r1}}, up: {{revision: {up}}}}}\n"
        ))
        .expect("reads")
    }

    fn record(id: &str, upstream: Option<&str>) -> EvidenceRecord {
        let upstream = upstream.map_or(String::new(), |revision| {
            format!("upstream_revisions: {{up: {revision}}}\n")
        });
        super::super::read_evidence(&format!(
            "format: canon-evidence/1\nid: {id}\nkind: k\nsubject: a\nsubject_revision: r1\n{upstream}"
        ))
        .expect("reads")
    }

    fn ids(set: Option<&BTreeSet<EvidenceId>>) -> Vec<&str> {
        set.into_iter().flatten().map(EvidenceId::as_str).collect()
    }

    /// A moved upstream revision invalidates the record for the named claim and every claim built
    /// on it, at any depth, and for no other claim; an equal one, or none recorded, invalidates
    /// nothing.
    #[test]
    fn exclusion_is_per_claim_and_follows_claims_built_on_a_named_one() {
        let records = [
            record("moved", Some("u1")),
            record("same", Some("u2")),
            record("none", None),
        ];
        let given: Vec<&EvidenceRecord> = records.iter().collect();
        let invalidated = exclude(&ir(), &case("u2"), &given).expect("evaluates");
        for claim in ["named", "on", "on_on"] {
            assert_eq!(
                ids(invalidated.for_claim(&ClaimId::new(claim))),
                ["moved"],
                "{claim}"
            );
        }
        assert_eq!(
            ids(invalidated.for_claim(&ClaimId::new("other"))),
            Vec::<&str>::new()
        );

        let unmoved: Vec<&EvidenceRecord> = records[1..].iter().collect();
        let nothing = exclude(&ir(), &case("u2"), &unmoved).expect("evaluates");
        assert_eq!(nothing, Invalidated::default());
    }
}

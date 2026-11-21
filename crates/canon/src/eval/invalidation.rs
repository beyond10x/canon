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
use crate::model::{
    ArtifactId, Case, ClaimId, EvidenceId, EvidenceKindId, EvidenceMatch, EvidenceRecord,
    Predicate, Revision, one_line,
};

/// What this stage kept from which claim: for each claim, the records invalidated for it.
///
/// A record invalidated for a claim is invalidated for every claim built on it
/// ([`invalidated_claims`]), so a claim tested from another claim never has a record invalidated
/// that the testing claim has not: claim evaluation relies on it (`claims.rs`).
///
/// Each record a rule moved is held once, in evidence-id order, and a claim's records are their
/// places in that order: a chain of `n` claims each built on the next, each with a rule of its own,
/// keeps `n (n + 1) / 2` places and `n` records, not `n (n + 1) / 2` copies of an identifier.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Invalidated {
    /// Every record a rule moved, in evidence-id order.
    moved: Vec<EvidenceRecord>,
    /// For each claim the stage kept a record from, the places in `moved` of the records
    /// invalidated for it, ascending, so in evidence-id order.
    by_claim: BTreeMap<ClaimId, Vec<usize>>,
}

impl Invalidated {
    /// Every record a rule moved, in evidence-id order: [`Self::kept`] names each by its place here.
    pub(crate) fn moved(&self) -> &[EvidenceRecord] {
        &self.moved
    }

    /// The places in [`Self::moved`] of the records invalidated for `claim`, ascending; none when
    /// the stage kept nothing from it.
    pub(crate) fn kept(&self, claim: &ClaimId) -> &[usize] {
        self.by_claim.get(claim).map_or(&[], Vec::as_slice)
    }

    /// The place in [`Self::moved`] of the record `id`, when a rule moved it.
    pub(crate) fn place(&self, id: &EvidenceId) -> Option<usize> {
        self.moved.binary_search_by(|record| record.id.cmp(id)).ok()
    }

    /// Whether `record` is invalidated for `claim`.
    pub(crate) fn excludes(&self, claim: &ClaimId, record: &EvidenceId) -> bool {
        self.place(record)
            .is_some_and(|place| self.kept(claim).binary_search(&place).is_ok())
    }

    /// Whether `record` is listed as `invalidated` under `claim`: invalidated for it, and read by
    /// one of `reached`, the evidence matches the claim reaches by kind and subject, in its own
    /// predicate or in a claim it tests through any number of claim references
    /// ([`reached_by_claim`]). Every such match is evaluated in the claim's context, so none of them
    /// reads the record there. The caller finds the matches once per claim, not once per record.
    pub(crate) fn lists<'m>(
        &self,
        reached: impl IntoIterator<Item = (&'m EvidenceKindId, Option<&'m ArtifactId>)>,
        claim: &ClaimId,
        record: &EvidenceRecord,
    ) -> bool {
        self.excludes(claim, &record.id) && read_by(reached, record)
    }

    /// The records listed as `invalidated` under `claim` ([`Self::lists`]), in evidence-id order:
    /// each record invalidated for it that one of `reached` reads.
    pub(crate) fn listed<'s, 'm>(
        &'s self,
        reached: impl IntoIterator<Item = (&'m EvidenceKindId, Option<&'m ArtifactId>)> + Clone,
        claim: &ClaimId,
    ) -> impl Iterator<Item = &'s EvidenceId> {
        self.kept(claim)
            .iter()
            .map(|place| &self.moved[*place])
            .filter(move |record| read_by(reached.clone(), record))
            .map(|record| &record.id)
    }
}

/// Whether one of `matches`, by kind and subject, reads `record`.
fn read_by<'m>(
    matches: impl IntoIterator<Item = (&'m EvidenceKindId, Option<&'m ArtifactId>)>,
    record: &EvidenceRecord,
) -> bool {
    matches
        .into_iter()
        .any(|(kind, subject)| reads(kind, subject, record))
}

/// Every evidence match each declared claim reaches, by claim: those of its own predicate and of
/// every claim it tests, through any number of claim references. Found once for every claim, after
/// the claims it tests, each match once, so a claim reached from many others is walked once, not
/// once for each. In no particular order: a reader asks whether any of them does something. The
/// walk runs only after claim evaluation has refused a cycle; were there one, a claim on it would
/// not have the matches of the claim that closes it.
pub(crate) fn reached_by_claim(ir: &Ir) -> BTreeMap<&ClaimId, Vec<&EvidenceMatch>> {
    let ids: Vec<&ClaimId> = ir.claims.keys().collect();
    let at: BTreeMap<&ClaimId, usize> = ids
        .iter()
        .enumerate()
        .map(|(number, id)| (*id, number))
        .collect();
    // Each claim's own matches and the declared claims it tests, by number.
    let mut own: Vec<Vec<&EvidenceMatch>> = Vec::with_capacity(ids.len());
    let mut tests: Vec<Vec<usize>> = Vec::with_capacity(ids.len());
    for claim in ir.claims.values() {
        let (mut matches, mut tested) = (Vec::new(), Vec::new());
        claim.true_when.visit(&mut |node| match node {
            Predicate::Evidence(matching) => matches.push(matching),
            Predicate::Claim(test) => tested.extend(at.get(&test.claim).copied()),
            _ => {}
        });
        own.push(matches);
        tests.push(tested);
    }
    // After every claim a claim tests, each claim's matches: its own and theirs, each once.
    let mut reached: Vec<Option<Vec<&EvidenceMatch>>> = vec![None; ids.len()];
    let mut entered = vec![false; ids.len()];
    for root in 0..ids.len() {
        let mut pending = vec![root];
        while let Some(&claim) = pending.last() {
            if reached[claim].is_some() {
                pending.pop();
                continue;
            }
            if !entered[claim] {
                entered[claim] = true;
                pending.extend(tests[claim].iter().filter(|tested| !entered[**tested]));
                continue;
            }
            pending.pop();
            let mut matches = own[claim].clone();
            for tested in &tests[claim] {
                matches.extend(reached[*tested].iter().flatten());
            }
            matches.sort_unstable_by_key(|matching| std::ptr::from_ref(*matching));
            matches.dedup_by_key(|matching| std::ptr::from_ref(*matching));
            reached[claim] = Some(matches);
        }
    }
    ids.into_iter()
        .zip(reached.into_iter().map(Option::unwrap_or_default))
        .collect()
}

/// The records of `evidence` this stage keeps from each claim, with the reason `invalidated`.
/// `evidence` is what earlier stages left. A rule whose upstream artifact the case snapshot does
/// not name, which only an IR a caller builds can hold, is refused as `undeclared-artifact`.
pub(super) fn exclude(
    ir: &Ir,
    case: &Case,
    evidence: &[&EvidenceRecord],
) -> Result<Invalidated, Refusal> {
    if ir.invalidation.is_empty() {
        return Ok(Invalidated::default());
    }
    // Each lookup a rule makes, indexed once, so the stage costs no more per rule than the claims
    // and records that rule reaches: the case's revision of each artifact (the first declaration,
    // as `Declarations::get` finds it), and each record's revision of each upstream artifact
    // (likewise), with the record's place in `evidence`, in the order of `evidence`.
    let mut revisions: BTreeMap<&ArtifactId, &Revision> = BTreeMap::new();
    for (artifact, declared) in case.artifacts.iter() {
        revisions.entry(artifact).or_insert(&declared.revision);
    }
    let mut recorded: BTreeMap<&ArtifactId, Vec<(&Revision, usize)>> = BTreeMap::new();
    for (at, record) in evidence.iter().enumerate() {
        for (artifact, revision) in record.upstream_revisions.iter() {
            let first = record.upstream_revisions.get(artifact);
            if first.is_some_and(|first| std::ptr::eq(first, revision)) {
                recorded.entry(artifact).or_default().push((revision, at));
            }
        }
    }
    // The claims each rule names, with the places in `evidence` of the records it moved.
    let mut moving: Vec<(&[ClaimId], Vec<usize>)> = Vec::new();
    for (id, rule) in &ir.invalidation {
        let Some(current) = revisions.get(&rule.upstream) else {
            return Err(Refusal::new(
                "undeclared-artifact",
                format!(
                    "invalidation rule `{}` watches artifact `{}`, which the protocol does not declare",
                    one_line(id.as_str()),
                    one_line(rule.upstream.as_str())
                ),
            ));
        };
        let moved: Vec<usize> = recorded
            .get(&rule.upstream)
            .into_iter()
            .flatten()
            .filter(|(revision, _)| *revision != *current)
            .map(|(_, at)| *at)
            .collect();
        if !moved.is_empty() {
            moving.push((&rule.invalidates, moved));
        }
    }
    if moving.is_empty() {
        return Ok(Invalidated::default());
    }

    // Every moved record once, in evidence-id order (the evidence check has refused a repeated
    // id), and the place there of each record of `evidence` a rule moved.
    let mut order: Vec<usize> = moving
        .iter()
        .flat_map(|(_, moved)| moved.iter().copied())
        .collect();
    order.sort_unstable();
    order.dedup();
    order.sort_by(|one, other| evidence[*one].id.cmp(&evidence[*other].id));
    let mut place_of = vec![0; evidence.len()];
    for (place, at) in order.iter().enumerate() {
        place_of[*at] = place;
    }

    // Each rule's records go to the claims it names and every claim built on them, each claim
    // found once per rule; `found` holds, for each claim, the last rule that found it.
    let claims = Index::of(ir);
    let mut kept: Vec<Vec<usize>> = vec![Vec::new(); claims.ids.len()];
    let mut found: Vec<usize> = vec![0; claims.ids.len()];
    let mut pending: Vec<usize> = Vec::new();
    for (rule, (named, moved)) in moving.iter().enumerate() {
        let rule = rule + 1;
        let places: Vec<usize> = moved.iter().map(|at| place_of[*at]).collect();
        pending.extend(
            named
                .iter()
                .filter_map(|claim| claims.at.get(claim).copied()),
        );
        while let Some(claim) = pending.pop() {
            if found[claim] == rule {
                continue;
            }
            found[claim] = rule;
            kept[claim].extend_from_slice(&places);
            pending.extend(claims.testers[claim].iter().copied());
        }
    }
    let by_claim = kept
        .into_iter()
        .enumerate()
        .filter(|(_, places)| !places.is_empty())
        .map(|(claim, mut places)| {
            if !places.is_sorted() {
                places.sort_unstable();
            }
            places.dedup();
            (claims.ids[claim].clone(), places)
        })
        .collect();
    Ok(Invalidated {
        moved: order.iter().map(|at| evidence[*at].clone()).collect(),
        by_claim,
    })
}

/// The claims a rule naming `named` keeps an invalidated record from: `named`, and every declared
/// claim built on one of them through any number of claim references. `canon check` reads it too,
/// to know which evidence dimensions a rule can move. Costs time linear in the size of the
/// protocol's claims; [`exclude`] indexes the claims once for all its rules instead.
pub(crate) fn invalidated_claims(ir: &Ir, named: &[ClaimId]) -> BTreeSet<ClaimId> {
    let claims = Index::of(ir);
    let mut found: BTreeSet<ClaimId> = named.iter().cloned().collect();
    let mut pending: Vec<usize> = named
        .iter()
        .filter_map(|claim| claims.at.get(claim).copied())
        .collect();
    while let Some(claim) = pending.pop() {
        for tester in &claims.testers[claim] {
            if found.insert(claims.ids[*tester].clone()) {
                pending.push(*tester);
            }
        }
    }
    found
}

/// Every claim by number: each claim the protocol declares, in claim-id order, then each claim a
/// predicate tests or a rule names that it does not declare, which only an IR a caller builds can
/// hold. For each claim, the declared claims that test it (`Predicate::claim_references`), so
/// following them looks nothing up.
struct Index<'a> {
    ids: Vec<&'a ClaimId>,
    at: BTreeMap<&'a ClaimId, usize>,
    testers: Vec<Vec<usize>>,
}

impl<'a> Index<'a> {
    fn of(ir: &'a Ir) -> Self {
        let mut index = Self {
            ids: Vec::new(),
            at: BTreeMap::new(),
            testers: Vec::new(),
        };
        for id in ir.claims.keys() {
            index.number(id);
        }
        for (tester, claim) in ir.claims.values().enumerate() {
            for tested in claim.true_when.claim_references() {
                let tested = index.number(tested);
                index.testers[tested].push(tester);
            }
        }
        for rule in ir.invalidation.values() {
            for named in &rule.invalidates {
                index.number(named);
            }
        }
        index
    }

    /// The number of `id`, given it now when it has none.
    fn number(&mut self, id: &'a ClaimId) -> usize {
        let next = self.ids.len();
        let number = *self.at.entry(id).or_insert(next);
        if number == next {
            self.ids.push(id);
            self.testers.push(Vec::new());
        }
        number
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

    impl Invalidated {
        /// The records invalidated for `claim`, by id.
        fn for_claim(&self, claim: &ClaimId) -> BTreeSet<&EvidenceId> {
            self.kept(claim)
                .iter()
                .map(|place| &self.moved[*place].id)
                .collect()
        }
    }

    fn ids(set: BTreeSet<&EvidenceId>) -> Vec<&str> {
        set.into_iter().map(EvidenceId::as_str).collect()
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

    /// What claim evaluation relies on (`claims.rs`): a claim tested by another never has a record
    /// invalidated that the testing claim has not, whichever rules moved what.
    #[test]
    fn a_claim_testing_another_has_every_record_invalidated_for_it() {
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}, up: {}, down: {}}\nevidence_kinds: {k: {}}\n\
                 claims: {x: {true_when: {evidence: {kind: k}}}, \
                 w: {true_when: {evidence: {kind: k}}}, \
                 y: {true_when: {all: [{claim: x}, {evidence: {kind: k}}]}}, \
                 z: {true_when: {any: [{claim: y}, {not: {claim: w}}]}}}\n\
                 invalidation: {r1: {upstream: up, invalidates: [x]}, \
                 r2: {upstream: down, invalidates: [w, y]}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let case = super::super::read_case(
            "format: canon-case/1\nid: C-1\nprotocol: p\n\
             artifacts: {a: {revision: r1}, up: {revision: u2}, down: {revision: d2}}\n",
        )
        .expect("reads");
        let moved = |id: &str, upstream: &str| {
            super::super::read_evidence(&format!(
                "format: canon-evidence/1\nid: {id}\nkind: k\nsubject: a\nsubject_revision: r1\n\
                 upstream_revisions: {{{upstream}}}\n"
            ))
            .expect("reads")
        };
        let records = [
            moved("by_up", "up: u1"),
            moved("by_down", "down: d1"),
            moved("by_both", "up: u1, down: d1"),
        ];
        let given: Vec<&EvidenceRecord> = records.iter().collect();
        let invalidated = exclude(&ir, &case, &given).expect("evaluates");
        assert_eq!(
            ids(invalidated.for_claim(&ClaimId::new("z"))),
            ["by_both", "by_down", "by_up"]
        );
        for (id, claim) in &ir.claims {
            for tested in claim.true_when.claim_references() {
                assert!(
                    invalidated
                        .for_claim(tested)
                        .is_subset(&invalidated.for_claim(id)),
                    "{id:?} tests {tested:?}"
                );
            }
        }
    }

    /// Found once for every claim, the matches a claim reaches are those a walk from it alone
    /// finds, through a diamond and a match reached along two paths, each match once.
    #[test]
    fn the_matches_each_claim_reaches_are_those_a_walk_from_it_finds() {
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}}\nevidence_kinds: {k: {}, l: {}}\n\
                 claims: {top: {true_when: {all: [{claim: left}, {claim: right}]}}, \
                 left: {true_when: {any: [{claim: bottom}, {evidence: {kind: l}}]}}, \
                 right: {true_when: {not: {claim: bottom}}}, \
                 bottom: {true_when: {evidence: {kind: k, subject: a}}}, \
                 alone: {true_when: {all: []}}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let walk = |claim: &ClaimId| -> Vec<*const EvidenceMatch> {
            let mut found = BTreeSet::new();
            let mut visited = BTreeSet::new();
            let mut pending = vec![claim];
            while let Some(next) = pending.pop() {
                if visited.insert(next) {
                    ir.claims[next].true_when.visit(&mut |node| match node {
                        Predicate::Evidence(matching) => {
                            found.insert(std::ptr::from_ref(matching));
                        }
                        Predicate::Claim(test) => pending.push(&test.claim),
                        _ => {}
                    });
                }
            }
            found.into_iter().collect()
        };
        let reached = reached_by_claim(&ir);
        assert_eq!(reached.len(), ir.claims.len());
        for (claim, matches) in &reached {
            let mut found: Vec<*const EvidenceMatch> = matches
                .iter()
                .map(|matching| std::ptr::from_ref(*matching))
                .collect();
            found.sort_unstable();
            assert_eq!(found, walk(claim), "{claim:?}");
        }
        assert_eq!(reached[&ClaimId::new("top")].len(), 2);
        assert!(reached[&ClaimId::new("alone")].is_empty());
    }
}

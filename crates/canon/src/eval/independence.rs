//! Independence: which evidence records an evidence match that demands independence reads
//! (story:evidence-independence, CANON-INDEPENDENCE-001).
//!
//! A protocol declares independence dimensions (`independence_dimensions`); an evidence match may
//! demand that its records' producer be independent on some of them (`independent_on`). Canon gives
//! no dimension a meaning and resolves no identity: the trusted context that writes the case
//! snapshot lists, under `independence`, the dimensions on which each producer is independent of
//! the case's work, and a record names its `producer` as given. A record satisfies a demand only
//! when it names a producer and the case lists that producer on every dimension demanded; a
//! producer the case does not list, or lists with `[]`, is independent on no dimension, and a
//! record without a producer satisfies no demand. A match that demands nothing reads its records
//! whoever produced them.
//!
//! A record of a demanding match's kind and subject that fails the demand is not of the match: it
//! neither establishes nor contradicts it, whatever its result, so a match only such records would
//! decide is `unknown`, not `false`. It is not set aside for every claim, as the exclusion stages
//! set records aside: another match, demanding less, still reads it. Under a claim it is listed as
//! excluded with the reason `not_independent` when a match the claim reaches would read it by kind
//! and subject, no earlier reason keeps it from the claim, and no match the claim reaches that
//! reads it accepts its producer.
//!
//! This file checks the case's `independence` map against the protocol ([`check`]) and answers
//! whether a record satisfies a demand ([`Independence::accepts`]).

use std::collections::{BTreeMap, BTreeSet};

use super::claims::reads;
use super::{Refusal, identifier};
use crate::ir::Ir;
use crate::model::{
    Case, EvidenceMatch, EvidenceRecord, IndependenceDimensionId, Principal, one_line,
};

/// The case's `independence` map, as the evaluator reads it: for each producer, the dimensions on
/// which it is independent of the case's work. The default lists no producer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Independence {
    producers: BTreeMap<Principal, BTreeSet<IndependenceDimensionId>>,
}

impl Independence {
    /// The independence `case` gives each producer. The case check ([`check`]) has refused a
    /// producer listed twice.
    pub(crate) fn of(case: &Case) -> Self {
        Self {
            producers: case
                .independence
                .iter()
                .map(|(producer, dimensions)| {
                    (producer.clone(), dimensions.iter().cloned().collect())
                })
                .collect(),
        }
    }

    /// Whether `record` satisfies the independence `demanded` of it: always, when nothing is
    /// demanded; otherwise only when it names a producer the case lists on every dimension
    /// demanded.
    pub(crate) fn accepts(
        &self,
        demanded: Option<&[IndependenceDimensionId]>,
        record: &EvidenceRecord,
    ) -> bool {
        let Some(demanded) = demanded else {
            return true;
        };
        let Some(producer) = &record.producer else {
            return false;
        };
        let listed = self.producers.get(producer);
        demanded
            .iter()
            .all(|dimension| listed.is_some_and(|listed| listed.contains(dimension)))
    }

    /// Each producer the case lists, in producer order, with its dimensions in identifier order.
    pub(crate) fn producers(
        &self,
    ) -> impl Iterator<Item = (&Principal, &BTreeSet<IndependenceDimensionId>)> {
        self.producers.iter()
    }
}

/// Whether `record` is `not_independent` among `matches`, the evidence matches one claim reaches:
/// some of them would read it by kind and subject, and none of those accepts its producer.
pub(crate) fn rejected(
    matches: &[&EvidenceMatch],
    record: &EvidenceRecord,
    independence: &Independence,
) -> bool {
    let mut read = false;
    for matching in matches {
        if reads(&matching.kind, matching.subject.as_ref(), record) {
            if independence.accepts(matching.independent_on.as_deref(), record) {
                return false;
            }
            read = true;
        }
    }
    read
}

/// Checks the case's `independence` map against the protocol: each producer, in the order written,
/// is an identifier (`invalid-identifier`) listed once (`duplicate-identifier`), and each
/// dimension listed for it, in the order written, is an identifier (`invalid-identifier`), listed
/// once for that producer (`duplicate-identifier`) and declared by the protocol
/// (`undeclared-dimension`). Each refusal names the producer, and the dimension where there is one.
pub(super) fn check(ir: &Ir, case: &Case) -> Result<(), Refusal> {
    let case_id = one_line(case.id.as_str());
    let mut producers = BTreeSet::new();
    for (producer, dimensions) in case.independence.iter() {
        identifier("case producer identifier", producer.as_str())?;
        let shown = one_line(producer.as_str());
        if !producers.insert(producer) {
            return Err(Refusal::new(
                "duplicate-identifier",
                format!("case `{case_id}` lists producer `{shown}` more than once"),
            ));
        }
        let mut listed = BTreeSet::new();
        for dimension in dimensions {
            identifier("independence dimension identifier", dimension.as_str())?;
            let dimension_shown = one_line(dimension.as_str());
            if !listed.insert(dimension) {
                return Err(Refusal::new(
                    "duplicate-identifier",
                    format!(
                        "case `{case_id}` lists producer `{shown}` as independent on dimension `{dimension_shown}` more than once"
                    ),
                ));
            }
            if !ir.independence_dimensions.contains_key(dimension) {
                return Err(Refusal::new(
                    "undeclared-dimension",
                    format!(
                        "case `{case_id}` lists producer `{shown}` as independent on dimension `{dimension_shown}`, which the protocol does not declare"
                    ),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{read_case, read_evidence};

    const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\nindependence_dimensions: {d: {}, e: {}}\nevidence_kinds: {k: {}}\n";

    fn ir() -> Ir {
        crate::ir::compile(&crate::model::parse(PROTOCOL).expect("parses")).expect("compiles")
    }

    fn case(independence: &str) -> Case {
        read_case(&format!(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {{a: {{revision: r1}}}}\n{independence}"
        ))
        .expect("reads")
    }

    fn record(producer: Option<&str>) -> EvidenceRecord {
        let producer = producer.map_or(String::new(), |producer| format!("producer: {producer}\n"));
        read_evidence(&format!(
            "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n{producer}"
        ))
        .expect("reads")
    }

    fn dimensions(ids: &[&str]) -> Vec<IndependenceDimensionId> {
        ids.iter()
            .copied()
            .map(IndependenceDimensionId::new)
            .collect()
    }

    /// A demand is met only by a named producer the case lists on every dimension demanded; no
    /// demand is met by every record.
    #[test]
    fn a_demand_is_met_only_by_a_producer_listed_on_every_dimension() {
        let independence = Independence::of(&case("independence: {p1: [d], p2: [d, e], p3: []}\n"));
        let both = dimensions(&["d", "e"]);
        let one = dimensions(&["d"]);
        for (producer, demanded, accepted) in [
            (Some("p1"), Some(one.as_slice()), true),
            (Some("p1"), Some(both.as_slice()), false),
            (Some("p2"), Some(both.as_slice()), true),
            (Some("p3"), Some(one.as_slice()), false),
            (Some("unlisted"), Some(one.as_slice()), false),
            (None, Some(one.as_slice()), false),
            (None, None, true),
            (Some("p3"), None, true),
        ] {
            assert_eq!(
                independence.accepts(demanded, &record(producer)),
                accepted,
                "{producer:?} {demanded:?}"
            );
        }
    }

    /// The case's map is refused, naming the producer and the dimension, for an undeclared
    /// dimension, a producer or a dimension listed twice, and a producer or a dimension that is
    /// not an identifier.
    #[test]
    fn the_case_map_is_checked_against_the_protocol() {
        assert_eq!(
            check(&ir(), &case("independence: {p1: [d, e], p2: []}\n")),
            Ok(())
        );
        for (map, code, message) in [
            (
                "independence: {p1: [x]}\n",
                "undeclared-dimension",
                "case `C-1` lists producer `p1` as independent on dimension `x`, which the protocol does not declare",
            ),
            (
                "independence: {p1: [d, d]}\n",
                "duplicate-identifier",
                "case `C-1` lists producer `p1` as independent on dimension `d` more than once",
            ),
            (
                "independence: {'p 1': [d]}\n",
                "invalid-identifier",
                "case producer identifier `p 1` is empty or contains whitespace or a control character",
            ),
            (
                "independence: {p1: ['d d']}\n",
                "invalid-identifier",
                "independence dimension identifier `d d` is empty or contains whitespace or a control character",
            ),
        ] {
            let refusal = check(&ir(), &case(map)).expect_err(map);
            assert_eq!(refusal.code(), code, "{map}");
            assert_eq!(refusal.to_string(), message, "{map}");
        }
        // YAML refuses a repeated key when the case is read; a case a caller builds can still
        // repeat a producer.
        let mut repeated = case("");
        repeated.independence = crate::model::Declarations::new(vec![
            (Principal::new("p1"), dimensions(&["d"])),
            (Principal::new("p1"), dimensions(&["e"])),
        ]);
        let refusal = check(&ir(), &repeated).expect_err("a repeated producer");
        assert_eq!(refusal.code(), "duplicate-identifier");
        assert_eq!(
            refusal.to_string(),
            "case `C-1` lists producer `p1` more than once"
        );
    }
}

//! The `outcomes` section of `canon-decision/1`: each declared outcome legitimate or blocked, and
//! the check that a case terminates only through a declared outcome. Not built yet
//! (story:outcomes): no section, and any termination is accepted.

use std::collections::BTreeMap;

use super::Refusal;
use super::decisions::Decisions;
use crate::ir::Ir;
use crate::model::{Case, ClaimId, EvidenceRecord, Json, Truth};

/// The section, or `None` to leave the slot empty; or a refusal of the case snapshot. `evidence`
/// is what the claims were evaluated over (after the exclusion stages); a requirement is evaluated
/// over it and `claims` with `super::claims::predicate`.
pub(super) fn section(
    _ir: &Ir,
    _case: &Case,
    _claims: &BTreeMap<ClaimId, Truth>,
    _evidence: &[EvidenceRecord],
    _decisions: Option<&Decisions>,
) -> Result<Option<Json>, Refusal> {
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::super::{evaluate, read_case};
    use crate::ir::Ir;

    const PROTOCOL: &str = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims:\n\
          \x20\x20c: {true_when: {evidence: {kind: k}}}\n\
        outcomes:\n\
          \x20\x20done: {requires: {claim: c}}\n";

    fn ir() -> Ir {
        crate::ir::compile(&crate::model::parse(PROTOCOL).expect("parses")).expect("compiles")
    }

    fn terminated_through(outcome: &str) -> crate::model::Case {
        read_case(&format!(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {{a: {{revision: r1}}}}\ntermination: {outcome}\n"
        ))
        .expect("case reads")
    }

    /// CANON-OUTCOME-001: a case terminates only through an outcome its protocol declares. The
    /// conformance scenario checks the refusal's code; this checks that it names the outcome.
    #[test]
    fn a_termination_through_an_undeclared_outcome_is_refused_naming_it() {
        let refusal = evaluate(&ir(), &terminated_through("abandoned"), &[])
            .expect_err("a termination through an undeclared outcome is refused");
        assert_eq!(refusal.code(), "undeclared-outcome", "{refusal}");
        assert_eq!(
            refusal.to_string(),
            "case `C-1` terminated through outcome `abandoned`, which the protocol does not declare"
        );
    }

    /// The control: the same snapshot terminated through the declared outcome is evaluated.
    #[test]
    fn a_termination_through_a_declared_outcome_is_evaluated() {
        evaluate(&ir(), &terminated_through("done"), &[])
            .expect("a termination through a declared outcome is evaluated");
    }
}

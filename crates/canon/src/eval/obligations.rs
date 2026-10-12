//! The `obligations` section of `canon-decision/1`: each obligation the protocol declares, open or
//! discharged by its discharge predicate over the claim values (story:obligations), or not
//! applicable by its applicability predicate (story:case-inputs).
//!
//! The section is an array with one entry per declared obligation, in identifier order, each
//! `{"id": <obligation id>, "status": "open" | "discharged" | "not_applicable"}`, decided in this
//! order:
//!
//! 1. `not_applicable` when the obligation's `applicable_when` predicate is `false` over the claim
//!    values and the case's inputs: it holds back nothing;
//! 2. otherwise `discharged` when its `discharged_when` predicate evaluates `true`, whatever its
//!    applicability: an obligation that is met where it applies is met;
//! 3. otherwise `open`: `unknown` and `false` both leave it open, since only `true` satisfies a
//!    positive requirement (design § 8). When its applicability is `unknown` (a claim it tests is
//!    `unknown`), it is open because `unknown` is not `false`, and its entry carries `reasons`
//!    naming what is unknown: the reasons an action precondition that is not `true` gives
//!    (`actions.rs`), `{"claim": <id>, "value": <value>}` and `{"input": <id>, "value":
//!    <value>}`.
//!
//! An obligation without an `applicable_when` always applies. Whether
//! deciding a claim or establishing it discharges an obligation is the protocol author's choice,
//! written with the claim tests of the predicate (`{claim: c, is: unknown}` is itself decided). A
//! protocol that declares no obligation has no section, so its decision is written as before, and
//! an obligation without an applicability predicate has the entry it had before.

use std::collections::BTreeMap;

use super::claims::{Inputs, predicate};
use crate::ir::{Ir, Obligation};
use crate::model::{ClaimId, Json, ObligationStatus, Truth};

/// The section, or `None` when the protocol declares no obligation. A discharge or applicability
/// predicate tests only claim values and case inputs (the validator refuses evidence in one), so
/// it is evaluated over `claims` and `inputs` with `super::claims::predicate`, given no evidence:
/// an evidence match in an IR a caller builds is `unknown`.
pub(super) fn section(ir: &Ir, claims: &BTreeMap<ClaimId, Truth>, inputs: &Inputs) -> Option<Json> {
    if ir.obligations.is_empty() {
        return None;
    }
    let entries = ir
        .obligations
        .iter()
        .map(|(id, obligation)| {
            let (status, reasons) = obligation_status(obligation, claims, inputs);
            let mut entry = serde_json::Map::new();
            entry.insert("id".to_owned(), Json::from(id.as_str()));
            if !reasons.is_empty() {
                entry.insert("reasons".to_owned(), Json::Array(reasons));
            }
            entry.insert("status".to_owned(), Json::from(status.as_str()));
            Json::Object(entry)
        })
        .collect();
    Some(Json::Array(entries))
}

/// One obligation's status, by the order the module docs give, with the reasons whether it applies
/// is unknown when it is open for that; no reasons otherwise.
fn obligation_status(
    obligation: &Obligation,
    claims: &BTreeMap<ClaimId, Truth>,
    inputs: &Inputs,
) -> (ObligationStatus, Vec<Json>) {
    let applies = obligation.applicable_when.as_ref().map(|applicable_when| {
        (
            applicable_when,
            predicate(applicable_when, claims, inputs, &[]),
        )
    });
    if let Some((_, Truth::False)) = applies {
        return (ObligationStatus::NotApplicable, Vec::new());
    }
    if predicate(&obligation.discharged_when, claims, inputs, &[]) == Truth::True {
        return (ObligationStatus::Discharged, Vec::new());
    }
    let reasons = match applies {
        Some((applicable_when, Truth::Unknown)) => {
            super::actions::unmet(applicable_when, claims, inputs, &[])
        }
        _ => Vec::new(),
    };
    (ObligationStatus::Open, reasons)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROTOCOL: &str = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        evidence_kinds: {k: {}}\n\
        claims:\n\
          \x20\x20c: {true_when: {evidence: {kind: k}}}\n\
        obligations:\n\
          \x20\x20z.established: {discharged_when: {claim: c}}\n\
          \x20\x20a.decided: {discharged_when: {not: {claim: c, is: unknown}}}\n\
          \x20\x20m.refuted: {discharged_when: {claim: c, is: false}}\n";

    fn ir(text: &str) -> Ir {
        crate::ir::compile(&crate::model::parse(text).expect("parses")).expect("compiles")
    }

    fn statuses(ir: &Ir, value: Truth) -> Vec<(String, String)> {
        let claims = BTreeMap::from([(ClaimId::new("c"), value)]);
        let section = section(ir, &claims, &Inputs::new())
            .expect("a protocol with obligations has the section");
        section
            .as_array()
            .expect("the section is an array")
            .iter()
            .map(|entry| {
                let field = |name: &str| entry[name].as_str().expect(name).to_owned();
                (field("id"), field("status"))
            })
            .collect()
    }

    /// Only a discharge predicate that evaluates `true` discharges its obligation; the entries
    /// come in identifier order, whatever order the protocol declares them in.
    #[test]
    fn an_obligation_is_discharged_only_when_its_predicate_is_true() {
        let ir = ir(PROTOCOL);
        let row = |a: &str, m: &str, z: &str| {
            vec![
                ("a.decided".to_owned(), a.to_owned()),
                ("m.refuted".to_owned(), m.to_owned()),
                ("z.established".to_owned(), z.to_owned()),
            ]
        };
        assert_eq!(statuses(&ir, Truth::Unknown), row("open", "open", "open"));
        assert_eq!(
            statuses(&ir, Truth::False),
            row("discharged", "discharged", "open")
        );
        assert_eq!(
            statuses(&ir, Truth::True),
            row("discharged", "open", "discharged")
        );
    }

    /// A claim the values do not hold is `unknown`, so its obligation stays open.
    #[test]
    fn a_claim_without_a_value_leaves_the_obligation_open() {
        let ir = ir(PROTOCOL);
        let section = section(&ir, &BTreeMap::new(), &Inputs::new()).expect("section");
        let opened: Vec<&str> = section
            .as_array()
            .expect("array")
            .iter()
            .map(|entry| entry["status"].as_str().expect("status"))
            .collect();
        assert_eq!(opened, ["open", "open", "open"]);
    }

    /// An applicability predicate that is `false` makes the obligation not_applicable whatever its
    /// discharge predicate says; otherwise the discharge predicate decides, and an obligation it
    /// leaves open whose applicability is `unknown` has the reasons naming what is unknown. An
    /// input test decides on the case's value.
    #[test]
    fn an_obligation_applies_by_its_applicability_predicate() {
        let ir = ir("format: protocol/1\nprotocol: {id: p, revision: 1}\n\
             case: {inputs: {s: {values: [low, high]}}}\n\
             evidence_kinds: {k: {}}\n\
             claims: {c: {true_when: {evidence: {kind: k}}}, d: {true_when: {evidence: {kind: k}}}}\n\
             obligations:\n  \
             by_input: {applicable_when: {input: s, in: [high]}, discharged_when: {claim: d}}\n  \
             by_claim: {applicable_when: {claim: c}, discharged_when: {claim: d}}\n");
        let decide = |c: Option<Truth>, d: Truth, s: &str| {
            let mut claims = BTreeMap::from([(ClaimId::new("d"), d)]);
            if let Some(c) = c {
                claims.insert(ClaimId::new("c"), c);
            }
            let inputs = Inputs::from([(
                crate::model::InputId::new("s"),
                crate::model::InputValue::new(s),
            )]);
            serde_json::to_string(&section(&ir, &claims, &inputs).expect("section"))
                .expect("serializes")
        };
        assert_eq!(
            decide(Some(Truth::False), Truth::True, "low"),
            r#"[{"id":"by_claim","status":"not_applicable"},{"id":"by_input","status":"not_applicable"}]"#
        );
        assert_eq!(
            decide(Some(Truth::True), Truth::True, "high"),
            r#"[{"id":"by_claim","status":"discharged"},{"id":"by_input","status":"discharged"}]"#
        );
        assert_eq!(
            decide(None, Truth::Unknown, "high"),
            r#"[{"id":"by_claim","reasons":[{"claim":"c","value":"unknown"}],"status":"open"},{"id":"by_input","status":"open"}]"#
        );
        // Discharged whatever its applicability, unknown included.
        assert_eq!(
            decide(None, Truth::True, "high"),
            r#"[{"id":"by_claim","status":"discharged"},{"id":"by_input","status":"discharged"}]"#
        );
    }

    #[test]
    fn a_protocol_without_obligations_has_no_section() {
        let ir = ir("format: protocol/1\nprotocol: {id: p, revision: 1}\n");
        assert_eq!(section(&ir, &BTreeMap::new(), &Inputs::new()), None);
    }
}

//! The `obligations` section of `canon-decision/1`: each obligation the protocol declares, open or
//! discharged by its discharge predicate over the claim values (story:obligations).
//!
//! The section is an array with one entry per declared obligation, in identifier order, each
//! `{"id": <obligation id>, "status": "open" | "discharged"}`. An obligation is `discharged` only
//! when its `discharged_when` predicate evaluates `true`; `unknown` and `false` both leave it
//! `open`, since only `true` satisfies a positive requirement (design § 8). Whether deciding a
//! claim or establishing it discharges an obligation is the protocol author's choice, written with
//! the claim tests of the predicate (`{claim: c, is: unknown}` is itself decided). A protocol that
//! declares no obligation has no section, so its decision is written as before.

use std::collections::BTreeMap;

use crate::ir::Ir;
use crate::model::{ClaimId, Json, Truth};

/// The section, or `None` when the protocol declares no obligation. A discharge predicate tests
/// only claim values (the validator refuses evidence in one), so it is evaluated over `claims`
/// alone with `super::claims::predicate`, given no evidence: an evidence match in an IR a caller
/// builds is `unknown`, and leaves its obligation open.
pub(super) fn section(ir: &Ir, claims: &BTreeMap<ClaimId, Truth>) -> Option<Json> {
    if ir.obligations.is_empty() {
        return None;
    }
    let entries = ir
        .obligations
        .iter()
        .map(|(id, obligation)| {
            let status = match super::claims::predicate(&obligation.discharged_when, claims, &[]) {
                Truth::True => "discharged",
                Truth::False | Truth::Unknown => "open",
            };
            serde_json::json!({ "id": id.as_str(), "status": status })
        })
        .collect();
    Some(Json::Array(entries))
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
        let section = section(ir, &claims).expect("a protocol with obligations has the section");
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
        let section = section(&ir, &BTreeMap::new()).expect("section");
        let opened: Vec<&str> = section
            .as_array()
            .expect("array")
            .iter()
            .map(|entry| entry["status"].as_str().expect("status"))
            .collect();
        assert_eq!(opened, ["open", "open", "open"]);
    }

    #[test]
    fn a_protocol_without_obligations_has_no_section() {
        let ir = ir("format: protocol/1\nprotocol: {id: p, revision: 1}\n");
        assert_eq!(section(&ir, &BTreeMap::new()), None);
    }
}

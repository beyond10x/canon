//! The `outcomes` section of `canon-decision/1`: each declared outcome legitimate or blocked, and
//! the check that a case terminates only through a declared outcome (story:outcomes,
//! CANON-OUTCOME-001).
//!
//! Every declared outcome gets an entry, keyed by outcome id:
//!
//! - `{"status": "legitimate"}` when its `requires` predicate, evaluated over the claim values
//!   and the evidence left after the exclusion stages, is `true`;
//! - `{"status": "blocked", "reasons": [...]}` otherwise (`false` or `unknown`). The reasons are
//!   the tests that decide the requirement against it, each named once: walking from `requires`
//!   with the wanted value `true`, `not` flips the wanted value, `all` and `any` descend only into
//!   the members whose value is not the wanted one, and a claim test or evidence match whose own
//!   value is not the wanted one is a reason. So `{not: {claim: c}}` gives the reasons
//!   `{claim: c, is: false}` gives, and a test that already has the wanted value is never named.
//!   A claim test gives `{"claim": <id>, "value": <the claim's own value>}`, an evidence match
//!   `{"evidence": <kind>, "present": <whether a record of the kind is in the evidence left after
//!   the exclusion stages>}`: `false` for a required record that is missing, `true` for a record
//!   present under `not`. Claims come first, in claim-id order, then evidence kinds, in kind
//!   order. A blocked outcome always states at least one reason: a requirement no test decides
//!   against is one that cannot be met (it is blocked only by an empty `any`, or a `not` over an
//!   empty `all`), and gives `{"requirement": "unsatisfiable"}`.
//!
//! A case snapshot whose `termination` names an outcome the protocol does not declare is refused
//! as `undeclared-outcome`, naming the outcome. One that names a declared outcome which is blocked
//! is refused as `illegitimate-termination`, naming the outcome and its status: an outcome is a
//! declared legitimate terminal interpretation of a case (design § 4.6).
//!
//! A protocol that declares no outcome leaves the section out. Outcomes whose requirement is an
//! explicit decision are story:decision-outcomes; `decisions` is not read here.

use std::collections::{BTreeMap, BTreeSet};

use super::Refusal;
use super::claims::predicate;
use super::decisions::Decisions;
use crate::ir::Ir;
use crate::model::{
    Case, ClaimId, EvidenceKindId, EvidenceRecord, Json, Predicate, Truth, one_line,
};

/// The section, or `None` to leave the slot empty; or a refusal of the case snapshot. `evidence`
/// is what the claims were evaluated over (after the exclusion stages); a requirement is evaluated
/// over it and `claims` with `super::claims::predicate`.
pub(super) fn section(
    ir: &Ir,
    case: &Case,
    claims: &BTreeMap<ClaimId, Truth>,
    evidence: &[EvidenceRecord],
    _decisions: Option<&Decisions>,
) -> Result<Option<Json>, Refusal> {
    if let Some(outcome) = &case.termination
        && !ir.outcomes.contains_key(outcome)
    {
        return Err(Refusal::new(
            "undeclared-outcome",
            format!(
                "case `{}` terminated through outcome `{}`, which the protocol does not declare",
                one_line(case.id.as_str()),
                one_line(outcome.as_str())
            ),
        ));
    }
    if ir.outcomes.is_empty() {
        return Ok(None);
    }
    let mut entries = serde_json::Map::new();
    for (id, outcome) in &ir.outcomes {
        let legitimate = predicate(&outcome.requires, claims, evidence) == Truth::True;
        if !legitimate && case.termination.as_ref() == Some(id) {
            return Err(Refusal::new(
                "illegitimate-termination",
                format!(
                    "case `{}` terminated through outcome `{}`, which is blocked",
                    one_line(case.id.as_str()),
                    one_line(id.as_str())
                ),
            ));
        }
        let entry = if legitimate {
            serde_json::json!({"status": "legitimate"})
        } else {
            serde_json::json!({
                "status": "blocked",
                "reasons": reasons(&outcome.requires, claims, evidence),
            })
        };
        entries.insert(id.as_str().to_owned(), entry);
    }
    Ok(Some(Json::Object(entries)))
}

/// What keeps a requirement from being met, collected by [`Unmet::walk`].
#[derive(Default)]
struct Unmet<'a> {
    /// Each claim whose test decides against the requirement, with the claim's own value.
    claims: BTreeMap<&'a ClaimId, Truth>,
    /// Each evidence kind whose match decides against the requirement.
    evidence: BTreeSet<&'a EvidenceKindId>,
}

impl<'a> Unmet<'a> {
    /// Collects the tests that keep `node` from having the value `wanted` (`true`, or `false`
    /// under an odd number of `not`s): a claim test or an evidence match whose own value is not
    /// `wanted`; through `not`, the inner predicate's tests against the opposite value; through
    /// `all` and `any`, those of each member whose value is not `wanted`. A member that already
    /// has the wanted value is not visited, so a test it holds is never named.
    fn walk(
        &mut self,
        node: &'a Predicate,
        wanted: Truth,
        claims: &BTreeMap<ClaimId, Truth>,
        evidence: &[EvidenceRecord],
    ) {
        match node {
            Predicate::Not(inner) => {
                let opposite = match wanted {
                    Truth::True => Truth::False,
                    _ => Truth::True,
                };
                self.walk(inner, opposite, claims, evidence);
            }
            Predicate::All(members) | Predicate::Any(members) => {
                for member in members {
                    if predicate(member, claims, evidence) != wanted {
                        self.walk(member, wanted, claims, evidence);
                    }
                }
            }
            Predicate::Claim(test) => {
                if predicate(node, claims, evidence) != wanted {
                    let value = claims.get(&test.claim).copied().unwrap_or(Truth::Unknown);
                    self.claims.insert(&test.claim, value);
                }
            }
            Predicate::Evidence(matching) => {
                if predicate(node, claims, evidence) != wanted {
                    self.evidence.insert(&matching.kind);
                }
            }
        }
    }
}

/// The reasons a blocked requirement states: claims first, in claim-id order, then evidence
/// kinds, in kind order, with whether a record of the kind is present, each once;
/// `{"requirement": "unsatisfiable"}` when no test is named.
fn reasons(
    requires: &Predicate,
    claims: &BTreeMap<ClaimId, Truth>,
    evidence: &[EvidenceRecord],
) -> Vec<Json> {
    let mut unmet = Unmet::default();
    unmet.walk(requires, Truth::True, claims, evidence);
    let mut reasons: Vec<Json> = unmet
        .claims
        .into_iter()
        .map(|(claim, value)| {
            serde_json::json!({"claim": claim.as_str(), "value": value.to_string()})
        })
        .chain(
            unmet
                .evidence
                .into_iter()
                .map(|kind| {
                    let present = evidence.iter().any(|record| record.kind == *kind);
                    serde_json::json!({"evidence": kind.as_str(), "present": present})
                }),
        )
        .collect();
    if reasons.is_empty() {
        reasons.push(serde_json::json!({"requirement": "unsatisfiable"}));
    }
    reasons
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

    /// One `k` record, which makes `c`, and so `done`, `true`.
    fn observed() -> crate::model::EvidenceRecord {
        super::super::read_evidence(
            "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n",
        )
        .expect("evidence reads")
    }

    /// The control: the same snapshot terminated through the declared outcome, legitimate, is
    /// evaluated.
    #[test]
    fn a_termination_through_a_declared_outcome_is_evaluated() {
        evaluate(&ir(), &terminated_through("done"), &[observed()])
            .expect("a termination through a declared legitimate outcome is evaluated");
    }

    /// An outcome is a declared legitimate terminal interpretation (design § 4.6): a termination
    /// through a declared outcome that is blocked is refused, naming the outcome and its status.
    #[test]
    fn a_termination_through_a_blocked_outcome_is_refused_naming_it_and_its_status() {
        let refusal = evaluate(&ir(), &terminated_through("done"), &[])
            .expect_err("a termination through a blocked outcome is refused");
        assert_eq!(refusal.code(), "illegitimate-termination", "{refusal}");
        assert_eq!(
            refusal.to_string(),
            "case `C-1` terminated through outcome `done`, which is blocked"
        );
    }

    /// A blocked outcome names each claim whose test is not `true` once, in claim-id order, with
    /// the claim's own value; a claim whose test is met is not named.
    #[test]
    fn a_blocked_outcome_names_each_unmet_claim_test_once_in_claim_id_order() {
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
                 evidence_kinds: {k: {}, l: {}}\n\
                 claims:\n\
                 \x20\x20c: {true_when: {evidence: {kind: k}}}\n\
                 \x20\x20d: {true_when: {evidence: {kind: l}}}\n\
                 outcomes:\n\
                 \x20\x20both: {requires: {all: [{claim: d}, {claim: c}, {claim: c, is: true}]}}\n\
                 \x20\x20without_c: {requires: {claim: c, is: false}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let case = read_case(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n",
        )
        .expect("case reads");
        let outcomes = |evidence: &[crate::model::EvidenceRecord]| {
            evaluate(&ir, &case, evidence)
                .expect("decides")
                .outcomes
                .expect("the protocol declares outcomes")
        };
        let blocked = |reasons: &[(&str, &str)]| {
            let reasons: Vec<_> = reasons
                .iter()
                .map(|(claim, value)| serde_json::json!({"claim": claim, "value": value}))
                .collect();
            serde_json::json!({"status": "blocked", "reasons": reasons})
        };
        assert_eq!(
            outcomes(&[]),
            serde_json::json!({
                "both": blocked(&[("c", "unknown"), ("d", "unknown")]),
                "without_c": blocked(&[("c", "unknown")]),
            })
        );
        let k = super::super::read_evidence(
            "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n",
        )
        .expect("evidence reads");
        assert_eq!(
            outcomes(&[k]),
            serde_json::json!({
                "both": blocked(&[("d", "unknown")]),
                "without_c": blocked(&[("c", "true")]),
            })
        );
    }

    /// An unmet evidence match is named by its kind and whether a record of the kind is present,
    /// after the claims; a requirement no test decides against, which cannot be met, says so.
    #[test]
    fn a_blocked_outcome_names_an_unmet_evidence_match_and_an_unsatisfiable_requirement() {
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
                 evidence_kinds: {k: {}, l: {}}\n\
                 claims:\n\
                 \x20\x20c: {true_when: {evidence: {kind: k}}}\n\
                 outcomes:\n\
                 \x20\x20mixed: {requires: {all: [{evidence: {kind: l}}, {claim: c}]}}\n\
                 \x20\x20never: {requires: {all: [{any: []}, {not: {all: []}}, {claim: c}]}}\n\
                 \x20\x20absent: {requires: {not: {evidence: {kind: k}}}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let case = read_case(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n",
        )
        .expect("case reads");
        let decided = evaluate(&ir, &case, &[observed()])
            .expect("decides")
            .outcomes
            .expect("the protocol declares outcomes");
        assert_eq!(
            decided,
            serde_json::json!({
                "absent": {"status": "blocked", "reasons": [{"evidence": "k", "present": true}]},
                "mixed": {"status": "blocked", "reasons": [{"evidence": "l", "present": false}]},
                "never": {"status": "blocked", "reasons": [{"requirement": "unsatisfiable"}]},
            })
        );
        let decided = evaluate(&ir, &case, &[])
            .expect("decides")
            .outcomes
            .expect("the protocol declares outcomes");
        assert_eq!(
            decided["mixed"],
            serde_json::json!({"status": "blocked", "reasons": [
                {"claim": "c", "value": "unknown"},
                {"evidence": "l", "present": false},
            ]})
        );
        assert_eq!(
            decided["never"],
            serde_json::json!({"status": "blocked", "reasons": [{"claim": "c", "value": "unknown"}]})
        );
        // With no `k` record the negated match is `unknown`, not `false`: still blocked, by a
        // record that is missing.
        assert_eq!(
            decided["absent"],
            serde_json::json!({"status": "blocked", "reasons": [{"evidence": "k", "present": false}]})
        );
    }

    /// A protocol that declares no outcome leaves the section out.
    #[test]
    fn a_protocol_without_outcomes_leaves_the_section_out() {
        let ir = crate::ir::compile(
            &crate::model::parse(
                &PROTOCOL.replace("outcomes:\n  done: {requires: {claim: c}}\n", ""),
            )
            .expect("parses"),
        )
        .expect("compiles");
        assert!(ir.outcomes.is_empty());
        let case = read_case(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n",
        )
        .expect("case reads");
        assert_eq!(evaluate(&ir, &case, &[]).expect("decides").outcomes, None);
    }
}

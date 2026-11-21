//! Adversary cases for story:authority-revision-binding (wave 2026-11-20-w3, pass 1), through the
//! evaluator library over the fixture of CANON-AUTHORITY-002
//! (`fixtures/investigation/capability-scope.yaml`): action `inspect` requires `metrics.read` at
//! `{cluster: prod-eu, namespace: foo}`.
//!
//! Driven from the story's Outcome: "An entry with `case_revision` applies only when that revision
//! equals the `canon-case/1` snapshot's `revision`", whatever its decision (`## Decided`, option
//! A), and "An entry that does not apply takes no part in either" coverage or precedence. The
//! unit's scenario and unit tests bind `granted` and `denied` entries only; no test in the tree
//! binds a `require_approval` one, so a filter that kept a bound `require_approval` at every
//! revision stayed green. These cases pin that third decision and the scoped lapse.

use std::path::PathBuf;

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Json};
use serde_json::json;

fn fixture() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/investigation/capability-scope.yaml");
    std::fs::read_to_string(path).expect("the CANON-AUTHORITY-002 fixture reads")
}

/// The `inspect` entry of the `actions` section, with the case snapshot at `revision` (none:
/// no `revision` key).
fn inspect(authority: &str, revision: Option<&str>) -> Json {
    let ir = ir::compile(&model::parse(&fixture()).expect("parses")).expect("compiles");
    let mut case = "format: canon-case/1\nid: OPS-1\nprotocol: metrics\nartifacts: {}\n".to_owned();
    if let Some(revision) = revision {
        case.push_str(&format!("revision: {revision}\n"));
    }
    let case = eval::read_case(&case).expect("case reads");
    let decision = eval::evaluate_with(
        &ir,
        &case,
        &[],
        Supplied {
            authority: Some(authority),
            ..Supplied::default()
        },
    )
    .expect("decides");
    decision.actions.expect("an actions section")["inspect"].clone()
}

fn reason(decision: &str) -> Json {
    json!({
        "reasons": [{
            "capability": "metrics.read",
            "decision": decision,
            "scope": {"cluster": "prod-eu", "namespace": "foo"}
        }],
        "status": if decision == "denied" { "blocked" } else { "approval-required" }
    })
}

/// A bound `require_approval` lapses like a bound `granted` or `denied`: beside an unbound grant
/// it decides at its own revision and takes no part at another one or without a revision.
#[test]
fn a_bound_require_approval_applies_only_at_its_case_revision() {
    let authority = "- {capability: metrics.read, decision: require_approval, case_revision: c1}\n\
                     - {capability: metrics.read, scope: {cluster: prod-eu}, decision: granted}\n";
    assert_eq!(inspect(authority, Some("c1")), reason("require_approval"));
    assert_eq!(
        inspect(authority, Some("c2")),
        json!({"status": "admissible"}),
        "the c1 require_approval lapsed at c2"
    );
    assert_eq!(
        inspect(authority, None),
        json!({"status": "admissible"}),
        "no current revision: the bound require_approval applies to nothing"
    );
    let alone = "- {capability: metrics.read, decision: require_approval, case_revision: c1}\n";
    assert_eq!(inspect(alone, Some("c2")), reason("none"));
}

/// Coverage runs over the entries that apply only, from the wildcard outwards: a lapsed wide
/// denial leaves an unbound narrow grant deciding, and a lapsed narrow grant leaves an unbound wide
/// require_approval deciding. Two principals at one revision are two entries, and the denial wins.
#[test]
fn a_lapsed_entry_takes_no_part_in_scoped_coverage_or_precedence() {
    let wide_denied = "- {capability: metrics.read, scope: {cluster: prod-eu}, decision: denied, case_revision: c1}\n\
                       - {capability: metrics.read, scope: {cluster: prod-eu, namespace: foo}, decision: granted}\n";
    assert_eq!(inspect(wide_denied, Some("c1")), reason("denied"));
    assert_eq!(
        inspect(wide_denied, Some("c2")),
        json!({"status": "admissible"})
    );
    let narrow_granted = "- {capability: metrics.read, scope: {cluster: prod-eu, namespace: foo}, decision: granted, case_revision: c1}\n\
                          - {capability: metrics.read, decision: require_approval}\n";
    assert_eq!(
        inspect(narrow_granted, Some("c2")),
        reason("require_approval")
    );
    let principals = "- {capability: metrics.read, decision: granted, case_revision: c2, principal: p}\n\
                      - {capability: metrics.read, decision: denied, case_revision: c2, principal: q}\n";
    assert_eq!(inspect(principals, Some("c2")), reason("denied"));
    assert_eq!(inspect(principals, Some("c1")), reason("none"));
}

//! story:pending-decisions against the evaluator library, over the fixture of CANON-PENDING-001
//! (`fixtures/investigation/pending-decisions.yaml`): what the conformance runner cannot hold,
//! because it compares only the sections a step lists. A decision with nothing pending carries no
//! `pending_decisions` key, and the order of the entries in the authority and decisions documents
//! does not change the section.

use std::path::PathBuf;

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Decision};

/// Read at run time, so a test binary reused from another build directory reads this tree.
fn fixture() -> ir::Ir {
    let root = std::env::var_os("CARGO_MANIFEST_DIR").expect("run through cargo");
    let path = PathBuf::from(root).join("../../fixtures/investigation/pending-decisions.yaml");
    let text = std::fs::read_to_string(&path).expect("the fixture reads");
    ir::compile(&model::parse(&text).expect("the fixture parses")).expect("the fixture compiles")
}

const CASE: &str = "format: canon-case/1\nid: OPS-7\nprotocol: operations\nrevision: c2\n\
                    artifacts: {service: {revision: s1}}\n";

fn decide(evidence: &[model::EvidenceRecord], supplied: Supplied<'_>) -> Decision {
    let case = eval::read_case(CASE).expect("case reads");
    eval::evaluate_with(&fixture(), &case, evidence, supplied).expect("decides")
}

/// Every requirement granted, both decisions taken at c2 and the claim true: nothing is pending,
/// so the slot is empty and the document has no `pending_decisions` key (CANON-PENDING-001,
/// expectation 9). Remove one grant and the section is back, so the key's absence is not an
/// accident of the input.
#[test]
fn a_decision_with_nothing_pending_has_no_key() {
    let anomaly = eval::read_evidence(
        "format: canon-evidence/1\nid: anomaly-1\nkind: anomaly_report\nsubject: service\n\
         subject_revision: s1\n",
    )
    .expect("evidence reads");
    let decisions = "- {decision: explicitly_inconclusive, outcome: inconclusive, \
                     principal: lead-operator, case_revision: c2}\n\
                     - {decision: explicitly_abandoned, outcome: abandoned, \
                     principal: lead-operator, case_revision: c2}\n";
    let granted = "- {capability: metrics.read, decision: granted}\n\
                   - {capability: report.write, decision: granted}\n\
                   - {capability: pager.send, decision: granted}\n";
    let decision = decide(
        std::slice::from_ref(&anomaly),
        Supplied {
            authority: Some(granted),
            decisions: Some(decisions),
            ..Supplied::default()
        },
    );
    assert_eq!(decision.pending_decisions, None);
    assert!(!eval::render(&decision).contains("pending_decisions"));

    let one_short = "- {capability: metrics.read, decision: granted}\n\
                     - {capability: report.write, decision: granted}\n";
    let decision = decide(
        &[anomaly],
        Supplied {
            authority: Some(one_short),
            decisions: Some(decisions),
            ..Supplied::default()
        },
    );
    assert_eq!(
        decision.pending_decisions,
        Some(serde_json::json!({"authority": [{
            "action": "escalate", "capability": "pager.send", "current": "none",
            "case_revision": "c2", "options": ["granted", "denied"],
        }]}))
    );
}

/// Every order of the entries of the authority document, and of the decisions document, gives
/// the same section, byte for byte.
#[test]
fn the_order_of_the_input_documents_does_not_change_the_section() {
    let authority = [
        "- {capability: metrics.read, scope: {cluster: prod-eu, namespace: foo}, decision: require_approval}\n",
        "- {capability: metrics.read, scope: {namespace: foo}, decision: granted}\n",
        "- {capability: report.write, decision: granted}\n",
        "- {capability: metrics.read, scope: {cluster: prod-us}, decision: denied, case_revision: c1}\n",
    ];
    let decisions = [
        "- {decision: explicitly_inconclusive, outcome: inconclusive, principal: p, case_revision: c1}\n",
        "- {decision: explicitly_abandoned, outcome: abandoned, principal: p, case_revision: c1}\n",
    ];
    let listed = ["pending_decisions".to_owned()].into_iter().collect();
    let section = |authority: &str, decisions: &str| {
        let decision = decide(
            &[],
            Supplied {
                authority: Some(authority),
                decisions: Some(decisions),
                ..Supplied::default()
            },
        );
        eval::render_sections(&decision, &listed)
    };
    let first = section(&authority.concat(), &decisions.concat());
    assert!(first.contains("\"require_approval\""), "{first}");
    let mut orders = 0;
    for a in permutations(authority.len()) {
        let authority: String = a.iter().map(|index| authority[*index]).collect();
        for d in permutations(decisions.len()) {
            let decisions: String = d.iter().map(|index| decisions[*index]).collect();
            assert_eq!(
                section(&authority, &decisions),
                first,
                "{authority}{decisions}"
            );
            orders += 1;
        }
    }
    assert_eq!(orders, 24 * 2);
}

/// Every order of `0..n`.
fn permutations(n: usize) -> Vec<Vec<usize>> {
    if n == 0 {
        return vec![Vec::new()];
    }
    let mut all = Vec::new();
    for shorter in permutations(n - 1) {
        for at in 0..=shorter.len() {
            let mut order = shorter.clone();
            order.insert(at, n - 1);
            all.push(order);
        }
    }
    all
}

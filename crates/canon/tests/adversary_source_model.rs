//! Adversary cases for story:protocol-source-model (wave 2026-10-04-w1).

use b10x_canon::model;

/// `is` admits exactly `true`, `false` or `unknown` (model/predicate.rs). Written with no value it
/// must not silently become the positive test `is: true`: an author who left the value out did not
/// write one.
#[test]
fn a_claim_test_whose_is_has_no_value_is_a_parse_error() {
    for written in ["is:", "is: ~", "is: null"] {
        let source = format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nclaims:\n  a: {{true_when: {{all: []}}}}\noutcomes:\n  o:\n    requires:\n      claim: a\n      {written}\n"
        );
        let parsed = model::parse(&source);
        assert!(
            parsed.is_err(),
            "`{written}` was accepted as {:?}",
            parsed.map(|protocol| protocol
                .outcomes
                .iter()
                .map(|(_, outcome)| outcome.requires.clone())
                .collect::<Vec<_>>())
        );
    }
}

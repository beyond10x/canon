//! Adversarial case for `canon check` over evidence matches that demand independence
//! (story:evidence-independence, fix 2a1ff3a): the state space a plausible software-change
//! protocol needs.

use b10x_canon::check;
use b10x_canon::{ir, model};

/// A software-change review: four evidence kinds, each demanded independent by one claim, review
/// demanded on two nested demands (`[different_principal]` and `[different_principal,
/// different_team]`), and a gate "no independent reviewer rejected". Six claims, two gated
/// actions, two outcomes.
///
/// Without any `independent_on` this protocol has 8 * 4 * 4 * 4 = 512 states. With the demands
/// the space gives each dimension `2^(c * p)` values, `p` producer profiles, so `review` alone has
/// 2^(3 * 3) = 512 values and the whole space 524288 states, and `canon check` refuses it at the
/// bound of 65536. The documentation calls `2^(c * 2^m * p)` "the smallest set that loses
/// nothing", but it is not: a match reads class `c` exactly when some present profile of `c`
/// contains its demand, so what a state decides per class is the down-closed set of demands that
/// read it, not the subset of profiles; and a producer-less record of a kind every reader of which
/// demands independence is read by no match at all. For `review` that is 3^3 = 27 values (per class:
/// read by no demand, by `[different_principal]` only, or by both), for each other kind 2 * 2 = 4,
/// 1728 states in all, well inside the bound.
const PROTOCOL: &str = r#"format: protocol/1
protocol: {id: software-change, revision: 1}
artifacts: {change: {}}
independence_dimensions: {different_principal: {}, different_agent_run: {}, different_team: {}}
evidence_kinds: {review: {}, security_scan: {}, rollback_test: {}, tests: {}}
claims:
  change.reviewed:
    true_when: {evidence: {kind: review, result: approved, independent_on: [different_principal]}}
  change.security_reviewed:
    true_when: {evidence: {kind: review, result: approved, independent_on: [different_principal, different_team]}}
  change.not_rejected:
    true_when: {not: {evidence: {kind: review, result: rejected, independent_on: [different_principal]}}}
  security.clear:
    true_when: {evidence: {kind: security_scan, result: pass, independent_on: [different_agent_run]}}
  rollback.verified:
    true_when: {evidence: {kind: rollback_test, result: pass, independent_on: [different_principal]}}
  tests.green:
    true_when: {evidence: {kind: tests, result: pass}}
actions:
  run_tests: {may_produce: [{evidence: tests}]}
  request_review: {may_produce: [{evidence: review}]}
  scan: {may_produce: [{evidence: security_scan}]}
  verify_rollback: {may_produce: [{evidence: rollback_test}]}
  merge:
    precondition:
      all:
        - {claim: change.reviewed, is: true}
        - {claim: change.not_rejected, is: true}
        - {claim: tests.green, is: true}
        - {claim: security.clear, is: true}
  deploy:
    precondition:
      all:
        - {claim: change.security_reviewed, is: true}
        - {claim: rollback.verified, is: true}
outcomes:
  shipped:
    requires:
      all:
        - {claim: change.reviewed, is: true}
        - {claim: rollback.verified, is: true}
  rejected:
    requires: {claim: change.reviewed, is: false}
"#;

/// `canon check` decides the software-change protocol rather than refusing it at the bound.
#[test]
fn check_decides_a_software_change_protocol_with_four_demanded_kinds() {
    let ir = ir::compile(&model::parse(PROTOCOL).expect("parses")).expect("compiles");
    let without = PROTOCOL
        .replace(
            ", independent_on: [different_principal, different_team]",
            "",
        )
        .replace(", independent_on: [different_principal]", "")
        .replace(", independent_on: [different_agent_run]", "");
    let plain = ir::compile(&model::parse(&without).expect("parses")).expect("compiles");
    let plain_states = check::check(&plain, None).expect("checked").states;
    assert_eq!(plain_states, 512, "the protocol without demands");
    match check::check(&ir, None) {
        Ok(report) => assert!(report.states <= 65536, "{report}"),
        Err(refusal) => panic!(
            "refused although the demand-free protocol has {plain_states} states: {}: {refusal}",
            refusal.code()
        ),
    }
}

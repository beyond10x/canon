//! E4 of the wave 9 review: `canon check` decided each slice and then kept its states and
//! evaluations until it returned, so its peak memory was the sum of the slices'. Only `canon
//! generate` reads them again (`check_keeping`); a plain check holds one slice at a time.

use super::{ENUMERATIONS_HELD, Strategy, check_by, check_keeping};
use crate::ir::{self, Ir};
use crate::model;

/// Three outcomes, each reading four evidence kinds of its own, and an action reading none: four
/// spaces under `Strategy::Own`, three of 16 states and the action's of one (one product of 4096
/// otherwise).
fn four_spaces() -> Ir {
    let mut text = String::from(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\nevidence_kinds:\n",
    );
    for group in 0..3 {
        for kind in 0..4 {
            text.push_str(&format!("  k{group}x{kind}: {{}}\n"));
        }
    }
    text.push_str("claims:\n");
    for group in 0..3 {
        for kind in 0..4 {
            text.push_str(&format!(
                "  c{group}x{kind}: {{true_when: {{evidence: {{kind: k{group}x{kind}}}}}}}\n"
            ));
        }
    }
    text.push_str("outcomes:\n");
    for group in 0..3 {
        text.push_str(&format!("  o{group}:\n    requires:\n      all:\n"));
        for kind in 0..4 {
            text.push_str(&format!("        - {{claim: c{group}x{kind}}}\n"));
        }
    }
    text.push_str("actions:\n  act:\n    may_produce:\n");
    for group in 0..3 {
        for kind in 0..4 {
            text.push_str(&format!("      - {{evidence: k{group}x{kind}}}\n"));
        }
    }
    ir::compile(&model::parse(&text).expect("parses")).expect("compiles")
}

#[test]
fn a_check_holds_one_slice_at_a_time_and_a_check_that_keeps_them_holds_all() {
    let ir = four_spaces();
    let report = check_by(&ir, None, Strategy::Own).expect("checks");
    assert!(report.is_clean(), "{report}");
    assert_eq!(
        ENUMERATIONS_HELD.with(std::cell::Cell::get),
        1,
        "check_by enumerations held at once"
    );
    let (kept_report, kept) = check_keeping(&ir, None, Strategy::Own).expect("checks");
    assert_eq!(kept_report, report, "the report is the same either way");
    assert_eq!(kept.len(), 4);
    assert_eq!(
        ENUMERATIONS_HELD.with(std::cell::Cell::get),
        4,
        "check_keeping enumerations held at once"
    );
}

//! Adversary pass 2 for story:decision-outcomes (wave 2026-10-04-w8): what a `canon-decisions/1`
//! entry with a missing key is refused with.
//!
//! `eval/decisions.rs` documents "an entry with another key or a missing one, as
//! `malformed-input`". The code is right; the message is not. Every key of an entry is an
//! identifier, and identifiers deserialize through `present::required`, which reads a missing
//! field as an explicit null and says "an explicit null is not allowed here; leave the key out to
//! take its default" — telling the author to do what they already did, for a key that has no
//! default. `--authority` names the key in the same situation (`missing field `decision``).
//! Measured with `canon evaluate --decisions` on a file holding
//! `[{decision: d, outcome: o, principal: p}]` (exit 1, that message). The same message comes for
//! a `canon-case/1` snapshot without `id`, so the mechanism predates this unit
//! (`git show 8fc260a:crates/canon/src/model/ids.rs`, line 21).

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model;

const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
    outcomes: {inconclusive: {requires: {decision: stop}}}\n";

/// A missing key is refused naming the key, not as a null to leave out.
#[test]
fn a_decision_entry_missing_a_key_is_refused_naming_the_key() {
    let ir = ir::compile(&model::parse(PROTOCOL).expect("parses")).expect("compiles");
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nrevision: c2\nartifacts: {a: {revision: r1}}\n",
    )
    .expect("case reads");
    for (decisions, missing) in [
        (
            "[{decision: stop, outcome: inconclusive, principal: p}]",
            "case_revision",
        ),
        (
            "[{outcome: inconclusive, principal: p, case_revision: c2}]",
            "decision",
        ),
    ] {
        let refusal = eval::evaluate_with(
            &ir,
            &case,
            &[],
            Supplied {
                decisions: Some(decisions),
                ..Supplied::default()
            },
        )
        .expect_err(decisions);
        assert_eq!(refusal.code(), "malformed-input", "{decisions}: {refusal}");
        let message = refusal.to_string();
        assert!(
            message.contains(missing) && !message.contains("leave the key out"),
            "{decisions}: the refusal should name the missing key `{missing}`, and not tell the \
             author to leave a key out:\n{message}"
        );
    }
}

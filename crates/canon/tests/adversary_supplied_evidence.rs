//! Adversary passes 1 and 2 for story:supplied-evidence, against the library.
//!
//! Pass 1 asserted the story's first wording ("a supplied kind is not governed") and was red. The
//! coordinator decided the rule the code holds (story:supplied-evidence, last section; the module
//! docs of `check`): a kind that declares `supplied_by` counts as produced without authority and
//! stays governed when an action that requires a capability may also produce it, so `canon check`
//! reports `authority-bypass` on an outcome a supplied record alone makes legitimate. Pass 2
//! asserts that rule (`a_supplied_kind_an_authority_action_produces_is_a_bypass`) and attacks it:
//! a kind both a governed and a capability-free action produce, a kind read two claims down, the
//! witness with a second governed kind, sliced and whole alike, and a second governed kind no
//! capability-free action produces. Then the `canon diff` rule: a changed supplier is
//! `BREAKING`, alone and beside another change of the same kind, in a fixed order. Every case is
//! expected green.
//!
//! The pass 1 probes stay: a supplied record about an undeclared subject is still refused, and an
//! import whose kind names the root as its supplier composes into a self-supplied protocol, which
//! is refused.

use b10x_canon::check;
use b10x_canon::diff;
use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model;

fn compiled(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

fn checked(source: &str) -> String {
    check::check(&compiled(source), None)
        .expect("checked")
        .to_string()
}

/// `s` is supplied by `q`, and `approve`, which requires the capability `c`, may also produce it.
/// No action that requires no capability produces anything.
const GOVERNED_AND_SUPPLIED: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}}\n\
    evidence_kinds: {s: {supplied_by: q}}\n\
    claims: {ok: {true_when: {evidence: {kind: s}}}}\n\
    actions: {approve: {requires: [{capability: c}], may_produce: [{evidence: s}]}}\n\
    outcomes: {done: {requires: {claim: ok}}}\n";

const BYPASS_ON_S: &str = "authority-bypass: outcome `done` is legitimate without authority in state {evidence `s`}\n\
     checked: protocol `p` revision 1: 6 states, 0 properties, 1 finding\n";

/// The decided rule: with `supplied_by`, the supplied record alone witnesses the bypass; without
/// it, only the governed action produces `s` and nothing is found.
#[test]
fn a_supplied_kind_an_authority_action_produces_is_a_bypass() {
    assert_eq!(checked(GOVERNED_AND_SUPPLIED), BYPASS_ON_S);
    let unsupplied = GOVERNED_AND_SUPPLIED.replace("s: {supplied_by: q}", "s: {}");
    assert_eq!(
        checked(&unsupplied),
        "checked: protocol `p` revision 1: 6 states, 0 properties, 0 findings\n"
    );
}

/// A supplied kind that a governed and a capability-free action both produce: the bypass is
/// reported once, with the same witness and state count, whichever route makes `s` free.
#[test]
fn a_supplied_kind_a_free_action_also_produces_is_one_bypass() {
    let both = GOVERNED_AND_SUPPLIED.replace(
        "may_produce: [{evidence: s}]}}",
        "may_produce: [{evidence: s}]}, self_approve: {may_produce: [{evidence: s}]}}",
    );
    assert_ne!(both, GOVERNED_AND_SUPPLIED);
    assert_eq!(checked(&both), BYPASS_ON_S);
    assert_eq!(
        checked(&both.replace("s: {supplied_by: q}", "s: {}")),
        BYPASS_ON_S
    );
}

/// The outcome reads the supplied kind only through a claim that tests a claim that tests it.
#[test]
fn a_supplied_kind_read_two_claims_down_is_a_bypass() {
    let deep = GOVERNED_AND_SUPPLIED.replace(
        "claims: {ok: {true_when: {evidence: {kind: s}}}}",
        "claims: {ok: {true_when: {claim: mid}}, mid: {true_when: {claim: low}}, \
         low: {true_when: {evidence: {kind: s}}}}",
    );
    assert_ne!(deep, GOVERNED_AND_SUPPLIED);
    assert_eq!(checked(&deep), BYPASS_ON_S);
}

/// `done` needs the supplied `s` and `k`, both governed by `approve`; `self_approve` also produces
/// `k`. The witness holds both. `w`, read only by `other`, doubles the whole product, so `done` is
/// decided over its slice: the witness is the one the protocol without `w` gives over its whole
/// product.
#[test]
fn the_witness_beside_a_second_governed_kind_is_the_same_sliced_and_whole() {
    let pair = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {s: {supplied_by: q}, k: {}}\n\
        actions:\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: s}, {evidence: k}]}\n\
        \x20\x20self_approve: {may_produce: [{evidence: k}]}\n\
        outcomes:\n\
        \x20\x20done: {requires: {all: [{evidence: {kind: s}}, {evidence: {kind: k}}]}}\n";
    assert_eq!(
        checked(pair),
        "authority-bypass: outcome `done` is legitimate without authority in state \
         {evidence `k`, evidence `s`}\n\
         checked: protocol `p` revision 1: 12 states, 0 properties, 1 finding\n"
    );
    let wide = pair.replace("k: {}}", "k: {}, w: {}}").replace(
        "outcomes:\n",
        "\x20\x20watch: {may_produce: [{evidence: w}]}\noutcomes:\n",
    ) + "\x20\x20other: {requires: {evidence: {kind: w}}}\n";
    assert_eq!(
        checked(&wide),
        "authority-bypass: outcome `done` is legitimate without authority in state \
         {evidence `k`, evidence `s`}\n\
         checked: protocol `p` revision 1: 24 states, 0 properties, 1 finding\n"
    );
}

/// A second governed kind `g` no capability-free action produces: an outcome that needs both is
/// no bypass (a state holding `g` is no witness); one that takes either rests on the supplied `s`
/// alone and is.
#[test]
fn a_second_governed_kind_only_authority_produces_is_needed_or_not() {
    let both = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {s: {supplied_by: q}, g: {}}\n\
        actions: {approve: {requires: [{capability: c}], \
        may_produce: [{evidence: s}, {evidence: g}]}}\n\
        outcomes: {done: {requires: {all: [{evidence: {kind: s}}, {evidence: {kind: g}}]}}}\n";
    assert_eq!(
        checked(both),
        "checked: protocol `p` revision 1: 12 states, 0 properties, 0 findings\n"
    );
    assert_eq!(
        checked(&both.replace("{all: [", "{any: [")),
        "authority-bypass: outcome `done` is legitimate without authority in state {evidence `s`}\n\
         checked: protocol `p` revision 1: 12 states, 0 properties, 1 finding\n"
    );
}

fn kinds(body: &str) -> ir::Ir {
    compiled(&format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nevidence_kinds: {body}\n"
    ))
}

fn lines(from: &ir::Ir, to: &ir::Ir) -> Vec<String> {
    diff::diff(from, to)
        .expect("one protocol")
        .changes
        .iter()
        .map(ToString::to_string)
        .collect()
}

/// A to B is `BREAKING`, A to A no change, added `EXPANSION` and removed `BREAKING`.
#[test]
fn a_changed_supplier_breaks_and_an_unchanged_one_is_no_change() {
    let none = kinds("{a: {}}");
    let x = kinds("{a: {supplied_by: x}}");
    let y = kinds("{a: {supplied_by: y}}");
    assert_eq!(
        lines(&x, &y),
        ["BREAKING: evidence kind `a` supplier changed from protocol `x` to protocol `y`"]
    );
    assert!(lines(&x, &x).is_empty());
    assert_eq!(
        lines(&none, &x),
        ["EXPANSION: evidence kind `a` supplier introduced: protocol `x`"]
    );
    assert_eq!(
        lines(&x, &none),
        ["BREAKING: evidence kind `a` supplier removed: was protocol `x`"]
    );
}

/// A changed supplier beside a changed maximum age of the same kind is two lines, the maximum age
/// first, and two kinds come in identifier order, whatever order they are written in.
#[test]
fn a_changed_supplier_beside_another_change_is_its_own_line_in_a_fixed_order() {
    let from = kinds("{b: {supplied_by: x}, a: {max_age: 2h, supplied_by: x}}");
    let to = kinds("{a: {max_age: 1h, supplied_by: y}, b: {supplied_by: y}}");
    let expected = [
        "TIGHTENING: evidence kind `a` maximum age shortened from 2h to 1h",
        "BREAKING: evidence kind `a` supplier changed from protocol `x` to protocol `y`",
        "BREAKING: evidence kind `b` supplier changed from protocol `x` to protocol `y`",
    ];
    assert_eq!(lines(&from, &to), expected);
    assert_eq!(lines(&from, &to), lines(&from, &to));
}

/// A record of a supplied kind may name undeclared upstream artifacts, not an undeclared subject:
/// the revision-binding stage refuses it as before.
#[test]
fn a_supplied_record_about_an_undeclared_subject_is_refused() {
    let ir = compiled(GOVERNED_AND_SUPPLIED);
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts:\n  a: {revision: r1}\n",
    )
    .expect("case reads");
    let record = eval::read_evidence(
        "format: canon-evidence/1\nid: e1\nkind: s\nsubject: elsewhere\nsubject_revision: r1\n\
         upstream_revisions: {gone: g1}\n",
    )
    .expect("record reads");
    let refusal =
        eval::evaluate_with(&ir, &case, &[record], Supplied::default()).expect_err("refused");
    assert_eq!(refusal.code(), "undeclared-artifact");
    assert_eq!(
        refusal.to_string(),
        "evidence `e1` is about artifact `elsewhere`, which the protocol does not declare"
    );
}

/// An import may name the root as a kind's supplier, valid alone; composed into the root, the
/// kind is supplied by the protocol's own id, and the composed document is refused.
#[test]
fn an_import_supplied_by_the_root_composes_into_a_self_supplied_protocol() {
    let part = model::parse(
        "format: protocol/1\nprotocol: {id: lib.a, revision: 1}\n\
         evidence_kinds: {s: {supplied_by: root}}\n",
    )
    .expect("part parses");
    let digest = ir::digest(&ir::compile(&part).expect("the part alone is valid"));
    let root = model::parse(&format!(
        "format: protocol/1\nprotocol: {{id: root, revision: 1}}\n\
         imports:\n  lib.a: {{revision: 1, digest: '{digest}'}}\n"
    ))
    .expect("root parses");
    let problems: Vec<String> = match ir::compose(&root, &[part]) {
        Ok(composed) => ir::compile(&composed)
            .expect_err("self-supplied")
            .iter()
            .map(|problem| format!("{}: {problem}", problem.code()))
            .collect(),
        Err(problems) => problems
            .iter()
            .map(|problem| format!("{}: {problem}", problem.code()))
            .collect(),
    };
    assert_eq!(
        problems,
        [
            "self-supplied-evidence: evidence kind `s` is supplied by protocol `root`, the protocol's own id"
        ]
    );
}

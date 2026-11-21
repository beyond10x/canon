//! Adversary pass 1 for story:supplied-evidence, against the library.
//!
//! - `a_supplied_kind_is_not_governed_even_when_an_authority_action_produces_it` (expected red):
//!   the story's § Outcome 3 says "For `authority-bypass`, a supplied kind is not governed in this
//!   protocol", and the brief's claim 2 repeats it. The checker counts a supplied kind as produced
//!   without authority (`free`) but leaves it governed when an action requiring a capability also
//!   produces it, so an outcome that rests only on such a kind is reported as a bypass, its
//!   witness the state holding the supplied record. Without `supplied_by` the same protocol has no
//!   finding.
//! - the rest are probes (expected green): a supplied record about an undeclared subject is still
//!   refused, and an import whose kind names the root as its supplier composes into a
//!   self-supplied protocol, which is refused.

use b10x_canon::check;
use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model;

fn compiled(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// `s` is supplied by `q`, and `approve`, which requires the capability `c`, may also produce it.
/// No action that requires no capability produces anything.
const GOVERNED_AND_SUPPLIED: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}}\n\
    evidence_kinds: {s: {supplied_by: q}}\n\
    claims: {ok: {true_when: {evidence: {kind: s}}}}\n\
    actions: {approve: {requires: [{capability: c}], may_produce: [{evidence: s}]}}\n\
    outcomes: {done: {requires: {claim: ok}}}\n";

#[test]
fn a_supplied_kind_is_not_governed_even_when_an_authority_action_produces_it() {
    let unsupplied = GOVERNED_AND_SUPPLIED.replace("s: {supplied_by: q}", "s: {}");
    assert_eq!(
        check::check(&compiled(&unsupplied), None)
            .expect("checked")
            .to_string(),
        "checked: protocol `p` revision 1: 6 states, 0 properties, 0 findings\n"
    );
    assert_eq!(
        check::check(&compiled(GOVERNED_AND_SUPPLIED), None)
            .expect("checked")
            .to_string(),
        "checked: protocol `p` revision 1: 6 states, 0 properties, 0 findings\n",
        "story § Outcome 3: a supplied kind is not governed in this protocol"
    );
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

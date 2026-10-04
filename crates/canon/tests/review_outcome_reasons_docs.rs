//! Independent review of wave 2026-10-04-w7: the published evaluation reference against the
//! evaluator, for the reasons a blocked outcome gives.
//!
//! `crates/canon/src/eval/mod.rs` (module docs, rendered as `website/docs/reference/evaluation.md`
//! by `canon-docs`) says, under "Sections": an outcome that is not legitimate maps to
//! `{"status": "blocked", "reasons": [...]}`, "the reasons written as an action precondition's".
//!
//! They are not. For an action precondition, "In an `all` that is `false`, only its `false`
//! members decide it" (`eval/actions.rs` module docs); an outcome walk descends into every member
//! whose value is not the wanted one (`eval/outcomes.rs` module docs), so an `unknown` member of a
//! `false` `all` is a reason for the outcome and not for the action. The same predicate, as a
//! precondition and as a requirement, gives two different reason lists.

use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model;

#[test]
fn an_outcome_blocked_by_a_predicate_gives_the_reasons_the_same_precondition_gives() {
    let protocol = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {k: {}, l: {}}\n\
        claims:\n\
        \x20\x20refuted: {true_when: {evidence: {kind: k, result: pass}}}\n\
        \x20\x20undecided: {true_when: {evidence: {kind: l}}}\n\
        actions:\n\
        \x20\x20act: {precondition: {all: [{claim: refuted}, {claim: undecided}]}}\n\
        outcomes:\n\
        \x20\x20done: {requires: {all: [{claim: refuted}, {claim: undecided}]}}\n";
    let ir = ir::compile(&model::parse(protocol).expect("parses")).expect("compiles");
    let case = eval::read_case(
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n",
    )
    .expect("case reads");
    // `refuted` is false (a `k` record without `pass`); `undecided` is unknown (no `l` record).
    let evidence = [eval::read_evidence(
        "format: canon-evidence/1\nid: e1\nkind: k\nresult: fail\nsubject: a\nsubject_revision: r1\n",
    )
    .expect("evidence reads")];
    let decision = eval::evaluate(&ir, &case, &evidence).expect("decides");

    let action = &decision.actions.as_ref().expect("actions")["act"];
    let outcome = &decision.outcomes.as_ref().expect("outcomes")["done"];
    assert_eq!(action["status"], "blocked", "{action}");
    assert_eq!(outcome["status"], "blocked", "{outcome}");
    assert_eq!(
        outcome["reasons"], action["reasons"],
        "evaluation.md: an outcome's reasons are \"written as an action precondition's\"; \
         action reasons {} vs outcome reasons {}",
        action["reasons"], outcome["reasons"]
    );
}

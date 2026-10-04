//! Adversary pass 1 on story:review-hardening-w7.
//!
//! The story bounds a caller-built IR by `MAX_IR_DEPTH`, measured per predicate (`check_depth`,
//! `eval/read.rs`). Claim evaluation (`eval/claims.rs`) recurses through claim references as well
//! as through `all`, `any` and `not`, so the stack an evaluation needs is the sum of the depths along
//! a chain of claims, not the depth of one predicate. These cases build IRs whose every predicate is
//! within the bound and check that the process survives.
//!
//! They also check the other public entry point that takes a caller-built IR (`check::check`),
//! the advice given for an explicit null on a required key, and the story outcome that a
//! conformance scenario holds the subject-bound reason shape in `actions` and `outcomes`.
//!
//! An abort cannot be caught inside one test, so the abort cases run the work in a child process
//! (this same test binary, one ignored test selected by name).

use std::path::PathBuf;
use std::process::Command;

use b10x_canon::eval;
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, ClaimId, Predicate};
use b10x_canon::{check, conform};

const CHILD_ENV: &str = "CANON_ADVERSARY_HARDENING_CHILD";

/// Claims in the chain, and `not`s around each claim reference: every predicate is
/// `NOTS + 1` levels deep, within `MAX_IR_DEPTH`.
const CHAIN: usize = 64;
const NOTS: usize = 4000;

fn claim(at: usize) -> String {
    format!("c{at:04}")
}

/// A compiled protocol whose claims form one chain `c0000 -> c0001 -> ... -> cNNNN -> evidence k`,
/// then, as a caller may, each claim reference wrapped in `NOTS` `not`s. Every predicate is within
/// the bound; the chain of them is not.
fn chained_ir() -> Ir {
    let mut source = String::from(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\nclaims:\n",
    );
    for at in 0..CHAIN {
        source.push_str(&format!(
            "  {}: {{true_when: {{claim: {}}}}}\n",
            claim(at),
            claim(at + 1)
        ));
    }
    source.push_str(&format!(
        "  {}: {{true_when: {{evidence: {{kind: k}}}}}}\n",
        claim(CHAIN)
    ));
    let mut built = ir::compile(&model::parse(&source).expect("parses")).expect("compiles");
    for at in 0..CHAIN {
        let declared = built
            .claims
            .get_mut(&ClaimId::new(claim(at)))
            .expect("compiled");
        let mut deep = declared.true_when.clone();
        for _ in 0..NOTS {
            deep = Predicate::Not(Box::new(deep));
        }
        declared.true_when = deep;
    }
    const _: () = assert!(
        NOTS < eval::MAX_IR_DEPTH,
        "each predicate is within the bound"
    );
    built
}

fn case() -> model::Case {
    eval::read_case("format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r}}\n")
        .expect("case reads")
}

fn run_child(test: &str) -> (std::process::ExitStatus, String, String) {
    let child = Command::new(std::env::current_exe().expect("test binary path"))
        .args([
            "--ignored",
            "--exact",
            test,
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_ENV, "1")
        .output()
        .expect("child test process runs");
    (
        child.status,
        String::from_utf8_lossy(&child.stdout).into_owned(),
        String::from_utf8_lossy(&child.stderr)
            .lines()
            .take(4)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

#[test]
#[ignore = "run as a child process by a_chain_of_claims_each_within_the_bound_does_not_abort"]
fn child_evaluates_a_chain_of_claims_each_within_the_bound() {
    if std::env::var_os(CHILD_ENV).is_none() {
        return;
    }
    let built = chained_ir();
    let result = eval::evaluate(&built, &case(), &[]);
    println!(
        "evaluated: {}",
        result.map_or_else(|r| r.code().to_owned(), |_| "decision".to_owned())
    );
    std::mem::forget(built);
}

/// `evaluate_with` accepts an IR whose every predicate is within `MAX_IR_DEPTH` (the module docs:
/// "an IR at the bound reads and evaluates from any caller's thread"), so it must return a
/// decision or a refusal for one. A chain of 64 claims, each testing the next under 4000 `not`s,
/// passes `check_depth` and is evaluated on the 64 MiB thread.
#[test]
fn a_chain_of_claims_each_within_the_bound_does_not_abort() {
    let (status, stdout, stderr) =
        run_child("child_evaluates_a_chain_of_claims_each_within_the_bound");
    assert!(
        status.success() && stdout.contains("evaluated: "),
        "a caller-built IR of {CHAIN} chained claims, each predicate {NOTS} levels deep, must be \
         evaluated or refused; the process ended with {status:?}\nstderr:\n{stderr}"
    );
}

#[test]
#[ignore = "run as a child process by check_of_a_caller_built_ir_beyond_the_bound_does_not_abort"]
fn child_checks_a_caller_built_ir_beyond_the_bound() {
    if std::env::var_os(CHILD_ENV).is_none() {
        return;
    }
    let mut built = ir::compile(
        &model::parse(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
             evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\n",
        )
        .expect("parses"),
    )
    .expect("compiles");
    let declared = built.claims.get_mut(&ClaimId::new("c")).expect("c");
    let mut deep = declared.true_when.clone();
    for _ in 0..2_000_000 {
        deep = Predicate::Not(Box::new(deep));
    }
    declared.true_when = deep;
    let result = check::check(&built, None);
    println!(
        "checked: {}",
        result.map_or_else(|r| r.code().to_owned(), |_| "report".to_owned())
    );
    std::mem::forget(built);
}

/// `check::check` is the other public function that takes any `Ir` a caller builds, and it
/// evaluates each state with `evaluate_with`. An IR nested beyond `MAX_IR_DEPTH` must be refused
/// (`predicate-too-deep`) there too, not abort the process.
#[test]
fn check_of_a_caller_built_ir_beyond_the_bound_does_not_abort() {
    let (status, stdout, stderr) = run_child("child_checks_a_caller_built_ir_beyond_the_bound");
    assert!(
        status.success() && stdout.contains("checked: predicate-too-deep"),
        "check::check of an IR 2,000,000 `not`s deep must be refused as predicate-too-deep; the \
         process ended with {status:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

/// `levels` predicate levels: an evidence match inside `levels - 1` `not`s.
fn nested(levels: usize) -> Predicate {
    let mut predicate = Predicate::Evidence(model::EvidenceMatch {
        kind: model::EvidenceKindId::new("k"),
        result: None,
        subject: None,
    });
    for _ in 1..levels {
        predicate = Predicate::Not(Box::new(predicate));
    }
    predicate
}

/// The bound holds for the deepest member, wherever it stands: a shallow first member of an `all`
/// or `any` and a deep last one. One level over the bound is refused; at the bound it evaluates.
/// Every existing depth test nests `not` alone, so a check that walked only the first member of a
/// list would pass them.
#[test]
fn depth_in_a_later_member_of_all_or_any_is_held_to_the_bound() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
                  evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\n";
    let compiled = ir::compile(&model::parse(source).expect("parses")).expect("compiles");
    let shallow = || nested(1);
    type List = fn(Vec<Predicate>) -> Predicate;
    let wrap: [(&str, List); 2] = [("all", Predicate::All), ("any", Predicate::Any)];
    for (name, list) in wrap {
        let mut over = compiled.clone();
        over.claims
            .get_mut(&ClaimId::new("c"))
            .expect("c")
            .true_when = list(vec![shallow(), nested(eval::MAX_IR_DEPTH)]);
        let refusal = eval::evaluate(&over, &case(), &[]).expect_err(name);
        assert_eq!(refusal.code(), "predicate-too-deep", "{name}: {refusal}");

        let mut at = compiled.clone();
        at.claims.get_mut(&ClaimId::new("c")).expect("c").true_when =
            list(vec![shallow(), nested(eval::MAX_IR_DEPTH - 1)]);
        eval::evaluate(&at, &case(), &[])
            .unwrap_or_else(|refusal| panic!("{name} at the bound: {refusal}"));
    }
}

/// A required key written with an explicit null is refused with the advice to "leave the key out
/// to take its default". A required key has no default: following the advice gives
/// `missing field`. The refusal of a null must not send the author to a second refusal.
#[test]
fn an_explicit_null_on_a_required_key_does_not_advise_leaving_it_out() {
    let full = "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n";
    let nulled = full.replace("subject: a", "subject: ~");
    let left_out = full.replace("subject: a\n", "");
    let null_refusal = eval::read_evidence(&nulled)
        .expect_err("null refused")
        .to_string();
    let left_out_refusal = eval::read_evidence(&left_out)
        .expect_err("left out refused")
        .to_string();
    assert!(
        left_out_refusal.contains("missing field `subject`"),
        "{left_out_refusal}"
    );
    assert!(
        !null_refusal.contains("leave the key out"),
        "an explicit null on the required key `subject` is refused as: {null_refusal}\n\
         leaving the key out, as advised, is refused as: {left_out_refusal}"
    );
}

/// The story's last outcome: a conformance scenario holds the subject-bound evidence reason
/// shape `{evidence, present, subject}` in the `actions` section and in the `outcomes` section.
#[test]
fn a_scenario_holds_the_subject_bound_reason_shape_in_actions_and_outcomes() {
    let root =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"))
            .join("../../conformance/scenarios");
    let mut found = [("actions", false), ("outcomes", false)];
    let mut read = 0;
    for entry in std::fs::read_dir(&root).expect("scenario registry") {
        let path = entry.expect("entry").path();
        let text = std::fs::read_to_string(&path).expect("scenario reads");
        let scenario = conform::parse(&text).expect("scenario parses");
        read += 1;
        for step in scenario.steps {
            let conform::StepKind::Evaluate {
                expected: conform::EvaluateExpectation::Decision(decision),
                ..
            } = step.kind
            else {
                continue;
            };
            let decision: serde_json::Value =
                serde_json::from_str(&decision).expect("expected decision is JSON");
            for (section, seen) in &mut found {
                let Some(entries) = decision[*section].as_object() else {
                    continue;
                };
                for entry in entries.values() {
                    for reason in entry["reasons"].as_array().into_iter().flatten() {
                        if reason.get("evidence").is_some()
                            && reason.get("present").is_some()
                            && reason.get("subject").is_some()
                        {
                            *seen = true;
                        }
                    }
                }
            }
        }
    }
    assert!(read > 0, "no scenario read from {}", root.display());
    assert_eq!(
        found,
        [("actions", true), ("outcomes", true)],
        "a scenario expects a reason {{evidence, present, subject}} in each section"
    );
}

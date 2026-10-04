//! Adversary cases for story:conformance-runner (wave 2026-10-04-w3).
//!
//! Each case drives `b10x_canon::conform` from the `canon-conformance/1` format its own module
//! documentation states, on a line the unit's suite leaves unobserved.

use b10x_canon::conform::{self, Verdict};
use b10x_canon::{ir, model};

const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n";

fn compiled() -> String {
    ir::compile(&model::parse(PROTOCOL).expect("parses"))
        .expect("compiles")
        .canonical_json()
}

fn literal(text: &str) -> String {
    text.lines()
        .map(|line| format!("        {line}\n"))
        .collect()
}

const HEAD: &str = "format: canon-conformance/1\nid: S\ncovers: []\nfixture: p.yaml\n";

/// "runs each scenario's steps in order, stopping a scenario at its first failing step." Every
/// scenario the unit's suite runs has exactly one step, so evaluating only the first step (a
/// scenario passes when its first step does) keeps that suite green.
#[test]
fn a_failing_second_step_fails_the_scenario() {
    let source = format!(
        "{HEAD}steps:\n  - id: first\n    compile: {{}}\n    expect:\n      canon-ir: |\n{}  - id: second\n    compile: {{}}\n    expect:\n      canon-ir: |\n        not the ir\n",
        literal(&compiled())
    );
    let scenario = conform::parse(&source).expect("parses");
    assert_eq!(
        conform::run(&scenario, Ok(PROTOCOL)),
        Verdict::Failed {
            step: "second".to_owned(),
            reason: "canon-ir/1 differs from the expectation at line 1".to_owned(),
        }
    );
}

/// The first failing step is the one reported, not a later one.
#[test]
fn the_first_failing_step_is_reported_not_the_last() {
    let source = format!(
        "{HEAD}steps:\n  - id: first\n    compile: {{}}\n    expect: {{canon-ir: x}}\n  - id: second\n    evaluate: {{case: {{}}, evidence: []}}\n    expect: {{refusal: r}}\n"
    );
    let scenario = conform::parse(&source).expect("parses");
    assert_eq!(
        conform::run(&scenario, Ok(PROTOCOL)),
        Verdict::Failed {
            step: "first".to_owned(),
            reason: "canon-ir/1 differs from the expectation at line 1".to_owned(),
        }
    );
}

/// "the step id, unique within the scenario" and reported in the report line; the unit's suite
/// refuses a bad scenario id but never a bad step id.
#[test]
fn a_step_id_that_is_not_an_identifier_is_refused() {
    let source =
        format!("{HEAD}steps:\n  - id: 'a b'\n    compile: {{}}\n    expect: {{canon-ir: x}}\n");
    let message = conform::parse(&source).expect_err("refused").to_string();
    assert!(
        message.contains("step id `a b` is not an identifier"),
        "{message}"
    );
}

/// `covers` lists "the CANON-* requirements it covers". `CANON-` names no requirement: the
/// prefix with nothing after it is accepted today.
#[test]
fn covers_naming_only_the_prefix_is_refused() {
    let source = "format: canon-conformance/1\nid: S\ncovers: [CANON-]\nfixture: p.yaml\nsteps:\n  - id: c\n    compile: {}\n    expect: {canon-ir: x}\n";
    let refused = conform::parse(source);
    assert!(
        refused.is_err(),
        "`covers: [CANON-]` parsed: {:?}",
        refused.map(|scenario| scenario.covers)
    );
}

/// A requirement with whitespace after the prefix is not an identifier; the unit's suite only
/// refuses a requirement that lacks the prefix, so dropping the identifier check stays green.
#[test]
fn covers_with_whitespace_is_refused() {
    let source = "format: canon-conformance/1\nid: S\ncovers: ['CANON-A B']\nfixture: p.yaml\nsteps:\n  - id: c\n    compile: {}\n    expect: {canon-ir: x}\n";
    let message = conform::parse(source).expect_err("refused").to_string();
    assert!(
        message.contains("covers `CANON-A B`, which is not a CANON-* requirement"),
        "{message}"
    );
}

/// An alias bomb in the uninterpreted evaluate inputs is refused promptly rather than expanded.
#[test]
fn an_alias_bomb_in_evaluate_inputs_is_refused() {
    let mut bomb = String::from("a0: &a0 [x, x, x, x, x, x, x, x, x, x]\n");
    for level in 1..10 {
        let previous = level - 1;
        bomb.push_str(&format!(
            "a{level}: &a{level} [*a{previous}, *a{previous}, *a{previous}, *a{previous}, *a{previous}, *a{previous}, *a{previous}, *a{previous}, *a{previous}, *a{previous}]\n"
        ));
    }
    let case: String = bomb
        .lines()
        .map(|line| format!("        {line}\n"))
        .collect();
    let source = format!(
        "{HEAD}steps:\n  - id: e\n    evaluate:\n      case:\n{case}      evidence: []\n    expect: {{refusal: r}}\n"
    );
    let started = std::time::Instant::now();
    let result = conform::parse(&source);
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "took {:?}",
        started.elapsed()
    );
    assert!(result.is_err(), "an alias bomb parsed");
}

/// The module documentation's own example scenario is a `canon-conformance/1` document: read at
/// run time from this tree, it parses into the compile step and the evaluate step it shows.
#[test]
fn the_documented_example_scenario_parses() {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").expect("run through cargo");
    let source =
        std::fs::read_to_string(std::path::PathBuf::from(manifest_dir).join("src/conform/mod.rs"))
            .expect("module source");
    let mut example = String::new();
    let mut inside = false;
    for line in source.lines() {
        let line = line.strip_prefix("//!").unwrap_or(line);
        let line = line.strip_prefix(' ').unwrap_or(line);
        if inside && line.trim_end() == "```" {
            break;
        }
        if inside {
            example.push_str(line);
            example.push('\n');
        }
        if line.trim_end() == "```yaml" {
            inside = true;
        }
    }
    assert!(!example.is_empty(), "no ```yaml block in the module doc");
    let scenario = conform::parse(&example).expect("the documented example parses");
    assert_eq!(scenario.id, "CANON-CLAIM-001");
    assert_eq!(scenario.covers, ["CANON-CLAIM-001", "CANON-CLAIM-002"]);
    let steps: Vec<&str> = scenario.steps.iter().map(|step| step.id.as_str()).collect();
    assert_eq!(steps, ["compile", "no-evidence"]);
}

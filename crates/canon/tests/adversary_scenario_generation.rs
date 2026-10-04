//! Adversary cases for story:scenario-generation, against the library.
//!
//! - Every protocol under `fixtures/investigation/` that generates: each scenario passes, and
//!   removing any one evidence record, authority decision or explicit decision fails it. Covers
//!   protocols with explicit decisions, authority requirements and subject-bound matches, not
//!   only the base protocol the acceptance uses.
//! - Two subjects whose scenario file names differ only in letter case, or only in Unicode
//!   normalisation, name one file on a case-insensitive or normalisation-insensitive filesystem
//!   (the default on macOS and Windows), so the second write replaces the first.

use std::path::{Path, PathBuf};

use b10x_canon::conform::{self, EvaluateInputs, StepKind, Verdict};
use b10x_canon::generate::{self, Scenario};
use b10x_canon::{ir, model};

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn generated(source: &str, fixture: &str) -> Result<Vec<Scenario>, String> {
    let protocol = model::parse(source).map_err(|error| format!("parse: {error}"))?;
    let compiled = ir::compile(&protocol).map_err(|problems| format!("compile: {problems:?}"))?;
    generate::generate(&compiled, fixture, source)
        .map_err(|refusal| format!("{}: {refusal}", refusal.code()))
}

/// Every `protocol/1` document under `dir`, outside `generate/` and `invalid/`, sorted.
fn protocols(dir: &Path, found: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("a fixture directory is readable")
        .map(|entry| entry.expect("an entry is readable").path())
        .collect();
    entries.sort();
    for path in entries {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if path.is_dir() {
            if name != "generate" && name != "invalid" {
                protocols(&path, found);
            }
        } else if name.ends_with(".yaml")
            && std::fs::read_to_string(&path)
                .expect("a fixture is readable")
                .starts_with("format: protocol/1")
        {
            found.push(path);
        }
    }
}

fn inputs(scenario: &mut conform::Scenario) -> &mut EvaluateInputs {
    match &mut scenario.steps[0].kind {
        StepKind::Evaluate { inputs, .. } => inputs,
        StepKind::Compile { .. } => panic!("a witness step evaluates"),
    }
}

/// Each way to remove one record from the scenario's evidence, authority or decisions, named.
fn removals(scenario: &conform::Scenario) -> Vec<(String, conform::Scenario)> {
    let mut parsed = scenario.clone();
    let base = inputs(&mut parsed).clone();
    let mut out = Vec::new();
    for index in 0..base.evidence.len() {
        let mut changed = scenario.clone();
        inputs(&mut changed).evidence.remove(index);
        out.push((format!("evidence {index}"), changed));
    }
    for index in 0..base.authority.as_ref().map_or(0, Vec::len) {
        let mut changed = scenario.clone();
        let list = inputs(&mut changed).authority.as_mut().expect("listed");
        list.remove(index);
        out.push((format!("authority {index}"), changed));
    }
    for index in 0..base.decisions.as_ref().map_or(0, Vec::len) {
        let mut changed = scenario.clone();
        let list = inputs(&mut changed).decisions.as_mut().expect("listed");
        list.remove(index);
        out.push((format!("decisions {index}"), changed));
    }
    out
}

/// A protocol whose outcome reads a subject-bound match and an unbound one of the same kind.
const SUBJECT_BOUND: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {explanation: {}, dataset: {}}\n\
    evidence_kinds: {attempt: {}}\n\
    claims:\n  \
      bound: {true_when: {evidence: {kind: attempt, result: survived, subject: explanation}}}\n  \
      unbound: {true_when: {evidence: {kind: attempt, result: refuted}}}\n\
    actions: {try: {may_produce: [{evidence: attempt}]}, \
    publish: {precondition: {claim: bound}, requires: [{capability: cap}]}}\n\
    outcomes: {survived: {requires: {all: [{claim: bound}, {not: {claim: unbound}}]}}, \
    refuted: {requires: {claim: unbound}}, closed: {requires: {decision: close}}}\n";

#[test]
fn every_generated_witness_passes_and_is_minimal_over_every_investigation_protocol() {
    let root = repository_root();
    let mut found = Vec::new();
    protocols(&root.join("fixtures/investigation"), &mut found);
    let mut sources: Vec<(String, String)> = found
        .iter()
        .map(|path| {
            let relative = path
                .strip_prefix(&root)
                .expect("below the root")
                .to_str()
                .expect("UTF-8")
                .to_owned();
            (relative, std::fs::read_to_string(path).expect("readable"))
        })
        .collect();
    sources.push(("subject-bound.yaml".to_owned(), SUBJECT_BOUND.to_owned()));

    let mut failures = Vec::new();
    let (mut scenarios_run, mut removals_run) = (0, 0);
    for (fixture, source) in &sources {
        let scenarios = match generated(source, fixture) {
            Ok(scenarios) => scenarios,
            Err(why) => {
                // The two fixtures written to be refused, and the ones with nothing to witness
                // (coordinator decision 3, adversary pass 1: refused as `nothing-to-witness`;
                // invalidation-rules.yaml, from story:invalidation-rules, declares no outcome or
                // action either).
                let expected = (fixture.ends_with("state-space-bound/protocol.yaml")
                    && why.starts_with("state-space-bound"))
                    || (fixture.ends_with("unreachable-outcome/protocol.yaml")
                        && why.starts_with("unreachable-outcome"))
                    || ((fixture.ends_with("subject-bound-evidence-match.yaml")
                        || fixture.ends_with("invalidation-rules.yaml"))
                        && why.starts_with("nothing-to-witness"));
                if !expected {
                    failures.push(format!("{fixture}: refused: {why}"));
                }
                continue;
            }
        };
        for scenario in &scenarios {
            let parsed = conform::parse(&scenario.text).expect("a generated scenario parses");
            scenarios_run += 1;
            if let Verdict::Failed { step, reason } = conform::run(&parsed, Ok(source)) {
                failures.push(format!(
                    "{fixture} {}: fails `{step}`: {reason}",
                    scenario.file
                ));
            }
            for (what, changed) in removals(&parsed) {
                removals_run += 1;
                if matches!(conform::run(&changed, Ok(source)), Verdict::Passed) {
                    failures.push(format!(
                        "{fixture} {}: still passes without {what}",
                        scenario.file
                    ));
                }
            }
        }
    }
    assert!(scenarios_run > 20, "{scenarios_run} scenarios ran");
    assert!(removals_run > 10, "{removals_run} removals ran");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The file names `files` would leave on a filesystem that folds letter case and composes `e`
/// followed by U+0301 into `é`, grouped: every group of more than one is a collision.
fn collisions(files: &[&str]) -> Vec<Vec<String>> {
    let mut groups: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for file in files {
        let folded = file.replace("e\u{301}", "\u{e9}").to_lowercase();
        groups.entry(folded).or_default().push((*file).to_owned());
    }
    groups
        .into_values()
        .filter(|group| group.len() > 1)
        .collect()
}

#[test]
fn scenario_file_names_do_not_collide_on_a_case_or_normalisation_insensitive_filesystem() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        outcomes: {A: {requires: {decision: x}}, a: {requires: {decision: y}}, \
        \"\\u00e9\": {requires: {decision: z}}, \"e\\u0301\": {requires: {decision: w}}}\n";
    match generated(source, "p.yaml") {
        // Refusing such a protocol is one acceptable answer.
        Err(_) => {}
        Ok(scenarios) => {
            let files: Vec<&str> = scenarios.iter().map(|s| s.file.as_str()).collect();
            assert_eq!(scenarios.len(), 4);
            let collided = collisions(&files);
            assert!(
                collided.is_empty(),
                "{} scenario(s) generated, but these file names name one file on a \
                 case- or normalisation-insensitive filesystem, so `canon generate` would \
                 overwrite one with the other and still report every scenario written: {collided:?}",
                scenarios.len()
            );
        }
    }
}

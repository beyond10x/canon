//! Adversary cases for story:scenario-generation, against the library.
//!
//! - Every protocol under `fixtures/investigation/` that generates: each scenario passes, it
//!   witnesses its subject (an outcome `legitimate`, an action `blocked`), and removing any one
//!   evidence record, authority decision or explicit decision fails it. Covers protocols with
//!   explicit decisions, authority requirements and subject-bound matches, not only the base
//!   protocol the acceptance uses.
//! - Each fixture's scenarios are committed (`COMMITTED`) with a digest of the fixtures they come
//!   from. The gate lane reads the committed scenarios and runs those checks on them, regenerates
//!   the fixtures that generate quickly and compares bytes, and checks that exactly the fixtures
//!   on `REFUSED` are refused. The `slow` lane (`task test-slow`) regenerates every fixture and
//!   compares bytes; `task regenerate-scenarios` rewrites the file.
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

/// A protocol whose outcomes read `after` over two kinds and over one, under `not` too, small
/// enough to check: `fixtures/investigation/evidence-order-after.yaml` reads the same kinds bound
/// to a subject and unbound at once, and its space is beyond the bound.
const AFTER: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {change: {}}\n\
    evidence_kinds: {tests_change: {}, test_result: {}}\n\
    claims:\n  \
      red: {true_when: {after: {first: tests_change, then: {kind: test_result, result: failed}}}}\n  \
      again: {true_when: {after: {first: test_result, then: {kind: test_result, result: failed}}}}\n\
    actions: {run: {may_produce: [{evidence: test_result}]}, \
    merge: {precondition: {claim: red}, requires: [{capability: cap}]}}\n\
    outcomes: {done: {requires: {claim: red}}, \
    flaky: {requires: {all: [{claim: again}, {not: {claim: red}}]}}}\n";

/// A protocol whose outcomes read `since` over two kinds, under `not` too, small
/// enough to check: `fixtures/investigation/evidence-order-since.yaml` adds an evidence match on
/// a result of the kind it compares, and its space is beyond the bound.
const SINCE: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {change: {}}\n\
    evidence_kinds: {tests_change: {}, test_result: {}}\n\
    claims:\n  \
      red: {true_when: {since: {last: tests_change, then: {kind: test_result, result: failed}}}}\n\
    actions: {run: {may_produce: [{evidence: test_result}]}, \
    merge: {precondition: {claim: red}, requires: [{capability: cap}]}}\n\
    outcomes: {done: {requires: {claim: red}}, \
    green: {requires: {not: {claim: red}}}}\n";

/// The committed scenarios of every fixture below `fixtures/investigation/` and of the three
/// protocols above, as `task regenerate-scenarios` writes them, below the repository root.
const COMMITTED: &str = "crates/canon/tests/generated/investigation-scenarios.txt";

/// The environment variable that makes the slow test write `COMMITTED` instead of comparing to it.
const REGENERATE: &str = "CANON_REGENERATE_SCENARIOS";

/// The fixtures written to be refused, each with the start of the refusal it gets. A fixture on
/// this list that generates and a fixture off it that is refused both fail the tests below; so
/// does a fixture refused for another reason than the list gives. Why each is refused:
///
/// - `state-space-bound`: the whole product is beyond the bound (`check/sliced` and
///   `check/unread-dimension`, story:check-subject-slices, are checked slice by slice, but
///   generating enumerates the whole product; `evidence-order-after.yaml` reads its kinds bound to
///   a subject and unbound at once, for CANON-ORDER-002's evaluate steps, `after.yaml` above is
///   the one generated from; `evidence-order-since.yaml`, for CANON-ORDER-004, likewise with
///   `since.yaml`);
/// - `unreachable-outcome`: an outcome is legitimate in no state;
/// - `nothing-to-witness`: no outcome and no action blocked in any state (coordinator decision 3,
///   adversary pass 1; `invalidation-rules.yaml`, from story:invalidation-rules, declares no
///   outcome or action either);
/// - `parse:` and `compile:`: written to be refused by the reader or by compile
///   (story:decision-condition);
/// - `compile: [MissingImport`: a document of `imports/` or `floor/` that imports others compiles
///   only with the documents it imports, which generation does not take (story:protocol-imports,
///   story:protocol-floor); `canon generate` refuses them before compiling.
const REFUSED: &[(&str, &str)] = &[
    (
        "fixtures/investigation/check/sliced/protocol.yaml",
        "state-space-bound",
    ),
    (
        "fixtures/investigation/check/state-space-bound/protocol.yaml",
        "state-space-bound",
    ),
    (
        "fixtures/investigation/check/unreachable-outcome/protocol.yaml",
        "unreachable-outcome",
    ),
    (
        "fixtures/investigation/check/unread-dimension/protocol.yaml",
        "state-space-bound",
    ),
    (
        "fixtures/investigation/decision-condition/decision-inside-when.yaml",
        "parse:",
    ),
    (
        "fixtures/investigation/decision-condition/evidence-in-when.yaml",
        "compile: [EvidenceInDecisionCondition",
    ),
    (
        "fixtures/investigation/decision-condition/when-beside-claim.yaml",
        "parse:",
    ),
    (
        "fixtures/investigation/evidence-order-after.yaml",
        "state-space-bound",
    ),
    (
        "fixtures/investigation/evidence-order-since.yaml",
        "state-space-bound",
    ),
    (
        "fixtures/investigation/floor/composed.yaml",
        "compile: [MissingImport",
    ),
    (
        "fixtures/investigation/floor/refines-alternative.yaml",
        "compile: [MissingImport",
    ),
    (
        "fixtures/investigation/floor/refines-conjunct.yaml",
        "compile: [MissingImport",
    ),
    (
        "fixtures/investigation/imports/falsification.yaml",
        "compile: [MissingImport",
    ),
    (
        "fixtures/investigation/imports/review.yaml",
        "compile: [MissingImport",
    ),
    (
        "fixtures/investigation/imports/root.yaml",
        "compile: [MissingImport",
    ),
    (
        "fixtures/investigation/invalidation-rules.yaml",
        "nothing-to-witness",
    ),
    (
        "fixtures/investigation/simulate/reachable-state-space-bound/protocol.yaml",
        "state-space-bound",
    ),
    (
        "fixtures/investigation/simulate/state-space-bound/protocol.yaml",
        "state-space-bound",
    ),
    (
        "fixtures/investigation/subject-bound-evidence-match.yaml",
        "nothing-to-witness",
    ),
];

/// The fixtures whose generation takes over 5 s in a debug build (a state space in the thousands,
/// measured 2026-11-22 on a loaded host: 7.8 to 17.9 s each, the other fixtures 2 s or less): only
/// the slow lane regenerates them and compares them with the committed file. The gate lane
/// regenerates every other fixture.
const SLOW_ONLY: &[&str] = &[
    "fixtures/investigation/child-outcome-subject.yaml",
    "fixtures/investigation/evidence-order-subsecond.yaml",
    "fixtures/investigation/missing-evidence.yaml",
    "after.yaml",
    "since.yaml",
];

/// One fixture's result: its scenarios as (file name, text), or the refusal on one line.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Generated {
    Scenarios(Vec<(String, String)>),
    Refused(String),
}

type Entry = (String, Generated);

/// Every protocol the tests read as (name, text): the documents under `fixtures/investigation`
/// and the three inline protocols above, in that order.
fn sources() -> Vec<(String, String)> {
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
    sources.push(("after.yaml".to_owned(), AFTER.to_owned()));
    sources.push(("since.yaml".to_owned(), SINCE.to_owned()));
    sources
}

/// The digest of the sources the committed scenarios were generated from: SHA-256 over each name
/// and text in order, each followed by a NUL. The generator's own code is not part of it: the slow
/// lane compares what the generator writes today with the committed file byte for byte.
fn digest(sources: &[(String, String)]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for (name, text) in sources {
        hasher.update(name.as_bytes());
        hasher.update([0]);
        hasher.update(text.as_bytes());
        hasher.update([0]);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn generate_one((fixture, source): &(String, String)) -> Entry {
    let result = match generated(source, fixture) {
        Ok(scenarios) => Generated::Scenarios(
            scenarios
                .into_iter()
                .map(|scenario| (scenario.file, scenario.text))
                .collect(),
        ),
        Err(why) => Generated::Refused(why.replace('\n', "\\n")),
    };
    (fixture.clone(), result)
}

/// Generates each of `sources` on its own, on four threads (`threads-required` in the `slow` lane
/// of `.config/nextest.toml`); the entries come back in the order of `sources`.
fn generate_all(sources: &[(String, String)]) -> Vec<Entry> {
    std::thread::scope(|scope| {
        let chunk = sources.len().div_ceil(4).max(1);
        let workers: Vec<_> = sources
            .chunks(chunk)
            .map(|part| scope.spawn(move || part.iter().map(generate_one).collect::<Vec<_>>()))
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| {
                worker
                    .join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect()
    })
}

/// The committed file: the digest line, the regeneration line, then per fixture a `=== fixture`
/// line followed by one `=== scenario <file>` line and the scenario's text for each scenario, or by
/// one `=== refused <why>` line.
fn render(digest: &str, entries: &[Entry]) -> String {
    let mut out = format!("digest: {digest}\nregenerate: task regenerate-scenarios\n");
    for (fixture, result) in entries {
        out.push_str(&format!("=== fixture {fixture}\n"));
        match result {
            Generated::Scenarios(scenarios) => {
                for (file, text) in scenarios {
                    out.push_str(&format!("=== scenario {file}\n{text}"));
                }
            }
            Generated::Refused(why) => out.push_str(&format!("=== refused {why}\n")),
        }
    }
    out
}

/// The digest and the entries of the committed file.
fn parse_committed(text: &str) -> (String, Vec<Entry>) {
    let mut lines = text.split_inclusive('\n');
    let digest = lines
        .next()
        .and_then(|line| line.strip_prefix("digest: "))
        .expect("the committed file starts with a digest line")
        .trim_end()
        .to_owned();
    let mut entries: Vec<Entry> = Vec::new();
    for line in lines {
        if let Some(fixture) = line.strip_prefix("=== fixture ") {
            entries.push((
                fixture.trim_end().to_owned(),
                Generated::Scenarios(Vec::new()),
            ));
        } else if let Some(file) = line.strip_prefix("=== scenario ") {
            if let Some((_, Generated::Scenarios(list))) = entries.last_mut() {
                list.push((file.trim_end().to_owned(), String::new()));
            }
        } else if let Some(why) = line.strip_prefix("=== refused ") {
            entries.last_mut().expect("a fixture line comes first").1 =
                Generated::Refused(why.trim_end().to_owned());
        } else if let Some((_, Generated::Scenarios(list))) = entries.last_mut()
            && let Some((_, scenario)) = list.last_mut()
        {
            scenario.push_str(line);
        }
    }
    (digest, entries)
}

fn committed() -> (String, Vec<Entry>) {
    let path = repository_root().join(COMMITTED);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "{} is unreadable ({error}); run `task regenerate-scenarios` to write it",
            path.display()
        )
    });
    parse_committed(&text)
}

/// What the file name `action.<id>.blocked.yaml` or `outcome.<id>.legitimate.yaml` says a scenario
/// witnesses: the decision section, the identifier and the status.
fn subject_of(file: &str) -> Option<(&'static str, &str, &'static str)> {
    if let Some(id) = file
        .strip_prefix("action.")
        .and_then(|rest| rest.strip_suffix(".blocked.yaml"))
    {
        return Some(("actions", id, "blocked"));
    }
    file.strip_prefix("outcome.")
        .and_then(|rest| rest.strip_suffix(".legitimate.yaml"))
        .map(|id| ("outcomes", id, "legitimate"))
}

/// What is wrong with `entries`, one line each, and how many scenarios and removals were run.
///
/// - a fixture on `REFUSED` that generates, one off it that is refused, or one refused for another
///   reason than the list gives;
/// - a scenario that does not parse or fails against its fixture;
/// - a scenario whose expectation does not give its subject the status its file name promises (an
///   outcome `legitimate`, an action `blocked`): a witness taken in a state where the subject has
///   another status still passes against the evaluator that wrote it;
/// - removing one evidence record, authority decision or explicit decision still passing.
fn problems(entries: &[Entry], sources: &[(String, String)]) -> (Vec<String>, usize, usize) {
    let mut failures = Vec::new();
    let (mut scenarios_run, mut removals_run) = (0, 0);
    for (fixture, result) in entries {
        let expected = REFUSED
            .iter()
            .find(|(path, _)| path == fixture)
            .map(|(_, start)| *start);
        let scenarios = match (expected, result) {
            (Some(start), Generated::Refused(why)) => {
                if !why.starts_with(start) {
                    failures.push(format!(
                        "{fixture}: refused as `{why}`, expected a refusal starting `{start}`"
                    ));
                }
                continue;
            }
            (Some(start), Generated::Scenarios(_)) => {
                failures.push(format!(
                    "{fixture}: on the list of expected refusals (`{start}`) but it generates"
                ));
                continue;
            }
            (None, Generated::Refused(why)) => {
                failures.push(format!("{fixture}: refused: {why}"));
                continue;
            }
            (None, Generated::Scenarios(scenarios)) => scenarios,
        };
        let source = &sources
            .iter()
            .find(|(name, _)| name == fixture)
            .expect("an entry is about one of the sources")
            .1;
        for (file, text) in scenarios {
            let parsed = conform::parse(text).expect("a generated scenario parses");
            scenarios_run += 1;
            if let Verdict::Failed { step, reason } = conform::run(&parsed, Ok(source)) {
                failures.push(format!("{fixture} {file}: fails `{step}`: {reason}"));
            }
            let status = match (subject_of(file), &parsed.steps[0].kind) {
                (
                    Some((section, id, wanted)),
                    StepKind::Evaluate {
                        expected: conform::EvaluateExpectation::Decision(decision),
                        ..
                    },
                ) => serde_json::from_str::<serde_json::Value>(decision)
                    .ok()
                    .map(|decision| (decision[section][id]["status"].clone(), wanted)),
                _ => None,
            };
            match status {
                Some((given, wanted)) if given == wanted => {}
                Some((given, wanted)) => failures.push(format!(
                    "{fixture} {file}: expects the status {given} of its subject, a witness must have `{wanted}`"
                )),
                None => failures.push(format!(
                    "{fixture} {file}: expects no status of its subject"
                )),
            }
            for (what, changed) in removals(&parsed) {
                removals_run += 1;
                if matches!(conform::run(&changed, Ok(source)), Verdict::Passed) {
                    failures.push(format!("{fixture} {file}: still passes without {what}"));
                }
            }
        }
    }
    (failures, scenarios_run, removals_run)
}

/// The fixtures `REFUSED` and `SLOW_ONLY` name that are not among `sources`.
fn stale_names(sources: &[(String, String)]) -> Vec<String> {
    REFUSED
        .iter()
        .map(|(path, _)| *path)
        .chain(SLOW_ONLY.iter().copied())
        .filter(|name| !sources.iter().any(|(source, _)| source == name))
        .map(str::to_owned)
        .collect()
}

#[test]
fn committed_scenarios_are_those_of_the_current_fixtures() {
    let sources = sources();
    let (committed_digest, entries) = committed();
    let names: Vec<&str> = sources.iter().map(|(name, _)| name.as_str()).collect();
    let listed: Vec<&str> = entries.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        committed_digest,
        digest(&sources),
        "a fixture under fixtures/investigation or an inline protocol of this file changed since \
         {COMMITTED} was written; run `task regenerate-scenarios` and commit the file"
    );
    assert_eq!(names, listed, "the committed file lists other fixtures");
    assert_eq!(
        stale_names(&sources),
        Vec::<String>::new(),
        "REFUSED or SLOW_ONLY names no fixture"
    );
}

#[test]
fn committed_scenarios_witness_their_subject_and_each_removal_fails_them() {
    let (_, entries) = committed();
    let (failures, scenarios_run, removals_run) = problems(&entries, &sources());
    assert!(scenarios_run > 20, "{scenarios_run} scenarios ran");
    assert!(removals_run > 10, "{removals_run} removals ran");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_fixtures_the_gate_lane_regenerates_give_the_committed_bytes() {
    let sources = sources();
    let (_, entries) = committed();
    let quick: Vec<(String, String)> = sources
        .iter()
        .filter(|(name, _)| !SLOW_ONLY.contains(&name.as_str()))
        .cloned()
        .collect();
    let fresh = generate_all(&quick);
    let (failures, _, _) = problems(&fresh, &sources);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    let differ: Vec<&str> = fresh
        .iter()
        .filter(|entry| !entries.contains(entry))
        .map(|(name, _)| name.as_str())
        .collect();
    assert!(
        differ.is_empty(),
        "the generator writes other scenarios than {COMMITTED} holds for {differ:?}; run \
         `task regenerate-scenarios` if that is meant"
    );
}

/// The slow lane: every fixture is generated again, each scenario passes, witnesses its subject and
/// fails without any one of its records, and the result is `COMMITTED` byte for byte. With
/// `CANON_REGENERATE_SCENARIOS` set it writes the file instead (`task regenerate-scenarios`), when
/// nothing is wrong.
#[test]
fn every_generated_witness_passes_and_is_minimal_over_every_investigation_protocol() {
    let sources = sources();
    let fresh = generate_all(&sources);
    let (failures, scenarios_run, removals_run) = problems(&fresh, &sources);
    assert!(scenarios_run > 20, "{scenarios_run} scenarios ran");
    assert!(removals_run > 10, "{removals_run} removals ran");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(
        stale_names(&sources),
        Vec::<String>::new(),
        "REFUSED or SLOW_ONLY names no fixture"
    );
    let text = render(&digest(&sources), &fresh);
    let path = repository_root().join(COMMITTED);
    if std::env::var_os(REGENERATE).is_some() {
        std::fs::create_dir_all(path.parent().expect("a directory")).expect("created");
        std::fs::write(&path, &text).expect("written");
        return;
    }
    let held = std::fs::read_to_string(&path).unwrap_or_default();
    let (_, held_entries) = parse_committed(&held);
    let differ: Vec<&str> = fresh
        .iter()
        .filter(|entry| !held_entries.contains(entry))
        .map(|(name, _)| name.as_str())
        .collect();
    assert!(
        differ.is_empty() && held == text,
        "the generator writes other scenarios than {COMMITTED} holds (fixtures {differ:?}); run \
         `task regenerate-scenarios` if that is meant"
    );
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

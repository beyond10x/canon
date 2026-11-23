//! The checker reads each state's action and outcome statuses only, and builds no explanation for
//! a state (story:check-evaluates-statuses-only): the statuses it reads equal those of the full
//! decision on every state it enumerates of the protocol fixtures, and a check builds no
//! explanation.

use std::path::{Path, PathBuf};

use super::*;
use crate::model::parse;

/// The fixtures above [`FAST_STATES`], in the `slow` lane.
mod slow;

/// The largest whole product of a fixture this module's test compares state by state: about 90 of
/// the 136 fixtures that compile on their own, a few seconds. Each state is evaluated twice, and
/// the larger fixtures hold up to 17,160 states the checker enumerates.
const FAST_STATES: u128 = 64;

/// The repository's `fixtures/` directory.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// Every `protocol/1` document under `fixtures/` that compiles on its own, in path order, and how
/// many `protocol/1` documents there are.
fn compiled_fixtures() -> (Vec<(PathBuf, Ir)>, usize) {
    let mut found = Vec::new();
    let mut pending = vec![fixtures()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("a fixture directory is readable") {
            let path = entry.expect("an entry is readable").path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "yaml")
                && std::fs::read_to_string(&path)
                    .is_ok_and(|text| text.lines().any(|line| line == "format: protocol/1"))
            {
                found.push(path);
            }
        }
    }
    found.sort();
    let documents = found.len();
    let compiled = found
        .into_iter()
        .filter_map(|path| {
            let text = std::fs::read_to_string(&path).ok()?;
            let ir = crate::ir::compile(&parse(&text).ok()?).ok()?;
            Some((path, ir))
        })
        .collect();
    (compiled, documents)
}

/// The `properties.yaml` beside `path`, when there is one and it reads.
fn properties_beside(path: &Path) -> Option<Properties> {
    let text = std::fs::read_to_string(path.with_file_name("properties.yaml")).ok()?;
    read_properties(&text).ok()
}

/// The statuses of `state` of `space` read from the full decision, explanation and all.
fn full(ir: &Ir, space: &Space<'_>, state: &State) -> Evaluated {
    evaluate_records_by(ir, space, state, space.evidence(state), eval::evaluate_with)
        .expect("a state the checker evaluated is evaluated")
}

/// The statuses the checker read of `state` of `space`, `read`, equal those of the full decision.
fn agrees(ir: &Ir, space: &Space<'_>, state: &State, read: &Evaluated) {
    let decided = full(ir, space, state);
    assert_eq!(read.actions, decided.actions, "{}", space.render(state));
    assert_eq!(read.outcomes, decided.outcomes, "{}", space.render(state));
    assert_eq!(read.denied, decided.denied, "{}", space.render(state));
}

/// How many states of `ir` under `properties` were compared: each state of each space `canon
/// check` enumerates (the spaces `canon generate` reads, without properties), and each probe when
/// every space is a slice, its statuses as the checker read them against those of the full
/// decision. `None` when the check is refused.
fn compare(ir: &Ir, properties: Option<&Properties>) -> Option<usize> {
    let (_, enumerations) = check_keeping(ir, properties, Strategy::Chosen).ok()?;
    let whole = Space::new(ir);
    let mut compared = 0;
    for enumeration in &enumerations {
        let sliced;
        let space = match &enumeration.slice {
            Some(slice) => {
                sliced = whole.slice(slice);
                &sliced
            }
            None => &whole,
        };
        for (state, read) in enumeration.states.iter().zip(&enumeration.evaluated) {
            agrees(ir, space, state, read);
            compared += 1;
        }
    }
    if enumerations.iter().all(|e| e.slice.is_some()) {
        for probe in whole.probes() {
            let read = evaluate(ir, &whole, &probe).expect("a probe the checker evaluated");
            agrees(ir, &whole, &probe, &read);
            compared += 1;
        }
    }
    Some(compared)
}

/// How many states of the fixture `ir` at `path` were compared: without properties and, when a
/// `properties.yaml` lies beside it, with them. `None` when both checks are refused.
fn compare_fixture(path: &Path, ir: &Ir) -> Option<usize> {
    let properties = properties_beside(path);
    let alone = compare(ir, None);
    let with = properties.and_then(|properties| compare(ir, Some(&properties)));
    match (alone, with) {
        (None, None) => None,
        (alone, with) => Some(alone.unwrap_or_default() + with.unwrap_or_default()),
    }
}

/// Every state the checker enumerates of every `protocol/1` fixture that compiles on its own and
/// whose whole product holds at most [`FAST_STATES`] states: the statuses the checker reads equal
/// those of the full decision.
#[test]
fn the_checker_reads_the_statuses_of_the_full_decision_in_every_state_of_a_small_fixture() {
    let (compiled, documents) = compiled_fixtures();
    let small: Vec<&(PathBuf, Ir)> = compiled
        .iter()
        .filter(|(_, ir)| Space::new(ir).size().is_some_and(|n| n <= FAST_STATES))
        .collect();
    let mut checked = 0;
    let mut states = 0;
    for (path, ir) in &small {
        if let Some(compared) = compare_fixture(path, ir) {
            checked += 1;
            states += compared;
        }
    }
    eprintln!(
        "{documents} protocol fixtures, {} compile, {} small, {checked} checked, {states} states \
         compared",
        compiled.len(),
        small.len()
    );
    assert!(documents >= 173, "found {documents} protocol fixtures");
    assert!(checked >= 80, "only {checked} small fixtures were checked");
}

/// `canon check` on the shipped 48-state fixture, with its properties, builds no explanation; the
/// full decision of one of its states builds one, so the count counts.
#[test]
fn the_checker_builds_no_explanation() {
    let path = fixtures().join("investigation/check/base/protocol.yaml");
    let text = std::fs::read_to_string(&path).expect("the fixture is readable");
    let mut ir = crate::ir::compile(&parse(&text).expect("parses")).expect("compiles");
    let mut properties = properties_beside(&path).expect("the fixture's properties read");
    // An id no other test gives its protocol, so its count is this test's alone.
    let id = ProtocolId::new("check-evaluates-statuses-only");
    ir.protocol.id = id.clone();
    properties.protocol = id.clone();
    let count = || {
        eval::EXPLAINED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(id.as_str())
            .copied()
            .unwrap_or_default()
    };

    let report = check(&ir, Some(&properties)).expect("the fixture is checked");
    assert_eq!(report.states, 48);
    assert_eq!(count(), 0, "explanations built by `canon check`");

    let whole = Space::new(&ir);
    full(&ir, &whole, &whole.state(0));
    assert_eq!(count(), 1, "the full decision builds its explanation");
}

//! The fixtures above [`FAST_STATES`], state by state, in the `slow` lane (`.config/nextest.toml`):
//! the statuses the checker reads equal those of the full decision in every state it enumerates.
//! Four tests, two by directory and the other fixtures in two halves, each on [`WORKERS`] threads.

use super::*;

/// The threads each test runs on: the `threads-required` `.config/nextest.toml` gives the tests
/// of the `slow` lane.
const WORKERS: usize = 4;

/// The states of one enumeration given to one thread at a time.
const CHUNK: usize = 256;

/// `f` of every item, in item order, computed on [`WORKERS`] threads, each taking every
/// [`WORKERS`]th item.
fn par<'t, T: Sync, R: Send>(items: &'t [T], f: impl Fn(&'t T) -> R + Sync) -> Vec<R> {
    let mut results: Vec<Option<R>> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..WORKERS)
            .map(|first| {
                let f = &f;
                scope.spawn(move || {
                    (first..items.len())
                        .step_by(WORKERS)
                        .map(|at| (at, f(&items[at])))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        let mut results: Vec<Option<R>> = (0..items.len()).map(|_| None).collect();
        for worker in workers {
            let done = worker
                .join()
                .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
            for (at, result) in done {
                results[at] = Some(result);
            }
        }
        results
    });
    results
        .iter_mut()
        .map(|r| r.take().expect("done"))
        .collect()
}

/// One check of a fixture: its slices, each with its states and the statuses the checker read,
/// taken out of the enumerations on the thread that made them, and the probes compared.
struct Checked<'f> {
    ir: &'f Ir,
    spaces: Vec<(Option<Slice>, Vec<State>, Vec<Evaluated>)>,
    probes: usize,
}

/// The check of `ir` under `properties`, with the probes compared on the way; `None` when it is
/// refused.
fn checked<'f>(ir: &'f Ir, properties: Option<&Properties>) -> Option<Checked<'f>> {
    let (_, enumerations) = check_keeping(ir, properties, Strategy::Chosen).ok()?;
    let mut probes = 0;
    if enumerations.iter().all(|e| e.slice.is_some()) {
        let whole = Space::new(ir);
        for probe in whole.probes() {
            let read = evaluate(ir, &whole, &probe).expect("a probe the checker evaluated");
            agrees(ir, &whole, &probe, &read);
            probes += 1;
        }
    }
    let spaces = enumerations
        .into_iter()
        .map(|e| {
            let Enumeration {
                slice,
                states,
                evaluated,
                ..
            } = e;
            (slice, states, evaluated)
        })
        .collect();
    Some(Checked { ir, spaces, probes })
}

/// Which large fixtures a test of this module compares.
#[derive(Clone, Copy)]
enum Group {
    /// The fixtures under this directory of `fixtures/`.
    Under(&'static str),
    /// Of the fixtures under no directory [`NAMED`] holds, in path order, every second one from
    /// this index.
    Other(usize),
}

/// The directories of `fixtures/` a test of their own compares.
const NAMED: [&str; 2] = ["record-group-large", "child-outcome-large"];

/// Compares every state the checker enumerates of each fixture of `group` whose whole product is
/// above [`FAST_STATES`], without properties and with those beside it; returns how many fixtures
/// were checked and how many states compared.
fn compare_large(group: Group) -> (usize, usize) {
    let started = std::time::Instant::now();
    let (compiled, _) = compiled_fixtures();
    let root = fixtures();
    let under = |path: &Path| -> String {
        path.strip_prefix(&root)
            .ok()
            .and_then(|rest| rest.iter().next())
            .and_then(|first| first.to_str())
            .unwrap_or_default()
            .to_owned()
    };
    let large = compiled
        .into_iter()
        .filter(|(_, ir)| Space::new(ir).size().is_none_or(|n| n > FAST_STATES));
    let ours: Vec<(PathBuf, Ir, Option<Properties>)> = match group {
        Group::Under(dir) => large
            .filter(|(path, _)| under(path) == dir)
            .collect::<Vec<_>>(),
        Group::Other(first) => large
            .filter(|(path, _)| !NAMED.contains(&under(path).as_str()))
            .skip(first)
            .step_by(2)
            .collect(),
    }
    .into_iter()
    .map(|(path, ir)| {
        let properties = properties_beside(&path);
        (path, ir, properties)
    })
    .collect();
    let mut runs: Vec<(usize, &Ir, Option<&Properties>)> = Vec::new();
    for (at, (_, ir, properties)) in ours.iter().enumerate() {
        runs.push((at, ir, None));
        if let Some(properties) = properties {
            runs.push((at, ir, Some(properties)));
        }
    }
    let checks: Vec<(usize, Option<Checked<'_>>)> = par(&runs, |(at, ir, properties)| {
        (*at, checked(ir, *properties))
    });
    let fixtures = checks
        .iter()
        .filter_map(|(at, c)| c.as_ref().map(|_| at))
        .collect::<BTreeSet<_>>()
        .len();
    let probes: usize = checks.iter().flat_map(|(_, c)| c).map(|c| c.probes).sum();
    let mut work: Vec<(&Checked<'_>, usize, std::ops::Range<usize>)> = Vec::new();
    for checked in checks.iter().flat_map(|(_, c)| c) {
        for (at, (_, states, _)) in checked.spaces.iter().enumerate() {
            for start in (0..states.len()).step_by(CHUNK) {
                work.push((checked, at, start..(start + CHUNK).min(states.len())));
            }
        }
    }
    let compared: usize = par(&work, |(checked, at, range)| {
        let (slice, states, evaluated) = &checked.spaces[*at];
        let whole = Space::new(checked.ir);
        let sliced;
        let space = match slice {
            Some(slice) => {
                sliced = whole.slice(slice);
                &sliced
            }
            None => &whole,
        };
        for index in range.clone() {
            agrees(checked.ir, space, &states[index], &evaluated[index]);
        }
        range.len()
    })
    .into_iter()
    .sum();
    eprintln!(
        "{} large fixtures, {fixtures} checked, {} states compared in {:?}",
        ours.len(),
        compared + probes,
        started.elapsed()
    );
    (fixtures, compared + probes)
}

/// Every state the checker enumerates of the fixtures under `record-group-large/`.
#[test]
fn the_checker_reads_the_full_statuses_in_every_state_of_record_group_large() {
    let (fixtures, states) = compare_large(Group::Under("record-group-large"));
    assert!(
        fixtures >= 1 && states >= 10_000,
        "{fixtures} fixtures, {states} states"
    );
}

/// Every state the checker enumerates of the fixtures under `child-outcome-large/`.
#[test]
fn the_checker_reads_the_full_statuses_in_every_state_of_child_outcome_large() {
    let (fixtures, states) = compare_large(Group::Under("child-outcome-large"));
    assert!(
        fixtures >= 2 && states >= 10_000,
        "{fixtures} fixtures, {states} states"
    );
}

/// Every state the checker enumerates of every other fixture above [`FAST_STATES`], the first of
/// two halves.
#[test]
fn the_checker_reads_the_full_statuses_in_every_state_of_another_large_fixture_0() {
    let (fixtures, states) = compare_large(Group::Other(0));
    assert!(
        fixtures >= 15 && states >= 5_000,
        "{fixtures} fixtures, {states} states"
    );
}

/// Every state the checker enumerates of every other fixture above [`FAST_STATES`], the second of
/// two halves.
#[test]
fn the_checker_reads_the_full_statuses_in_every_state_of_another_large_fixture_1() {
    let (fixtures, states) = compare_large(Group::Other(1));
    assert!(
        fixtures >= 15 && states >= 5_000,
        "{fixtures} fixtures, {states} states"
    );
}

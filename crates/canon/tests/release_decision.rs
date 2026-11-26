//! story:musl-workload-measurement: the measurement of the gnu, musl and musl-with-mimalloc
//! builds committed in `docs/design/linux-release.md` decides two things, and these tests hold
//! the tree to them without building or measuring anything. The runs are read from the document
//! (`run <workload> <build> <index> <seconds> <peak RSS KiB>`, as `scripts/measure-release.sh`
//! prints them), the medians are computed again here, and the rule the document states is applied:
//!
//! - the musl archive links mimalloc only when its median is not above the system allocator's on
//!   every workload (`MUSL_ALLOCATOR` in `scripts/package.sh`);
//! - the installation page names the musl archive first only when the chosen musl build's median
//!   is at most 5 % above gnu's on every workload, else the gnu archive first.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const WORKLOADS: [&str; 2] = ["conform", "check"];
const BUILDS: [&str; 3] = ["gnu", "musl", "musl-mimalloc"];
const RUNS: usize = 10;

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("run this test through cargo"))
        .join("../..")
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|error| panic!("{path} reads: {error}"))
}

/// The wall times in seconds of each (workload, build), in the order the document lists them.
fn runs(doc: &str) -> BTreeMap<(String, String), Vec<f64>> {
    let mut runs: BTreeMap<(String, String), Vec<f64>> = BTreeMap::new();
    for line in doc.lines().filter(|line| line.starts_with("run ")) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 6, "a run line has six fields: {line}");
        assert!(WORKLOADS.contains(&fields[1]), "a known workload: {line}");
        assert!(BUILDS.contains(&fields[2]), "a known build: {line}");
        let seconds: f64 = fields[4]
            .parse()
            .unwrap_or_else(|_| panic!("seconds: {line}"));
        let _kib: u64 = fields[5].parse().unwrap_or_else(|_| panic!("KiB: {line}"));
        runs.entry((fields[1].to_owned(), fields[2].to_owned()))
            .or_default()
            .push(seconds);
    }
    runs
}

/// The median of an even or odd number of values: the middle one, or the mean of the two middle
/// ones.
fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len();
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

fn medians() -> BTreeMap<(String, String), f64> {
    runs(&read("docs/design/linux-release.md"))
        .into_iter()
        .map(|(key, values)| (key, median(&values)))
        .collect()
}

fn at(medians: &BTreeMap<(String, String), f64>, workload: &str, build: &str) -> f64 {
    *medians
        .get(&(workload.to_owned(), build.to_owned()))
        .unwrap_or_else(|| panic!("the document holds runs of {build} on {workload}"))
}

/// mimalloc is used when its median is not above the system allocator's on every workload.
fn mimalloc_chosen(medians: &BTreeMap<(String, String), f64>) -> bool {
    WORKLOADS
        .iter()
        .all(|w| at(medians, w, "musl-mimalloc") <= at(medians, w, "musl"))
}

/// musl is the default when the chosen musl build's median is at most 5 % above gnu's on every
/// workload.
fn musl_default(medians: &BTreeMap<(String, String), f64>) -> bool {
    let musl = if mimalloc_chosen(medians) {
        "musl-mimalloc"
    } else {
        "musl"
    };
    WORKLOADS
        .iter()
        .all(|w| at(medians, w, musl) <= 1.05 * at(medians, w, "gnu"))
}

/// The document holds 10 runs of each build on each workload, the commit measured, and a summary
/// row per build and workload whose median is the median of those runs.
#[test]
fn the_document_holds_ten_runs_per_build_and_workload() {
    let doc = read("docs/design/linux-release.md");
    let runs = runs(&doc);
    for workload in WORKLOADS {
        for build in BUILDS {
            let found = runs
                .get(&(workload.to_owned(), build.to_owned()))
                .map_or(0, Vec::len);
            assert_eq!(found, RUNS, "{RUNS} runs of {build} on {workload}");
        }
    }
    let commit = doc
        .lines()
        .find_map(|line| line.strip_prefix("Commit measured: `"))
        .and_then(|rest| rest.strip_suffix('`'))
        .expect("a line `Commit measured: `<sha>``");
    assert!(
        commit.len() == 40 && commit.chars().all(|c| c.is_ascii_hexdigit()),
        "the commit measured is a full SHA-1: {commit}"
    );
    for ((workload, build), values) in &runs {
        let row = format!("| {workload} | {build} | {} |", values.len());
        let line = doc
            .lines()
            .find(|line| line.starts_with(&row))
            .unwrap_or_else(|| panic!("a summary row starting `{row}`"));
        let shown: f64 = line
            .split('|')
            .nth(4)
            .unwrap_or_default()
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("the fourth column of `{line}` is the median in seconds"));
        assert!(
            (shown - median(values)).abs() < 0.0015,
            "the median of {build} on {workload} is {:.3}, the table says {shown}",
            median(values)
        );
    }
}

/// The musl archive links mimalloc exactly when the table shows it not slower.
#[test]
fn the_musl_archive_links_mimalloc_only_when_the_table_shows_it_not_slower() {
    let expected = if mimalloc_chosen(&medians()) {
        "mimalloc"
    } else {
        "system"
    };
    let script = read("scripts/package.sh");
    let set = script
        .lines()
        .find_map(|line| line.strip_prefix("MUSL_ALLOCATOR="))
        .expect("scripts/package.sh sets MUSL_ALLOCATOR=");
    assert_eq!(
        set, expected,
        "the table decides MUSL_ALLOCATOR={expected} in scripts/package.sh"
    );
}

/// The installation page names the default archive first.
#[test]
fn the_installation_page_names_the_default_first() {
    let page = read("website/docs/getting-started.md");
    let section = page
        .split("\n## Linux binaries\n")
        .nth(1)
        .expect("getting-started has ## Linux binaries")
        .split("\n## ")
        .next()
        .unwrap_or_default();
    let gnu = section
        .find("x86_64-unknown-linux-gnu")
        .expect("the section names the gnu archive");
    let musl = section
        .find("x86_64-unknown-linux-musl")
        .expect("the section names the musl archive");
    if musl_default(&medians()) {
        assert!(
            musl < gnu,
            "the table makes musl the default; name it first"
        );
    } else {
        assert!(gnu < musl, "the table makes gnu the default; name it first");
    }
}

/// mimalloc is a feature of `canon-cli`, off by default, pinned in Cargo.lock, set as the global
/// allocator only under that feature, and asked for by the musl build alone.
#[test]
fn mimalloc_is_an_optional_feature_of_the_musl_build_only() {
    let manifest = read("crates/canon-cli/Cargo.toml");
    assert!(
        manifest.contains("\n[features]\nmimalloc = [\"dep:mimalloc\"]\n"),
        "canon-cli declares the feature `mimalloc = [\"dep:mimalloc\"]`"
    );
    assert!(
        !manifest.lines().any(|line| line.starts_with("default =")),
        "canon-cli has no default feature, so mimalloc is off by default"
    );
    let dependency = manifest
        .lines()
        .find(|line| line.starts_with("mimalloc = {"))
        .expect("canon-cli depends on mimalloc");
    assert!(dependency.contains("optional = true"), "{dependency}");

    let lock = read("Cargo.lock");
    let pinned: Vec<&str> = lock
        .split("[[package]]\n")
        .filter(|entry| entry.starts_with("name = \"mimalloc\"\n"))
        .collect();
    assert_eq!(pinned.len(), 1, "Cargo.lock pins one mimalloc");

    let main = read("crates/canon-cli/src/main.rs");
    assert!(
        main.contains(
            "#[cfg(feature = \"mimalloc\")]\n#[global_allocator]\nstatic GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;\n"
        ),
        "main.rs sets mimalloc as the global allocator under the feature only"
    );

    let script = read("scripts/package.sh");
    assert_eq!(
        script.matches("--features mimalloc").count(),
        1,
        "package.sh asks for the feature in one place, the musl build"
    );
    let out = Command::new("sh")
        .arg(root().join("scripts/package.sh"))
        .args([
            "binary",
            "x86_64-unknown-linux-gnu",
            "mimalloc",
            "target/never-built",
        ])
        .current_dir(root())
        .stdin(Stdio::null())
        .output()
        .expect("sh runs scripts/package.sh");
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !out.status.success() && said.contains("only the musl build"),
        "a gnu build with mimalloc is refused before anything is built:\n{said}"
    );
    assert!(!root().join("target/never-built").exists());
}

/// The rule is applied as written: the medians decide, and 5 % is the bound.
#[test]
fn the_rule_reads_medians() {
    let table = |rows: &[(&str, &str, f64)]| -> BTreeMap<(String, String), f64> {
        rows.iter()
            .map(|(w, b, m)| ((w.to_string(), b.to_string()), *m))
            .collect()
    };
    let close = table(&[
        ("conform", "gnu", 1.00),
        ("conform", "musl", 1.10),
        ("conform", "musl-mimalloc", 1.04),
        ("check", "gnu", 2.00),
        ("check", "musl", 2.30),
        ("check", "musl-mimalloc", 2.10),
    ]);
    assert!(mimalloc_chosen(&close));
    assert!(musl_default(&close));
    let slower = table(&[
        ("conform", "gnu", 1.00),
        ("conform", "musl", 1.10),
        ("conform", "musl-mimalloc", 1.04),
        ("check", "gnu", 2.00),
        ("check", "musl", 2.30),
        ("check", "musl-mimalloc", 2.31),
    ]);
    assert!(!mimalloc_chosen(&slower));
    assert!(!musl_default(&slower));
    assert_eq!(median(&[3.0, 1.0, 2.0, 4.0]), 2.5);
    assert_eq!(median(&[3.0, 1.0, 2.0]), 2.0);
}

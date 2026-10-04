//! The Atlas ADR 0076 hard gate on Canon's ESS specification under `ess/`.
//!
//! Four steps, each of which fails `task check`: `ess specify validate --strict-requires`,
//! `ess specify compile`, `ess verify conform synthesize` with 0 refusals, and no `UNMAPPED:`
//! anywhere under `ess/`. The last step is this file's own scan, because ess 0.52.0 does not parse
//! comments; it goes when the ESS release that refuses open questions is pinned.
//!
//! [`gate`] runs the four steps in order on any specification directory. The cases below run it on
//! `ess/` and on copies of `ess/` broken so that exactly one step must fail.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The ESS release `ess/ess-inputs.yaml` pins and CI installs.
const PINNED: &str = "0.52.0";

/// The marker of an open question, written as two halves so this file does not carry it whole.
const OPEN_QUESTION: &str = concat!("UNMAPPED", ":");

/// A command whose `never` outcome no input satisfies. ess 0.52.0 refuses it (`ESS-SYNTH-003`) and
/// still exits 0, so the refusal count on the summary line is the only signal that step 3 does not
/// hold. Observed 2026-10-04 on a copy of `ess/`: `1 scenario(s) (0 authored), 1 refusal(s),
/// written to …`, exit 0.
const UNSATISFIABLE_COMMAND: &str = "
commands:
  - name: canon.protocol.Probe
    naming:
      wire: probe
      display: Probe
    input:
      - name: truth
        type: canon.protocol.Truth
    outcomes:
      - name: never
        when:
          all:
            - truth == unknown
            - truth != unknown
        emits: [canon.protocol.Probed]
        payload:
          canon.protocol.Probed:
            truth: input.truth
      - name: always
        emits: [canon.protocol.Probed]
        payload:
          canon.protocol.Probed:
            truth: input.truth

events:
  - name: canon.protocol.Probed
    naming:
      wire: probed
      display: Probed
    fields:
      - name: truth
        type: canon.protocol.Truth
";

/// The tree under test, read at run time: a test binary reused from a shared target directory
/// would otherwise check the tree it was built from.
fn repo_root() -> PathBuf {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect(
        "CARGO_MANIFEST_DIR is unset: run this test through cargo, which sets it to the \
         crates/canon directory of the tree under test",
    );
    Path::new(&manifest)
        .join("../..")
        .canonicalize()
        .expect("repository root resolves")
}

fn spec_dir() -> PathBuf {
    repo_root().join("ess")
}

/// A fresh directory under `CARGO_TARGET_TMPDIR` for one case, distinct per tree under test.
fn scratch(case: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    repo_root().hash(&mut hasher);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("canon-ess-gate")
        .join(format!("{:016x}", hasher.finish()))
        .join(case);
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("remove the previous scratch copy");
    }
    fs::create_dir_all(&dir).expect("create scratch");
    dir
}

/// A copy of `ess/` at `<scratch>/ess`, so the gate runs on it exactly as on the real one.
fn spec_copy(case: &str) -> PathBuf {
    let copy = scratch(case).join("ess");
    copy_dir(&spec_dir(), &copy);
    copy
}

/// Runs `ess` in `dir`. Fails, rather than skips, when `ess` is not on `PATH`.
fn ess(dir: &Path, args: &[&str]) -> Output {
    match Command::new("ess").args(args).current_dir(dir).output() {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => panic!(
            "`ess` is not on PATH: the ESS gate needs ess {PINNED}, the release ess/ess-inputs.yaml pins \
             (https://github.com/beyond10x/ess/releases/tag/{PINNED})"
        ),
        Err(error) => panic!("cannot run `ess {}`: {error}", args.join(" ")),
    }
}

/// Runs `ess <args> --path <spec dir name>` from the directory holding `spec`: for `ess/` that is
/// the repository root and `--path ess`. Its stdout when it exits 0.
fn ess_on(spec: &Path, args: &[&str], tail: &[&str]) -> Result<String, String> {
    let parent = spec.parent().expect("spec dir has a parent");
    let name = spec
        .file_name()
        .and_then(|name| name.to_str())
        .expect("spec dir name is UTF-8");
    let mut all: Vec<&str> = args.to_vec();
    all.extend(["--path", name]);
    all.extend(tail);
    let output = ess(parent, &all);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!(
            "`ess {}` exited {}\nstdout:\n{stdout}\nstderr:\n{}",
            all.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Step 1: `ess specify validate --path <spec> --strict-requires` exits 0.
fn step_validate(spec: &Path) -> Result<(), String> {
    ess_on(spec, &["specify", "validate"], &["--strict-requires"]).map(drop)
}

/// Step 2: `ess specify compile --path <spec>` exits 0.
fn step_compile(spec: &Path) -> Result<(), String> {
    ess_on(spec, &["specify", "compile"], &[]).map(drop)
}

/// Where step 3 writes the suite of `spec`: under `CARGO_TARGET_TMPDIR`, never in the tree.
fn suite_path(spec: &Path) -> PathBuf {
    let mut tree = DefaultHasher::new();
    repo_root().hash(&mut tree);
    let mut dir = DefaultHasher::new();
    spec.hash(&mut dir);
    let suites = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("canon-ess-gate")
        .join(format!("{:016x}", tree.finish()))
        .join("suites");
    fs::create_dir_all(&suites).expect("create the suites directory");
    suites.join(format!("{:016x}-canon-ess-suite.json", dir.finish()))
}

/// Step 3: `ess verify conform synthesize --path <spec>` exits 0 and its summary reports 0
/// refusals.
fn step_synthesize(spec: &Path) -> Result<(), String> {
    let out = suite_path(spec);
    let out = out.to_str().expect("suite path is UTF-8");
    let stdout = ess_on(spec, &["verify", "conform", "synthesize"], &["--out", out])?;
    match refusal_count(&stdout) {
        Some(0) => Ok(()),
        Some(count) => Err(format!(
            "`ess verify conform synthesize` exited 0 and reported {count} refusal(s):\n{stdout}"
        )),
        None => Err(format!(
            "`ess verify conform synthesize` printed no refusal count:\n{stdout}"
        )),
    }
}

/// Step 4: no file under `spec` carries an open-question marker.
fn step_no_open_question(spec: &Path) -> Result<(), String> {
    let found = open_questions(spec);
    if found.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} declares {} open question(s); Atlas ADR 0076 allows none:\n{}",
            spec.display(),
            found.len(),
            found.join("\n")
        ))
    }
}

/// The four steps in order; the first failure, naming its step.
fn gate(spec: &Path) -> Result<(), String> {
    step_validate(spec).map_err(|e| format!("step 1 (validate --strict-requires): {e}"))?;
    step_compile(spec).map_err(|e| format!("step 2 (compile): {e}"))?;
    step_synthesize(spec).map_err(|e| format!("step 3 (synthesize, 0 refusals): {e}"))?;
    step_no_open_question(spec).map_err(|e| format!("step 4 (no open question): {e}"))
}

/// The refusal count of `ess verify conform synthesize`'s summary, which is its last line naming
/// `refusal(s)`: `N scenario(s) (M authored), R refusal(s), written to …`. Detail lines printed
/// before the summary may quote other counts.
fn refusal_count(stdout: &str) -> Option<u64> {
    let summary = stdout
        .lines()
        .rev()
        .find(|line| line.contains("refusal(s)"))?;
    let before = &summary[..summary.rfind("refusal(s)")?];
    before
        .split(|c: char| c.is_whitespace() || c == ',')
        .rfind(|token| !token.is_empty())?
        .parse()
        .ok()
}

/// Every line under `dir` that carries an open-question marker, as `path:line: text`, in path
/// order.
fn open_questions(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_files(dir, &mut files);
    files.sort();
    let mut found = Vec::new();
    for file in files {
        let bytes =
            fs::read(&file).unwrap_or_else(|error| panic!("read {}: {error}", file.display()));
        let text = String::from_utf8_lossy(&bytes);
        for (index, line) in text.lines().enumerate() {
            if line.contains(OPEN_QUESTION) {
                found.push(format!("{}:{}: {}", file.display(), index + 1, line.trim()));
            }
        }
    }
    found
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|error| panic!("read dir {}: {error}", dir.display()));
    for entry in entries {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|error| panic!("create {}: {error}", to.display()));
    for entry in
        fs::read_dir(from).unwrap_or_else(|error| panic!("read {}: {error}", from.display()))
    {
        let path = entry.expect("directory entry").path();
        let target = to.join(path.file_name().expect("entry has a name"));
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            fs::copy(&path, &target)
                .unwrap_or_else(|error| panic!("copy {}: {error}", path.display()));
        }
    }
}

fn append(path: &Path, text: &str) -> usize {
    let mut content = fs::read_to_string(path).expect("file reads");
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(text);
    fs::write(path, &content).expect("file written");
    content.lines().count()
}

fn replace(path: &Path, from: &str, to: &str) {
    let content = fs::read_to_string(path).expect("file reads");
    assert!(
        content.contains(from),
        "`{from}` is not in {}",
        path.display()
    );
    fs::write(path, content.replacen(from, to, 1)).expect("file written");
}

fn expect_failure_at(spec: &Path, step: &str, needles: &[&str]) {
    let error = gate(spec).expect_err("the gate refuses the broken copy");
    assert!(
        error.starts_with(step) && needles.iter().all(|needle| error.contains(needle)),
        "expected the gate to fail at `{step}` naming {needles:?}, found:\n{error}"
    );
}

/// ADR 0076 step 1 (expectation 1).
#[test]
fn validate_with_strict_requires_exits_0() {
    step_validate(&spec_dir()).unwrap_or_else(|e| panic!("{e}"));
}

/// ADR 0076 step 2 (expectation 2).
#[test]
fn compile_exits_0() {
    step_compile(&spec_dir()).unwrap_or_else(|e| panic!("{e}"));
}

/// ADR 0076 step 3 (expectation 3).
#[test]
fn synthesize_reports_0_refusals() {
    step_synthesize(&spec_dir()).unwrap_or_else(|e| panic!("{e}"));
}

/// ADR 0076 step 4 (expectation 4).
#[test]
fn no_open_question_under_ess() {
    step_no_open_question(&spec_dir()).unwrap_or_else(|e| panic!("{e}"));
}

/// All four steps, in order, on `ess/`.
#[test]
fn gate_holds_on_the_specification() {
    gate(&spec_dir()).unwrap_or_else(|e| panic!("{e}"));
}

/// Negative control for step 4 (expectation 5): a marker appended to a copy of `ess/` is
/// reported, it is the only thing reported, and the gate fails at step 4.
#[test]
fn open_question_scan_reports_an_added_marker() {
    let copy = spec_copy("open-question");
    let domain = copy.join("domains/protocol.yaml");
    let line = append(&domain, &format!("# {OPEN_QUESTION} probe\n"));

    assert_eq!(
        open_questions(&copy),
        [format!(
            "{}:{line}: # {OPEN_QUESTION} probe",
            domain.display()
        )]
    );
    expect_failure_at(&copy, "step 4", &[&format!("protocol.yaml:{line}")]);
}

/// Negative control for step 3: synthesize refuses and exits 0, so only the count fails it.
#[test]
fn a_refusal_with_exit_0_fails_step_3() {
    let copy = spec_copy("refused-exit-0");
    append(&copy.join("domains/protocol.yaml"), UNSATISFIABLE_COMMAND);

    let out = suite_path(&copy);
    let printed = ess_on(
        &copy,
        &["verify", "conform", "synthesize"],
        &["--out", out.to_str().expect("UTF-8")],
    )
    .expect("synthesize exits 0 on this copy, so the exit status alone would pass it");
    assert_eq!(refusal_count(&printed), Some(1), "{printed}");
    expect_failure_at(&copy, "step 3", &["1 refusal(s)", "ESS-SYNTH-003"]);
}

/// Negative control for step 1: a `requires:` older than the `ess` on `PATH` passes validation
/// without `--strict-requires` and fails the gate with it.
#[test]
fn an_older_requires_fails_step_1() {
    let copy = spec_copy("older-requires");
    replace(
        &copy.join("ess-inputs.yaml"),
        &format!("requires: ess {PINNED}"),
        "requires: ess 0.51.0",
    );

    ess_on(&copy, &["specify", "validate"], &[])
        .expect("without --strict-requires the older pin only warns, so the flag is what fails it");
    expect_failure_at(&copy, "step 1", &["--strict-requires refuses"]);
}

#[test]
fn refusal_count_reads_the_summary_line() {
    let stdout = "refused: canon.protocol.Protocol: the probe quotes 0 refusal(s)\n\
                  3 scenario(s) (0 authored), 2 refusal(s), written to suite.json\n";
    assert_eq!(refusal_count(stdout), Some(2));
    assert_eq!(
        refusal_count("0 scenario(s) (0 authored), 0 refusal(s), written to suite.json\n"),
        Some(0)
    );
    assert_eq!(
        refusal_count("0 scenario(s), written to suite.json\n"),
        None
    );
}

/// Expectation 6: run from `ess/`, where `ess-inputs.yaml` is, `ess specify toolchain which` names
/// the pinned release and takes it from the pin. From the repository root the dispatcher does not
/// look inside `ess/` and reports whichever `ess` is on `PATH` (`reason: self`), which says
/// nothing about the pin.
#[test]
fn toolchain_is_the_pinned_release() {
    let args = ["specify", "toolchain", "which"];
    let output = ess(&spec_dir(), &args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "`ess {}` failed", args.join(" "));
    assert_eq!(
        stdout.lines().next(),
        Some(format!("ess {PINNED}").as_str()),
        "`ess {}` from ess/:\n{stdout}",
        args.join(" ")
    );
    let reason = stdout
        .lines()
        .find_map(|line| line.strip_prefix("reason: "))
        .unwrap_or_default();
    assert!(
        reason.starts_with(&format!("pin: `requires: ess {PINNED}` in"))
            && reason.ends_with("ess/ess-inputs.yaml"),
        "`ess {}` from ess/ does not take its release from ess/ess-inputs.yaml:\n{stdout}",
        args.join(" ")
    );
}

/// The pin is written in three places; they name one release.
#[test]
fn pin_agrees_across_inputs_and_ci() {
    let inputs =
        fs::read_to_string(spec_dir().join("ess-inputs.yaml")).expect("ess/ess-inputs.yaml reads");
    let inputs: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&inputs).expect("ess/ess-inputs.yaml is YAML");
    assert_eq!(
        inputs["requires"].as_str(),
        Some(format!("ess {PINNED}").as_str()),
        "ess/ess-inputs.yaml `requires:`"
    );

    let workflow = fs::read_to_string(repo_root().join(".github/workflows/check.yml"))
        .expect(".github/workflows/check.yml reads");
    let installs: Vec<&str> = workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("ESS_VERSION:"))
        .map(|value| value.trim().trim_matches('"'))
        .collect();
    assert_eq!(
        installs,
        [PINNED],
        "ESS_VERSION in .github/workflows/check.yml"
    );
}

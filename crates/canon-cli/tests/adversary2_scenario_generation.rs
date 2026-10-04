//! Adversary pass 2 for story:scenario-generation, against the `canon generate` staging
//! directory, the protocol-path confinement and the `file-name-collision` refusal.
//!
//! Red when written:
//! - an existing, empty, writable `--out` whose parent is not writable is refused: the staging
//!   directory has to be created beside it;
//! - an existing, empty `--out` loses its own permissions: it is replaced by the staging
//!   directory, not written into;
//! - an `--out` name legal on its own, but too long once the staging prefix and suffix are added,
//!   is refused;
//! - an `--out` written as `<dir>/.` is refused;
//! - a failed write leaves behind the parent directories created for `--out`;
//! - an `--out` that is a symbolic link to an empty directory is refused;
//! - identifiers equal under Unicode case folding (`s`/`ſ`, `σ`/`ς`, `μ`/`µ`) are not refused as
//!   `file-name-collision`, although the refusal says it catches every pair a filesystem that
//!   ignores letter case would hold as one file.
//!
//! Green when written, kept as guards: a symbolic link out of the working directory in a parent
//! component of `--path` is refused; a working directory that is itself a symbolic link generates
//! scenarios `canon conform run` passes from the same directory; the Kelvin sign and the capital
//! sharp s are refused as collisions; and no failure removes a symbolic link `--out` or what it
//! points to.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("the canon binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// A fresh, empty directory under the test scratch area.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary2-generate-{name}"));
    if dir.exists() {
        make_writable(&dir);
        std::fs::remove_dir_all(&dir).expect("an old scratch directory is removable");
    }
    std::fs::create_dir_all(&dir).expect("a scratch directory is creatable");
    dir
}

#[cfg(unix)]
fn make_writable(dir: &Path) {
    use std::os::unix::fs::PermissionsExt;
    for entry in walk(dir) {
        if entry.is_dir() && !entry.is_symlink() {
            let _ = std::fs::set_permissions(&entry, std::fs::Permissions::from_mode(0o755));
        }
    }
}

#[cfg(not(unix))]
fn make_writable(_dir: &Path) {}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut all = vec![dir.to_path_buf()];
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !path.is_symlink() {
                all.extend(walk(&path));
            }
        }
    }
    all
}

fn entries(dir: &Path) -> Vec<String> {
    if !dir.exists() {
        return Vec::new();
    }
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("readable")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

/// A protocol with one outcome per name, each requiring its own explicit decision, so every
/// outcome is reachable and gets one scenario.
fn protocol_with_outcomes(names: &[&str]) -> String {
    let outcomes: Vec<String> = names
        .iter()
        .enumerate()
        .map(|(at, name)| format!("\"{name}\": {{requires: {{decision: d{at}}}}}"))
        .collect();
    format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\noutcomes: {{{}}}\n",
        outcomes.join(", ")
    )
}

/// The base investigation protocol copied into `work`, as `p.yaml`.
fn base_protocol_in(work: &Path) {
    let base = repository_root().join("fixtures/investigation/check/base/protocol.yaml");
    std::fs::copy(base, work.join("p.yaml")).expect("the base protocol copies");
}

/// Restores a directory's mode when the test ends, so the scratch area stays removable.
#[cfg(unix)]
struct Mode(PathBuf);

#[cfg(unix)]
impl Drop for Mode {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o755));
    }
}

#[cfg(unix)]
#[test]
fn an_existing_empty_out_under_a_parent_that_is_not_writable_is_written() {
    use std::os::unix::fs::PermissionsExt;
    let work = scratch("read-only-parent");
    base_protocol_in(&work);
    let parent = work.join("shared");
    let out = parent.join("out");
    std::fs::create_dir_all(&out).expect("creatable");
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o555)).expect("chmod");
    let _restore = Mode(parent.clone());
    if std::fs::write(parent.join("probe"), "").is_ok() {
        // Running with a privilege that ignores the mode: the case cannot be built here.
        let _ = std::fs::remove_file(parent.join("probe"));
        return;
    }
    let output = canon(
        &work,
        &["generate", "--path", "p.yaml", "--out", "shared/out"],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "--out `shared/out` exists, is empty and is writable, but canon generate refuses it \
         because its parent is not: {}",
        text(&output.stderr).trim()
    );
    assert!(!entries(&out).is_empty());
}

#[cfg(unix)]
#[test]
fn an_existing_empty_out_keeps_its_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let work = scratch("out-mode");
    base_protocol_in(&work);
    let out = work.join("out");
    std::fs::create_dir(&out).expect("creatable");
    std::fs::set_permissions(&out, std::fs::Permissions::from_mode(0o700)).expect("chmod");
    let output = canon(&work, &["generate", "--path", "p.yaml", "--out", "out"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let mode = std::fs::metadata(&out)
        .expect("out exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        mode, 0o700,
        "the existing empty --out, mode 0700, was replaced by a new directory of mode {mode:o}"
    );
}

#[test]
fn an_out_name_within_the_file_name_limit_is_written() {
    let work = scratch("long-out");
    base_protocol_in(&work);
    // 240 bytes is a legal name on every common filesystem; with `.` before it and
    // `.canon-generate-<pid>` after it, the staging name is over 255 bytes.
    let name = "o".repeat(240);
    std::fs::create_dir(work.join(&name)).expect("a 240-byte directory name is legal");
    std::fs::remove_dir(work.join(&name)).expect("removable");
    let output = canon(&work, &["generate", "--path", "p.yaml", "--out", &name]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "a 240-byte --out name, which the filesystem accepts, is refused: {}",
        text(&output.stderr).trim()
    );
    assert!(!entries(&work.join(&name)).is_empty());
}

#[test]
fn an_out_written_with_a_trailing_dot_component_is_written() {
    let work = scratch("trailing-dot");
    base_protocol_in(&work);
    std::fs::create_dir(work.join("out")).expect("creatable");
    let output = canon(&work, &["generate", "--path", "p.yaml", "--out", "out/."]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "--out `out/.` names the existing empty directory `out`, but is refused: {}",
        text(&output.stderr).trim()
    );
    assert!(!entries(&work.join("out")).is_empty());
}

#[test]
fn a_failed_write_leaves_no_directory_it_created() {
    let work = scratch("created-parents");
    let long = "b".repeat(240);
    std::fs::write(
        work.join("p.yaml"),
        protocol_with_outcomes(&["a", long.as_str()]),
    )
    .expect("writable");
    let output = canon(
        &work,
        &["generate", "--path", "p.yaml", "--out", "new/deeper/out"],
    );
    assert_eq!(output.status.code(), Some(2), "{}", text(&output.stderr));
    assert_eq!(
        entries(&work),
        ["p.yaml"],
        "canon generate failed ({}) and left the parent directories it created for --out",
        text(&output.stderr).trim()
    );
}

#[cfg(unix)]
#[test]
fn an_out_that_is_a_symbolic_link_to_an_empty_directory_is_written_through() {
    let work = scratch("symlink-out");
    base_protocol_in(&work);
    std::fs::create_dir(work.join("real")).expect("creatable");
    std::os::unix::fs::symlink("real", work.join("out")).expect("a symlink is creatable");
    let output = canon(&work, &["generate", "--path", "p.yaml", "--out", "out"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "--out `out` is a symbolic link to an empty directory, which --out's help calls \
         acceptable, but canon generate fails: {}",
        text(&output.stderr).trim()
    );
}

#[cfg(unix)]
#[test]
fn no_failure_removes_a_symbolic_link_out_or_what_it_points_to() {
    let work = scratch("symlink-out-kept");
    // A protocol whose second scenario cannot be written, so generation fails after staging.
    let long = "b".repeat(240);
    std::fs::write(
        work.join("p.yaml"),
        protocol_with_outcomes(&["a", long.as_str()]),
    )
    .expect("writable");
    std::fs::create_dir(work.join("real")).expect("creatable");
    std::os::unix::fs::symlink("real", work.join("out")).expect("a symlink is creatable");
    let output = canon(&work, &["generate", "--path", "p.yaml", "--out", "out"]);
    assert_ne!(output.status.code(), Some(0));
    assert!(
        work.join("out").is_symlink(),
        "the --out symlink was removed"
    );
    assert!(
        work.join("real").is_dir(),
        "the directory --out points to was removed"
    );
    assert_eq!(entries(&work), ["out", "p.yaml", "real"]);

    // And with a protocol that generates: whatever the outcome, both are still there.
    base_protocol_in(&work);
    let _ = canon(&work, &["generate", "--path", "p.yaml", "--out", "out"]);
    assert!(
        work.join("real").is_dir(),
        "the directory --out points to was removed"
    );
}

#[cfg(unix)]
#[test]
fn a_symbolic_link_out_of_the_working_directory_in_a_parent_component_is_refused() {
    let work = scratch("parent-link");
    let outside = scratch("parent-link-outside");
    base_protocol_in(&outside);
    std::os::unix::fs::symlink(&outside, work.join("dir")).expect("a symlink is creatable");
    let output = canon(&work, &["generate", "--path", "dir/p.yaml", "--out", "out"]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    assert!(
        text(&output.stderr).starts_with("error[fixture-not-confined]: "),
        "{}",
        text(&output.stderr)
    );
    assert_eq!(entries(&work), ["dir"]);
}

#[cfg(unix)]
#[test]
fn a_working_directory_that_is_a_symbolic_link_generates_scenarios_conform_run_passes() {
    let real = scratch("linked-cwd-real");
    base_protocol_in(&real);
    let holder = scratch("linked-cwd");
    let linked = holder.join("cwd");
    std::os::unix::fs::symlink(&real, &linked).expect("a symlink is creatable");
    let generated = canon(&linked, &["generate", "--path", "p.yaml", "--out", "out"]);
    assert_eq!(
        generated.status.code(),
        Some(0),
        "{}",
        text(&generated.stderr)
    );
    let run = canon(&linked, &["conform", "run", "--scenarios", "out"]);
    assert_eq!(run.status.code(), Some(0), "{}", text(&run.stdout));
    assert!(
        text(&run.stdout).contains(" 0 failed, 0 unreadable"),
        "{}",
        text(&run.stdout)
    );
}

/// Runs `canon generate` over a protocol with `names` as outcomes and returns its exit status and
/// standard error.
fn generate_outcomes(case: &str, names: &[&str]) -> (Option<i32>, String) {
    let work = scratch(case);
    std::fs::write(work.join("p.yaml"), protocol_with_outcomes(names)).expect("writable");
    let output = canon(&work, &["generate", "--path", "p.yaml", "--out", "out"]);
    (output.status.code(), text(&output.stderr))
}

fn is_collision((code, stderr): &(Option<i32>, String)) -> bool {
    *code == Some(1) && stderr.starts_with("error[file-name-collision]: ")
}

#[test]
fn identifiers_equal_under_unicode_case_folding_are_refused_as_a_file_name_collision() {
    // Each pair is one name under Unicode default case folding (CaseFolding.txt, status C):
    // U+017F folds to `s`, U+03C2 to U+03C3, U+00B5 to U+03BC.
    let pairs = [
        ("fold-long-s", "s", "\u{17F}"),
        ("fold-final-sigma", "\u{3C3}", "\u{3C2}"),
        ("fold-micro", "\u{3BC}", "\u{B5}"),
    ];
    let missed: Vec<String> = pairs
        .iter()
        .filter_map(|(case, a, b)| {
            let result = generate_outcomes(case, &[a, b]);
            (!is_collision(&result))
                .then(|| format!("{a:?} and {b:?}: exit {:?}, {}", result.0, result.1.trim()))
        })
        .collect();
    assert!(
        missed.is_empty(),
        "identifiers one under Unicode case folding were not refused as file-name-collision: \
         {missed:#?}"
    );
}

#[test]
fn the_kelvin_sign_and_the_capital_sharp_s_are_refused_as_a_file_name_collision() {
    for (case, a, b) in [
        ("fold-kelvin", "k", "\u{212A}"),
        ("fold-sharp-s", "\u{DF}", "\u{1E9E}"),
        ("fold-ascii", "Go", "gO"),
    ] {
        let result = generate_outcomes(case, &[a, b]);
        assert!(is_collision(&result), "{a:?} and {b:?}: {result:?}");
    }
}

#[test]
fn a_capital_letter_and_its_final_lower_case_form_are_refused_as_a_file_name_collision() {
    // U+03A3 is the capital of U+03C2 (UnicodeData.txt: 03C2's simple uppercase is 03A3), so a
    // filesystem that ignores letter case holds `outcome.Σ…` and `outcome.ς…` as one file.
    // Lowercasing U+03A3 inside `outcome.Σ.legitimate.yaml` gives U+03C3, not U+03C2, because it is
    // followed by a letter.
    let result = generate_outcomes("fold-capital-sigma", &["\u{3A3}", "\u{3C2}"]);
    assert!(
        is_collision(&result),
        "outcomes `Σ` and `ς` were not refused as file-name-collision: exit {:?} {}",
        result.0,
        result.1.trim()
    );
}

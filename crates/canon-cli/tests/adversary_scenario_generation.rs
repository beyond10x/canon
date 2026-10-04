//! Adversary cases for story:scenario-generation, against the `canon generate` command line.
//!
//! - A scenario file that cannot be created (a file name over the filesystem's limit) leaves the
//!   scenarios written before it in `--out`: a partial output, which also makes every re-run
//!   refuse with `out-not-empty`.
//! - A protocol path that is confined as written but resolves, through a symbolic link, outside
//!   the working directory is accepted, and `canon conform run` from that same directory then
//!   refuses every scenario it wrote.
//! - A protocol with no outcome and no action ever blocked generates no scenario and exits 0;
//!   `canon conform run` refuses the empty directory it leaves.
//! - A non-empty `--out` is refused with `out-not-empty`, exit 1, and left as it was.

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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary-generate-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("an old scratch directory is removable");
    }
    std::fs::create_dir_all(&dir).expect("a scratch directory is creatable");
    dir
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

#[test]
fn a_scenario_that_cannot_be_written_leaves_no_partial_output() {
    let work = scratch("long");
    // `outcome.a.legitimate.yaml` is written first; the second file name is 264 bytes, over the
    // 255-byte limit of every common filesystem.
    let long = "b".repeat(240);
    std::fs::write(
        work.join("p.yaml"),
        format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
             outcomes: {{a: {{requires: {{decision: stop}}}}, {long}: {{requires: {{decision: go}}}}}}\n"
        ),
    )
    .expect("writable");
    let output = canon(&work, &["generate", "--path", "p.yaml", "--out", "out"]);
    assert_ne!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let left = entries(&work.join("out"));
    assert!(
        left.is_empty(),
        "canon generate exited {:?} ({}) and left {left:?} in --out: a partial output, which a \
         re-run refuses as out-not-empty",
        output.status.code(),
        text(&output.stderr).trim()
    );
}

#[cfg(unix)]
#[test]
fn a_protocol_path_through_a_symbolic_link_out_of_the_working_directory_runs_or_is_refused() {
    let work = scratch("symlink");
    let target = repository_root().join("fixtures/investigation/check/base/protocol.yaml");
    std::os::unix::fs::symlink(&target, work.join("link.yaml")).expect("a symlink is creatable");
    let generated = canon(&work, &["generate", "--path", "link.yaml", "--out", "out"]);
    if generated.status.code() != Some(0) {
        // Refusing the path is one acceptable answer.
        return;
    }
    let run = canon(&work, &["conform", "run", "--scenarios", "out"]);
    assert_eq!(
        run.status.code(),
        Some(0),
        "canon generate accepted `link.yaml` and wrote {:?}, but canon conform run from the same \
         working directory refuses them: {}",
        entries(&work.join("out")),
        text(&run.stdout).trim()
    );
}

#[test]
fn a_protocol_that_yields_no_scenario_is_refused_or_leaves_a_runnable_registry() {
    let root = repository_root();
    let out = scratch("empty").join("out");
    let out_text = out.to_str().expect("a UTF-8 scratch path");
    let generated = canon(
        &root,
        &[
            "generate",
            "--path",
            "fixtures/investigation/subject-bound-evidence-match.yaml",
            "--out",
            out_text,
        ],
    );
    if generated.status.code() != Some(0) {
        return;
    }
    let run = canon(&root, &["conform", "run", "--scenarios", out_text]);
    assert_eq!(
        run.status.code(),
        Some(0),
        "canon generate exited 0 ({}), but canon conform run refuses what it left: {}",
        text(&generated.stdout).trim(),
        text(&run.stdout).trim()
    );
}

#[test]
fn a_non_empty_out_is_refused_and_left_as_it_was() {
    let root = repository_root();
    let out = scratch("non-empty");
    std::fs::write(out.join("keep.txt"), "mine\n").expect("writable");
    let output = canon(
        &root,
        &[
            "generate",
            "--path",
            "fixtures/investigation/check/base/protocol.yaml",
            "--out",
            out.to_str().expect("a UTF-8 scratch path"),
        ],
    );
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    assert!(
        text(&output.stderr).starts_with("error[out-not-empty]: "),
        "{}",
        text(&output.stderr)
    );
    assert_eq!(text(&output.stdout), "");
    assert_eq!(entries(&out), ["keep.txt"]);
    assert_eq!(
        std::fs::read_to_string(out.join("keep.txt")).expect("readable"),
        "mine\n"
    );
}

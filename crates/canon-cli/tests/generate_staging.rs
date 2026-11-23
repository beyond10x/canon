//! `canon generate` stages a run in a private directory (story:generate-rollback-in-private-staging)
//! and still fails, and succeeds, where 0.12.0, which wrote into `--out` directly, did:
//!
//! - under a umask that masks the owner's read bit (`0477`, `0777`) the run succeeds and writes its
//!   files with the mode that umask gives (`0200`, `0`), as 0.12.0 did;
//! - a read-only `--out` exits 2 naming the first file the run would have written, as 0.12.0 did;
//! - a staging directory left by a run that was stopped makes `--out` non-empty, and the refusal
//!   names it.

#![cfg(unix)]

use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PROTOCOL: &str = "fixtures/investigation/check/base/protocol.yaml";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

/// A fresh, empty directory under the test scratch area.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("generate-staging-{name}"));
    if dir.exists() {
        make_removable(&dir);
        std::fs::remove_dir_all(&dir).expect("an old scratch directory is removable");
    }
    std::fs::create_dir_all(&dir).expect("a scratch directory is creatable");
    dir
}

/// Gives every directory under `dir` mode 0700, so a run of a test that failed leaves nothing
/// the next run cannot remove.
fn make_removable(dir: &Path) {
    let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                make_removable(&entry.path());
            }
        }
    }
}

/// `canon generate --path <PROTOCOL> --out <out>` from the repository root, under `umask`.
fn generate(out: &Path, umask: &str) -> Output {
    Command::new("sh")
        .current_dir(repository_root())
        .args([
            "-c",
            "umask \"$1\" && shift && exec \"$@\"",
            "sh",
            umask,
            env!("CARGO_BIN_EXE_canon"),
            "generate",
            "--path",
            PROTOCOL,
            "--out",
        ])
        .arg(out)
        .output()
        .expect("sh runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn entries(dir: &Path) -> Vec<String> {
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

/// The files a run under umask 022 writes, in the order it reports them.
fn written(work: &Path) -> Vec<PathBuf> {
    let out = work.join("reference");
    let output = generate(&out, "022");
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    text(&output.stdout)
        .lines()
        .filter_map(|line| line.strip_prefix("wrote: "))
        .map(PathBuf::from)
        .collect()
}

#[test]
fn generate_under_a_umask_that_masks_the_owner_read_bit_writes_as_0_12_0_did() {
    let work = scratch("umask");
    let names: Vec<String> = written(&work)
        .iter()
        .map(|file| {
            file.file_name()
                .expect("a name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    for (umask, file_mode) in [("0477", 0o200), ("0777", 0o000)] {
        let out = work.join(format!("out-{umask}"));
        std::fs::create_dir(&out).expect("creatable");
        let output = generate(&out, umask);
        assert_eq!(
            (output.status.code(), text(&output.stderr).as_str()),
            (Some(0), ""),
            "umask {umask}"
        );
        assert_eq!(entries(&out), sorted, "umask {umask}");
        for name in &names {
            let mode = std::fs::metadata(out.join(name))
                .expect("there")
                .permissions()
                .mode()
                & 0o7777;
            assert_eq!(mode, file_mode, "umask {umask}: {name} has mode {mode:o}");
        }
    }
    // An absent `--out` is created under the umask, as 0.12.0 created it.
    let out = work.join("absent/out");
    let output = generate(&out, "0477");
    assert_eq!(
        (output.status.code(), text(&output.stderr).as_str()),
        (Some(0), ""),
        "umask 0477, absent --out"
    );
    assert_eq!(entries(&out), sorted);
    make_removable(&work);
}

#[test]
fn generate_into_a_read_only_out_names_the_first_file_as_0_12_0_did() {
    let work = scratch("read-only");
    let first = written(&work)[0].file_name().expect("a name").to_owned();
    let out = work.join("out");
    std::fs::create_dir(&out).expect("creatable");
    std::fs::set_permissions(&out, std::fs::Permissions::from_mode(0o555)).expect("chmod");
    let output = generate(&out, "022");
    std::fs::set_permissions(&out, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    assert_eq!(output.status.code(), Some(2), "{}", text(&output.stderr));
    assert_eq!(
        text(&output.stderr),
        format!(
            "error[unreadable]: {}: Permission denied (os error 13)\n",
            out.join(first).display()
        )
    );
    assert_eq!(text(&output.stdout), "");
    assert!(entries(&out).is_empty(), "left {:?}", entries(&out));
}

#[test]
fn generate_names_a_left_staging_directory_that_makes_out_non_empty() {
    let work = scratch("left-staging");
    let out = work.join("out");
    std::fs::create_dir_all(out.join(".canon-generate-1-0")).expect("creatable");
    let output = generate(&out, "022");
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    let stderr = text(&output.stderr);
    assert!(
        stderr.starts_with(&format!(
            "error[out-not-empty]: {}: the output directory is not empty",
            out.display()
        )),
        "{stderr}"
    );
    assert!(
        stderr.contains("`.canon-generate-1-0`"),
        "the refusal does not name the entry that makes --out non-empty: {stderr}"
    );
    assert_eq!(entries(&out), [".canon-generate-1-0"]);
}

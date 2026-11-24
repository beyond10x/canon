//! Wave 15 review F2: `canon digest --help` says what it refuses and what it does not check, and
//! the command does as the help says. It refuses a file that does not read as a
//! `canon-evidence/1` record; it does not run the identifier and instant checks of
//! `canon evaluate`, so a record with the id `e 1` or the `observed_at` `yesterday` prints a
//! digest and exits 0.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use clap::CommandFactory;

/// The help of `canon digest`, lower case, whitespace runs as one space.
fn digest_help() -> String {
    let mut command = canon_cli::Cli::command();
    let digest = command
        .find_subcommand_mut("digest")
        .expect("canon digest exists");
    let help = digest.render_long_help().to_string().to_lowercase();
    help.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .args(args)
        .output()
        .expect("canon runs")
}

fn scratch() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("digest-help-refusals");
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

#[test]
fn digest_help_names_the_checks_it_does_not_run_and_the_command_does_not_run_them() {
    let help = digest_help();
    for words in [
        "a file that does not read as a `canon-evidence/1` record is refused",
        "the identifier and instant checks of `canon evaluate` are not run",
    ] {
        assert!(help.contains(words), "help does not say `{words}`: {help}");
    }
    assert!(
        !help.contains("a file that is not a `canon-evidence/1` record is refused"),
        "help still promises a refusal the command does not make: {help}"
    );

    let dir = scratch();
    for (name, record) in [
        (
            "space-in-id.yaml",
            "format: canon-evidence/1\nid: e 1\nkind: k\nsubject: spec\nsubject_revision: r2\n",
        ),
        (
            "bare-instant.yaml",
            "format: canon-evidence/1\nid: e1\nkind: k\nsubject: spec\nsubject_revision: r2\n\
             observed_at: yesterday\n",
        ),
    ] {
        let file = dir.join(name);
        std::fs::write(&file, record).expect("record written");
        let out = canon(&["digest", "--evidence", file.to_str().expect("utf-8 path")]);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{name}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            stdout.starts_with("sha256:") && stdout.len() == "sha256:".len() + 64 + 1,
            "{name}: {stdout}"
        );
    }
}

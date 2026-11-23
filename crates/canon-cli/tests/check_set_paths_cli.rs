//! Story:check-across-protocol-files, `canon check-set` against the command line: which file a
//! `--path` reads, two files with one protocol id, and the exit statuses the help states.
//!
//! `canon check-set --help` says of `--path`: "a path given twice is read once". A path is one
//! file wherever it resolves to: `review.yaml` and `./review.yaml`, or a symbolic link and the
//! file it points at, are read once, so the summary counts the file's openings and supplied kinds
//! once. A shell glob next to a path written by hand (`--path ./*.yaml --path review.yaml`) gives
//! exactly that. Two different files with one protocol id are not pooled: a runner resolves an id
//! to one protocol and `case_open` names no revision, so the set is refused with
//! `duplicate-protocol-id`. The help states exit 2 for a file that cannot be read, and that the
//! first file refused in sorted order decides the status.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The fixture family, relative to the repository root.
const FIXTURES: &str = "fixtures/check-across-protocol-files";

/// The clean set's members.
const CLEAN_MEMBERS: [&str; 4] = ["change.yaml", "check.yaml", "notes.yaml", "review.yaml"];

fn set_dir(set: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(FIXTURES)
        .join(set)
        .canonicalize()
        .expect("the set exists")
}

fn canon(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("the canon binary runs")
}

/// `canon check-set` over `paths`, run in `dir`.
fn check_set_in(dir: &Path, paths: &[&str]) -> Output {
    let mut args = vec!["check-set"];
    for path in paths {
        args.push("--path");
        args.push(path);
    }
    canon(dir, &args)
}

/// `canon check-set` over `paths`, run in the clean set's directory: standard output and status.
fn check_set(paths: &[&str]) -> (String, Option<i32>) {
    let output = check_set_in(&set_dir("clean"), paths);
    (text(&output.stdout), output.status.code())
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("utf-8 output")
}

#[test]
fn a_path_given_twice_in_two_spellings_is_read_once() {
    let expected = std::fs::read_to_string(set_dir("clean").join("report.txt")).expect("report");
    assert_eq!(check_set(&CLEAN_MEMBERS), (expected.clone(), Some(0)));
    let twice = [
        "change.yaml",
        "check.yaml",
        "notes.yaml",
        "review.yaml",
        "./review.yaml",
    ];
    assert_eq!(
        check_set(&twice),
        (expected, Some(0)),
        "review.yaml and ./review.yaml are one file, read once"
    );
}

/// A symbolic link and the file it points at are one file: read once, so the set is the clean set
/// and not two protocols with the id `review`.
#[test]
fn a_file_and_a_symbolic_link_to_it_are_read_once() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("check_set_paths_cli/symlink");
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old copy is removed");
    }
    std::fs::create_dir_all(&dir).expect("the copy's directory is made");
    for member in CLEAN_MEMBERS {
        std::fs::copy(set_dir("clean").join(member), dir.join(member)).expect("a member is copied");
    }
    std::os::unix::fs::symlink("review.yaml", dir.join("again.yaml")).expect("the link is made");
    let expected = std::fs::read_to_string(set_dir("clean").join("report.txt")).expect("report");
    let mut paths = CLEAN_MEMBERS.to_vec();
    paths.push("again.yaml");
    let output = check_set_in(&dir, &paths);
    assert_eq!(text(&output.stderr), "");
    assert_eq!(
        text(&output.stdout),
        expected,
        "again.yaml is review.yaml, read once"
    );
    assert_eq!(output.status.code(), Some(0));
}

/// A file that cannot be read still exits 2, named as it was typed.
#[test]
fn a_missing_path_is_named_as_typed_and_exits_2() {
    let mut paths = CLEAN_MEMBERS.to_vec();
    paths.push("./absent.yaml");
    let output = check_set_in(&set_dir("clean"), &paths);
    let stderr = text(&output.stderr);
    assert!(
        stderr.starts_with("error[unreadable]: ./absent.yaml: "),
        "{stderr}"
    );
    assert_eq!(text(&output.stdout), "");
    assert_eq!(output.status.code(), Some(2));
}

/// Two files with the id `change` are one finding naming both, whichever order they are given in,
/// and nothing is resolved against either; one of them given under two spellings is still one
/// file. Without the second file, the review's name `accepted` resolves against the one `change`
/// and is a finding when that one does not declare it.
#[test]
fn a_protocol_id_of_two_files_is_a_finding_and_one_file_twice_is_not() {
    let dir = set_dir("duplicate-id");
    let expected = "duplicate-protocol-id: protocol `change` is declared by more than one file of \
                    the set: `./change.yaml` and `change-other.yaml`\n\
                    checked: 3 protocols, 1 opening, 0 supplied kinds, 1 finding\n";
    for paths in [
        [
            "review.yaml",
            "change.yaml",
            "./change.yaml",
            "change-other.yaml",
        ],
        [
            "change-other.yaml",
            "./change.yaml",
            "change.yaml",
            "review.yaml",
        ],
    ] {
        let output = check_set_in(&dir, &paths);
        assert_eq!(text(&output.stderr), "");
        assert_eq!(text(&output.stdout), expected, "{paths:?}");
        assert_eq!(output.status.code(), Some(1));
    }
    let alone = check_set_in(&dir, &["review.yaml", "change-other.yaml"]);
    assert_eq!(
        text(&alone.stdout),
        "undeclared-child-outcome: protocol `review` claim `work.accepted` names outcome \
         `accepted` of a child case about `work`, which no protocol it opens about `work` declares \
         (it opens `change`)\n\
         checked: 2 protocols, 1 opening, 0 supplied kinds, 1 finding\n"
    );
    assert_eq!(alone.status.code(), Some(1));
}

/// When one file cannot be read and another is refused, the first in sorted order of the paths
/// decides: exit 2 when the unreadable one sorts first, 1 when the refused one does.
#[test]
fn the_first_refused_file_in_sorted_order_decides_the_status() {
    let dir = set_dir("imports");
    let first = check_set_in(&dir, &["root.yaml", "change.yaml", "absent.yaml"]);
    assert!(
        text(&first.stderr).starts_with("error[unreadable]: absent.yaml: "),
        "{}",
        text(&first.stderr)
    );
    assert_eq!(first.status.code(), Some(2));
    let last = check_set_in(&dir, &["zz-absent.yaml", "root.yaml", "change.yaml"]);
    assert!(
        text(&last.stderr).starts_with("error[protocol-imports]: root.yaml: "),
        "{}",
        text(&last.stderr)
    );
    assert_eq!(last.status.code(), Some(1));
}

/// The help of each command that states its exit statuses states exit 2 for a file it cannot
/// read: `check-set` with the rule that the first file refused in sorted order decides, and
/// `mutate`, whose ESS declaration (`ess/domains/cli.yaml`) has the same `unreadable` outcome.
#[test]
fn the_help_states_exit_2_for_a_file_that_cannot_be_read() {
    let dir = set_dir("clean");
    let check_set = text(&canon(&dir, &["check-set", "--help"]).stdout);
    for needle in [
        "Exits 0 when the check found nothing, 1 when it found something",
        "a file that cannot be read exits 2",
        "the first of them in sorted order of the paths decides the exit status",
        "`duplicate-protocol-id`",
    ] {
        assert!(
            check_set.contains(needle),
            "check-set --help lacks {needle:?}: {check_set}"
        );
    }
    let mutate = text(&canon(&dir, &["mutate", "--help"]).stdout);
    assert!(
        mutate.contains("exits 2"),
        "mutate --help lacks exit 2: {mutate}"
    );
}

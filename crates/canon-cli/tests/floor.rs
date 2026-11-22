//! Acceptance for story:protocol-floor: `canon floor --floor <ir> --ir <ir>` answers whether a
//! protocol stays above an organization-owned floor (Atlas ADR 0086 § Decision 2): it classifies
//! every change from the floor to the protocol with the classifier of `canon diff`, exits 0 when
//! each is `TIGHTENING`, `EXPANSION` or `NO SEMANTIC CHANGE`, and exits 1 printing each
//! `RELAXATION` and `BREAKING` change otherwise.
//!
//! The specification is `fixtures/investigation/floor/`: `floor.yaml`, a `protocol/1` document
//! whose id and description are its own, and the protocols compared with it. The floor requires
//! `explanation.supported` and the capability `finding.publish` to publish a finding. Every
//! protocol has an id and a revision of its own; the three that import the floor are pinned to its
//! revision and digest.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const DIR: &str = "fixtures/investigation/floor";

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

/// Runs `canon <args>` from the repository root, so that fixture paths resolve against it.
fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(args)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

/// A fresh, empty directory under this test's scratch space.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("floor")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("stale scratch directory removes");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory creates");
    dir
}

/// Compiles the fixture `name` (with the floor as its import when `with_floor`) and writes its
/// `canon-ir/1` to `dir`; returns the IR's path and its text.
fn compiled(name: &str, with_floor: bool, dir: &Path) -> (String, String) {
    let source = format!("{DIR}/{name}.yaml");
    let floor = format!("{DIR}/floor.yaml");
    let mut args = vec!["compile", "--path", source.as_str()];
    if with_floor {
        args.extend(["--import", floor.as_str()]);
    }
    let output = canon(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{name} compiles: {}",
        text(&output.stderr)
    );
    let ir = text(&output.stdout).to_owned();
    let path = dir.join(format!("{name}.json"));
    std::fs::write(&path, &ir).expect("the IR is written");
    (path.display().to_string(), ir)
}

/// Runs `canon floor --floor <floor> --ir <ir>` and checks its exit status, then that it printed
/// exactly `stdout` and nothing on standard error. Returns what it printed.
fn floor(floor: &str, ir: &str, status: i32, stdout: Option<&str>) -> String {
    let output = canon(&["floor", "--floor", floor, "--ir", ir]);
    assert_eq!(
        output.status.code(),
        Some(status),
        "exit status of `canon floor --ir {ir}`, standard error: {}",
        text(&output.stderr)
    );
    assert_eq!(text(&output.stderr), "");
    if let Some(stdout) = stdout {
        assert_eq!(text(&output.stdout), stdout);
    }
    text(&output.stdout).to_owned()
}

/// What the floor does with inputs it cannot compare: a floor against itself changes nothing, an
/// unreadable file exits 2 and a file that is not `canon-ir/1` exits 1, each naming the file.
#[test]
fn floor_refuses_what_it_cannot_read_and_holds_against_itself() {
    let dir = scratch("refuses");
    let (floor_path, _) = compiled("floor", false, &dir);
    floor(
        &floor_path,
        &floor_path,
        0,
        Some(
            "floor: holds\n\
             TIGHTENING: 0\n\
             EXPANSION: 0\n\
             NO SEMANTIC CHANGE: 0\n",
        ),
    );

    let missing = dir.join("missing.json").display().to_string();
    for (arguments, named) in [
        (
            ["--floor", missing.as_str(), "--ir", floor_path.as_str()],
            0,
        ),
        (
            ["--floor", floor_path.as_str(), "--ir", missing.as_str()],
            1,
        ),
    ] {
        let output = canon(&[
            "floor",
            arguments[0],
            arguments[1],
            arguments[2],
            arguments[3],
        ]);
        assert_eq!(output.status.code(), Some(2), "{arguments:?}");
        assert_eq!(text(&output.stdout), "");
        let stderr = text(&output.stderr);
        assert!(
            stderr.starts_with(&format!("error[unreadable]: {missing}: ")),
            "{named}: {stderr}"
        );
    }

    let source = format!("{DIR}/floor.yaml");
    for (arguments, named) in [
        (["--floor", source.as_str(), "--ir", floor_path.as_str()], 0),
        (["--floor", floor_path.as_str(), "--ir", source.as_str()], 1),
    ] {
        let output = canon(&[
            "floor",
            arguments[0],
            arguments[1],
            arguments[2],
            arguments[3],
        ]);
        assert_eq!(output.status.code(), Some(1), "{arguments:?}");
        assert_eq!(text(&output.stdout), "");
        let stderr = text(&output.stderr);
        assert!(
            stderr.starts_with(&format!("error[malformed-input]: {source}: ")),
            "{named}: {stderr}"
        );
    }
}

#[test]
fn floor_admits_only_tightening_and_expansion() {
    let dir = scratch("admits");
    let (floor_path, floor_ir) = compiled("floor", false, &dir);

    // 1. A protocol that adds an obligation to the floor exits 0: one `TIGHTENING`.
    let (adds_obligation, adds_obligation_ir) = compiled("adds-obligation", false, &dir);
    floor(
        &floor_path,
        &adds_obligation,
        0,
        Some(
            "floor: holds\n\
             TIGHTENING: 1\n\
             EXPANSION: 0\n\
             NO SEMANTIC CHANGE: 0\n",
        ),
    );

    // 2. A protocol composed by importing the floor and adding an action exits 0: one
    // `EXPANSION`.
    let (composed, composed_ir) = compiled("composed", true, &dir);
    floor(
        &floor_path,
        &composed,
        0,
        Some(
            "floor: holds\n\
             TIGHTENING: 0\n\
             EXPANSION: 1\n\
             NO SEMANTIC CHANGE: 0\n",
        ),
    );

    // 3. A protocol that drops the floor's capability requirement exits 1 and prints
    // `RELAXATION` with the action's identifier.
    let (drops_capability, _) = compiled("drops-capability", false, &dir);
    floor(
        &floor_path,
        &drops_capability,
        1,
        Some(
            "floor: breached\n\
             RELAXATION: action `publish_finding` no longer requires capability `finding.publish`\n",
        ),
    );

    // 4. A protocol that changes an outcome's requirement exits 1 and prints `BREAKING` with the
    // outcome's identifier.
    let (changes_outcome, _) = compiled("changes-outcome", false, &dir);
    floor(
        &floor_path,
        &changes_outcome,
        1,
        Some(
            "floor: breached\n\
             BREAKING: outcome `supported` requirement changed\n",
        ),
    );

    // 5. In 1 and 2 the protocol's id and revision differ from the floor's, and nothing is printed
    // about them, nor about the descriptions, which differ too.
    assert!(
        floor_ir.contains("\"id\": \"floor.investigation\"")
            && floor_ir.contains("\"revision\": 1\n"),
        "the floor is `floor.investigation` revision 1: {floor_ir}"
    );
    for (path, ir, id, revision) in [
        (
            &adds_obligation,
            &adds_obligation_ir,
            "investigation.code-change",
            "4",
        ),
        (&composed, &composed_ir, "investigation.composed", "2"),
    ] {
        assert!(
            ir.contains(&format!("\"id\": \"{id}\""))
                && ir.contains(&format!("\"revision\": {revision}\n")),
            "the protocol is `{id}` revision {revision}: {ir}"
        );
        let printed = floor(&floor_path, path, 0, None);
        for word in [
            id,
            "floor.investigation",
            "revision",
            "protocol",
            "description",
        ] {
            assert!(!printed.contains(word), "`{word}` is not said: {printed}");
        }
    }

    // 6. A protocol that imports the floor and refines `explanation.supported` exits 0 when the
    // refinement adds a conjunct under its `all`, and exits 1 printing `RELAXATION` with the
    // claim's identifier when it adds an alternative under an `any`.
    let (strict, _) = compiled("refines-conjunct", true, &dir);
    floor(
        &floor_path,
        &strict,
        0,
        Some(
            "floor: holds\n\
             TIGHTENING: 1\n\
             EXPANSION: 1\n\
             NO SEMANTIC CHANGE: 0\n",
        ),
    );
    let (lenient, _) = compiled("refines-alternative", true, &dir);
    floor(
        &floor_path,
        &lenient,
        1,
        Some(
            "floor: breached\n\
             RELAXATION: claim `explanation.supported` predicate weakened\n",
        ),
    );
}

/// Compiles the protocol `source` to `dir/<name>.json` and returns that path.
fn compiled_source(dir: &Path, name: &str, source: &str) -> String {
    let yaml = dir.join(format!("{name}.yaml"));
    std::fs::write(&yaml, source).expect("the source is written");
    let output = canon(&["compile", "--path", &yaml.display().to_string()]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{name} compiles: {}",
        text(&output.stderr)
    );
    let path = dir.join(format!("{name}.json"));
    std::fs::write(&path, &output.stdout).expect("the IR is written");
    path.display().to_string()
}

fn floor_source() -> String {
    std::fs::read_to_string(repository_root().join(DIR).join("floor.yaml"))
        .expect("the floor fixture reads")
}

/// The floor under the id `investigation.added`, with `publish_finding` unchanged and a second
/// action `name` whose declaration after its description is `body`.
fn floor_with_added_action(name: &str, body: &str) -> String {
    let anchor = "    requires:\n      - capability: finding.publish\n";
    let source = floor_source();
    assert_eq!(
        source.matches(anchor).count(),
        1,
        "the floor names its capability once"
    );
    source
        .replacen("id: floor.investigation", "id: investigation.added", 1)
        .replacen(
            anchor,
            &format!("{anchor}\n  {name}:\n    description: An added action.\n{body}"),
            1,
        )
}

/// Story:floor-guards-added-actions (decision D124). An added action is an `EXPANSION` for the
/// classifier, but one that has the precondition of an action the floor guards with a capability
/// publishes the floor's finding without the capability: below the floor, exit 1, and the added
/// action is named. An added action that requires what the floor requires, or that is not the
/// floor's finding, stays an `EXPANSION`.
#[test]
fn floor_refuses_an_added_action_that_publishes_without_the_floors_capability() {
    let dir = scratch("added-action");
    let help = canon(&["floor", "--help"]);
    assert!(
        !text(&help.stdout).contains("is not caught"),
        "the help no longer states the limit: {}",
        text(&help.stdout)
    );
    let floor_path = compiled_source(&dir, "floor", &floor_source());
    let same_precondition =
        "    precondition:\n      claim: explanation.supported\n      is: true\n";
    let expansion = "floor: holds\n\
                     TIGHTENING: 0\n\
                     EXPANSION: 1\n\
                     NO SEMANTIC CHANGE: 0\n";

    // 1. The same precondition and no capability: breached, naming the action and the capability.
    let quiet = compiled_source(
        &dir,
        "quiet",
        &floor_with_added_action("publish_finding_quietly", same_precondition),
    );
    floor(
        &floor_path,
        &quiet,
        1,
        Some(
            "floor: breached\n\
             RELAXATION: action `publish_finding_quietly` added without capability \
             `finding.publish`, which the floor requires of `publish_finding`\n",
        ),
    );

    // 2. The same precondition and the capability: an expansion, which passes.
    let capable = compiled_source(
        &dir,
        "capable",
        &floor_with_added_action(
            "publish_finding_again",
            &format!("{same_precondition}    requires:\n      - capability: finding.publish\n"),
        ),
    );
    floor(&floor_path, &capable, 0, Some(expansion));

    // 3. A different precondition: not the floor's finding, an expansion, which passes.
    let other = compiled_source(
        &dir,
        "other",
        &floor_with_added_action(
            "note_progress",
            "    precondition:\n      claim: explanation.supported\n      is: false\n",
        ),
    );
    floor(&floor_path, &other, 0, Some(expansion));

    // 4. The same precondition and a capability other than the floor's: still without the floor's.
    let other_capability = compiled_source(
        &dir,
        "other-capability",
        &floor_with_added_action(
            "publish_finding_elsewhere",
            &format!("{same_precondition}    requires:\n      - capability: finding.draft\n"),
        ),
    );
    floor(
        &floor_path,
        &other_capability,
        1,
        Some(
            "floor: breached\n\
             RELAXATION: action `publish_finding_elsewhere` added without capability \
             `finding.publish`, which the floor requires of `publish_finding`\n",
        ),
    );
}

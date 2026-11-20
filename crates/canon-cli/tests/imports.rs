//! Acceptance for story:protocol-imports against the `canon` command line, over
//! `fixtures/investigation/imports/`: the root protocol `investigation` (`root.yaml`) imports
//! `investigation.review` (`review.yaml`), which declares the case input `severity` a root claim
//! reads, and `investigation.falsification` (`falsification.yaml`); both import
//! `investigation.evidence` (`evidence.yaml`). `flattened.yaml` holds the same declarations
//! inline. The refusals are the documents under `invalid/`.
//!
//! `imports_compose_into_closed_ir` holds the story's eighteen expectations, one block each,
//! numbered as the story numbers them. CANON-IMPORT-001 (`conformance/scenarios/imports.yaml`) is
//! run by `canon conform run` over the registry (`conform_requirements.rs`) and here.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const DIR: &str = "fixtures/investigation/imports";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

/// Runs `canon` from the repository root.
fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(args)
        .output()
        .expect("the canon binary runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

/// A fixture path, relative to the repository root.
fn fixture(name: &str) -> String {
    format!("{DIR}/{name}")
}

/// `command --path <root>` followed by `--import <part>` for each part, every name a fixture.
fn with_imports(command: &str, root: &str, parts: &[&str]) -> Output {
    let root = fixture(root);
    let parts: Vec<String> = parts.iter().map(|part| fixture(part)).collect();
    let mut args = vec![command, "--path", root.as_str()];
    for part in &parts {
        args.extend(["--import", part.as_str()]);
    }
    canon(&args)
}

/// The three parts `root.yaml` imports, directly or through another import.
const PARTS: [&str; 3] = ["review.yaml", "falsification.yaml", "evidence.yaml"];

/// A refusal: exit 1, nothing on standard output, and exactly `stderr`.
fn assert_refused(output: &Output, stderr: &str) {
    assert_eq!(text(&output.stderr), stderr);
    assert_eq!(text(&output.stdout), "");
    assert_eq!(output.status.code(), Some(1));
}

/// A scratch directory of this test binary, inside the build directory, emptied first.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("imports")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

/// The digest `root.yaml` pins `investigation.falsification` to, which is the digest of
/// `falsification.yaml` compiled with `evidence.yaml`.
const FALSIFICATION_DIGEST: &str =
    "sha256:1a2cc72944b479e53776adc362125e87607f587ce05b3d1cee4d95186ba0ab68";

#[test]
fn imports_compose_into_closed_ir() {
    // 1. The root compiled with its parts prints the bytes `flattened.yaml` compiles to.
    let composed = with_imports("compile", "root.yaml", &PARTS);
    assert_eq!(text(&composed.stderr), "");
    assert_eq!(composed.status.code(), Some(0));
    let flattened = canon(&["compile", "--path", &fixture("flattened.yaml")]);
    assert_eq!(
        flattened.status.code(),
        Some(0),
        "{}",
        text(&flattened.stderr)
    );
    let ir = text(&composed.stdout);
    assert_eq!(ir, text(&flattened.stdout));
    assert!(!ir.contains("imports"), "the IR holds no import: {ir}");
    assert!(
        ir.contains("  \"protocol\": {\n    \"description\": \"Decide whether an explanation is supported, and publish the finding once every obligation that applies to the case is discharged.\",\n    \"id\": \"investigation\",\n    \"revision\": 3\n  }\n"),
        "the IR names the root's id and revision only: {ir}"
    );

    // 2. The shared part's declarations appear once.
    for declared in [
        "\"explanation\": {",
        "\"observation\": {",
        "\"explanation.supported\": {",
        "\"inspect\": {",
        "\"explanation.refuted\": {",
    ] {
        assert_eq!(ir.matches(declared).count(), 1, "{declared} in {ir}");
    }

    // 3. The claim the root refines has the root's predicate.
    assert!(
        ir.contains("    \"peer.approved\": {\n      \"description\": \"A peer approved the finding.\",\n      \"true_when\": {\n        \"evidence\": {\n          \"kind\": \"peer_review\",\n          \"result\": \"approved\"\n        }\n      }\n    }"),
        "{ir}"
    );
    assert!(!ir.contains("A peer reviewed the finding."), "{ir}");

    // 4. The imports passed in the opposite order print identical bytes.
    let mut reversed = PARTS;
    reversed.reverse();
    let reordered = with_imports("compile", "root.yaml", &reversed);
    assert_eq!(
        reordered.status.code(),
        Some(0),
        "{}",
        text(&reordered.stderr)
    );
    assert_eq!(text(&reordered.stdout), ir);

    // 5. `canon validate` of the root with its parts exits 0.
    let valid = with_imports("validate", "root.yaml", &PARTS);
    assert_eq!(text(&valid.stderr), "");
    assert_eq!(
        text(&valid.stdout),
        "valid: protocol `investigation` revision 3\n"
    );
    assert_eq!(valid.status.code(), Some(0));

    // 6. `canon check` of the root with its parts prints the report of `flattened.yaml`.
    let checked = with_imports("check", "root.yaml", &PARTS);
    let checked_flat = canon(&["check", "--path", &fixture("flattened.yaml")]);
    assert_eq!(text(&checked.stderr), text(&checked_flat.stderr));
    assert_eq!(text(&checked.stdout), text(&checked_flat.stdout));
    assert_eq!(checked.status.code(), checked_flat.status.code());
    assert!(
        text(&checked.stdout).starts_with("checked: protocol `investigation` revision 3: "),
        "{}",
        text(&checked.stdout)
    );

    // 7. `canon generate` of the root exits 1 naming its first import, and writes nothing.
    let out = scratch("generate").join("out");
    let generated = canon(&[
        "generate",
        "--path",
        &fixture("root.yaml"),
        "--out",
        out.to_str().expect("UTF-8"),
    ]);
    assert_refused(
        &generated,
        "error[protocol-imports]: the protocol imports `investigation.falsification`: a generated \
         scenario names one fixture, and a protocol with imports compiles only with its imported \
         documents\n",
    );
    assert!(!out.exists());

    // 8. An import with no supplied document.
    assert_refused(
        &with_imports("compile", "root.yaml", &["review.yaml", "evidence.yaml"]),
        "error[missing-import]: protocol `investigation` imports `investigation.falsification`, \
         and no supplied document is that protocol\n",
    );

    // 9. A supplied document no import names.
    assert_refused(
        &with_imports(
            "compile",
            "root.yaml",
            &[
                "review.yaml",
                "falsification.yaml",
                "evidence.yaml",
                "invalid/stray.yaml",
            ],
        ),
        "error[unused-import]: the supplied protocol `investigation.stray` is imported neither by \
         the root nor by any document it imports\n",
    );

    // 10. A digest that differs from the pin, printing the pinned and the actual digest.
    assert_refused(
        &with_imports("compile", "invalid/root-wrong-digest.yaml", &PARTS),
        &format!(
            "error[import-digest-mismatch]: protocol `investigation` imports \
             `investigation.falsification` pinned to digest \
             sha256:0000000000000000000000000000000000000000000000000000000000000000, and the \
             supplied document compiles to {FALSIFICATION_DIGEST}\n"
        ),
    );

    // 11. An import cycle, printed from the first protocol on it back to that protocol. Its pins
    // are wrong, as every pin of a cycle is; the cycle is refused first.
    assert_refused(
        &with_imports(
            "compile",
            "invalid/cycle-root.yaml",
            &["invalid/cycle-a.yaml", "invalid/cycle-b.yaml"],
        ),
        "error[import-cycle]: protocols import each other in a cycle: investigation.cycle-a -> \
         investigation.cycle-b -> investigation.cycle-a\n",
    );

    // 12. An identifier the root and an import both declare, without `refines`.
    assert_refused(
        &with_imports(
            "compile",
            "invalid/root-unrefined.yaml",
            &["review.yaml", "evidence.yaml"],
        ),
        "error[unrefined-redeclaration]: protocol `investigation` declares claim `peer.approved`, \
         which its import `investigation.review` declares and does not list under `refines`\n",
    );

    // 13. An identifier two imports declare differently.
    assert_refused(
        &with_imports(
            "compile",
            "invalid/root-conflict.yaml",
            &["review.yaml", "evidence.yaml", "invalid/second-review.yaml"],
        ),
        "error[conflicting-imports]: protocol `investigation` imports evidence kind `peer_review` \
         from `investigation.review` and from `investigation.second-review`, declared \
         differently\n",
    );

    // 14. The root claim that reads the imported case input is TRUE for `high` and FALSE for
    // `low`, the two values `investigation.review` declares.
    let dir = scratch("evaluate");
    let ir_path = dir.join("investigation.json");
    std::fs::write(&ir_path, ir).expect("IR written");
    for (case, value) in [("case-high.yaml", "true"), ("case-low.yaml", "false")] {
        let evaluated = canon(&[
            "evaluate",
            "--ir",
            ir_path.to_str().expect("UTF-8"),
            "--case",
            &fixture(case),
            "--evidence",
            &fixture("records"),
        ]);
        assert_eq!(text(&evaluated.stderr), "", "{case}");
        assert_eq!(evaluated.status.code(), Some(0), "{case}");
        let decision: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(text(&evaluated.stdout)).expect("JSON");
        assert_eq!(
            decision["claims"]["case.severe"]["value"].as_str(),
            Some(value),
            "{case}: {}",
            text(&evaluated.stdout)
        );
    }

    // 15. `canon conform run` reports CANON-IMPORT-001 passed.
    let registry = scratch("registry");
    std::fs::copy(
        repository_root().join("conformance/scenarios/imports.yaml"),
        registry.join("imports.yaml"),
    )
    .expect("scenario copied");
    let ran = canon(&[
        "conform",
        "run",
        "--scenarios",
        registry.to_str().expect("UTF-8"),
    ]);
    assert_eq!(text(&ran.stderr), "");
    assert_eq!(
        text(&ran.stdout),
        "passed: scenario `CANON-IMPORT-001`\nconform: 1 passed, 0 failed, 0 unreadable\n"
    );
    assert_eq!(ran.status.code(), Some(0));

    // 16. A supplied document whose revision differs from the pin.
    assert_refused(
        &with_imports(
            "compile",
            "invalid/root-wrong-revision.yaml",
            &["review.yaml", "evidence.yaml"],
        ),
        "error[import-revision-mismatch]: protocol `investigation` imports `investigation.review` \
         pinned to revision 1, and the supplied document is revision 2\n",
    );

    // 17. A `refines` entry naming an identifier its import does not declare.
    assert_refused(
        &with_imports(
            "compile",
            "invalid/root-undeclared-refinement.yaml",
            &["review.yaml", "evidence.yaml"],
        ),
        "error[undeclared-refinement]: protocol `investigation` refines `peer.rejected` of its \
         import `investigation.review`, which declares no such identifier\n",
    );

    // 18. A part whose action opens a case under the root's id.
    assert_refused(
        &with_imports(
            "compile",
            "invalid/root-reopen.yaml",
            &["invalid/reopen.yaml", "evidence.yaml"],
        ),
        "error[self-opened-case]: action `reopen` opens a case under protocol `investigation`, \
         the protocol's own id\n",
    );
}

/// A refinement of a section other than claims: the root redeclares the action `request_review`,
/// which `investigation.review` declares, and lists it under `refines`; the redeclaration replaces
/// the imported action whole.
#[test]
fn refining_an_action_replaces_the_imported_action() {
    let source =
        std::fs::read_to_string(repository_root().join(fixture("root.yaml"))).expect("root reads");
    let refined = source.replace(
        "    refines: [peer.approved]\n",
        "    refines: [peer.approved, request_review]\n",
    ) + "\nactions:\n  request_review:\n    description: Ask a peer to review the finding of a severe case.\n    precondition:\n      claim: case.severe\n";
    assert_ne!(refined, source);
    let dir = scratch("refined-action");
    let root = dir.join("root.yaml");
    std::fs::write(&root, refined).expect("root written");
    let parts: Vec<String> = PARTS
        .iter()
        .map(|part| {
            repository_root()
                .join(fixture(part))
                .to_str()
                .expect("UTF-8")
                .to_owned()
        })
        .collect();
    let mut args = vec!["compile", "--path", root.to_str().expect("UTF-8")];
    for part in &parts {
        args.extend(["--import", part.as_str()]);
    }
    let compiled = canon(&args);
    assert_eq!(text(&compiled.stderr), "");
    assert_eq!(compiled.status.code(), Some(0));
    assert!(
        text(&compiled.stdout).contains("    \"request_review\": {\n      \"description\": \"Ask a peer to review the finding of a severe case.\",\n      \"effect\": null,\n      \"may_produce\": [],\n      \"precondition\": {\n        \"claim\": {\n          \"id\": \"case.severe\",\n          \"is\": \"true\"\n        }\n      },\n      \"requires\": []\n    }"),
        "{}",
        text(&compiled.stdout)
    );

    // Without the `refines` entry, the same redeclaration is refused.
    let unrefined = std::fs::read_to_string(&root)
        .expect("root reads")
        .replace("[peer.approved, request_review]", "[peer.approved]");
    std::fs::write(&root, unrefined).expect("root written");
    let refused = canon(&args);
    assert_refused(
        &refused,
        "error[unrefined-redeclaration]: protocol `investigation` declares action \
         `request_review`, which its import `investigation.review` declares and does not list \
         under `refines`\n",
    );
}

/// A root with imports passed without them is refused by every command that reads a
/// `protocol/1` document, naming each import; no import of a protocol without imports changes
/// anything.
#[test]
fn a_root_without_its_imports_is_refused_by_each_command() {
    for command in ["validate", "compile", "check"] {
        assert_refused(
            &with_imports(command, "root.yaml", &[]),
            "error[missing-import]: protocol `investigation` imports \
             `investigation.falsification`, and no supplied document is that protocol\n\
             error[missing-import]: protocol `investigation` imports `investigation.review`, and \
             no supplied document is that protocol\n",
        );
    }
    let alone = canon(&["compile", "--path", &fixture("evidence.yaml")]);
    let with_nothing = with_imports("compile", "evidence.yaml", &[]);
    assert_eq!(alone.status.code(), Some(0));
    assert_eq!(text(&alone.stdout), text(&with_nothing.stdout));
}

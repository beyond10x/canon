//! Adversary pass 2 on story:ess-hard-gate: `ess_model_matches` against model and specification
//! changes it does not see.
//!
//! The unit's `crates/canon/tests/ess_model_matches.rs` is read at run time, copied unchanged into a
//! probe package under `CARGO_TARGET_TMPDIR`, and built once into that package's own target
//! directory. Its test binary resolves the tree under test from `CARGO_MANIFEST_DIR` at run time, so
//! the one binary is run against several scratch roots, each holding a copy of `ess/` and
//! `crates/canon/src/model/` with one change applied. Every case asserts that the unit's own
//! `specification_and_model_are_equal` fails on a copy where the model and the specification
//! differ. The tree is never edited.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

fn repo_root() -> PathBuf {
    Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"))
        .join("../..")
        .canonicalize()
        .expect("repository root resolves")
}

fn base() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("adversary2-ess-model")
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

/// The unit's test, built once from the text on disk now, as an executable path.
fn unit_binary() -> &'static Path {
    static BINARY: OnceLock<PathBuf> = OnceLock::new();
    BINARY.get_or_init(|| {
        let root = repo_root();
        let package = base().join("probe");
        fs::create_dir_all(package.join("src")).expect("probe src");
        fs::create_dir_all(package.join("tests")).expect("probe tests");
        fs::write(package.join("src/lib.rs"), "").expect("probe lib");
        fs::write(
            package.join("Cargo.toml"),
            "[package]\nname = \"adversary2-ess-probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
             publish = false\n\n[dev-dependencies]\nserde_yaml_ng = \"0.10\"\n\n[workspace]\n",
        )
        .expect("probe manifest");
        fs::copy(root.join("Cargo.lock"), package.join("Cargo.lock")).expect("lockfile");
        fs::copy(
            root.join("crates/canon/tests/ess_model_matches.rs"),
            package.join("tests/ess_model_matches.rs"),
        )
        .expect("the unit's test copied unchanged");

        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
        let output = Command::new(cargo)
            .args([
                "test",
                "--offline",
                "--no-run",
                "--test",
                "ess_model_matches",
                "--message-format=json",
            ])
            .current_dir(&package)
            .env("CARGO_TARGET_DIR", base().join("target"))
            .output()
            .expect("cargo runs");
        assert!(
            output.status.success(),
            "building the unit's test failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let executable = stdout
            .lines()
            .filter_map(|line| serde_yaml_ng::from_str::<serde_yaml_ng::Value>(line).ok())
            .filter(|message| message["target"]["name"].as_str() == Some("ess_model_matches"))
            .find_map(|message| message["executable"].as_str().map(PathBuf::from))
            .expect("cargo names the test executable");
        assert!(
            executable.starts_with(base().join("target")),
            "the executable is this probe's own build: {}",
            executable.display()
        );
        executable
    })
}

/// A scratch root holding copies of `ess/` and `crates/canon/src/model/`.
fn scratch_root(name: &str) -> PathBuf {
    let root = base().join("roots").join(name);
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove the previous copy");
    }
    copy_dir(&repo_root().join("ess"), &root.join("ess"));
    copy_dir(
        &repo_root().join("crates/canon/src/model"),
        &root.join("crates/canon/src/model"),
    );
    root
}

fn edit(path: &Path, from: &str, to: &str) {
    let text = fs::read_to_string(path).expect("file reads");
    assert!(
        text.contains(from),
        "`{from}` is not in {}: the probe's edit no longer applies",
        path.display()
    );
    fs::write(path, text.replacen(from, to, 1)).expect("file written");
}

/// Runs the unit's `specification_and_model_are_equal` with `root` as the tree under test.
fn unit_test_on(root: &Path) -> (bool, String) {
    let output = Command::new(unit_binary())
        .args(["--exact", "specification_and_model_are_equal"])
        .env("CARGO_MANIFEST_DIR", root.join("crates/canon"))
        .output()
        .expect("the unit's test binary runs");
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        text.contains("running 1 test"),
        "the unit's test did not run:\n{text}"
    );
    (output.status.success(), text)
}

fn assert_unit_fails(root: &Path, what: &str) {
    let (passed, output) = unit_test_on(root);
    assert!(
        !passed,
        "{what}, and ess_model_matches::specification_and_model_are_equal still passes:\n{output}"
    );
}

/// The harness is faithful: the unchanged copy passes the unit's test.
#[test]
fn control_the_unchanged_copy_passes() {
    let root = scratch_root("control");
    let (passed, output) = unit_test_on(&root);
    assert!(passed, "the unchanged copy fails:\n{output}");
}

/// A model field retyped to a type outside `crates/canon/src/model/` whose last path segment is a
/// model type's name. The unit reads only the last segment (`last_segment`,
/// ess_model_matches.rs:569-571 via `rust_ty`), so `crate::ir::Claim` — a struct story:canon-ir
/// adds in this wave with different fields — is compared as the model's `Claim`, contrary to the
/// unit's own rule "Any other Rust type matches nothing" (ess_model_matches.rs:18).
#[test]
fn a_field_retyped_to_a_foreign_type_of_the_same_name_is_caught() {
    let root = scratch_root("foreign-claim");
    edit(
        &root.join("crates/canon/src/model/mod.rs"),
        "pub claims: Declarations<ClaimId, Claim>,",
        "pub claims: Declarations<ClaimId, crate::ir::Claim>,",
    );
    assert_unit_fails(
        &root,
        "Protocol.claims now holds crate::ir::Claim, which is not the model's Claim",
    );
}

/// A model field compiled out of every build: b10x-canon declares no features, so a field under
/// `#[cfg(feature = "effects")]` is not in the model that is built. The unit strips every attribute
/// from a struct body (`strip_attributes`, ess_model_matches.rs:430-441) and so still sees the
/// field; only `#[cfg(test)]` items are excluded, and only at module level.
#[test]
fn a_field_compiled_out_by_a_feature_is_caught() {
    let root = scratch_root("cfg-feature-field");
    edit(
        &root.join("crates/canon/src/model/mod.rs"),
        "    pub effect: Option<EffectClass>,",
        "    #[cfg(feature = \"effects\")]\n    pub effect: Option<EffectClass>,",
    );
    assert_unit_fails(
        &root,
        "Action.effect exists only under a feature b10x-canon does not declare, while ess/ declares it unconditionally",
    );
}

/// The specification changes a map key and an unlisted copy of the old domain file sits beside it.
/// `ess` compiles only the files `ess/ess-inputs.yaml` lists, so the spec now says
/// `Map<OutcomeId, Claim>`; the unit reads key types from every file in `ess/domains/`
/// (`authored_map_keys`, ess_model_matches.rs:135-159), last read wins, and the stale copy supplies
/// `ClaimId`, which is what the Rust model says.
#[test]
fn a_map_key_changed_in_the_compiled_spec_is_caught_beside_a_stale_copy() {
    let without_copy = scratch_root("map-key-no-stale-copy");
    let domain = without_copy.join("ess/domains/protocol.yaml");
    let from = "type: Map<canon.protocol.ClaimId, canon.protocol.Claim>";
    let to = "type: Map<canon.protocol.OutcomeId, canon.protocol.Claim>";
    edit(&domain, from, to);
    assert_unit_fails(
        &without_copy,
        "control: ess/ says claims is Map<OutcomeId, Claim> and the model says Declarations<ClaimId, Claim>",
    );

    let root = scratch_root("map-key-stale-copy");
    let domains = root.join("ess/domains");
    let original = fs::read_to_string(domains.join("protocol.yaml")).expect("domain reads");
    edit(&domains.join("protocol.yaml"), from, to);
    // A name `ess` does not compile, placed after protocol.yaml in directory order (which is the
    // file system's, often hash order, so several names are tried).
    let mut placed = None;
    let names = ["protocol.yaml.orig", "protocol.yaml~", "protocol.yaml.bak"]
        .map(str::to_owned)
        .into_iter()
        .chain((1..=64).map(|n| format!("protocol.yaml.orig.{n}")));
    for name in names {
        fs::write(domains.join(&name), &original).expect("stale copy written");
        let order: Vec<String> = fs::read_dir(&domains)
            .expect("domains dir reads")
            .map(|entry| {
                entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        let at = |file: &str| order.iter().position(|n| n == file).expect("listed");
        if at(&name) > at("protocol.yaml") {
            placed = Some(name);
            break;
        }
        fs::remove_file(domains.join(&name)).expect("stale copy removed");
    }
    let placed = placed.expect("some stale name is read after protocol.yaml");

    let validate = Command::new("ess")
        .args(["specify", "validate", "--path", "ess", "--strict-requires"])
        .current_dir(&root)
        .output()
        .expect("ess runs");
    assert!(
        validate.status.success(),
        "ess accepts the changed spec with {placed} beside it:\n{}",
        String::from_utf8_lossy(&validate.stderr)
    );
    assert_unit_fails(
        &root,
        &format!(
            "ess/ compiles claims as Map<OutcomeId, Claim> (ess ignores the unlisted {placed}) and the model says Declarations<ClaimId, Claim>"
        ),
    );
}

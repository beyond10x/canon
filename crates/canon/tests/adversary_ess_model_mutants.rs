//! Adversary pass 1 on story:ess-hard-gate.
//!
//! Runs the unit's own `ess_model_matches` test, unchanged, against mutated copies of the Rust
//! model, and asserts that it fails on each mutant that changes the model. Each copy is a scratch
//! repository under `CARGO_TARGET_TMPDIR`: `ess/` and `crates/canon/src/model/` copied, the edits
//! applied to the copy, and `crates/canon/tests/ess_model_matches.rs` as the only test of a probe
//! package whose `CARGO_MANIFEST_DIR` makes the copy the repository root. The tree is never edited.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"))
        .join("../..")
        .canonicalize()
        .expect("repository root resolves")
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

/// One textual edit to a model file: `(file under src/model, from, to)`. `from` must occur.
type Edit<'a> = (&'a str, &'a str, &'a str);

/// Whether the unit's `specification_and_model_are_equal` passes on a copy of the model with
/// `edits` applied, and its output.
fn unit_test_on_mutant(name: &str, edits: &[Edit]) -> (bool, String) {
    let root = repo_root();
    let base = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary-ess")
        .join(name);
    if base.exists() {
        fs::remove_dir_all(&base).expect("remove previous mutant copy");
    }
    copy_dir(&root.join("ess"), &base.join("ess"));
    let model = base.join("crates/canon/src/model");
    copy_dir(&root.join("crates/canon/src/model"), &model);
    for (file, from, to) in edits {
        let path = model.join(file);
        let text = fs::read_to_string(&path).expect("model file reads");
        assert!(text.contains(from), "mutant {name}: `{from}` not in {file}");
        fs::write(&path, text.replacen(from, to, 1)).expect("mutant written");
    }

    let package = base.join("crates/canon");
    fs::write(package.join("src/lib.rs"), "").expect("probe lib");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"adversary-ess-probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
         publish = false\n\n[dev-dependencies]\nserde_yaml_ng = \"0.10\"\n\n[workspace]\n",
    )
    .expect("probe manifest");
    fs::copy(root.join("Cargo.lock"), package.join("Cargo.lock")).expect("lockfile copied");
    fs::create_dir_all(package.join("tests")).expect("probe tests dir");
    fs::copy(
        root.join("crates/canon/tests/ess_model_matches.rs"),
        package.join("tests/ess_model_matches.rs"),
    )
    .expect("the unit's test copied unchanged");

    // One target directory per mutant: a shared one lets parallel probes run each other's binary.
    let target = base.join("target");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = Command::new(cargo)
        .args([
            "test",
            "--offline",
            "--test",
            "ess_model_matches",
            "--",
            "--exact",
            "specification_and_model_are_equal",
        ])
        .current_dir(&package)
        .env("CARGO_TARGET_DIR", &target)
        .output()
        .expect("cargo runs");
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        text.contains("running 1 test")
            && text.contains(&format!(
                "Running tests/ess_model_matches.rs ({}",
                target.display()
            )),
        "mutant {name}: the unit's test did not run from this mutant's own build:\n{text}"
    );
    (output.status.success(), text)
}

fn assert_caught(name: &str, edits: &[Edit]) {
    let (passed, output) = unit_test_on_mutant(name, edits);
    assert!(
        !passed,
        "mutant {name} changes the Rust model and ess_model_matches::specification_and_model_are_equal \
         still passes:\n{output}"
    );
}

/// The harness is faithful: the unmutated copy passes the unit's test.
#[test]
fn control_the_unmutated_copy_passes() {
    let (passed, output) = unit_test_on_mutant("control", &[]);
    assert!(passed, "the unmutated copy fails:\n{output}");
}

/// Every identifier newtype now wraps `u64`, while `ess/` still says `of: String`. The test reads
/// `identifier!(X)` as `newtype of String` without reading what the macro wraps
/// (ess_model_matches.rs:479-484).
#[test]
fn a_changed_identifier_representation_is_caught() {
    assert_caught(
        "identifier-wraps-u64",
        &[
            (
                "ids.rs",
                "pub struct $name(String);",
                "pub struct $name(u64);",
            ),
            (
                "ids.rs",
                "super::present::required::<D, String>(deserializer)",
                "super::present::required::<D, u64>(deserializer)",
            ),
        ],
    );
}

/// A field added to the real `Artifact`, while a later model file declares another item named
/// `Artifact` (a test fixture). Every `struct`/`enum` across the model files goes into one map by
/// bare name, last one wins (ess_model_matches.rs:485-498), so the real struct is never compared.
#[test]
fn a_field_added_beside_a_same_named_fixture_is_caught() {
    assert_caught(
        "shadowed-artifact",
        &[
            (
                "mod.rs",
                "pub struct Artifact {",
                "pub struct Artifact {\n    pub owner: String,",
            ),
            (
                "predicate.rs",
                "impl Predicate {",
                "#[cfg(test)]\nmod tests {\n    #[allow(dead_code)]\n    struct Artifact {\n        \
                 description: Option<String>,\n    }\n}\n\nimpl Predicate {",
            ),
        ],
    );
}

/// Holds-probe: a field type changed in a struct reached only through the union, written over
/// several lines under a doc comment and a serde attribute.
#[test]
fn a_multiline_attributed_type_change_in_a_union_payload_is_caught() {
    assert_caught(
        "multiline-result",
        &[(
            "predicate.rs",
            "    pub result: Option<String>,",
            "    /// The result, if any.\n    #[serde(rename = \"outcome\")]\n    pub result:\n        \
             Option<\n            Vec<String>,\n        >,",
        )],
    );
}

/// Holds-probe: a struct renamed at its definition and its use.
#[test]
fn a_renamed_struct_is_caught() {
    assert_caught(
        "renamed-obligation",
        &[
            ("mod.rs", "pub struct Obligation {", "pub struct Duty {"),
            (
                "mod.rs",
                "Declarations<ObligationId, Obligation>",
                "Declarations<ObligationId, Duty>",
            ),
        ],
    );
}

/// Holds-probe: a union variant's payload changed from one predicate to a list.
#[test]
fn a_changed_union_payload_is_caught() {
    assert_caught(
        "not-holds-list",
        &[("predicate.rs", "Not(Box<Predicate>)", "Not(Vec<Predicate>)")],
    );
}

/// `ess/` types `revision` as ESS `Integer`, which admits `[i64::MIN, i64::MAX]`
/// (ess docs/design/review-primitive-semantics.md:34); the model's `revision: u64` admits no
/// negative value. The specification states no `revision >= 0` invariant, so it admits a protocol
/// the model refuses, and ess_model_matches.rs:439 maps `u64` to `integer` as if equal.
#[test]
fn the_specification_bounds_revision_as_the_model_does() {
    let output = Command::new("ess")
        .args(["specify", "compile", "--path", "ess", "--format", "json"])
        .current_dir(repo_root())
        .output()
        .expect("ess runs");
    assert!(output.status.success(), "ess specify compile failed");
    let ir: serde_yaml_ng::Value =
        serde_yaml_ng::from_slice(&output.stdout).expect("compiled IR is JSON");
    let invariants = &ir["entities"]["canon.protocol.Protocol"]["invariants"];
    let text = serde_yaml_ng::to_string(invariants).expect("invariants render");
    assert!(
        text.contains("revision"),
        "canon.protocol.Protocol declares no invariant on revision; ESS Integer admits negative \
         revisions that the Rust model (u64) refuses. Invariants: {text}"
    );
}

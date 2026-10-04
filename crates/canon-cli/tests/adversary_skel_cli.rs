//! Adversary cases for story:evaluator-skeleton (wave 2026-10-04-w6, pass 1).
//!
//! Each case drives the `canon` binary from a statement the unit wrote about itself: the model and
//! `ess/` say a discharge predicate is "over claim values", and the story says "every stub is
//! inert", including for a protocol that declares an obligation and a case that records a
//! termination, neither of which any test of the unit evaluates.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SKELETON_FIXTURE: &str = "fixtures/investigation/evaluator-skeleton.yaml";

/// The decision the base fixture gives `INV-18` with no evidence (CANON-CLAIM-001's
/// `no-evidence` step): no section slot is written.
const NO_EVIDENCE_DECISION: &str = "{\n  \"case\": \"INV-18\",\n  \"claims\": {\n    \"explanation.supported\": {\n      \"value\": \"unknown\"\n    }\n  },\n  \"format\": \"canon-decision/1\",\n  \"protocol\": \"investigation\",\n  \"protocol_revision\": 1\n}\n";

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

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

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary_skel_cli")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("stale scratch directory removes");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory creates");
    dir
}

fn utf8(path: &Path) -> &str {
    path.to_str().expect("utf-8 path")
}

/// Compiles `fixture`, writes `case` and an empty evidence directory under `name`, and runs
/// `canon evaluate` over them.
fn evaluate(name: &str, fixture: &str, case: &str) -> Output {
    let dir = scratch(name);
    let compiled = canon(&["compile", "--path", fixture]);
    assert_eq!(
        compiled.status.code(),
        Some(0),
        "{fixture} compiles: {}",
        text(&compiled.stderr)
    );
    let ir = dir.join("protocol.ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("IR writes");
    let case_path = dir.join("case.yaml");
    std::fs::write(&case_path, case).expect("case writes");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory creates");
    canon(&[
        "evaluate",
        "--ir",
        utf8(&ir),
        "--case",
        utf8(&case_path),
        "--evidence",
        utf8(&evidence),
    ])
}

const CASE: &str = "format: canon-case/1\nid: INV-18\nprotocol: investigation\nartifacts:\n  explanation: {revision: r1}\n";

/// "Nothing evaluates differently: every stub is inert." The unit's tests evaluate only protocols
/// without obligations, and CANON-CLAIM-001 now compares only the sections it lists, so neither
/// would see an `obligations` section the stub wrote for a protocol that declares one. This
/// evaluates the unit's own fixture, which declares `establish.explanation`, and pins the whole
/// decision: it is the base fixture's, byte for byte. It also drives `read_ir` over an IR carrying
/// `discharged_when` from the binary.
#[test]
fn a_declared_obligation_changes_no_decision() {
    let run = evaluate("obligation", SKELETON_FIXTURE, CASE);
    assert_eq!(text(&run.stderr), "", "stderr");
    assert_eq!(text(&run.stdout), NO_EVIDENCE_DECISION, "canon-decision/1");
    assert_eq!(run.status.code(), Some(0), "exit");
}

/// `canon-case/1` gains `termination`, and "its check stays in `outcomes.rs`" which is a stub that
/// accepts any termination. A case recording one, declared or not, evaluates exactly as without.
#[test]
fn a_recorded_termination_changes_no_decision() {
    for (name, outcome) in [("declared", "supported"), ("undeclared", "abandoned")] {
        let case = format!("{CASE}termination: {outcome}\n");
        let run = evaluate(&format!("termination-{name}"), SKELETON_FIXTURE, &case);
        assert_eq!(text(&run.stderr), "", "{outcome}: stderr");
        assert_eq!(
            text(&run.stdout),
            NO_EVIDENCE_DECISION,
            "{outcome}: canon-decision/1"
        );
        assert_eq!(run.status.code(), Some(0), "{outcome}: exit");
    }
}

/// The model (`crates/canon/src/model/mod.rs`, `Obligation`) and `ess/domains/protocol.yaml`
/// (`canon.protocol.Obligation.discharged_when`) both say an obligation "is discharged only when
/// its discharge predicate, over claim values, evaluates true", and the skeleton hands
/// `obligations::section` only the claim values. A discharge predicate that tests evidence
/// directly is outside that contract, and nothing `obligations.rs` is given can evaluate it; the
/// validator accepts it today.
#[test]
fn a_discharge_predicate_that_tests_evidence_is_refused() {
    let fixture = std::fs::read_to_string(repository_root().join(SKELETON_FIXTURE))
        .unwrap_or_else(|error| panic!("{SKELETON_FIXTURE}: {error}"));
    let over_claims = "    discharged_when:\n      claim: explanation.supported\n      is: true\n";
    assert!(
        fixture.contains(over_claims),
        "the fixture's discharge predicate is where this case expects it"
    );
    let over_evidence = fixture.replace(
        over_claims,
        "    discharged_when:\n      evidence:\n        kind: supporting_observation\n",
    );
    let dir = scratch("discharge-over-evidence");
    let path = dir.join("protocol.yaml");
    std::fs::write(&path, over_evidence).expect("protocol writes");
    let run = canon(&["validate", "--path", utf8(&path)]);
    assert_eq!(
        run.status.code(),
        Some(1),
        "a discharge predicate over evidence is refused; stdout: {} stderr: {}",
        text(&run.stdout),
        text(&run.stderr)
    );
    assert_eq!(text(&run.stdout), "", "stdout");
    assert!(
        text(&run.stderr).contains("establish.explanation"),
        "the refusal names the obligation: {}",
        text(&run.stderr)
    );
}

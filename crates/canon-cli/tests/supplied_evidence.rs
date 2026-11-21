//! Acceptance for story:supplied-evidence against the `canon` command line, over
//! `fixtures/investigation/supplied-evidence/`: `supplied.yaml` is the protocol
//! `investigation.publication`, whose evidence kind `peer_review` declares
//! `supplied_by: investigation.review`; `unsupplied.yaml` is the same document without that line.
//! `bypass.yaml` and `bypass-unsupplied.yaml` are the pair for `authority-bypass`. `case.yaml` is a
//! case of the protocol, and `copied/peer-review.yaml` a record copied byte for byte from a case
//! under the supplier, which names, in `upstream_revisions`, the declared artifact `dataset` and
//! the artifact `guideline` only the supplier declares (built from the SDC session's saved case
//! `docs/limitations/copied-record-upstream/`).
//!
//! `supplied_evidence_is_produced_outside` holds the story's expectations, one block each,
//! numbered as the story numbers them, with expectation 11 the coordinator's option B (a record of
//! a supplied kind may name upstream artifacts the protocol does not declare). Expectation 9 is
//! built as the tree's precedents class a supplier introduced, since an evaluation reads whether
//! there is one (`diff/mod.rs`). Expectation 10, the model parity, is
//! `crates/canon/tests/ess_model_matches.rs` itself. CANON-SUPPLY-001
//! (`conformance/scenarios/supplied-evidence.yaml`) is also run over the whole registry
//! (`conform_requirements.rs`).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::{ir, model};

const DIR: &str = "fixtures/investigation/supplied-evidence";
const SCENARIO: &str = "conformance/scenarios/supplied-evidence.yaml";
/// The line `supplied.yaml` holds and `unsupplied.yaml` does not.
const SUPPLIED_LINE: &str = "    supplied_by: investigation.review\n";
/// The digest of `unsupplied.yaml`'s `canon-ir/1` as canon 0.9.0 compiled it at `be2c861`, before
/// the field existed.
const UNSUPPLIED_DIGEST_BEFORE: &str =
    "sha256:6bc8605cc7a5c2fd25d5529373e3b08ba126a9355f0440e0add781c2f020c137";

/// Read at run time, not compile time: a test binary reused from a shared build directory must
/// still read this tree's scenarios and fixtures.
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
        .expect("the canon binary runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

fn fixture(name: &str) -> String {
    format!("{DIR}/{name}")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repository_root().join(path))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// A scratch directory of this test binary, inside the build directory, emptied first.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("supplied-evidence")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

fn path(path: &Path) -> &str {
    path.to_str().expect("utf-8 path")
}

/// Exit 0, nothing on standard error; standard output.
fn succeeded(output: &Output) -> String {
    assert_eq!(text(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0));
    text(&output.stdout).to_owned()
}

/// A refusal: exit 1, nothing on standard output, and exactly `stderr`.
fn assert_refused(output: &Output, stderr: &str) {
    assert_eq!(text(&output.stderr), stderr);
    assert_eq!(text(&output.stdout), "");
    assert_eq!(output.status.code(), Some(1));
}

/// `canon compile` of the fixture `name`, written to `dir`; its bytes and its path.
fn compiled(name: &str, dir: &Path) -> (String, PathBuf) {
    let bytes = succeeded(&canon(&["compile", "--path", &fixture(name)]));
    let at = dir.join(name.replace(".yaml", ".ir.json"));
    std::fs::write(&at, &bytes).expect("IR written");
    (bytes, at)
}

/// The observation record every evidence directory below holds.
const OBSERVATION: &str = "format: canon-evidence/1\nid: observation-1\nkind: observation\n\
    result: consistent\nsubject: explanation\nsubject_revision: r1\n";

/// An evidence directory under `dir`: the observation, and the peer review when given.
fn evidence(dir: &Path, name: &str, review: Option<&str>) -> PathBuf {
    let at = dir.join(name);
    std::fs::create_dir_all(&at).expect("evidence directory");
    std::fs::write(at.join("observation.yaml"), OBSERVATION).expect("observation written");
    if let Some(review) = review {
        std::fs::write(at.join("peer-review.yaml"), review).expect("review written");
    }
    at
}

fn evaluate(ir: &Path, evidence: &Path) -> Output {
    canon(&[
        "evaluate",
        "--ir",
        path(ir),
        "--case",
        &fixture("case.yaml"),
        "--evidence",
        path(evidence),
    ])
}

#[test]
fn supplied_evidence_is_produced_outside() {
    let dir = scratch("acceptance");
    let copied = read(&fixture("copied/peer-review.yaml"));
    // The copied record with its undeclared upstream entry left out: what a runner had to write
    // before this story.
    let stripped = copied.replace("  guideline: g7\n", "");
    assert_ne!(stripped, copied);
    let (supplied_ir, supplied_at) = compiled("supplied.yaml", &dir);
    let (unsupplied_ir, unsupplied_at) = compiled("unsupplied.yaml", &dir);

    // 1. CANON-SUPPLY-001 passes, run on its own; with no supplied record the claim is UNKNOWN,
    // with one TRUE (the scenario's steps, and the same evaluations on the command line).
    let registry = scratch("registry");
    std::fs::write(registry.join("supplied-evidence.yaml"), read(SCENARIO)).expect("copied");
    assert_eq!(
        succeeded(&canon(&["conform", "run", "--scenarios", path(&registry)])),
        "passed: scenario `CANON-SUPPLY-001`\nconform: 1 passed, 0 failed, 0 unreadable\n"
    );
    let claim = |output: &str, value: &str| {
        output.contains(&format!(
            "    \"peer.approved\": {{\n      \"value\": \"{value}\"\n    }}"
        ))
    };
    let none = succeeded(&evaluate(&supplied_at, &evidence(&dir, "none", None)));
    assert!(claim(&none, "unknown"), "{none}");
    let given = succeeded(&evaluate(
        &supplied_at,
        &evidence(&dir, "given", Some(&stripped)),
    ));
    assert!(claim(&given, "true"), "{given}");
    assert!(
        given.contains("  \"outcomes\": {\n    \"published\": {\n      \"status\": \"legitimate\""),
        "{given}"
    );

    // 2. The supplied kind compiles with its supplier, after its other members, as the scenario
    // expects byte for byte.
    assert!(
        supplied_ir.contains(
            "      \"description\": \"A peer's review of the finding, recorded in an \
             investigation.review case and copied here unchanged; its result is approved or \
             rejected.\",\n      \"supplied_by\": \"investigation.review\"\n    }\n"
        ),
        "{supplied_ir}"
    );
    let scenario = b10x_canon::conform::parse(&read(SCENARIO)).expect("the scenario parses");
    let b10x_canon::conform::StepKind::Compile { expected_ir } = &scenario.steps[0].kind else {
        panic!("the first step compiles");
    };
    assert_eq!(*expected_ir, supplied_ir);

    // 3. A protocol without `supplied_by` compiles to the bytes it did before the story: the
    // unsupplied twin's digest is the one recorded on the base, and its IR is the supplied one
    // without the `supplied_by` member.
    let unsupplied = ir::compile(
        &model::parse(&read(&fixture("unsupplied.yaml"))).expect("unsupplied.yaml parses"),
    )
    .expect("unsupplied.yaml compiles");
    assert_eq!(unsupplied.canonical_json(), unsupplied_ir);
    assert_eq!(ir::digest(&unsupplied), UNSUPPLIED_DIGEST_BEFORE);
    assert_eq!(
        supplied_ir.replace(
            "rejected.\",\n      \"supplied_by\": \"investigation.review\"\n",
            "rejected.\"\n"
        ),
        unsupplied_ir
    );
    assert_eq!(
        read(&fixture("supplied.yaml")).replace(SUPPLIED_LINE, ""),
        read(&fixture("unsupplied.yaml"))
    );

    // 4. Unsupplied, every reader of the kind is reported, exit 1.
    let unproduced = canon(&["check", "--path", &fixture("unsupplied.yaml")]);
    assert_eq!(text(&unproduced.stderr), "");
    assert_eq!(
        text(&unproduced.stdout),
        "unproduced-evidence: claim `peer.approved` reads evidence kind `peer_review`, which no \
         action may produce\n\
         unproduced-evidence: outcome `published` reads evidence kind `peer_review`, which no \
         action may produce\n\
         checked: protocol `investigation.publication` revision 1: 64 states, 0 properties, 2 \
         findings\n"
    );
    assert_eq!(unproduced.status.code(), Some(1));

    // 5. Supplied, no finding, exit 0, over the same states.
    assert_eq!(
        succeeded(&canon(&["check", "--path", &fixture("supplied.yaml")])),
        "checked: protocol `investigation.publication` revision 1: 64 states, 0 properties, 0 \
         findings\n"
    );

    // 6. An outcome that reads the supplied kind and a governed kind a capability-free action
    // also produces is a bypass, witnessed by a state holding the supplied kind.
    let bypassed = canon(&["check", "--path", &fixture("bypass.yaml")]);
    assert_eq!(text(&bypassed.stderr), "");
    assert_eq!(
        text(&bypassed.stdout),
        "authority-bypass: outcome `approved_for_publication` is legitimate without authority in \
         state {evidence `observation` result `consistent`, evidence `peer_review` result \
         `approved`, evidence `publication_approval`}\n\
         checked: protocol `investigation.publication` revision 1: 96 states, 0 properties, 1 \
         finding\n"
    );
    assert_eq!(bypassed.status.code(), Some(1));

    // 7. With the kind neither supplied nor produced, no state holding it is a witness, so the
    // bypass is not reported; only the unproduced kind is.
    let hidden = canon(&["check", "--path", &fixture("bypass-unsupplied.yaml")]);
    assert_eq!(text(&hidden.stderr), "");
    assert_eq!(
        text(&hidden.stdout),
        "unproduced-evidence: claim `peer.approved` reads evidence kind `peer_review`, which no \
         action may produce\n\
         unproduced-evidence: outcome `approved_for_publication` reads evidence kind \
         `peer_review`, which no action may produce\n\
         checked: protocol `investigation.publication` revision 1: 96 states, 0 properties, 2 \
         findings\n"
    );
    assert_eq!(hidden.status.code(), Some(1));

    // 8. A protocol that names its own id as the supplier is refused, naming the kind.
    assert_refused(
        &canon(&[
            "validate",
            "--path",
            "fixtures/investigation/invalid/supplied-by-self.yaml",
        ]),
        "error[self-supplied-evidence]: evidence kind `peer_review` is supplied by protocol \
         `investigation.publication`, the protocol's own id\n",
    );

    // 9. An evaluation reads whether a kind has a supplier (expectation 11), so introducing one
    // is not NO SEMANTIC CHANGE: it admits records once refused and changes no other decision,
    // which the tree classes `EXPANSION` (a value added to a case input); removing one refuses
    // them again, `BREAKING`. Another supplier changes no evaluation, but it is a name the runner
    // acts on and nothing resolves, as a `case_open` protocol is, so it is `BREAKING` too.
    let diff = |from: &Path, to: &Path| {
        succeeded(&canon(&["diff", "--from", path(from), "--to", path(to)]))
    };
    assert_eq!(
        diff(&unsupplied_at, &supplied_at),
        "diff: protocol `investigation.publication` revision 1 to 1\n\
         EXPANSION: evidence kind `peer_review` supplier introduced: protocol \
         `investigation.review`\n"
    );
    assert_eq!(
        diff(&supplied_at, &unsupplied_at),
        "diff: protocol `investigation.publication` revision 1 to 1\n\
         BREAKING: evidence kind `peer_review` supplier removed: was protocol \
         `investigation.review`\n"
    );
    let resupplied_at = dir.join("resupplied.ir.json");
    std::fs::write(
        &resupplied_at,
        supplied_ir.replace(
            "\"supplied_by\": \"investigation.review\"",
            "\"supplied_by\": \"investigation.audit\"",
        ),
    )
    .expect("written");
    assert_eq!(
        diff(&supplied_at, &resupplied_at),
        "diff: protocol `investigation.publication` revision 1 to 1\n\
         BREAKING: evidence kind `peer_review` supplier changed from protocol \
         `investigation.review` to protocol `investigation.audit`\n"
    );

    // 11a. The record copied byte for byte, naming `guideline`, which only the supplier declares,
    // is evaluated, and the decision is byte-identical to the one without that entry.
    let copied_dir = dir.join("copied");
    std::fs::create_dir_all(&copied_dir).expect("evidence directory");
    std::fs::copy(
        repository_root().join(fixture("copied/peer-review.yaml")),
        copied_dir.join("peer-review.yaml"),
    )
    .expect("copied");
    std::fs::write(copied_dir.join("observation.yaml"), OBSERVATION).expect("written");
    assert_eq!(succeeded(&evaluate(&supplied_at, &copied_dir)), given);

    // 11b. The same record under the kind without `supplied_by` is refused, as before.
    assert_refused(
        &evaluate(&unsupplied_at, &copied_dir),
        &format!(
            "error[undeclared-artifact]: {}: evidence `peer-review-1` records a revision of \
             upstream artifact `guideline`, which the protocol does not declare\n",
            path(&copied_dir.join("peer-review.yaml"))
        ),
    );

    // 11c. The invalidation rule cannot name `guideline`, and never fires on it: the copied record
    // is invalidated exactly when, and exactly as, the record without that entry is, by its
    // declared upstream `dataset`.
    let moved = |record: &str| record.replace("  dataset: d2\n", "  dataset: d1\n");
    let copied_moved = succeeded(&evaluate(
        &supplied_at,
        &evidence(&dir, "copied-moved", Some(&moved(&copied))),
    ));
    let stripped_moved = succeeded(&evaluate(
        &supplied_at,
        &evidence(&dir, "stripped-moved", Some(&moved(&stripped))),
    ));
    assert_eq!(copied_moved, stripped_moved);
    assert!(
        copied_moved.contains(
            "    \"peer.approved\": {\n      \"excluded_evidence\": [\n        {\n          \
             \"evidence\": \"peer-review-1\",\n          \"reason\": \"invalidated\"\n        }\n      \
             ],\n      \"value\": \"unknown\""
        ),
        "{copied_moved}"
    );
    let watching = dir.join("watching-guideline.yaml");
    std::fs::write(
        &watching,
        read(&fixture("supplied.yaml")).replace("upstream: dataset", "upstream: guideline"),
    )
    .expect("written");
    assert_refused(
        &canon(&["validate", "--path", path(&watching)]),
        "error[undeclared-artifact]: invalidation rule `dataset.revised` references artifact \
         `guideline`, which is not declared\n",
    );
}

/// story:protocol-imports refuses an identifier two imports declare with different `canon-ir/1`
/// entries: two parts whose `peer_review` names different suppliers stay refused, and two that
/// name the same one are one declaration.
#[test]
fn two_imports_with_different_suppliers_conflict() {
    let dir = scratch("imports");
    let part = |id: &str, supplier: &str| {
        let source = format!(
            "format: protocol/1\nprotocol: {{id: {id}, revision: 1}}\n\
             evidence_kinds: {{peer_review: {{supplied_by: {supplier}}}}}\n"
        );
        let digest = ir::digest(
            &ir::compile(&model::parse(&source).expect("the part parses"))
                .expect("the part compiles"),
        );
        let at = dir.join(format!("{id}-{supplier}.yaml"));
        std::fs::write(&at, source).expect("part written");
        (at, digest)
    };
    let compose = |first: (PathBuf, String), second: (PathBuf, String)| {
        let root = dir.join("root.yaml");
        std::fs::write(
            &root,
            format!(
                "format: protocol/1\nprotocol: {{id: root, revision: 1}}\n\
                 imports:\n  lib.a: {{revision: 1, digest: '{}'}}\n  \
                 lib.b: {{revision: 1, digest: '{}'}}\n",
                first.1, second.1
            ),
        )
        .expect("root written");
        canon(&[
            "compile",
            "--path",
            path(&root),
            "--import",
            path(&first.0),
            "--import",
            path(&second.0),
        ])
    };
    assert_refused(
        &compose(
            part("lib.a", "investigation.review"),
            part("lib.b", "investigation.audit"),
        ),
        "error[conflicting-imports]: protocol `root` imports evidence kind `peer_review` from \
         `lib.a` and from `lib.b`, declared differently\n",
    );
    let shared = succeeded(&compose(
        part("lib.a", "investigation.review"),
        part("lib.b", "investigation.review"),
    ));
    assert_eq!(
        shared
            .matches("\"supplied_by\": \"investigation.review\"")
            .count(),
        1,
        "{shared}"
    );
}

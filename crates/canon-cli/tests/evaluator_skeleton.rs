//! Acceptance for story:evaluator-skeleton: splitting the evaluator into one module per concept and
//! landing the shared slots changes no evaluation. `canon conform run` still passes
//! `CANON-CLAIM-001` unedited; an evaluate step compares only the sections of `canon-decision/1` its
//! expectation lists, each byte for byte; `canon compile` carries an obligation's discharge
//! predicate into `canon-ir/1`; and `canon validate` refuses a discharge predicate that tests an
//! undeclared claim, naming it.
//!
//! The CLI inputs later stories own are declared here and inert: `canon evaluate --at` parses and
//! is refused naming the flag, and `canon diff --from --to` parses and is refused as not built.
//! `--authority` was inert here too; story:action-admissibility now reads it, and its test below
//! checks that it is read (refused naming the flag when malformed, evaluated when well-formed).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The scenario story:three-valued-claims added, which this story must leave passing unedited.
const CLAIM_SCENARIO: &str = "conformance/scenarios/three-valued-claims.yaml";
/// The base fixture plus the obligation `establish.explanation`.
const SKELETON_FIXTURE: &str = "fixtures/investigation/evaluator-skeleton.yaml";
/// The same obligation, discharged by a claim the protocol does not declare.
const UNDECLARED_DISCHARGE: &str = "fixtures/investigation/invalid/undeclared-discharge-claim.yaml";

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

/// Runs `canon <args>` from the repository root, so that scenario fixture paths resolve against it.
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

/// A fresh registry directory under this test's scratch space holding exactly one scenario file.
fn registry(name: &str, file: &str, scenario: &str) -> PathBuf {
    let dir = scratch(name);
    std::fs::write(dir.join(file), scenario).expect("scenario writes");
    dir
}

/// A fresh, empty directory under this test's scratch space; each test uses its own name.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("evaluator_skeleton")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("stale scratch directory removes");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory creates");
    dir
}

fn conform_run(dir: &Path) -> Output {
    canon(&[
        "conform",
        "run",
        "--scenarios",
        dir.to_str().expect("utf-8 path"),
    ])
}

/// One evaluate step over the three-valued-claims fixture with no evidence, so
/// `explanation.supported` is `unknown`; `decision` is the expected document, indented to fit.
fn one_step_scenario(id: &str, decision: &str) -> String {
    let mut scenario = format!(
        "format: canon-conformance/1\n\
         id: {id}\n\
         covers: []\n\
         fixture: fixtures/investigation/three-valued-claims.yaml\n\
         steps:\n  \
           - id: no-evidence\n    \
             evaluate:\n      \
               case:\n        \
                 format: canon-case/1\n        \
                 id: INV-18\n        \
                 protocol: investigation\n        \
                 artifacts:\n          \
                   explanation: {{revision: r1}}\n      \
               evidence: []\n    \
             expect:\n      \
               decision: |\n"
    );
    for line in decision.lines() {
        scenario.push_str("        ");
        scenario.push_str(line);
        scenario.push('\n');
    }
    scenario
}

/// The decision without its `claims` section, which the evaluation carries.
const WITHOUT_CLAIMS: &str = r#"{
  "case": "INV-18",
  "format": "canon-decision/1",
  "protocol": "investigation",
  "protocol_revision": 1
}"#;

/// Only the `claims` section, with a value the evaluation does not give (`unknown` is right).
const CLAIMS_DIFFER: &str = r#"{
  "claims": {
    "explanation.supported": {
      "value": "true"
    }
  }
}"#;

/// The `canon-ir/1` `fixtures/investigation/evaluator-skeleton.yaml` compiles to: the base
/// fixture's, with `establish.explanation` and its discharge predicate under `obligations`.
const SKELETON_IR: &str = r#"{
  "actions": {
    "attempt_falsification": {
      "description": "Try to refute the explanation.",
      "effect": null,
      "may_produce": [
        {
          "evidence": "falsification_attempt"
        }
      ],
      "precondition": {
        "all": []
      },
      "requires": []
    },
    "inspect": {
      "description": "Look for observations bearing on the explanation.",
      "effect": null,
      "may_produce": [
        {
          "evidence": "supporting_observation"
        }
      ],
      "precondition": {
        "all": []
      },
      "requires": []
    }
  },
  "artifacts": {
    "explanation": {
      "description": "The explanation under investigation."
    }
  },
  "claims": {
    "explanation.supported": {
      "description": "The explanation is supported by observation and has survived falsification.",
      "true_when": {
        "all": [
          {
            "evidence": {
              "kind": "falsification_attempt",
              "result": "survived"
            }
          },
          {
            "evidence": {
              "kind": "supporting_observation",
              "result": null
            }
          }
        ]
      }
    }
  },
  "evidence_kinds": {
    "falsification_attempt": {
      "description": "An attempt to refute the explanation; its result is survived or refuted."
    },
    "supporting_observation": {
      "description": "An observation consistent with the explanation."
    }
  },
  "format": "canon-ir/1",
  "obligations": {
    "establish.explanation": {
      "description": "Establish whether the explanation is supported.",
      "discharged_when": {
        "claim": {
          "id": "explanation.supported",
          "is": "true"
        }
      }
    }
  },
  "outcomes": {
    "supported": {
      "description": "The explanation is supported.",
      "requires": {
        "claim": {
          "id": "explanation.supported",
          "is": "true"
        }
      }
    }
  },
  "protocol": {
    "description": "Decide whether an explanation is supported by observation and survives an attempt to falsify it.",
    "id": "investigation",
    "revision": 1
  }
}
"#;

#[test]
fn evaluator_skeleton_is_inert() {
    // 1. CANON-CLAIM-001, byte for byte as committed, still passes on its own.
    let committed = std::fs::read_to_string(repository_root().join(CLAIM_SCENARIO))
        .unwrap_or_else(|error| panic!("{CLAIM_SCENARIO}: {error}"));
    let claims = registry("claim-scenario", "three-valued-claims.yaml", &committed);
    let run = conform_run(&claims);
    assert_eq!(text(&run.stderr), "", "CANON-CLAIM-001: stderr");
    assert_eq!(
        text(&run.stdout),
        "passed: scenario `CANON-CLAIM-001`\n\
         conform: 1 passed, 0 failed, 0 unreadable\n",
        "CANON-CLAIM-001: report"
    );
    assert_eq!(run.status.code(), Some(0), "CANON-CLAIM-001: exit");

    // 2a. An expectation that leaves out a section the decision carries (`claims`) passes: only
    // the listed sections are compared. Red before the per-section comparison: the whole document
    // was compared and differed.
    let omitted = registry(
        "section-omitted",
        "section-omitted.yaml",
        &one_step_scenario("SKELETON-SECTION-OMITTED", WITHOUT_CLAIMS),
    );
    let run = conform_run(&omitted);
    assert_eq!(text(&run.stderr), "", "omitted section: stderr");
    assert_eq!(
        text(&run.stdout),
        "passed: scenario `SKELETON-SECTION-OMITTED`\n\
         conform: 1 passed, 0 failed, 0 unreadable\n",
        "omitted section: report"
    );
    assert_eq!(run.status.code(), Some(0), "omitted section: exit");

    // 2b. A listed section that differs still fails, naming the scenario, the step and the
    // section, so leaving sections out cannot make every expectation pass.
    let differs = registry(
        "section-differs",
        "section-differs.yaml",
        &one_step_scenario("SKELETON-SECTION-DIFFERS", CLAIMS_DIFFER),
    );
    let run = conform_run(&differs);
    assert_eq!(text(&run.stderr), "", "differing section: stderr");
    let report = text(&run.stdout);
    let lines: Vec<&str> = report.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "differing section: one line and a summary: {report}"
    );
    let prefix = "failed: scenario `SKELETON-SECTION-DIFFERS` step `no-evidence`: ";
    assert!(
        lines[0].starts_with(prefix) && lines[0][prefix.len()..].contains("section `claims`"),
        "differing section: names scenario, step and section: {}",
        lines[0]
    );
    assert_eq!(
        lines[1], "conform: 0 passed, 1 failed, 0 unreadable",
        "differing section: summary"
    );
    assert_eq!(run.status.code(), Some(1), "differing section: exit");

    // 3. The obligation's discharge predicate reaches canon-ir/1.
    let compiled = canon(&["compile", "--path", SKELETON_FIXTURE]);
    assert_eq!(text(&compiled.stderr), "", "compile: stderr");
    assert_eq!(text(&compiled.stdout), SKELETON_IR, "compile: canon-ir/1");
    assert_eq!(compiled.status.code(), Some(0), "compile: exit");

    // 4. A discharge predicate testing an undeclared claim is refused, naming the claim.
    let refused = canon(&["validate", "--path", UNDECLARED_DISCHARGE]);
    assert_eq!(
        text(&refused.stderr),
        "error[undeclared-claim]: obligation `establish.explanation` references claim \
         `explanation.refuted`, which is not declared\n",
        "validate: refusal"
    );
    assert_eq!(text(&refused.stdout), "", "validate: stdout");
    assert_eq!(refused.status.code(), Some(1), "validate: exit");
}

/// The `canon evaluate` inputs of CANON-CLAIM-001's `no-evidence` step, written under `name`:
/// the compiled three-valued-claims fixture, the case snapshot and an empty evidence directory.
/// Returns the `--ir --case --evidence` arguments and the scratch directory.
fn evaluate_inputs(name: &str) -> (Vec<String>, PathBuf) {
    let dir = scratch(name);
    let compiled = canon(&[
        "compile",
        "--path",
        "fixtures/investigation/three-valued-claims.yaml",
    ]);
    assert_eq!(compiled.status.code(), Some(0), "the fixture compiles");
    let ir = dir.join("protocol.ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("IR writes");
    let case = dir.join("case.yaml");
    std::fs::write(
        &case,
        "format: canon-case/1\nid: INV-18\nprotocol: investigation\n\
         artifacts:\n  explanation: {revision: r1}\n",
    )
    .expect("case writes");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory creates");
    let path = |p: &Path| p.to_str().expect("utf-8 path").to_owned();
    let args = vec![
        "evaluate".to_owned(),
        "--ir".to_owned(),
        path(&ir),
        "--case".to_owned(),
        path(&case),
        "--evidence".to_owned(),
        path(&evidence),
    ];
    (args, dir)
}

fn canon_owned(args: &[String]) -> Output {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    canon(&args)
}

/// Asserts that `canon evaluate` without `flag` gives a decision, and with it is refused by the
/// library naming the flag: exit 1, one `error[<code>]: …` line, no decision. A flag clap did not
/// declare would instead be a usage error, exit 2.
fn assert_evaluate_flag_is_inert(flag: &str, value: &str, args: Vec<String>) {
    let control = canon_owned(&args);
    assert_eq!(
        control.status.code(),
        Some(0),
        "{flag}: the same inputs without it evaluate: {}",
        text(&control.stderr)
    );

    let mut with_flag = args;
    with_flag.push(flag.to_owned());
    with_flag.push(value.to_owned());
    let run = canon_owned(&with_flag);
    let stderr = text(&run.stderr);
    assert_eq!(run.status.code(), Some(1), "{flag}: exit; stderr: {stderr}");
    assert_eq!(text(&run.stdout), "", "{flag}: no decision is printed");
    assert!(
        stderr.starts_with("error[") && stderr.ends_with('\n') && stderr.lines().count() == 1,
        "{flag}: one refusal line: {stderr}"
    );
    assert!(
        stderr.contains(&format!("`{flag}`")),
        "{flag}: the refusal names the flag: {stderr}"
    );
}

/// `canon evaluate --authority <file>` is declared and read.
///
/// Changed by story:action-admissibility, which parses it: the skeleton refused any text naming
/// the flag. Text that is not `canon-authority/1` is still refused naming the flag (the same
/// checks as before); a well-formed list is now evaluated, and the decision carries the
/// `actions` section.
#[test]
fn evaluate_authority_parses_and_is_read() {
    let (args, dir) = evaluate_inputs("authority-flag");
    let authority = dir.join("authority.yaml");
    std::fs::write(&authority, "any text: not a canon-authority/1 list\n")
        .expect("authority file writes");
    assert_evaluate_flag_is_inert(
        "--authority",
        authority.to_str().expect("utf-8 path"),
        args.clone(),
    );

    let granted = dir.join("granted.yaml");
    std::fs::write(
        &granted,
        "- capability: finding.publish\n  decision: granted\n",
    )
    .expect("authority file writes");
    let mut with_flag = args;
    with_flag.push("--authority".to_owned());
    with_flag.push(granted.to_str().expect("utf-8 path").to_owned());
    let run = canon_owned(&with_flag);
    assert_eq!(text(&run.stderr), "", "well-formed --authority: stderr");
    assert_eq!(run.status.code(), Some(0), "well-formed --authority: exit");
    assert!(
        text(&run.stdout).contains("\"actions\": {"),
        "well-formed --authority: the decision carries actions: {}",
        text(&run.stdout)
    );
}

/// `canon evaluate --at <instant>` is declared and passed through unparsed, and refused until
/// story:evidence-freshness parses it.
#[test]
fn evaluate_at_parses_and_is_inert() {
    let (args, _dir) = evaluate_inputs("at-flag");
    assert_evaluate_flag_is_inert("--at", "2026-10-04T00:00:00Z", args);
}

/// `canon diff --from --to` is declared and refuses as not built until story:semantic-diff.
#[test]
fn diff_parses_and_refuses_as_not_built() {
    let (_, dir) = evaluate_inputs("diff-command");
    let ir = dir.join("protocol.ir.json");
    let ir = ir.to_str().expect("utf-8 path");
    let run = canon(&["diff", "--from", ir, "--to", ir]);
    let stderr = text(&run.stderr);
    assert_eq!(run.status.code(), Some(1), "diff: exit; stderr: {stderr}");
    assert_eq!(text(&run.stdout), "", "diff: stdout");
    assert!(
        stderr.starts_with("error[")
            && stderr.ends_with('\n')
            && stderr.lines().count() == 1
            && stderr.contains("`canon diff`")
            && stderr.contains("not built"),
        "diff: one line refusing `canon diff` as not built: {stderr}"
    );
}

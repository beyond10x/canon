//! Acceptance for story:canon-ir: `canon compile --path` turns a valid `protocol/1` document into
//! `canon-ir/1`, and two equivalent documents compile to byte-identical output whatever their path
//! and the working directory they are compiled from.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Read at run time, not compile time: a test binary reused from a shared build directory must
/// still read this tree's fixtures.
fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn fixtures() -> PathBuf {
    repository_root().join("fixtures/investigation")
}

/// Runs `canon compile --path <path>` from the working directory `cwd`.
fn compile_in(cwd: &Path, path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(cwd)
        .arg("compile")
        .arg("--path")
        .arg(path)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

/// The `canon-ir/1` the base investigation fixture compiles to.
const BASE_IR: &str = r#"{
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
  "obligations": {},
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
fn canon_ir_canonical_form() {
    let root = repository_root();
    let from_root = compile_in(&root, Path::new("fixtures/investigation/protocol.yaml"));
    let from_fixtures = compile_in(&fixtures(), Path::new("canon-ir.yaml"));

    for (name, output) in [
        ("protocol.yaml from the root", &from_root),
        ("canon-ir.yaml from fixtures/investigation", &from_fixtures),
    ] {
        assert_eq!(text(&output.stderr), "", "{name}");
        assert_eq!(output.status.code(), Some(0), "{name}");
        assert!(
            text(&output.stdout).contains("\"format\": \"canon-ir/1\""),
            "{name}: {}",
            text(&output.stdout)
        );
    }
    assert_eq!(
        text(&from_root.stdout),
        text(&from_fixtures.stdout),
        "equivalent documents compile to the same canon-ir/1"
    );

    let ir = text(&from_root.stdout);
    let root_text = root.to_str().expect("utf-8 repository root");
    for path_fragment in [root_text, "fixtures", "protocol.yaml", "canon-ir.yaml"] {
        assert!(
            !ir.contains(path_fragment),
            "the IR embeds `{path_fragment}`: {ir}"
        );
    }
}

#[test]
fn compile_emits_the_canonical_ir_of_the_base_fixture() {
    let output = compile_in(&repository_root(), &fixtures().join("protocol.yaml"));
    assert_eq!(text(&output.stderr), "");
    assert_eq!(text(&output.stdout), BASE_IR);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn compile_output_is_byte_identical_across_repeated_runs() {
    let path = fixtures().join("canon-ir.yaml");
    let first = compile_in(&repository_root(), &path);
    for _ in 0..4 {
        let again = compile_in(&repository_root(), &path);
        assert_eq!(again.stdout, first.stdout);
        assert_eq!(again.stderr, first.stderr);
        assert_eq!(again.status.code(), first.status.code());
    }
}

#[test]
fn compile_refuses_an_invalid_document_with_the_validator_problems() {
    let cases = [
        (
            "undeclared-claim.yaml",
            "error[undeclared-claim]: outcome `supported` references claim `explanation.refuted`, which is not declared\n",
        ),
        (
            "duplicate-identifier.yaml",
            "error[duplicate-identifier]: action `inspect` is declared more than once\n",
        ),
        (
            "undeclared-evidence-kind.yaml",
            "error[undeclared-evidence-kind]: action `attempt_falsification` references evidence kind `expert_opinion`, which is not declared\n",
        ),
    ];
    for (file, expected) in cases {
        let output = compile_in(&repository_root(), &fixtures().join("invalid").join(file));
        assert_eq!(text(&output.stderr), expected, "{file}");
        assert_eq!(text(&output.stdout), "", "{file}");
        assert_eq!(output.status.code(), Some(1), "{file}");
    }
}

#[test]
fn compile_refuses_an_unreadable_path() {
    let missing = Path::new(env!("CARGO_TARGET_TMPDIR")).join("no-such-protocol.yaml");
    let output = compile_in(&repository_root(), &missing);
    assert_eq!(text(&output.stdout), "");
    assert!(
        text(&output.stderr).starts_with("error[unreadable]: "),
        "{}",
        text(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn compile_escapes_document_text_as_json() {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("escaped-text.yaml");
    std::fs::write(
        &path,
        "format: protocol/1\nprotocol: {id: 'p\"q\\r', revision: 7, description: \"line one\\nline \\\"two\\\"\\t\\u0001\"}\n",
    )
    .expect("write fixture");
    let output = compile_in(&repository_root(), &path);
    assert_eq!(text(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0));
    assert!(
        text(&output.stdout).contains(
            "  \"protocol\": {\n    \"description\": \"line one\\nline \\\"two\\\"\\t\\u0001\",\n    \"id\": \"p\\\"q\\\\r\",\n    \"revision\": 7\n  }\n"
        ),
        "{}",
        text(&output.stdout)
    );
}

#[test]
fn validate_and_compile_read_through_one_leading_byte_order_mark() {
    let source = std::fs::read(fixtures().join("protocol.yaml")).expect("read base fixture");
    let mut marked = "\u{feff}".as_bytes().to_vec();
    marked.extend_from_slice(&source);
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("byte-order-mark.yaml");
    std::fs::write(&path, marked).expect("write marked copy");

    let validated = Command::new(env!("CARGO_BIN_EXE_canon"))
        .arg("validate")
        .arg("--path")
        .arg(&path)
        .output()
        .expect("canon runs");
    assert_eq!(text(&validated.stderr), "");
    assert_eq!(
        text(&validated.stdout),
        "valid: protocol `investigation` revision 1\n"
    );
    assert_eq!(validated.status.code(), Some(0));

    let compiled = compile_in(&repository_root(), &path);
    assert_eq!(text(&compiled.stderr), "");
    assert_eq!(text(&compiled.stdout), BASE_IR);
    assert_eq!(compiled.status.code(), Some(0));
}

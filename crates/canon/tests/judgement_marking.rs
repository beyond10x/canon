//! story:judgement-marking against the library: an action's `author` and the artifacts it
//! `revises`, and who decides each evidence kind it may produce (`decided_by`), through
//! `canon validate`, `canon compile`, the `canon-ir/1` read-back of `canon evaluate --ir`, and
//! `canon diff`; and that `canon evaluate`, `canon check` and `canon generate` give the same bytes
//! with and without the three fields. The story adds no evaluation meaning, so it has no
//! conformance scenario.

use std::path::Path;

use b10x_canon::check::check;
use b10x_canon::diff::diff;
use b10x_canon::eval;
use b10x_canon::generate::generate;
use b10x_canon::ir::{self, Ir};
use b10x_canon::model;
use b10x_canon::validate::validate;

/// The fixture's path as a generated scenario names it.
const FIXTURE: &str = "fixtures/investigation/judgement-marking.yaml";

fn repository_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(repository_root().join(relative))
        .unwrap_or_else(|error| panic!("{relative}: {error}"))
}

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// The fixture with the three fields removed: every line that writes `author`, `revises` or
/// `decided_by`.
fn unmarked(source: &str) -> String {
    let kept: Vec<&str> = source
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            !(line.starts_with("author:")
                || line.starts_with("revises:")
                || line.starts_with("decided_by:"))
        })
        .collect();
    format!("{}\n", kept.join("\n"))
}

/// Every problem `canon validate` reports of `source`, as `code: message`.
fn problems(source: &str) -> Vec<String> {
    match validate(&model::parse(source).expect("parses")) {
        Ok(()) => Vec::new(),
        Err(problems) => problems
            .iter()
            .map(|problem| format!("{}: {problem}", problem.code()))
            .collect(),
    }
}

/// The decision `canon evaluate` prints for `ir` over one case with the spec at `r1` and one
/// record of every kind.
fn evaluated(ir: &Ir) -> String {
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: investigation.drafting\nartifacts: {spec: {revision: r1}}\n",
    )
    .expect("case");
    let evidence: Vec<model::EvidenceRecord> = [
        ("e1", "spec_source", None),
        ("e2", "spec_rationale", None),
        ("e3", "spec_review", Some("accept")),
        ("e4", "spec_test", Some("pass")),
    ]
    .into_iter()
    .map(|(id, kind, result)| {
        let mut text = format!(
            "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: spec\nsubject_revision: r1\n"
        );
        if let Some(result) = result {
            text.push_str(&format!("result: {result}\n"));
        }
        eval::read_evidence(&text).expect("record")
    })
    .collect();
    eval::render(&eval::evaluate(ir, &case, &evidence).expect("evaluates"))
}

/// What `canon generate` writes for `ir` compiled from `source`: each file's name and text.
fn generated(ir: &Ir, source: &str) -> Vec<(String, String)> {
    generate(ir, FIXTURE, source)
        .expect("generates")
        .into_iter()
        .map(|scenario| (scenario.file, scenario.text))
        .collect()
}

/// `canon compile fixtures/investigation/protocol.yaml` at `557e75b`, before the three fields
/// existed. The fixture has not changed since `2dc2d881`.
const INVESTIGATION_IR: &str = r#"{
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

/// The story's eight expectations (story:judgement-marking § Acceptance), in order.
#[test]
fn judgement_marking_compiles_and_means_nothing_yet() {
    let source = read(FIXTURE);

    // 1. A judgement action with `revises: [spec]` and `{evidence: spec_source, decided_by:
    //    program}` compiles: `"author": "judgement"`, `"revises": ["spec"]`, and no `decided_by`
    //    on that production.
    // 2. Its production without `decided_by` is `"decided_by": "judgement"`.
    let marked = compiled(&source);
    let text = marked.canonical_json();
    assert!(
        text.contains(
            "    \"draft_spec\": {\n      \"author\": \"judgement\",\n      \"description\": \"Write a new revision of the specification.\",\n      \"effect\": \"write\",\n      \"may_produce\": [\n        {\n          \"decided_by\": \"judgement\",\n          \"evidence\": \"spec_rationale\"\n        },\n        {\n          \"evidence\": \"spec_source\"\n        }\n      ],\n      \"precondition\": {\n        \"all\": []\n      },\n      \"requires\": [],\n      \"revises\": [\n        \"spec\"\n      ]\n    },\n"
        ),
        "{text}"
    );

    // 3. `author: program` written out, and `decided_by` equal to the action's author, give the
    //    bytes of no key.
    assert!(
        text.contains(
            "    \"test_spec\": {\n      \"description\": \"Run the tests against the specification.\",\n      \"effect\": null,\n      \"may_produce\": [\n        {\n          \"evidence\": \"spec_test\"\n        }\n      ],\n"
        ),
        "{text}"
    );
    let without_program = source.replace("    author: program\n", "").replace(
        "      - evidence: spec_test\n        decided_by: program\n",
        "      - evidence: spec_test\n",
    );
    assert_eq!(without_program.matches("decided_by").count(), 1);
    assert_ne!(without_program, source);
    assert_eq!(compiled(&without_program).canonical_json(), text);

    // 4. The IR of a protocol without the fields is the IR it was before they existed.
    assert_eq!(
        compiled(&read("fixtures/investigation/protocol.yaml")).canonical_json(),
        INVESTIGATION_IR
    );

    // 5. `revises` naming an undeclared artifact is `undeclared-artifact`, naming the action; an
    //    author that is neither program nor judgement is refused when the document is read.
    assert_eq!(
        problems(&source.replace("revises: [spec]", "revises: [missing]")),
        [
            "undeclared-artifact: action `draft_spec` references artifact `missing`, which is not declared"
        ]
    );
    let refused = model::parse(&source.replace("author: judgement", "author: llm"))
        .expect_err("`author: llm` is refused");
    assert!(
        refused
            .to_string()
            .contains("unknown variant `llm`, expected `program` or `judgement`"),
        "{refused}"
    );

    // 6. The compiled IR reads back to the same IR and the same bytes, as `canon evaluate --ir`
    //    reads it.
    let read_back = eval::read_ir(&text).expect("reads back");
    assert_eq!(read_back.canonical_json(), text);
    assert_eq!(read_back, marked);

    // 7. `canon evaluate`, `canon check` and `canon generate` give the same bytes with the three
    //    fields and with them removed.
    let plain_source = unmarked(&source);
    assert!(!plain_source.contains("author") && !plain_source.contains("revises"));
    assert!(!plain_source.contains("decided_by"));
    let plain = compiled(&plain_source);
    assert_ne!(plain.canonical_json(), text);
    assert_eq!(evaluated(&marked), evaluated(&plain));
    assert_eq!(
        check(&marked, None).expect("checks").to_string(),
        check(&plain, None).expect("checks").to_string()
    );
    assert_eq!(
        generated(&marked, &source),
        generated(&plain, &plain_source)
    );

    // 8. `canon diff` from the unmarked fixture to the marked one: `TIGHTENING` for `author`,
    //    `RELAXATION` for `revises`, each naming the action.
    assert_eq!(
        diff(&plain, &marked).expect("one protocol").render(),
        "diff: protocol `investigation.drafting` revision 1 to 1\n\
         TIGHTENING: action `draft_spec` author changed from program to judgement\n\
         TIGHTENING: action `draft_spec` now decides evidence `spec_rationale` by judgement\n\
         RELAXATION: action `draft_spec` now revises artifact `spec`\n\
         TIGHTENING: action `review_spec` author changed from program to judgement\n\
         TIGHTENING: action `review_spec` now decides evidence `spec_review` by judgement\n"
    );
}

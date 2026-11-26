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
  "access_class": "write",
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
  "guarded": false,
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

/// The reverse of expectation 8: every judgement mark removed relaxes and `revises` emptied
/// tightens; the fixture against itself with `author: program` removed is no change.
#[test]
fn removing_the_marks_relaxes_and_emptying_revises_tightens() {
    let source = read(FIXTURE);
    let marked = compiled(&source);
    let plain = compiled(&unmarked(&source));
    assert_eq!(
        diff(&marked, &plain).expect("one protocol").render(),
        "diff: protocol `investigation.drafting` revision 1 to 1\n\
         RELAXATION: action `draft_spec` author changed from judgement to program\n\
         RELAXATION: action `draft_spec` now decides evidence `spec_rationale` by program\n\
         TIGHTENING: action `draft_spec` no longer revises artifact `spec`\n\
         RELAXATION: action `review_spec` author changed from judgement to program\n\
         RELAXATION: action `review_spec` now decides evidence `spec_review` by program\n"
    );
    let program_unwritten = compiled(&source.replace("    author: program\n", ""));
    assert_eq!(
        diff(&marked, &program_unwritten)
            .expect("one protocol")
            .render(),
        "diff: protocol `investigation.drafting` revision 1 to 1\n\
         NO SEMANTIC CHANGE: no declaration differs\n"
    );
}

/// `revises` is checked in validation step 9, after the evidence kinds the action may produce and
/// before the subject of the case it opens, once for every listing, in the order listed.
#[test]
fn an_undeclared_revised_artifact_comes_after_the_kinds_and_before_the_opened_subject() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
                  actions:\n  act:\n    may_produce: [{evidence: missing_kind}]\n    \
                  revises: [z, a, y, z]\n    case_open: {protocol: q, subject: missing_subject}\n";
    assert_eq!(
        problems(source),
        [
            "undeclared-evidence-kind: action `act` references evidence kind `missing_kind`, which is not declared",
            "undeclared-artifact: action `act` references artifact `z`, which is not declared",
            "undeclared-artifact: action `act` references artifact `y`, which is not declared",
            "undeclared-artifact: action `act` references artifact `z`, which is not declared",
            "undeclared-artifact: action `act` references artifact `missing_subject`, which is not declared",
        ]
    );
}

/// A part `id` holding `body` after its header.
fn part(id: &str, body: &str) -> String {
    format!("format: protocol/1\nprotocol:\n  id: {id}\n  revision: 1\n{body}")
}

/// The composition of a root holding `root_body` that imports `a`, pinned to `a`'s digest (or
/// to a digest of zeros when `a` does not compile alone), compiled, or the problems as
/// `code: message` lines.
fn composed(a: &str, root_body: &str) -> Result<String, Vec<String>> {
    let a = model::parse(&part("a", a)).expect("a parses");
    let digest = ir::compile(&a)
        .map(|compiled| ir::digest(&compiled))
        .unwrap_or_else(|_| format!("sha256:{}", "0".repeat(64)));
    let root = model::parse(&format!(
        "{}imports:\n  a:\n    revision: 1\n    digest: \"{digest}\"\n{root_body}",
        part("root", "")
    ))
    .expect("root parses");
    ir::compose(&root, &[a])
        .and_then(|composed| ir::compile(&composed))
        .map(|compiled| compiled.canonical_json())
        .map_err(|problems| {
            problems
                .iter()
                .map(|problem| format!("{}: {problem}", problem.code()))
                .collect()
        })
}

/// story:protocol-imports: an imported action keeps its three fields through the merge, so the
/// composition is the IR of one document declaring it inline. A root action may revise an artifact
/// only an import declares, as it may open a case about one; an imported action that revises an
/// artifact only the root declares makes the import invalid alone, as a case it opens about one
/// does.
#[test]
fn an_imported_action_keeps_its_marks_and_revises_is_judged_like_other_artifact_references() {
    let marked = "artifacts: {spec: {}}\nevidence_kinds: {k: {}}\nactions:\n  draft:\n    \
                  author: judgement\n    revises: [spec]\n    \
                  may_produce: [{evidence: k, decided_by: program}]\n";
    let composition = composed(marked, "").expect("composes");
    assert_eq!(
        composition,
        compiled(&part("root", marked)).canonical_json()
    );
    assert!(
        composition.contains("\"author\": \"judgement\""),
        "{composition}"
    );
    assert!(composition.contains("\"revises\": [\n        \"spec\"\n      ]"));
    assert!(!composition.contains("decided_by"), "{composition}");

    let declares = "artifacts: {spec: {}}\n";
    assert!(composed(declares, "actions: {draft: {revises: [spec]}}\n").is_ok());
    assert!(
        composed(
            declares,
            "actions: {open: {case_open: {protocol: q, subject: spec}}}\n"
        )
        .is_ok()
    );

    let revising = composed(
        "actions: {draft: {revises: [spec]}}\n",
        "artifacts: {spec: {}}\n",
    )
    .expect_err("the import does not compile alone");
    let opening = composed(
        "actions: {draft: {case_open: {protocol: q, subject: spec}}}\n",
        "artifacts: {spec: {}}\n",
    )
    .expect_err("the import does not compile alone");
    assert_eq!(
        revising,
        [
            "invalid-import: imported protocol `a`: undeclared-artifact: action `draft` references artifact `spec`, which is not declared"
        ]
    );
    assert_eq!(opening, revising);
}

/// Two imports that declare one action differently in its marks conflict, as for any other
/// field; `author: program` written out and no `author` are one declaration.
#[test]
fn imports_that_differ_in_a_mark_conflict_and_program_written_out_does_not() {
    let two = |first: &str, second: &str| {
        let a = model::parse(&part("a", first)).expect("a parses");
        let b = model::parse(&part("b", second)).expect("b parses");
        let pin = |document: &model::Protocol| {
            ir::digest(&ir::compile(document).expect("a part compiles"))
        };
        let root = model::parse(&format!(
            "{}imports:\n  a:\n    revision: 1\n    digest: \"{}\"\n  b:\n    revision: 1\n    digest: \"{}\"\n",
            part("root", ""),
            pin(&a),
            pin(&b)
        ))
        .expect("root parses");
        ir::compose(&root, &[a, b]).map(|_| ()).map_err(|problems| {
            problems
                .iter()
                .map(|problem| problem.code())
                .collect::<Vec<_>>()
        })
    };
    let plain = "actions: {act: {}}\n";
    assert_eq!(two(plain, "actions: {act: {author: program}}\n"), Ok(()));
    for marked in [
        "actions: {act: {author: judgement}}\n",
        "artifacts: {s: {}}\nactions: {act: {revises: [s]}}\n",
    ] {
        let first = if marked.starts_with("artifacts") {
            "artifacts: {s: {}}\nactions: {act: {}}\n"
        } else {
            plain
        };
        assert_eq!(
            two(first, marked),
            Err(vec!["conflicting-imports"]),
            "{marked}"
        );
    }
}

//! Adversary cases, second pass, for story:protocol-source-model (wave 2026-10-04-w1).

use b10x_canon::model;
use b10x_canon::validate::{Problem, validate};

fn outcome(source: &str) -> Result<(), String> {
    let protocol = model::parse(source).map_err(|error| format!("parse: {error}"))?;
    validate(&protocol).map_err(|problems| format!("{problems:?}"))
}

/// `model/present.rs` refuses `key: ~` and `key: null` wherever a default would apply, so an author
/// who wrote a key and forgot its value is told so. A required identifier written the same way must
/// not instead become an identifier spelled `~` or `null`: `id:` is refused (empty identifier), and
/// `id: ~` is the same omission written another way.
#[test]
fn an_explicit_null_identifier_is_not_read_as_the_text_of_the_null() {
    let mut accepted = Vec::new();
    for null in ["~", "null"] {
        for (field, body) in [
            (
                "protocol id",
                format!("protocol: {{id: {null}, revision: 1}}\n"),
            ),
            (
                "action capability",
                format!(
                    "protocol: {{id: p, revision: 1}}\nactions: {{a: {{requires: [{{capability: {null}}}]}}}}\n"
                ),
            ),
        ] {
            let source = format!("format: protocol/1\n{body}");
            if outcome(&source).is_ok() {
                accepted.push(format!("{field} = `{null}`"));
            }
        }
    }
    assert_eq!(accepted, Vec::<String>::new());
}

/// `model/present.rs` opens with "a key written with no value is an error". A declaration written
/// with no value is such a key: `deploy:` under `actions:` must not silently become an action with
/// no precondition, no capability requirement and no effect.
#[test]
fn a_declaration_written_with_no_value_is_refused() {
    let mut accepted = Vec::new();
    for body in [
        "actions:\n  a:\n",
        "actions: {a: ~}\n",
        "artifacts: {x: null}\n",
        "evidence_kinds: {k: ~}\n",
        "obligations:\n  o:\n",
    ] {
        let source = format!("format: protocol/1\nprotocol: {{id: p, revision: 1}}\n{body}");
        if outcome(&source).is_ok() {
            accepted.push(body.trim_end().replace('\n', " / "));
        }
    }
    assert_eq!(accepted, Vec::<String>::new());
}

/// `validate/mod.rs` documents the order: unresolved references come section by section, claims
/// first, then actions, then outcomes, whatever order the sections are written in. Here the outcome
/// is written first, the action second and the claim last (coordinator decision, pass 2: keep the
/// order, document it, assert it).
#[test]
fn unresolved_references_come_in_the_order_their_declarations_are_written() {
    let source = r#"
format: protocol/1
protocol: {id: p, revision: 1}
outcomes:
  first: {requires: {claim: missing_one}}
actions:
  second: {precondition: {claim: missing_two}}
claims:
  third: {true_when: {claim: missing_three}}
"#;
    let protocol = model::parse(source).expect("parses");
    let problems = validate(&protocol).expect_err("three unresolved references");
    let referenced: Vec<String> = problems
        .iter()
        .map(|problem| match problem {
            Problem::UndeclaredClaim { claim, .. } => claim.to_string(),
            other => format!("unexpected {other:?}"),
        })
        .collect();
    assert_eq!(
        referenced,
        ["missing_three", "missing_two", "missing_one"],
        "story:protocol-source-model: unresolved references are reported claims, then actions, then outcomes, as validate/mod.rs documents"
    );
}

/// `model::one_line` promises rendered document text "cannot be mistaken for other text". An
/// invisible or reordering character passes the identifier grammar and is rendered raw, so the
/// message below reads "references evidence kind `note`, which is not declared" for a document
/// that declares `note`.
#[test]
fn rendered_identifiers_carry_no_invisible_or_reordering_characters() {
    const HIDDEN: &[char] = &[
        '\u{200b}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}',
        '\u{202e}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}', '\u{feff}',
    ];
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {note: {}}\nactions:\n  a: {may_produce: [{evidence: \"note\\u200B\"}]}\n  b: {may_produce: [{evidence: \"eton\\u202Ebackwards\"}]}\n";
    let protocol = model::parse(source).expect("parses");
    let problems = validate(&protocol).expect_err("two undeclared kinds");
    assert_eq!(problems.len(), 2, "{problems:?}");
    for problem in &problems {
        let line = problem.to_string();
        assert!(
            !line.chars().any(|c| HIDDEN.contains(&c)),
            "rendered raw: {line:?} displays as: {line}"
        );
    }
}

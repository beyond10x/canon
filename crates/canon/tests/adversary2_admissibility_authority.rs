//! Adversary cases for story:action-admissibility (wave 2026-10-04-w7, pass 2), against the
//! `canon-authority/1` reader through the evaluator library: document edge cases the unit's own
//! tests do not exercise — comments, CRLF line ends, the JSON spelling, an empty, comment-only or
//! BOM-only document, and a very large list. A leading byte-order mark is not read here: the CLI
//! strips it before the library sees the text (`canon-cli/src/main.rs` `read_text`).
//!
//! Driven from the `authority.rs` module docs: "A YAML (or JSON) list of decisions", "an empty
//! document is not a list", and the refusal order (malformed, then entry by entry invalid
//! identifier, then duplicate).

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Json};
use serde_json::json;

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {}\n";
const PROTOCOL: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    actions:\n\
    \x20\x20gated: {requires: [{capability: finding.publish}]}\n";

fn evaluated(authority: &str) -> Result<Json, eval::Refusal> {
    let ir = ir::compile(&model::parse(PROTOCOL).expect("parses")).expect("compiles");
    let case = eval::read_case(CASE).expect("case reads");
    eval::evaluate_with(
        &ir,
        &case,
        &[],
        Supplied {
            authority: Some(authority),
            ..Supplied::default()
        },
    )
    .map(|decision| decision.actions.expect("an actions section")["gated"].clone())
}

/// Each spelling of "grant `finding.publish`" makes the action admissible.
#[test]
fn every_spelling_of_one_grant_is_read() {
    for text in [
        "- capability: finding.publish\n  decision: granted\n",
        "# authority for run 7\n- capability: finding.publish # the only one\n  decision: granted\n",
        "- capability: finding.publish\r\n  decision: granted\r\n",
        "[{\"capability\": \"finding.publish\", \"decision\": \"granted\"}]",
        "---\n- {capability: finding.publish, decision: granted}\n...\n",
    ] {
        assert_eq!(
            evaluated(text),
            Ok(json!({"status": "admissible"})),
            "{text:?}"
        );
    }
}

/// An empty document, whitespace only, a comment only or a BOM only is not a list.
#[test]
fn a_document_with_no_list_is_refused_as_malformed() {
    for text in [
        "",
        "\n\n",
        "   ",
        "# nothing decided\n",
        "\u{feff}",
        "---\n",
    ] {
        let refused = evaluated(text).expect_err(text);
        assert_eq!(refused.code(), "malformed-input", "{text:?}: {refused}");
        assert!(
            refused
                .to_string()
                .starts_with("`--authority` is not a canon-authority/1 document: "),
            "{text:?}: {refused}"
        );
    }
}

/// A list of 50 000 decisions is read; one capability decided again as the last entry is refused
/// naming it, and an invalid identifier earlier in the list is refused first.
#[test]
fn a_very_large_list_is_read_and_checked_entry_by_entry() {
    let mut text = String::new();
    for index in 0..50_000 {
        text.push_str(&format!("- {{capability: c.{index}, decision: denied}}\n"));
    }
    text.push_str("- {capability: finding.publish, decision: granted}\n");
    let started = std::time::Instant::now();
    assert_eq!(evaluated(&text), Ok(json!({"status": "admissible"})));
    assert!(started.elapsed().as_secs() < 20, "{:?}", started.elapsed());

    let duplicated = format!("{text}- {{capability: c.49999, decision: granted}}\n");
    let refused = evaluated(&duplicated).expect_err("a duplicate");
    assert_eq!(refused.code(), "duplicate-identifier");
    assert_eq!(
        refused.to_string(),
        "`--authority` decides capability `c.49999` more than once"
    );

    let invalid = format!("- {{capability: 'a b', decision: granted}}\n{duplicated}");
    assert_eq!(
        evaluated(&invalid).expect_err("invalid").code(),
        "invalid-identifier"
    );
}

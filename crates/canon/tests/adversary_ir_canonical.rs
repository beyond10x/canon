//! Adversary cases for story:canon-ir (wave 2026-10-04-w2).
//!
//! Each case pins a `canon-ir/1` promise (design § 30) on a line the unit's own suite leaves
//! unobserved: a mutant on that line keeps the unit's suite green and turns one of these red.

use b10x_canon::{ir, model};

fn ir(source: &str) -> String {
    ir::compile(&model::parse(source).expect("parses"))
        .expect("compiles")
        .canonical_json()
}

const HEADER: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n";

/// The three truth values are three different tests. Every claim test in the unit's suite either
/// reads `is: true` or writes the same non-true value on both sides of an equality, so rendering
/// `false` as `unknown` (or both as `true`) keeps it green.
#[test]
fn each_truth_value_compiles_to_its_own_name() {
    let source = format!(
        "{HEADER}claims:\n  c: {{true_when: {{all: []}}}}\noutcomes:\n  f: {{requires: {{claim: c, is: false}}}}\n  t: {{requires: {{claim: c, is: true}}}}\n  u: {{requires: {{claim: c, is: unknown}}}}\n"
    );
    let compiled = ir(&source);
    for (outcome, is) in [("f", "false"), ("t", "true"), ("u", "unknown")] {
        let expected = format!(
            "    \"{outcome}\": {{\n      \"description\": null,\n      \"requires\": {{\n        \"claim\": {{\n          \"id\": \"c\",\n          \"is\": \"{is}\"\n        }}\n      }}\n    }}"
        );
        assert!(
            compiled.contains(&expected),
            "{expected}\nnot in\n{compiled}"
        );
    }
}

/// Obligations are a declaration section of the IR. Every fixture the unit compiles has none, so
/// dropping the section's entries, or their descriptions, keeps its suite green.
#[test]
fn a_declared_obligation_compiles_with_its_description() {
    let source = format!(
        "{HEADER}obligations:\n  sign_off: {{description: Someone signs off.}}\n  archive: {{}}\n"
    );
    let compiled = ir(&source);
    assert!(
        compiled.contains(
            "  \"obligations\": {\n    \"archive\": {\n      \"description\": null\n    },\n    \"sign_off\": {\n      \"description\": \"Someone signs off.\"\n    }\n  },\n"
        ),
        "{compiled}"
    );
}

/// An action precondition is a predicate like any other, so its `all`/`any` members are in
/// canonical order. The unit's suite never compiles a precondition other than `{all: []}`, so
/// compiling the precondition without normalizing it keeps the suite green.
#[test]
fn action_precondition_members_compile_in_one_order() {
    let declarations = "evidence_kinds: {a: {}, b: {}}\nclaims:\n  c: {true_when: {all: []}}\n";
    let written = format!(
        "{HEADER}{declarations}actions:\n  act:\n    precondition:\n      all: [{{evidence: {{kind: b}}}}, {{not: {{any: [{{claim: c}}, {{evidence: {{kind: a}}}}]}}}}]\n"
    );
    let reordered = format!(
        "{HEADER}{declarations}actions:\n  act:\n    precondition:\n      all: [{{not: {{any: [{{evidence: {{kind: a}}}}, {{claim: c}}]}}}}, {{evidence: {{kind: b}}}}]\n"
    );
    assert_eq!(ir(&written), ir(&reordered));
}

/// The canonical order of predicate members is part of the `canon-ir/1` bytes, and those bytes are
/// what gets hashed (§ 30). The order rests on the derived `Ord` of a private JSON value type
/// (`ir/json.rs:12-20`): a missing evidence result (`null`) sorts before any result, and claim
/// tests sort by the names `false` < `true` < `unknown`. Nothing in the unit's suite compares two
/// members that differ only there, so reordering the value type's variants, or the truth names,
/// silently changes the hash of an unchanged protocol.
#[test]
fn predicate_member_order_is_pinned_where_members_differ_only_in_result_or_truth() {
    let source = format!(
        "{HEADER}evidence_kinds: {{a: {{}}}}\nclaims:\n  c: {{true_when: {{all: []}}}}\noutcomes:\n  o:\n    requires:\n      any: [{{claim: c, is: unknown}}, {{evidence: {{kind: a, result: x}}}}, {{claim: c}}, {{evidence: {{kind: a}}}}, {{claim: c, is: false}}]\n"
    );
    let compiled = ir(&source);
    let expected = "      \"requires\": {\n        \"any\": [\n          {\n            \"claim\": {\n              \"id\": \"c\",\n              \"is\": \"false\"\n            }\n          },\n          {\n            \"claim\": {\n              \"id\": \"c\",\n              \"is\": \"true\"\n            }\n          },\n          {\n            \"claim\": {\n              \"id\": \"c\",\n              \"is\": \"unknown\"\n            }\n          },\n          {\n            \"evidence\": {\n              \"kind\": \"a\",\n              \"result\": null\n            }\n          },\n          {\n            \"evidence\": {\n              \"kind\": \"a\",\n              \"result\": \"x\"\n            }\n          }\n        ]\n      }\n";
    assert!(compiled.contains(expected), "{compiled}");
}

/// Declarations are keyed by identifier in code-point order, whatever order they are written in,
/// including identifiers outside the Basic Multilingual Plane (where code-point and UTF-16 order
/// disagree: U+FF61 sorts before U+1F600 by code point, after it by UTF-16 code unit).
#[test]
fn declaration_keys_sort_by_code_point_including_astral_identifiers() {
    let ids = ["\u{1F600}", "\u{FF61}", "b", "B", "\u{e9}"];
    let section = |order: &[&str]| {
        let mut text = String::from("artifacts:\n");
        for id in order {
            text.push_str(&format!("  \"{id}\": {{}}\n"));
        }
        text
    };
    let mut reversed = ids;
    reversed.reverse();
    let forward = ir(&format!("{HEADER}{}", section(&ids)));
    let backward = ir(&format!("{HEADER}{}", section(&reversed)));
    assert_eq!(forward, backward);
    let positions: Vec<usize> = ["B", "b", "\u{e9}", "\u{FF61}", "\u{1F600}"]
        .iter()
        .map(|id| {
            forward
                .find(&format!("\"{id}\": {{"))
                .unwrap_or_else(|| panic!("{id} missing from {forward}"))
        })
        .collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "{positions:?}\n{forward}"
    );
}

/// The same document saved with CRLF line endings, or in flow style with an alias, is the same
/// protocol, and compiles to the same bytes as the base fixture.
#[test]
fn line_endings_and_yaml_style_do_not_change_the_ir() {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset: run this test through cargo");
    let fixture =
        std::path::Path::new(&manifest).join("../../fixtures/investigation/protocol.yaml");
    let base = std::fs::read_to_string(&fixture)
        .unwrap_or_else(|error| panic!("reading {}: {error}", fixture.display()));
    let base = base.as_str();
    let expected = ir(base);
    let crlf = base.replace('\n', "\r\n");
    let flow = "{format: protocol/1, protocol: {id: investigation, revision: 1, description: 'Decide whether an explanation is supported by observation and survives an attempt to falsify it.'}, artifacts: {explanation: {description: The explanation under investigation.}}, evidence_kinds: {supporting_observation: {description: An observation consistent with the explanation.}, falsification_attempt: {description: 'An attempt to refute the explanation; its result is survived or refuted.'}}, claims: {explanation.supported: {description: The explanation is supported by observation and has survived falsification., true_when: {all: [{evidence: {kind: falsification_attempt, result: survived}}, {evidence: {kind: &so supporting_observation}}]}}}, actions: {attempt_falsification: {description: Try to refute the explanation., may_produce: [{evidence: falsification_attempt}]}, inspect: {description: Look for observations bearing on the explanation., may_produce: [{evidence: *so}]}}, outcomes: {supported: {description: The explanation is supported., requires: {claim: explanation.supported, is: 'true'}}}}\n";
    for (name, variant) in [("crlf", crlf.as_str()), ("flow", flow)] {
        let parsed = model::parse(variant).unwrap_or_else(|error| panic!("{name}: {error}"));
        let compiled = ir::compile(&parsed)
            .unwrap_or_else(|problems| panic!("{name}: {problems:?}"))
            .canonical_json();
        assert_eq!(compiled, expected, "{name}");
    }
}

/// A revision is an unsigned 64-bit integer and is written exactly, at the top of its range too.
#[test]
fn the_largest_revision_is_written_exactly() {
    let compiled = ir("format: protocol/1\nprotocol: {id: p, revision: 18446744073709551615}\n");
    assert!(
        compiled.contains("\"revision\": 18446744073709551615\n"),
        "{compiled}"
    );
}

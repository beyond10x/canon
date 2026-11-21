//! story:protocol-writer: `b10x_canon::model::write` writes a `protocol/1` model as canonical JSON
//! text, and `model::parse` reads that text back to an equal model.
//!
//! The corpus is every file under `fixtures/` that starts with `format: protocol/1` and that
//! `parse` accepts, walked as `missing_keys.rs` walks it. Some fixtures are refused by `parse` on
//! purpose and are not part of it.
//!
//! The first group of tests is the story's seven expectations, in order. The second group holds the
//! four points the story did not verify, so the writer does not rely on a guess about
//! `serde_yaml_ng`; the third holds the spelling rules and the limits of what `parse` can read.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use b10x_canon::model::{
    self, Action, CapabilityId, CapabilityRequirement, CaseSection, Claim, ClaimId, Declarations,
    EvidenceKindId, EvidenceMatch, Predicate, Protocol, write,
};
use b10x_canon::validate;
use serde_json::Value as Json;
use serde_yaml_ng::Value as Yaml;

/// The repository root, read at run time: a test binary reused from a shared build directory
/// would otherwise read the tree it was built from.
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"))
        .join("../..")
}

const MINIMAL: &str = "format: protocol/1\nprotocol:\n  id: p\n  revision: 1\n";

fn minimal() -> Protocol {
    model::parse(MINIMAL).expect("the minimal protocol parses")
}

/// Every protocol fixture `parse` accepts, with its path below the repository root, in path order.
fn corpus() -> Vec<(String, Protocol)> {
    let mut found = Vec::new();
    let mut pending = vec![root().join("fixtures")];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("fixture directory reads") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "yaml") {
                let text = fs::read_to_string(&path).expect("fixture reads");
                if text.starts_with("format: protocol/1")
                    && let Ok(protocol) = model::parse(&text)
                {
                    let name = path
                        .strip_prefix(root().join("fixtures"))
                        .expect("below fixtures")
                        .display()
                        .to_string();
                    found.push((name, protocol));
                }
            }
        }
    }
    found.sort_by(|left, right| left.0.cmp(&right.0));
    assert!(
        found.len() >= 100,
        "{} protocol fixtures parse; the corpus is every one `parse` accepts",
        found.len()
    );
    found
}

/// `write` then `parse`, naming the text when `parse` refuses it.
fn round_trip(protocol: &Protocol) -> Protocol {
    let text = write(protocol);
    match model::parse(&text) {
        Ok(read) => read,
        Err(error) => {
            let shown: String = text.chars().take(4000).collect();
            panic!("parse refuses the written text: {error}\n{shown}")
        }
    }
}

/// Whether the build holds number text (`serde_json` `arbitrary_precision`, turned on by
/// `b10x-canon-expr` in a workspace build) and not doubles and 64-bit integers.
fn holds_number_text() -> bool {
    serde_json::from_str::<serde_json::Number>("1.50").is_ok_and(|n| n.to_string() == "1.50")
}

fn with_arguments(arguments: Json) -> Protocol {
    let mut protocol = minimal();
    protocol.actions = Declarations::new(vec![(
        "act".into(),
        Action {
            arguments: Some(arguments),
            ..Action::default()
        },
    )]);
    protocol
}

fn claims(ids: &[&str]) -> Declarations<ClaimId, Claim> {
    Declarations::new(
        ids.iter()
            .map(|id| {
                (
                    ClaimId::new(*id),
                    Claim {
                        description: None,
                        true_when: Predicate::All(Vec::new()),
                    },
                )
            })
            .collect(),
    )
}

/// A character `text` may not hold unescaped: one YAML cannot hold as written, or a line break
/// other than the layout's own.
fn assert_no_raw_unsafe_character(text: &str) {
    for c in text.chars() {
        let unsafe_raw = (c < ' ' && c != '\n')
            || ('\u{7f}'..='\u{9f}').contains(&c)
            || matches!(
                c,
                '\u{2028}' | '\u{2029}' | '\u{feff}' | '\u{fffe}' | '\u{ffff}'
            );
        assert!(!unsafe_raw, "U+{:04X} is written raw", u32::from(c));
    }
}

// ---------------------------------------------------------------------------------------------
// Expectation 1: the round trip over the corpus.

#[test]
fn every_corpus_protocol_reads_back_equal() {
    for (name, protocol) in corpus() {
        let text = write(&protocol);
        let read = model::parse(&text)
            .unwrap_or_else(|error| panic!("{name}: parse refuses the written text: {error}"));
        assert!(
            read == protocol,
            "{name}: the written text reads back unequal"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Expectation 2: the second write is a fixpoint, and the same model gives the same bytes.

#[test]
fn the_second_write_is_a_fixpoint_and_the_same_model_gives_the_same_bytes() {
    for (name, protocol) in corpus() {
        let first = write(&protocol);
        let read = model::parse(&first).unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(write(&read), first, "{name}: the second write differs");
        assert_eq!(write(&protocol), first, "{name}: two writes differ");
        assert_eq!(
            write(&protocol.clone()),
            first,
            "{name}: a clone writes differently"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Expectation 3: JSON that ends with one newline, declarations in model order.

#[test]
fn the_output_is_json_that_ends_with_one_newline() {
    for (name, protocol) in corpus() {
        let text = write(&protocol);
        assert!(
            text.ends_with("}\n"),
            "{name}: does not end with `}}` and a newline"
        );
        assert!(!text.ends_with("\n\n"), "{name}: ends with two newlines");
        assert!(!text.contains('\t'), "{name}: holds a tab");
        assert!(
            text.lines()
                .all(|line| (line.len() - line.trim_start_matches(' ').len()) % 2 == 0),
            "{name}: an indent is not a multiple of two spaces"
        );
        let value: Json = serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("{name}: serde_json refuses the text: {error}"));
        assert!(value.is_object(), "{name}: the document is not an object");
        assert_eq!(
            value["format"], "protocol/1",
            "{name}: format is not first-class"
        );
    }
}

#[test]
fn claims_declared_b_then_a_are_written_b_first() {
    let mut protocol = minimal();
    protocol.claims = claims(&["b", "a"]);
    let text = write(&protocol);
    let b = text.find("\"b\": ").expect("b is written");
    let a = text.find("\"a\": ").expect("a is written");
    assert!(b < a, "{text}");
}

#[test]
fn the_minimal_protocol_is_written_exactly() {
    assert_eq!(
        write(&minimal()),
        "{\n  \"format\": \"protocol/1\",\n  \"protocol\": {\n    \"id\": \"p\",\n    \"revision\": 1\n  }\n}\n"
    );
}

#[test]
fn keys_follow_the_order_of_the_struct_that_declares_them() {
    // The source writes every key in the reverse of the struct's order.
    let source = "format: protocol/1\n\
        protocol: {revision: 3, description: d, id: p}\n\
        imports: {q: {refines: [x], digest: 'sha256:00', revision: 2}}\n\
        independence_dimensions: {dim: {description: d}}\n\
        case: {inputs: {i: {values: [a], description: d}}}\n\
        invalidation: {r: {invalidates: [c], upstream: art, description: d}}\n\
        outcomes: {o: {requires: {claim: c}, description: d}}\n\
        actions: {a: {revises: [art], author: judgement, case_open: {subject: art, protocol: q}, arguments: {type: object}, may_produce: [{decided_by: program, evidence: ev}], effect: e, requires: [{scope: {k: v}, capability: cap}], precondition: {claim: c}, description: d}}\n\
        obligations: {ob: {applicable_when: {claim: c}, discharged_when: {claim: c}, description: d}}\n\
        claims:\n\
        \x20 c: {true_when: {evidence: {independent_on: [dim], subject: art, result: r, kind: ev}}, description: d}\n\
        \x20 f: {true_when: {first: {subject: art, result: r, kind: ev}}}\n\
        \x20 b: {true_when: {before: {subject: art, then: ev, first: ev}}}\n\
        \x20 af: {true_when: {after: {subject: art, then: {result: r, kind: ev}, first: ev}}}\n\
        \x20 s: {true_when: {since: {subject: art, then: {result: r, kind: ev}, last: ev}}}\n\
        \x20 ch: {true_when: {children: {subject: art, revision: current, in: [x]}}}\n\
        \x20 i: {true_when: {input: i, in: [a]}}\n\
        evidence_kinds: {ev: {supplied_by: other, max_age: 5m, description: d}}\n\
        artifacts: {art: {description: d}}\n";
    let protocol = model::parse(source).expect("parses");
    let text = write(&protocol);
    // A mapping keeps the order it is read in, which `serde_json::Value` does not.
    let value: Yaml = serde_yaml_ng::from_str(&text).expect("the text is YAML");
    let keys = |at: &Yaml| -> Vec<String> {
        at.as_mapping()
            .unwrap_or_else(|| panic!("not a mapping: {at:?}"))
            .keys()
            .map(|key| key.as_str().expect("a string key").to_owned())
            .collect()
    };
    assert_eq!(
        keys(&value),
        [
            "format",
            "protocol",
            "artifacts",
            "evidence_kinds",
            "claims",
            "obligations",
            "actions",
            "outcomes",
            "invalidation",
            "case",
            "independence_dimensions",
            "imports"
        ]
    );
    assert_eq!(keys(&value["protocol"]), ["id", "revision", "description"]);
    assert_eq!(keys(&value["artifacts"]["art"]), ["description"]);
    assert_eq!(
        keys(&value["evidence_kinds"]["ev"]),
        ["description", "max_age", "supplied_by"]
    );
    assert_eq!(keys(&value["claims"]["c"]), ["description", "true_when"]);
    let when = |claim: &str| value["claims"][claim]["true_when"].clone();
    assert_eq!(keys(&when("c")), ["evidence"]);
    assert_eq!(
        keys(&when("c")["evidence"]),
        ["kind", "result", "subject", "independent_on"]
    );
    assert_eq!(keys(&when("f")["first"]), ["kind", "result", "subject"]);
    assert_eq!(keys(&when("b")["before"]), ["first", "then", "subject"]);
    assert_eq!(keys(&when("af")["after"]), ["first", "then", "subject"]);
    assert_eq!(keys(&when("af")["after"]["then"]), ["kind", "result"]);
    assert_eq!(keys(&when("s")["since"]), ["last", "then", "subject"]);
    assert_eq!(keys(&when("ch")["children"]), ["in", "revision", "subject"]);
    assert_eq!(keys(&when("i")), ["input", "in"]);
    assert_eq!(
        keys(&value["obligations"]["ob"]),
        ["description", "discharged_when", "applicable_when"]
    );
    assert_eq!(
        keys(&value["actions"]["a"]),
        [
            "description",
            "precondition",
            "requires",
            "effect",
            "may_produce",
            "arguments",
            "case_open",
            "author",
            "revises"
        ]
    );
    assert_eq!(
        keys(&value["actions"]["a"]["requires"][0]),
        ["capability", "scope"]
    );
    assert_eq!(
        keys(&value["actions"]["a"]["may_produce"][0]),
        ["evidence", "decided_by"]
    );
    assert_eq!(
        keys(&value["actions"]["a"]["case_open"]),
        ["protocol", "subject"]
    );
    assert_eq!(keys(&value["outcomes"]["o"]), ["description", "requires"]);
    assert_eq!(
        keys(&value["invalidation"]["r"]),
        ["description", "upstream", "invalidates"]
    );
    assert_eq!(
        keys(&value["case"]["inputs"]["i"]),
        ["description", "values"]
    );
    assert_eq!(
        keys(&value["independence_dimensions"]["dim"]),
        ["description"]
    );
    assert_eq!(
        keys(&value["imports"]["q"]),
        ["revision", "digest", "refines"]
    );
    assert_eq!(model::parse(&text).expect("reads"), protocol);
}

// ---------------------------------------------------------------------------------------------
// Expectation 4: a repeated identifier is written as often as it is declared.

#[test]
fn claims_holding_a_twice_are_written_twice_and_read_back_twice() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        evidence_kinds: {e: {}}\n\
        claims:\n  a: {true_when: {evidence: {kind: e}}}\n  a: {true_when: {evidence: {kind: e, result: x}}}\n";
    let protocol = model::parse(source).expect("a repeated key parses");
    assert_eq!(protocol.claims.len(), 2);
    let text = write(&protocol);
    assert_eq!(text.matches("\"a\": {").count(), 2, "{text}");
    let read = round_trip(&protocol);
    assert_eq!(read.claims.len(), 2);
    assert!(read == protocol);

    let codes = |protocol: &Protocol| -> Vec<String> {
        validate::validate(protocol)
            .expect_err("the repeated identifier is refused")
            .iter()
            .map(|problem| format!("{}: {problem}", problem.code()))
            .collect()
    };
    let from_source = codes(&protocol);
    assert!(
        from_source
            .iter()
            .any(|line| line.starts_with("duplicate-identifier")),
        "{from_source:?}"
    );
    assert_eq!(codes(&read), from_source);
}

#[test]
fn a_protocol_the_validator_refuses_is_written_and_read_back() {
    let source = "format: protocol/2\n\
        protocol: {id: '', revision: 0}\n\
        claims: {c: {true_when: {all: [{claim: missing}, {evidence: {kind: nowhere, subject: nothing}}]}}}\n\
        outcomes: {o: {requires: {decision: x}}}\n\
        actions: {a: {precondition: {claim: dangling}, revises: [ghost]}}\n";
    let protocol = model::parse(source).expect("parses");
    assert!(validate::validate(&protocol).is_err());
    assert!(round_trip(&protocol) == protocol);
}

// ---------------------------------------------------------------------------------------------
// Expectation 5: the characters YAML cannot hold as written.

#[test]
fn a_description_holding_characters_yaml_cannot_hold_reads_back_equal() {
    let description = "a\u{0}b\tc\u{7f}d\u{85}e\u{2028}f\u{2029}g\u{feff}h \"q\" \\ end";
    let mut protocol = minimal();
    protocol.protocol.description = Some(description.to_owned());
    let text = write(&protocol);
    for raw in [
        '\u{0}', '\t', '\u{7f}', '\u{85}', '\u{2028}', '\u{2029}', '\u{feff}',
    ] {
        assert!(
            !text.contains(raw),
            "U+{:04X} is written raw",
            u32::from(raw)
        );
    }
    assert_eq!(
        round_trip(&protocol).protocol.description.as_deref(),
        Some(description)
    );
}

#[test]
fn every_unicode_scalar_value_reads_back_in_a_value_a_key_and_arguments() {
    let all: String = (0..=0x10FFFF).filter_map(char::from_u32).collect();
    let scalars: Vec<char> = all.chars().collect();
    let mut protocol = with_arguments(serde_json::json!({
        "type": "object",
        "description": all,
    }));
    protocol.protocol.description = Some(all.clone());
    // A key is read back whole only up to the simple key limit of YAML, so each key is a chunk.
    let keys: Vec<String> = scalars
        .chunks(64)
        .map(|chunk| format!("k{}", chunk.iter().collect::<String>()))
        .collect();
    protocol.claims = Declarations::new(
        keys.iter()
            .map(|key| {
                (
                    ClaimId::new(key.as_str()),
                    Claim {
                        description: Some(key.clone()),
                        true_when: Predicate::All(Vec::new()),
                    },
                )
            })
            .collect(),
    );
    let text = write(&protocol);
    assert_no_raw_unsafe_character(&text);
    let read = model::parse(&text).expect("every scalar value is written so parse reads it");
    assert!(read == protocol, "a scalar value does not read back");
}

// ---------------------------------------------------------------------------------------------
// Expectation 6: `Some` of an empty value is not `None`.

#[test]
fn an_empty_case_section_and_no_case_section_read_back_distinct() {
    let mut empty = minimal();
    empty.case = Some(CaseSection::default());
    let none = minimal();
    assert!(empty != none);
    assert!(write(&empty) != write(&none));
    assert!(write(&empty).contains("\"case\": {}"));
    assert!(round_trip(&empty) == empty);
    assert!(round_trip(&none) == none);
    assert_eq!(round_trip(&empty).case, Some(CaseSection::default()));
    assert_eq!(round_trip(&none).case, None);
}

#[test]
fn an_empty_independent_on_and_none_read_back_distinct() {
    let with = |independent_on: Option<Vec<model::IndependenceDimensionId>>| {
        let mut protocol = minimal();
        protocol.claims = Declarations::new(vec![(
            ClaimId::new("c"),
            Claim {
                description: None,
                true_when: Predicate::Evidence(EvidenceMatch {
                    kind: EvidenceKindId::new("e"),
                    result: None,
                    subject: None,
                    independent_on,
                }),
            },
        )]);
        protocol
    };
    let empty = with(Some(Vec::new()));
    let none = with(None);
    assert!(empty != none);
    assert!(write(&empty).contains("\"independent_on\": []"));
    assert!(!write(&none).contains("independent_on"));
    assert!(round_trip(&empty) == empty);
    assert!(round_trip(&none) == none);
}

// ---------------------------------------------------------------------------------------------
// Expectation 7: every field the specification declares is written by some corpus protocol.

#[derive(Debug, Clone)]
enum Ty {
    Name(String),
    Optional(Box<Ty>),
    List(Box<Ty>),
    Map(Box<Ty>),
}

enum Shape {
    Struct(Vec<(String, Ty)>),
    Union(Vec<(String, Ty)>),
}

fn local(name: &str) -> String {
    name.rsplit('.').next().unwrap_or(name).to_owned()
}

fn ty(text: &str) -> Ty {
    let text = text.trim();
    let inner = |prefix: &str| text.strip_prefix(prefix)?.strip_suffix('>');
    if let Some(of) = inner("Optional<") {
        Ty::Optional(Box::new(ty(of)))
    } else if let Some(of) = inner("List<") {
        Ty::List(Box::new(ty(of)))
    } else if let Some(args) = inner("Map<") {
        let mut depth = 0;
        let comma = args
            .char_indices()
            .find(|(_, c)| {
                match c {
                    '<' => depth += 1,
                    '>' => depth -= 1,
                    _ => {}
                }
                *c == ',' && depth == 0
            })
            .expect("a map has a key and a value")
            .0;
        Ty::Map(Box::new(ty(&args[comma + 1..])))
    } else {
        Ty::Name(local(text))
    }
}

/// The types of `canon.protocol` as `ess/domains/protocol.yaml` authors them, and the fields of
/// the entity `Protocol`. The entity is the Rust struct `Protocol` with its header written in
/// place, as `ess_model_matches.rs` reads it: the header's `id` is the identity `protocol_id`, and
/// `revision` and `description` are entity fields written under `protocol`.
fn specification() -> (BTreeMap<String, Shape>, Vec<(String, Ty)>) {
    let text = fs::read_to_string(root().join("ess/domains/protocol.yaml")).expect("spec reads");
    let doc: Yaml = serde_yaml_ng::from_str(&text).expect("spec is YAML");
    let fields = |declaration: &Yaml| -> Vec<(String, Ty)> {
        declaration["fields"]
            .as_sequence()
            .into_iter()
            .flatten()
            .map(|field| {
                (
                    field["name"].as_str().expect("field name").to_owned(),
                    ty(field["type"].as_str().expect("field type")),
                )
            })
            .collect()
    };
    let mut shapes = BTreeMap::new();
    for declaration in doc["types"].as_sequence().expect("types") {
        let name = local(declaration["name"].as_str().expect("type name"));
        match declaration["kind"].as_str().expect("type kind") {
            "struct" => {
                shapes.insert(name, Shape::Struct(fields(declaration)));
            }
            "union" => {
                let variants = declaration["variants"]
                    .as_mapping()
                    .expect("variants")
                    .iter()
                    .map(|(variant, of)| {
                        (
                            variant.as_str().expect("variant").to_owned(),
                            ty(of.as_str().expect("variant type")),
                        )
                    })
                    .collect();
                shapes.insert(name, Shape::Union(variants));
            }
            _ => {}
        }
    }
    let entity = doc["entities"]
        .as_sequence()
        .expect("entities")
        .iter()
        .find(|entity| entity["name"].as_str() == Some("canon.protocol.Protocol"))
        .expect("the Protocol entity is declared");
    (shapes, fields(entity))
}

/// The forms of a predicate written as the whole object beside `is` or `in` rather than under
/// their own key.
const INLINE: &[&str] = &["claim", "input"];

/// Records `Type.field` for every key written below `value`, read as `ty`, and `Type.variant`
/// for every form of a union.
fn walk(value: &Json, of: &Ty, shapes: &BTreeMap<String, Shape>, written: &mut BTreeSet<String>) {
    match of {
        Ty::Optional(inner) => walk(value, inner, shapes, written),
        Ty::List(inner) => {
            for item in value.as_array().into_iter().flatten() {
                walk(item, inner, shapes, written);
            }
        }
        Ty::Map(inner) => {
            for item in value
                .as_object()
                .into_iter()
                .flat_map(|object| object.values())
            {
                walk(item, inner, shapes, written);
            }
        }
        Ty::Name(name) => {
            let Some(object) = value.as_object() else {
                return;
            };
            match shapes.get(name) {
                Some(Shape::Struct(fields)) => {
                    for (key, inner) in object {
                        written.insert(format!("{name}.{key}"));
                        if let Some((_, field)) = fields.iter().find(|(field, _)| field == key) {
                            walk(inner, field, shapes, written);
                        }
                    }
                }
                Some(Shape::Union(variants)) if name == "OutcomeRequirement" => {
                    if object.contains_key("decision") {
                        written.insert("OutcomeRequirement.decision".to_owned());
                    } else {
                        written.insert("OutcomeRequirement.predicate".to_owned());
                        let predicate = variants
                            .iter()
                            .find(|(variant, _)| variant == "predicate")
                            .expect("a predicate variant");
                        walk(value, &predicate.1, shapes, written);
                    }
                }
                Some(Shape::Union(variants)) => {
                    for (variant, of) in variants {
                        if let Some(inner) = object.get(variant) {
                            written.insert(format!("{name}.{variant}"));
                            if INLINE.contains(&variant.as_str()) {
                                walk(value, of, shapes, written);
                            } else {
                                walk(inner, of, shapes, written);
                            }
                        }
                    }
                }
                None => {}
            }
        }
    }
}

/// Every `Type.field` and `Type.variant` of the types the entity `Protocol` reaches.
fn specified(shapes: &BTreeMap<String, Shape>, entity: &[(String, Ty)]) -> BTreeSet<String> {
    let mut expected: BTreeSet<String> = entity
        .iter()
        .map(|(field, _)| format!("Protocol.{field}"))
        .collect();
    expected.insert("Protocol.protocol_id".to_owned());
    let mut pending: Vec<String> = Vec::new();
    let mut seen = BTreeSet::new();
    fn names(of: &Ty, into: &mut Vec<String>) {
        match of {
            Ty::Name(name) => into.push(name.clone()),
            Ty::Optional(inner) | Ty::List(inner) | Ty::Map(inner) => names(inner, into),
        }
    }
    for (_, of) in entity {
        names(of, &mut pending);
    }
    while let Some(name) = pending.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        match shapes.get(&name) {
            Some(Shape::Struct(fields)) => {
                for (field, of) in fields {
                    expected.insert(format!("{name}.{field}"));
                    names(of, &mut pending);
                }
            }
            Some(Shape::Union(variants)) => {
                for (variant, of) in variants {
                    expected.insert(format!("{name}.{variant}"));
                    names(of, &mut pending);
                }
            }
            None => {}
        }
    }
    expected
}

/// What `write` writes for the protocols, as `Type.field` and `Type.variant`.
fn written_by<'a>(
    protocols: impl IntoIterator<Item = &'a Protocol>,
    shapes: &BTreeMap<String, Shape>,
    entity: &[(String, Ty)],
) -> BTreeSet<String> {
    let mut written = BTreeSet::new();
    for protocol in protocols {
        let value: Json = serde_json::from_str(&write(protocol)).expect("the text is JSON");
        for (key, inner) in value.as_object().expect("an object") {
            if key == "protocol" {
                for header in inner.as_object().expect("the header is an object").keys() {
                    let field = if header == "id" {
                        "protocol_id"
                    } else {
                        header
                    };
                    written.insert(format!("Protocol.{field}"));
                }
            } else {
                written.insert(format!("Protocol.{key}"));
                if let Some((_, of)) = entity.iter().find(|(field, _)| field == key) {
                    walk(inner, of, shapes, &mut written);
                }
            }
        }
    }
    written
}

#[test]
fn writer_covers_every_specified_field() {
    let (shapes, entity) = specification();
    let expected = specified(&shapes, &entity);
    assert!(expected.len() > 60, "{} specified fields", expected.len());
    let corpus = corpus();
    let written = written_by(
        corpus.iter().map(|(_, protocol)| protocol),
        &shapes,
        &entity,
    );
    let unwritten: Vec<&String> = expected.difference(&written).collect();
    assert!(
        unwritten.is_empty(),
        "no corpus protocol has `write` write: {unwritten:?}"
    );
}

#[test]
fn a_specified_field_no_protocol_writes_is_named() {
    let (shapes, entity) = specification();
    let expected = specified(&shapes, &entity);
    let minimal = minimal();
    let written = written_by([&minimal], &shapes, &entity);
    let unwritten: Vec<&String> = expected.difference(&written).collect();
    for named in [
        "CapabilityRequirement.scope",
        "Import.refines",
        "Action.author",
    ] {
        assert!(
            unwritten.iter().any(|field| field.as_str() == named),
            "{named} is not named in {unwritten:?}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The four points the story did not verify.

/// Point 1: `parse` accepts JSON text.
#[test]
fn parse_accepts_json_text() {
    let text = "{\n  \"format\": \"protocol/1\",\n  \"protocol\": {\n    \"id\": \"p\",\n    \"revision\": 1,\n    \"description\": \"a\\u0000\\t\\\"b\\\"\\\\\\u2028\"\n  },\n  \"claims\": {\n    \"a\": {\n      \"true_when\": {\n        \"claim\": \"a\",\n        \"is\": \"true\"\n      }\n    }\n  }\n}\n";
    let protocol = model::parse(text).expect("parse accepts JSON text");
    assert_eq!(
        protocol.protocol.description.as_deref(),
        Some("a\u{0}\t\"b\"\\\u{2028}")
    );
    assert_eq!(protocol.claims.len(), 1);
    let compact =
        model::parse(&serde_json::to_string(&serde_json::from_str::<Json>(text).unwrap()).unwrap())
            .expect("parse accepts JSON on one line");
    assert!(compact == protocol);
}

/// The most deeply nested predicate `parse` reads: `serde_yaml_ng` refuses a document nested
/// deeper than its recursion limit, whichever way the document is laid out.
fn deepest_predicate_parse_reads() -> usize {
    let nested = |depth: usize| {
        format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nclaims: {{a: {{true_when: {}{{all: []}}{}}}}}\n",
            "{not: ".repeat(depth),
            "}".repeat(depth)
        )
    };
    let mut depth = 0;
    while depth < 1000 && model::parse(&nested(depth + 1)).is_ok() {
        depth += 1;
    }
    assert!(
        depth < 1000,
        "parse reads a predicate nested 1000 deep: no limit"
    );
    assert!(model::parse(&nested(depth)).is_ok());
    assert!(model::parse(&nested(depth + 1)).is_err());
    depth
}

/// Point 2: the nesting limit of `serde_yaml_ng`: a predicate `parse` reads is written at the same
/// depth, so the written text is read; one nested a level deeper is refused by `parse` itself.
#[test]
fn a_predicate_nested_as_deep_as_parse_reads_is_written_and_read_back() {
    let limit = deepest_predicate_parse_reads();
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nclaims: {{a: {{true_when: {}{{all: []}}{}}}}}\noutcomes: {{o: {{requires: {}{{claim: a}}{}}}}}\n",
        "{not: ".repeat(limit),
        "}".repeat(limit),
        "{not: ".repeat(limit),
        "}".repeat(limit),
    );
    let protocol = model::parse(&source).expect("the deepest predicate parses");
    assert!(round_trip(&protocol) == protocol);
    // The same for `arguments`, which the model holds as JSON and the writer nests as deep.
    let arguments_limit = (1..200)
        .take_while(|depth| {
            let source = format!(
                "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nactions: {{a: {{arguments: {}0{}}}}}\n",
                "[".repeat(*depth),
                "]".repeat(*depth)
            );
            model::parse(&source).is_ok()
        })
        .count();
    let nested_arguments = |depth: usize| {
        serde_json::from_str::<Json>(&format!("{}0{}", "[".repeat(depth), "]".repeat(depth)))
            .expect("nested arrays")
    };
    let protocol = with_arguments(nested_arguments(arguments_limit));
    assert!(round_trip(&protocol) == protocol);
    assert!(
        limit > 50 && arguments_limit > 50,
        "{limit} {arguments_limit}"
    );
}

/// Point 3: whether `serde_json`'s spellings of a number read back as numbers.
#[test]
fn the_numbers_in_arguments_a_parse_gives_read_back_equal() {
    let mut numbers: Vec<serde_json::Number> = vec![
        serde_json::Number::from(0u64),
        serde_json::Number::from(1u64),
        serde_json::Number::from(-1i64),
        serde_json::Number::from(4_294_967_296u64),
        serde_json::Number::from(u64::MAX),
        serde_json::Number::from(i64::MIN),
    ];
    for float in [
        0.5,
        -0.5,
        1.5,
        100.0,
        std::f64::consts::PI,
        1e16,
        1e21,
        1e300,
        -1e300,
        1e-7,
        1.5e-7,
        2.2250738585072014e-308,
        5e-324,
        f64::MAX,
        f64::MIN_POSITIVE,
        -0.0,
        123456789012345680000.0,
    ] {
        numbers.push(serde_json::Number::from_f64(float).expect("finite"));
    }
    let arguments = Json::Array(numbers.iter().cloned().map(Json::Number).collect());
    let protocol = with_arguments(arguments);
    let text = write(&protocol);
    let read = round_trip(&protocol);
    assert!(read == protocol, "a number does not read back:\n{text}");
    // Each is written as `serde_json` spells it, and every spelling read back as a number.
    for number in &numbers {
        assert!(
            text.contains(&number.to_string()),
            "{number} is not written as spelled"
        );
    }
}

#[test]
fn a_number_spelled_otherwise_reads_back_as_the_double_it_names() {
    // `1.50` and `1E5` are numbers whose text is not what `parse` itself spells; a build that
    // holds number text writes them as given and reads back the number `parse` spells for the
    // same double, a build that holds doubles has already spelled them so.
    let arguments: Json = serde_json::from_str(
        r#"[1.50, 1E5, 100000000000000000000, 0.10, 12345678901234567890123]"#,
    )
    .expect("JSON");
    let protocol = with_arguments(arguments.clone());
    let read = round_trip(&protocol);
    let read = read
        .actions
        .iter()
        .next()
        .expect("an action")
        .1
        .arguments
        .clone()
        .expect("arguments");
    for (given, back) in arguments
        .as_array()
        .unwrap()
        .iter()
        .zip(read.as_array().unwrap())
    {
        let given = given.as_f64().expect("a double");
        assert_eq!(back.as_f64(), Some(given), "{back} reads {given}");
        let spelled = serde_json::Number::from_f64(given).expect("finite");
        assert_eq!(
            back,
            &Json::Number(spelled),
            "{back} is not the spelling parse gives"
        );
    }
    if !holds_number_text() {
        assert!(read == arguments);
    }
}

/// With number text, `1e400` is a number the build holds and no double holds. No spelling reads
/// back as it: `serde_yaml_ng` reads `1e400` as the string, in block YAML as in JSON, so `parse`
/// gives a string for it. The text is written as it is, and this pins what happens to it. Without
/// number text there is no such number.
#[test]
fn a_number_beyond_a_double_is_written_and_read_back_as_a_string() {
    match serde_json::from_str::<Json>("[1e400, -1e400]") {
        Ok(arguments) => {
            assert!(
                holds_number_text(),
                "only a build with number text holds it"
            );
            let spelled: Vec<String> = arguments
                .as_array()
                .unwrap()
                .iter()
                .map(Json::to_string)
                .collect();
            let text = write(&with_arguments(arguments));
            assert!(
                spelled
                    .iter()
                    .all(|spelling| text.contains(spelling.as_str())),
                "{text}"
            );
            let read = model::parse(&text).expect("the text is read");
            let arguments = read
                .actions
                .iter()
                .next()
                .expect("an action")
                .1
                .arguments
                .clone();
            assert_eq!(arguments, Some(serde_json::json!(spelled)));
        }
        Err(_) => assert!(!holds_number_text()),
    }
}

/// An integer beyond 64 bits is refused by `parse` as digits, up to 128 bits: the writer spells it
/// as the double it names, so the text is read back.
#[test]
fn an_integer_beyond_64_bits_is_written_as_a_double() {
    let arguments: Json = serde_json::from_str(
        "[100000000000000000000, -100000000000000000000, 18446744073709551616]",
    )
    .expect("JSON");
    let text = write(&with_arguments(arguments));
    assert!(
        !text.contains("00000000000"),
        "an integer beyond 64 bits is written as digits:\n{text}"
    );
    assert!(model::parse(&text).is_ok(), "{text}");
}

#[test]
fn arguments_with_number_text_are_a_fixpoint_from_the_first_read() {
    // A model built with number text is written as that text, which `parse` reads as the double;
    // from the model `parse` gives, every write is the same bytes.
    let arguments: Json =
        serde_json::from_str(r#"{"a": [1.50, 1E5, 100000000000000000000]}"#).unwrap();
    let read = round_trip(&with_arguments(arguments));
    let first = write(&read);
    let again = model::parse(&first).expect("reads");
    assert!(again == read);
    assert_eq!(write(&again), first);
}

/// Point 4: repeated keys in a flow mapping reach `Declarations`, as they do from block YAML.
#[test]
fn repeated_keys_in_a_flow_mapping_reach_the_declarations() {
    let text = "{\"format\": \"protocol/1\", \"protocol\": {\"id\": \"p\", \"revision\": 1}, \"claims\": {\"a\": {\"true_when\": {\"claim\": \"x\"}}, \"a\": {\"true_when\": {\"claim\": \"y\"}}}}";
    let protocol = model::parse(text).expect("parses");
    assert_eq!(protocol.claims.len(), 2);
    let ids: Vec<&str> = protocol.claims.ids().map(|id| id.as_str()).collect();
    assert_eq!(ids, ["a", "a"]);
    let first = protocol.claims.iter().next().expect("a claim").1;
    assert_eq!(first.true_when.claim_references()[0].as_str(), "x");
    assert!(round_trip(&protocol) == protocol);
}

// ---------------------------------------------------------------------------------------------
// Spelling rules beyond the story's outcome, and what a model value `parse` never gives may do.

#[test]
fn a_claim_test_is_always_written_with_is_and_a_decision_as_a_map() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        claims: {c: {true_when: {evidence: {kind: e}}}}\n\
        obligations: {o: {discharged_when: {claim: c}}}\n\
        outcomes: {done: {requires: {claim: c, is: unknown}}, held: {requires: {decision: hold}}}\n";
    let protocol = model::parse(source).expect("parses");
    let text = write(&protocol);
    assert!(
        text.contains("\"claim\": \"c\",\n        \"is\": \"true\""),
        "{text}"
    );
    assert!(text.contains("\"is\": \"unknown\""), "{text}");
    assert!(text.contains("\"decision\": \"hold\""), "{text}");
    assert!(round_trip(&protocol) == protocol);
}

#[test]
fn an_absent_option_and_an_empty_defaulted_section_are_left_out() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        actions: {a: {requires: [], may_produce: [], revises: []}}\n\
        artifacts: {}\n";
    let protocol = model::parse(source).expect("parses");
    let text = write(&protocol);
    for absent in [
        "requires",
        "may_produce",
        "revises",
        "artifacts",
        "description",
        "scope",
    ] {
        assert!(!text.contains(absent), "{absent} is written:\n{text}");
    }
    assert!(text.contains("\"a\": {}"), "{text}");
    assert!(round_trip(&protocol) == protocol);
}

#[test]
fn a_scalar_written_as_a_boolean_or_a_number_comes_back_as_text() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        case: {inputs: {flag: {values: [true, 2, 0.5, ~x]}}}\n";
    let protocol = model::parse(source).expect("parses");
    let text = write(&protocol);
    assert!(text.contains("\"true\"") && text.contains("\"2\"") && text.contains("\"0.5\""));
    assert!(!text.contains(": true") && !text.contains("  2"));
    assert!(round_trip(&protocol) == protocol);
}

#[test]
fn identifiers_yaml_would_read_as_something_else_are_written_as_text() {
    let ids = [
        "true",
        "false",
        "null",
        "~",
        "1",
        "0x1f",
        "1e3",
        ".inf",
        "<<",
        "=",
        "&a",
        "*a",
        "!tag",
        "? q",
        "- q",
        "---",
        "...",
        "#x",
        "a: b",
        "a #b",
        " lead",
        "trail ",
        "",
        "[",
        "}",
        "'",
        "\"",
        "a\\b",
        "ü",
        "日本",
        "\u{10000}",
        "\u{fffd}",
    ];
    let mut protocol = minimal();
    protocol.claims = claims(&ids);
    protocol.actions = Declarations::new(
        ids.iter()
            .map(|id| {
                (
                    model::ActionId::new(*id),
                    Action {
                        requires: vec![CapabilityRequirement {
                            capability: CapabilityId::new(*id),
                            scope: Declarations::new(vec![(
                                model::ScopeKey::new(*id),
                                model::ScopeValue::new(*id),
                            )]),
                        }],
                        ..Action::default()
                    },
                )
            })
            .collect(),
    );
    assert!(round_trip(&protocol) == protocol);
}

#[test]
fn the_largest_revision_reads_back() {
    let mut protocol = minimal();
    protocol.protocol.revision = u64::MAX;
    assert!(round_trip(&protocol) == protocol);
}

/// The longest key, in bytes as written between its quotes with the escapes, that `parse` reads
/// back: the YAML reader behind it drops a simple key more than 1024 bytes long before its `:`,
/// quotes included (decided in review, D1). Measured, not assumed: 1022 reads, 1023 does not, for
/// each kind of key below.
const KEY_LIMIT: usize = 1022;

/// A key that is `n` bytes as written: `n` letters.
fn plain_key(n: usize) -> String {
    "k".repeat(n)
}

/// A key that is `n` bytes as written, a part of it `\` and `"`, which are written as two bytes
/// each.
fn escaped_key(n: usize) -> String {
    let pairs = n / 8;
    format!("{}{}", "\\\"".repeat(pairs), "k".repeat(n - 4 * pairs))
}

/// How `write` spells `key` between the quotes, for a key made of letters, `\` and `"`.
fn spelled(key: &str) -> String {
    key.replace('\\', "\\\\").replace('"', "\\\"")
}

fn claim_key(key: &str) -> Protocol {
    let mut protocol = minimal();
    protocol.claims = claims(&[key]);
    protocol
}

fn property_key(key: &str) -> Protocol {
    let mut properties = serde_json::Map::new();
    properties.insert(key.to_owned(), serde_json::json!({"type": "string"}));
    with_arguments(serde_json::json!({"type": "object", "properties": properties}))
}

#[test]
fn a_key_reads_back_up_to_1022_bytes_as_written_and_not_beyond() {
    type Kind = (&'static str, fn(usize) -> String, fn(&str) -> Protocol);
    let kinds: [Kind; 4] = [
        ("plain identifier", plain_key, claim_key),
        ("identifier grown by escapes", escaped_key, claim_key),
        ("property name in arguments", plain_key, property_key),
        ("escaped property name", escaped_key, property_key),
    ];
    for (kind, key, protocol) in kinds {
        for (n, reads) in [
            (1, true),
            (1021, true),
            (KEY_LIMIT, true),
            (KEY_LIMIT + 1, false),
            (KEY_LIMIT + 2, false),
            (2000, false),
        ] {
            let key = key(n);
            assert_eq!(
                spelled(&key).len(),
                n,
                "{kind}: the key is {n} bytes as written"
            );
            let protocol = protocol(&key);
            let text = write(&protocol);
            assert!(
                text.contains(&format!("\"{}\": ", spelled(&key))),
                "{kind}: {n}"
            );
            assert_eq!(
                model::parse(&text).is_ok_and(|read| read == protocol),
                reads,
                "{kind}: a key of {n} bytes as written"
            );
        }
    }
}

#[test]
fn a_multibyte_key_counts_its_utf8_bytes_against_the_limit() {
    // 340 characters of three bytes and two letters make 1022 bytes; one more character makes 1025.
    let within = format!("{}kk", "\u{65e5}".repeat(340));
    let beyond = format!("{within}\u{65e5}");
    assert_eq!(within.len(), KEY_LIMIT);
    assert!(round_trip(&claim_key(&within)) == claim_key(&within));
    assert!(model::parse(&write(&claim_key(&beyond))).is_err());
}

#[test]
fn the_text_for_a_key_beyond_the_limit_is_json_holding_the_key() {
    for key in [plain_key(2000), escaped_key(2000)] {
        let text = write(&claim_key(&key));
        let value: Json = serde_json::from_str(&text).expect("serde_json reads the text");
        let claims = value["claims"].as_object().expect("claims");
        assert_eq!(
            claims.keys().collect::<Vec<_>>(),
            [&key],
            "the key is intact"
        );
        assert!(text.ends_with("}\n"));
    }
    let key = plain_key(2000);
    let text = write(&property_key(&key));
    let value: Json = serde_json::from_str(&text).expect("serde_json reads the text");
    assert!(value["actions"]["act"]["arguments"]["properties"][&key].is_object());
}

/// `Some(null)` arguments are the one model value with no `protocol/1` spelling at all: `arguments:
/// null` is refused as an explicit null, so `parse` never gives it and `write` writes the JSON
/// `null`.
#[test]
fn null_arguments_are_written_and_refused_by_parse_like_an_explicit_null() {
    let protocol = with_arguments(Json::Null);
    let text = write(&protocol);
    assert!(text.contains("\"arguments\": null"), "{text}");
    assert!(model::parse(&text).is_err());
}

/// A `null` inside arguments, not the whole of them, reads back.
#[test]
fn a_null_inside_arguments_reads_back() {
    let protocol = with_arguments(serde_json::json!({"enum": [null, 1, "a"], "x": {"y": null}}));
    assert!(round_trip(&protocol) == protocol);
}

/// A key inside `arguments` that YAML would read as a boolean, null, number or merge key stays
/// text, and so does an empty one.
#[test]
fn arguments_keys_that_yaml_would_read_otherwise_stay_text() {
    let protocol = with_arguments(serde_json::json!({
        "true": 1, "null": 2, "~": 3, "1": 4, "<<": 5, "": 6, "a: b": 7, "é": 8
    }));
    assert!(round_trip(&protocol) == protocol);
}

#[test]
fn a_hand_built_declaration_set_is_written_in_its_order_with_repeats() {
    let mut protocol = minimal();
    protocol.claims = claims(&["z", "m", "z", "a"]);
    let read = round_trip(&protocol);
    let ids: Vec<&str> = read.claims.ids().map(|id| id.as_str()).collect();
    assert_eq!(ids, ["z", "m", "z", "a"]);
}

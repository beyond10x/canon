//! A key left out of any document Canon reads is refused naming that key, never as an explicit
//! null (story:review-hardening-w7).
//!
//! Serde reads a left-out field of a type without `deserialize_with` by asking the type to read a
//! null, and every identifier refuses a null with advice to leave the key out — so a left-out
//! identifier was once refused as `an explicit null is not allowed here; leave the key out to take
//! its default`, naming nothing. Rather than list the fields that can do this, this test takes a
//! full document of every kind Canon reads — the case snapshot, an evidence record, the
//! authority and explicit decisions, the properties, every `protocol/1` fixture and every
//! conformance scenario — removes each key of it in turn, at every depth, and reads the result.
//! Where that is refused, the refusal must name the key and must not speak of an explicit null;
//! and since the key is then required, writing it with an explicit null must not be refused with
//! the advice to leave it out (`leave the key out to take its default`).

use std::fs;
use std::path::PathBuf;

use b10x_canon::eval::{self, Supplied};
use b10x_canon::{check, conform, ir, model};
use serde_yaml_ng::{Mapping, Value};

/// The repository root, read at run time: a test binary reused from a shared build directory
/// would otherwise read the tree it was built from.
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"))
        .join("../..")
}

/// Every mapping key of `value`, as the path of keys and list positions that leads to it.
fn key_paths(value: &Value, path: &mut Vec<Value>, out: &mut Vec<Vec<Value>>) {
    match value {
        Value::Mapping(mapping) => {
            for (key, inner) in mapping {
                path.push(key.clone());
                out.push(path.clone());
                key_paths(inner, path, out);
                path.pop();
            }
        }
        Value::Sequence(items) => {
            for (at, inner) in items.iter().enumerate() {
                path.push(Value::from(at as u64));
                key_paths(inner, path, out);
                path.pop();
            }
        }
        Value::Tagged(tagged) => key_paths(&tagged.value, path, out),
        _ => {}
    }
}

/// `value` with the key at the end of `path` removed.
fn without(value: &Value, path: &[Value]) -> Value {
    edited(value, path, false)
}

/// `value` with the key at the end of `path` written as an explicit null.
fn nulled(value: &Value, path: &[Value]) -> Value {
    edited(value, path, true)
}

fn edited(value: &Value, path: &[Value], null: bool) -> Value {
    let mut copy = value.clone();
    let (last, parents) = path.split_last().expect("a path names a key");
    let mut node = &mut copy;
    for step in parents {
        node = match (node, step) {
            (Value::Sequence(items), Value::Number(at)) => {
                &mut items[at.as_u64().expect("a position") as usize]
            }
            (Value::Mapping(mapping), key) => mapping.get_mut(key).expect("the path exists"),
            (Value::Tagged(tagged), _) => &mut tagged.value,
            _ => unreachable!("the path was read from this value"),
        };
    }
    let Value::Mapping(mapping) = node else {
        unreachable!("a key's parent is a mapping")
    };
    let removed: Option<Value> = if null {
        mapping.insert(last.clone(), Value::Null)
    } else {
        Mapping::remove(mapping, last)
    };
    assert!(removed.is_some(), "the key exists");
    copy
}

/// Removes each key of `text` in turn and reads the result with `read`; every refusal must name
/// the key and not speak of an explicit null. Returns how many removals were refused, so a
/// document whose every key is optional cannot pass unnoticed.
fn each_key_left_out(what: &str, text: &str, read: &dyn Fn(&str) -> Result<(), String>) -> usize {
    left_out(what, text, true, read)
}

/// [`each_key_left_out`], where `names_key` says whether the refusal must also name the key.
fn left_out(
    what: &str,
    text: &str,
    names_key: bool,
    read: &dyn Fn(&str) -> Result<(), String>,
) -> usize {
    read(text).unwrap_or_else(|why| panic!("{what}: the full document must read: {why}"));
    let value: Value = serde_yaml_ng::from_str(text).expect("the document is YAML");
    let mut paths = Vec::new();
    key_paths(&value, &mut Vec::new(), &mut paths);
    let mut refused = 0;
    let mut wrong = Vec::new();
    for path in paths {
        let key = path
            .last()
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let left_out = serde_yaml_ng::to_string(&without(&value, &path)).expect("serializes");
        if let Err(message) = read(&left_out) {
            refused += 1;
            if message.contains("explicit null") || (names_key && !message.contains(&key)) {
                wrong.push(format!("{what} without `{key}` (at {path:?}): {message}"));
            }
            // The key is required: a null on it must not send the author to leave it out.
            let null_text = serde_yaml_ng::to_string(&nulled(&value, &path)).expect("serializes");
            if let Err(message) = read(&null_text)
                && message.contains("leave the key out")
            {
                wrong.push(format!("{what} with `{key}: ~` (at {path:?}): {message}"));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "a left-out key refused without naming it, or as an explicit null:\n{}",
        wrong.join("\n")
    );
    refused
}

fn protocol_ir(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

#[test]
fn every_left_out_key_of_an_evaluation_document_is_named() {
    let case = "format: canon-case/1\nid: C-1\nprotocol: p\nrevision: c2\n\
                artifacts: {a: {revision: r1}}\ntermination: done\n";
    let refused = each_key_left_out("canon-case/1", case, &|text| {
        eval::read_case(text).map(drop).map_err(|r| r.to_string())
    });
    assert!(refused >= 5, "canon-case/1: {refused} removals refused");

    let evidence = "format: canon-evidence/1\nid: e1\nkind: k\nresult: pass\nsubject: a\n\
                    subject_revision: r1\nobserved_at: 2026-10-04T12:00:00Z\n\
                    upstream_revisions: {up: u1}\n";
    let refused = each_key_left_out("canon-evidence/1", evidence, &|text| {
        eval::read_evidence(text)
            .map(drop)
            .map_err(|r| r.to_string())
    });
    assert!(refused >= 5, "canon-evidence/1: {refused} removals refused");

    let ir = protocol_ir(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         outcomes: {inconclusive: {requires: {decision: stop}}}\n",
    );
    let snapshot = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nrevision: c2\nartifacts: {a: {revision: r1}}\n",
    )
    .expect("case reads");
    let supplied = |supplied: Supplied<'_>| {
        eval::evaluate_with(&ir, &snapshot, &[], supplied)
            .map(drop)
            .map_err(|r| r.to_string())
    };
    let refused = each_key_left_out(
        "canon-authority/1",
        "- {capability: x, decision: granted}\n- {capability: y, decision: denied}\n",
        &|text| {
            supplied(Supplied {
                authority: Some(text),
                ..Supplied::default()
            })
        },
    );
    assert_eq!(refused, 4, "canon-authority/1: every key is required");
    let refused = each_key_left_out(
        "canon-decisions/1",
        "- {decision: stop, outcome: inconclusive, principal: lead, case_revision: c2}\n",
        &|text| {
            supplied(Supplied {
                decisions: Some(text),
                ..Supplied::default()
            })
        },
    );
    assert_eq!(refused, 4, "canon-decisions/1: every key is required");

    let properties = fs::read_to_string(
        root().join("fixtures/investigation/check/failing-property/properties.yaml"),
    )
    .expect("the properties fixture reads");
    let refused = each_key_left_out("canon-properties/1", &properties, &|text| {
        check::read_properties(text)
            .map(drop)
            .map_err(|r| r.to_string())
    });
    assert!(
        refused >= 3,
        "canon-properties/1: {refused} removals refused"
    );
}

/// Every `protocol/1` fixture that parses, and every conformance scenario, with each key left out.
#[test]
fn every_left_out_key_of_a_protocol_or_a_scenario_is_named() {
    let mut protocols = Vec::new();
    let mut pending = vec![root().join("fixtures")];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("fixture directory reads") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "yaml") {
                let text = fs::read_to_string(&path).expect("fixture reads");
                // A fixture that repeats a key on purpose is not a YAML value this test can edit.
                if text.starts_with("format: protocol/1")
                    && model::parse(&text).is_ok()
                    && serde_yaml_ng::from_str::<Value>(&text).is_ok()
                {
                    protocols.push((path, text));
                }
            }
        }
    }
    assert!(
        protocols.len() >= 10,
        "{} protocol fixtures",
        protocols.len()
    );
    let mut refused = 0;
    for (path, text) in &protocols {
        refused += each_key_left_out(&path.display().to_string(), text, &|text| {
            model::parse(text).map(drop).map_err(|e| e.to_string())
        });
    }
    assert!(refused > 0, "no removal from a protocol was refused");

    // The scenario reader states only the kind and position of a shape it refuses, never the
    // deserializer's message (`parse_failure` in `conform/mod.rs`), so a scenario is held only to
    // not speaking of an explicit null.
    let mut scenarios = 0;
    for entry in fs::read_dir(root().join("conformance/scenarios")).expect("scenarios read") {
        let path = entry.expect("entry").path();
        let text = fs::read_to_string(&path).expect("scenario reads");
        scenarios += left_out(&path.display().to_string(), &text, false, &|text| {
            conform::parse(text).map(drop).map_err(|e| e.to_string())
        });
    }
    assert!(scenarios > 0, "no removal from a scenario was refused");
}

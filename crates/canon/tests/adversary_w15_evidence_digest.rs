//! Adversary of wave 15 (story:evidence-digest-binds-approvals). The digest an evidence ground
//! gives is computed here from the definition the reference writes (`website/docs/reference/
//! documents.md`, `EvidenceDigest`): SHA-256 over the RFC 8785 canonical JSON of one object with
//! `format`, `id`, `kind`, `subject`, `subject_revision` always, `result`, `observed_at`,
//! `producer`, `child_case`, `item` when given, `upstream_revisions` when not empty, every value a
//! string, keys sorted by UTF-16 code units. Nothing of Canon's own digest code is used to compute
//! the expected value: the canonical text is built from the raw YAML value with a JSON string
//! writer of this file, or written out by hand.

use std::path::{Path, PathBuf};

use b10x_canon::eval::{self, Supplied};
use b10x_canon::{ir, model};
use serde_yaml_ng::Value;
use sha2::{Digest, Sha256};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sha256(text: &str) -> String {
    let mut out = String::from("sha256:");
    for byte in Sha256::digest(text.as_bytes()) {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// A JSON string as RFC 8785 § 3.2.2.2 writes it (ECMAScript `JSON.stringify`): `"` and `\`
/// escaped, U+0008, U+0009, U+000A, U+000C and U+000D in their short forms, every other code
/// point below U+0020 as `\u00xx` in lowercase hex, everything else as itself.
fn jcs_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// An object of already-written members, keys sorted by their UTF-16 code units (RFC 8785
/// § 3.2.3).
fn jcs_object(mut members: Vec<(String, String)>) -> String {
    members.sort_by(|(a, _), (b, _)| {
        a.encode_utf16()
            .collect::<Vec<u16>>()
            .cmp(&b.encode_utf16().collect::<Vec<u16>>())
    });
    let body: Vec<String> = members
        .into_iter()
        .map(|(key, value)| format!("{}:{value}", jcs_string(&key)))
        .collect();
    format!("{{{}}}", body.join(","))
}

/// The documented canonical form of a record, from its raw YAML value; `None` when a field this
/// reads is not a plain string (Canon refuses such a record, or it is not one).
fn documented_canonical(record: &Value) -> Option<String> {
    let map = record.as_mapping()?;
    let text = |key: &str| map.get(key).map(|v| v.as_str().map(str::to_owned));
    let mut members = Vec::new();
    for key in ["format", "id", "kind", "subject", "subject_revision"] {
        members.push((key.to_owned(), jcs_string(&text(key)??)));
    }
    for key in ["result", "observed_at", "producer", "child_case", "item"] {
        if let Some(value) = text(key) {
            members.push((key.to_owned(), jcs_string(&value?)));
        }
    }
    if let Some(upstream) = map.get("upstream_revisions") {
        let upstream = upstream.as_mapping()?;
        if !upstream.is_empty() {
            let mut pairs = Vec::new();
            for (artifact, revision) in upstream {
                pairs.push((
                    artifact.as_str()?.to_owned(),
                    jcs_string(revision.as_str()?),
                ));
            }
            members.push(("upstream_revisions".to_owned(), jcs_object(pairs)));
        }
    }
    Some(jcs_object(members))
}

fn yaml_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            yaml_files(&path, out);
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "json")
        {
            out.push(path);
        }
    }
}

/// Every evidence record the repository holds: each `canon-evidence/1` file under `fixtures/` and
/// `conformance/`, and each record of an `evaluate` step of every `canon-conformance/1` scenario.
fn repository_records() -> Vec<(String, Value)> {
    let mut files = Vec::new();
    for dir in ["fixtures", "conformance"] {
        yaml_files(&root().join(dir), &mut files);
    }
    files.sort();
    let mut records = Vec::new();
    for file in files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let Ok(value) = serde_yaml_ng::from_str::<Value>(&text) else {
            continue;
        };
        let name = file.display().to_string();
        match value.get("format").and_then(Value::as_str) {
            Some("canon-evidence/1") => records.push((name, value)),
            Some("canon-conformance/1") => {
                let steps = value.get("steps").and_then(Value::as_sequence);
                for (at, step) in steps.into_iter().flatten().enumerate() {
                    let evidence = step
                        .get("evaluate")
                        .and_then(|e| e.get("evidence"))
                        .and_then(Value::as_sequence);
                    for (n, record) in evidence.into_iter().flatten().enumerate() {
                        records.push((format!("{name} step {at} record {n}"), record.clone()));
                    }
                }
            }
            _ => {}
        }
    }
    records
}

/// For every record of the repository that Canon reads as a `canon-evidence/1` record, the digest
/// Canon gives (the evaluator's `evidence_digest`, and `digest_evidence`, which `canon digest`
/// prints, over the record's text) is the SHA-256 of the documented canonical form.
#[test]
fn every_repository_record_has_the_documented_digest() {
    let mut compared = 0;
    for (name, value) in repository_records() {
        let Ok(record) = eval::evidence_from_value(&value) else {
            continue;
        };
        if record.format != model::EVIDENCE_FORMAT {
            continue;
        }
        let canonical = documented_canonical(&value)
            .unwrap_or_else(|| panic!("{name}: Canon read it, so its fields are strings"));
        let expected = sha256(&canonical);
        assert_eq!(
            eval::evidence_digest(&record).as_str(),
            expected,
            "{name}: {canonical}"
        );
        let text = serde_yaml_ng::to_string(&value).expect("the record is written back");
        assert_eq!(
            eval::digest_evidence(&text)
                .expect("canon digest reads it")
                .as_str(),
            expected,
            "{name}: canon digest"
        );
        compared += 1;
    }
    assert!(
        compared >= 200,
        "only {compared} records compared: the walk found too few to be a property"
    );
}

/// The protocol of `fixtures/investigation/decision-grounds.yaml` with two more artifacts whose
/// ids sort differently in UTF-8 and in UTF-16: `ｚ` (U+FF5A) and `𝒜` (U+1D49C, a surrogate pair).
const PROTOCOL: &str = "format: protocol/1\n\
protocol: {id: grounds, revision: 1, description: d}\n\
artifacts:\n  spec: {description: d}\n  up: {description: d}\n  ｚ: {description: d}\n  𝒜: {description: d}\n\
evidence_kinds:\n  k: {description: d}\n\
claims:\n  c: {description: d, true_when: {evidence: {kind: k}}}\n\
actions:\n  act: {description: d, requires: [{capability: x}]}\n";

const CASE: &str = "format: canon-case/1\nid: G-1\nprotocol: grounds\nrevision: c2\n\
artifacts: {spec: {revision: r2}, up: {revision: u2}, ｚ: {revision: z1}, 𝒜: {revision: a1}}\n";

/// A record whose `result` needs every escape RFC 8785 writes and some it must not (U+007F,
/// U+2028, `/`, non-ASCII), whose `producer` and `item` are not ASCII, and whose upstream keys
/// sort differently by UTF-8 bytes and by UTF-16 code units.
const RECORD: &str = r#"{"format": "canon-evidence/1", "id": "e1", "kind": "k", "subject": "spec", "subject_revision": "r2", "result": "a\u0001b\u001fc\"d\\e\u007ff gé😀\th\ni\bj\fk\rl/m", "producer": "ü", "item": "ⅹ", "upstream_revisions": {"ｚ": "z1", "up": "u2", "𝒜": "a1"}}"#;

/// [`RECORD`]'s canonical form, written out by hand from the reference's definition.
const RECORD_CANONICAL: &str = "{\"format\":\"canon-evidence/1\",\"id\":\"e1\",\"item\":\"ⅹ\",\
\"kind\":\"k\",\"producer\":\"ü\",\
\"result\":\"a\\u0001b\\u001fc\\\"d\\\\e\u{7f}f\u{2028}gé😀\\th\\ni\\bj\\fk\\rl/m\",\
\"subject\":\"spec\",\"subject_revision\":\"r2\",\
\"upstream_revisions\":{\"up\":\"u2\",\"𝒜\":\"a1\",\"ｚ\":\"z1\"}}";

/// The hand-written canonical form of a record that needs RFC 8785's string escapes and its
/// UTF-16 key order is the one whose digest holds a grant; one digit off, the grant lapses as
/// `content_changed`, so the comparison is not vacuous.
#[test]
fn escapes_and_utf16_key_order_follow_rfc_8785_and_hold_the_grant() {
    let value: Value = serde_yaml_ng::from_str(RECORD).expect("the record parses");
    assert_eq!(
        documented_canonical(&value).as_deref(),
        Some(RECORD_CANONICAL),
        "this file's writer agrees with the hand-written form"
    );
    let record = eval::read_evidence(RECORD).expect("the record reads");
    let digest = sha256(RECORD_CANONICAL);
    assert_eq!(eval::evidence_digest(&record).as_str(), digest);

    let compiled = ir::compile(&model::parse(PROTOCOL).expect("parses")).expect("compiles");
    let case = eval::read_case(CASE).expect("case reads");
    let decide = |digest: &str| {
        let authority = format!(
            "- {{capability: x, decision: granted, ground_evidence: [{{id: e1, digest: '{digest}'}}]}}\n"
        );
        eval::evaluate_with(
            &compiled,
            &case,
            std::slice::from_ref(&record),
            Supplied {
                authority: Some(&authority),
                ..Supplied::default()
            },
        )
        .expect("decides")
        .actions
        .expect("actions")["act"]
            .clone()
    };
    assert_eq!(decide(&digest), serde_json::json!({"status": "admissible"}));
    let last = digest.chars().last().expect("a digit");
    let other = format!(
        "{}{}",
        &digest[..digest.len() - 1],
        if last == '0' { '1' } else { '0' }
    );
    let lapsed = decide(&other);
    assert_eq!(
        lapsed["reasons"][0]["stale_grounds"],
        serde_json::json!([{"evidence": "e1", "reason": "content_changed"}]),
        "{lapsed}"
    );
}

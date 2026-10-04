//! Adversary case for story:evidence-freshness: the published JSON Schemas this unit regenerated
//! (`website/static/schemas/protocol-1.schema.json` and `evidence-1.schema.json`), read as the
//! contract they are, against what Canon accepts.
//!
//! The schemas declare `max_age` as `#/$defs/Age` and `observed_at` as `#/$defs/Instant`, each
//! with its own pattern. Every value a pattern accepts is one the schema promises Canon accepts,
//! except where the description names what Canon checks beyond the pattern (an instant's
//! calendar validity). This case evaluates both patterns by hand on values of that kind and asks
//! Canon.

use std::path::{Path, PathBuf};

use b10x_canon::eval::{self, Supplied};
use b10x_canon::{ir, model, validate};
use serde_json::Value;

/// Read at run time: a test binary reused from a shared build directory must read this tree.
fn repository_root() -> PathBuf {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    Path::new(&manifest)
        .join("../..")
        .canonicalize()
        .expect("repository root resolves")
}

fn schema(name: &str) -> Value {
    let path = repository_root().join("website/static/schemas").join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_str(&text).expect("schema is JSON")
}

/// The schema of the property `property`, wherever in `$defs` it is declared, with a local `$ref`
/// resolved.
fn property<'a>(root: &'a Value, property: &str) -> &'a Value {
    let defs = root["$defs"].as_object().expect("$defs");
    let declared = defs
        .values()
        .find_map(|def| def["properties"].get(property))
        .unwrap_or_else(|| panic!("no $defs entry declares `{property}`"));
    match declared["$ref"].as_str() {
        Some(reference) => {
            let name = reference
                .strip_prefix("#/$defs/")
                .unwrap_or_else(|| panic!("non-local $ref {reference}"));
            &defs[name]
        }
        None => declared,
    }
}

/// The `Age` pattern, evaluated by [`age_shape`].
const AGE_PATTERN: &str = "^(0|[1-9][0-9]*)[smhd]$";
/// The `Instant` pattern, evaluated by [`instant_shape`].
const INSTANT_PATTERN: &str = "^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$";

/// [`AGE_PATTERN`] by hand: `0`, or digits without a leading zero, then one of `s`, `m`, `h`, `d`.
fn age_shape(text: &str) -> bool {
    let Some(digits) = text.strip_suffix(['s', 'm', 'h', 'd']) else {
        return false;
    };
    !digits.is_empty()
        && digits.bytes().all(|byte| byte.is_ascii_digit())
        && (digits == "0" || !digits.starts_with('0'))
}

/// [`INSTANT_PATTERN`] by hand: 20 ASCII bytes, `-`, `-`, `T`, `:`, `:`, `Z` at their places and
/// a digit everywhere else.
fn instant_shape(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 20
        && bytes.iter().enumerate().all(|(at, &byte)| match at {
            4 | 7 => byte == b'-',
            10 => byte == b'T',
            13 | 16 => byte == b':',
            19 => byte == b'Z',
            _ => byte.is_ascii_digit(),
        })
}

/// Whether `schema` accepts the string `text`, for the keywords this case can evaluate; a schema
/// using a keyword it cannot evaluate is treated as refusing, so the case never claims a promise
/// the schema may not make.
fn accepts(schema: &Value, text: &str) -> bool {
    let object = schema.as_object().expect("schema object");
    for (keyword, value) in object {
        let ok = match keyword.as_str() {
            "description" | "$comment" | "title" => true,
            "type" => value == "string",
            "minLength" => text.chars().count() as u64 >= value.as_u64().expect("minLength"),
            "pattern" if value == "^\\S+$" => {
                !text.is_empty() && !text.chars().any(char::is_whitespace)
            }
            "pattern" if value == AGE_PATTERN => age_shape(text),
            "pattern" if value == INSTANT_PATTERN => instant_shape(text),
            _ => false,
        };
        if !ok {
            return false;
        }
    }
    true
}

fn protocol(max_age: &str) -> String {
    format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{a: {{}}}}\n\
         evidence_kinds: {{k: {{max_age: '{max_age}'}}}}\n"
    )
}

#[test]
fn every_max_age_the_published_protocol_schema_accepts_is_one_canon_accepts() {
    let root = schema("protocol-1.schema.json");
    let max_age = property(&root, "max_age");
    // The well-formed value must reach the comparison, or the case compares nothing.
    assert!(accepts(max_age, "5m"), "the schema accepts `5m`: {max_age}");
    let mut broken = Vec::new();
    for candidate in [
        "5m", "05m", "1H", "1w", "-1h", "+1h", "1.5h", "5min", "PT5M",
    ] {
        if !accepts(max_age, candidate) {
            continue;
        }
        let parsed = model::parse(&protocol(candidate)).expect("protocol parses");
        if let Err(problems) = validate::validate(&parsed) {
            broken.push(format!("{candidate}: {}", problems[0]));
        }
    }
    assert_eq!(
        broken,
        Vec::<String>::new(),
        "protocol-1.schema.json accepts these `max_age` values; canon validate refuses them"
    );
}

#[test]
fn every_observed_at_the_published_evidence_schema_accepts_is_one_canon_accepts() {
    let root = schema("evidence-1.schema.json");
    let observed_at = property(&root, "observed_at");
    let compiled = ir::compile(
        &model::parse(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
             evidence_kinds: {k: {}}\n",
        )
        .expect("parses"),
    )
    .expect("compiles");
    let case = eval::read_case(
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n",
    )
    .expect("case reads");
    // The well-formed value must reach the comparison, or the case compares nothing.
    assert!(
        accepts(observed_at, "2026-10-04T12:00:00Z"),
        "the schema accepts `2026-10-04T12:00:00Z`: {observed_at}"
    );
    // The documented exception: the `Instant` description in evidence-1.schema.json says
    // "Calendar validity is checked by Canon, not by the pattern: the date must exist (leap years
    // included) and the time be at most 23:59:59." These values match the pattern, are refused by
    // Canon, and differ from an accepted instant only in calendar validity.
    let calendar_only = ["2026-02-30T00:00:00Z"];
    assert!(
        observed_at["description"]
            .as_str()
            .is_some_and(|text| text.contains("Calendar validity is checked by Canon")),
        "the schema documents the calendar exception: {observed_at}"
    );
    let mut broken = Vec::new();
    for candidate in [
        "2026-10-04T12:00:00Z",
        "2026-10-04T12:00:00+02:00",
        "2026-10-04T12:00:00.5Z",
        "2026-10-04t12:00:00z",
        "2026-02-30T00:00:00Z",
        "2026-10-04",
        "yesterday",
    ] {
        if !accepts(observed_at, candidate) {
            continue;
        }
        let record = eval::read_evidence(&format!(
            "format: canon-evidence/1\nid: e\nkind: k\nsubject: a\nsubject_revision: r1\n\
             observed_at: '{candidate}'\n"
        ))
        .expect("record reads");
        let result = eval::evaluate_with(&compiled, &case, &[record], Supplied::default());
        if calendar_only.contains(&candidate) {
            // Exempt, but only while it is still the exception it is documented as.
            let refusal = result.expect_err("a calendar-only exemption is refused by Canon");
            assert_eq!(refusal.code(), "invalid-instant", "{candidate}: {refusal}");
            continue;
        }
        if let Err(refusal) = result {
            broken.push(format!("{candidate}: {}: {refusal}", refusal.code()));
        }
    }
    assert_eq!(
        broken,
        Vec::<String>::new(),
        "evidence-1.schema.json accepts these `observed_at` values; canon evaluate refuses them"
    );
}

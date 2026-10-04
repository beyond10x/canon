//! The canonical JSON a `canon-decision/1` document is written in: the same rules as `canon-ir/1`'s
//! serialization (`crate::ir`'s `json` module), for the values a decision holds, including any
//! JSON value a section slot carries. Object keys are sorted by Unicode code point, members are
//! indented by two spaces, an empty array or object is written `[]` or `{}`, and the document ends
//! with one newline. A number is written as `serde_json` writes it. Strings escape `"` and `\`,
//! write U+0008, U+0009, U+000A, U+000C and U+000D as `\b`, `\t`, `\n`, `\f` and `\r`, every
//! other character below U+0020 as `\u00xx` with lowercase hex, and every other character as
//! itself.
//!
//! The `tests` below hold this writer's string escaping equal to `canon-ir/1`'s.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Value {
    Null,
    Bool(bool),
    Integer(u64),
    /// Any other number, as `serde_json` writes it.
    Number(String),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl Value {
    pub(super) fn string(text: impl Into<String>) -> Self {
        Value::String(text.into())
    }

    /// An object from `(key, value)` pairs; the pairs' order does not matter.
    pub(super) fn object<'a>(entries: impl IntoIterator<Item = (&'a str, Value)>) -> Self {
        Value::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        )
    }

    /// A section slot's JSON value, every object re-keyed in canonical order.
    pub(super) fn json(value: &serde_json::Value) -> Self {
        use serde_json::Value as Json;
        match value {
            Json::Null => Value::Null,
            Json::Bool(flag) => Value::Bool(*flag),
            Json::Number(number) => match number.as_u64() {
                Some(integer) => Value::Integer(integer),
                None => Value::Number(number.to_string()),
            },
            Json::String(text) => Value::string(text.as_str()),
            Json::Array(items) => Value::Array(items.iter().map(Value::json).collect()),
            Json::Object(entries) => Value::Object(
                entries
                    .iter()
                    .map(|(key, item)| (key.clone(), Value::json(item)))
                    .collect(),
            ),
        }
    }

    /// The canonical rendering, ending with a newline.
    pub(super) fn render(&self) -> String {
        let mut out = String::new();
        write(self, 0, &mut out);
        out.push('\n');
        out
    }
}

fn write(value: &Value, indent: usize, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
        Value::Integer(number) => out.push_str(&number.to_string()),
        Value::Number(number) => out.push_str(number),
        Value::String(text) => write_string(text, out),
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push('\n');
                out.extend(std::iter::repeat_n(' ', indent + 2));
                write(item, indent + 2, out);
            }
            out.push('\n');
            out.extend(std::iter::repeat_n(' ', indent));
            out.push(']');
        }
        Value::Object(entries) if entries.is_empty() => out.push_str("{}"),
        Value::Object(entries) => {
            out.push('{');
            for (index, (key, item)) in entries.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push('\n');
                out.extend(std::iter::repeat_n(' ', indent + 2));
                write_string(key, out);
                out.push_str(": ");
                write(item, indent + 2, out);
            }
            out.push('\n');
            out.extend(std::iter::repeat_n(' ', indent));
            out.push('}');
        }
    }
}

fn write_string(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::Value;
    use crate::{ir, model};

    /// Text through every escaping rule: quote, backslash, every character below U+0020, DEL, a
    /// line separator and a non-ASCII letter.
    fn awkward() -> String {
        let every_low: String = (0u8..0x20).map(char::from).collect();
        format!("\"\\{every_low}\u{7f}\u{2028}é")
    }

    #[test]
    fn strings_are_escaped_exactly_as_canon_ir_escapes_them() {
        // A protocol description is free text that canon-ir/1 renders as a JSON string.
        let mut protocol =
            model::parse("format: protocol/1\nprotocol: {id: p, revision: 1}\n").expect("parses");
        protocol.protocol.description = Some(awkward());
        let ir = ir::compile(&protocol).expect("compiles").canonical_json();
        let ours = Value::string(awkward()).render();
        let ours = ours.trim_end();
        let line = ir
            .lines()
            .find(|line| line.trim_start().starts_with("\"description\": \"\\\""))
            .expect("the protocol description line");
        assert_eq!(
            line.trim_start()
                .trim_start_matches("\"description\": ")
                .trim_end_matches(','),
            ours
        );
    }

    #[test]
    fn objects_nest_with_two_space_indentation_and_sorted_keys() {
        let value = Value::object([
            ("b", Value::Integer(1)),
            (
                "a",
                Value::object([("é", Value::object([])), ("B", Value::string("x"))]),
            ),
        ]);
        assert_eq!(
            value.render(),
            "{\n  \"a\": {\n    \"B\": \"x\",\n    \"é\": {}\n  },\n  \"b\": 1\n}\n"
        );
    }

    /// A section slot's JSON value: arrays, null, booleans and numbers, objects re-keyed.
    #[test]
    fn a_section_slot_value_renders_canonically() {
        let slot: serde_json::Value = serde_json::from_str(
            r#"{"z": [1, {"y": null, "x": true}, []], "a": -1.5, "m": false, "e": {}}"#,
        )
        .expect("json");
        assert_eq!(
            Value::json(&slot).render(),
            "{\n  \"a\": -1.5,\n  \"e\": {},\n  \"m\": false,\n  \"z\": [\n    1,\n    {\n      \"x\": true,\n      \"y\": null\n    },\n    []\n  ]\n}\n"
        );
    }
}

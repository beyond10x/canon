//! The one serialization `canon-ir/1` has: canonical JSON.
//!
//! Object keys are sorted by Unicode code point (a `BTreeMap<String, _>`), members are indented by
//! two spaces, an empty array or object is written `[]` or `{}`, and the document ends with one
//! newline. Strings escape `"` and `\`, write the control characters U+0008, U+0009, U+000A, U+000C
//! and U+000D as `\b`, `\t`, `\n`, `\f` and `\r`, every other character below U+0020 as `\u00xx`
//! with lowercase hex, and every other character as itself. Nothing else varies, so the same value
//! always renders to the same bytes.

use std::collections::BTreeMap;

/// A JSON value. It has no ordering: the order of predicate members is
/// [`super::canonical_order`], defined on predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Value {
    Null,
    Integer(u64),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl Value {
    pub(crate) fn string(text: impl Into<String>) -> Self {
        Value::String(text.into())
    }

    pub(crate) fn optional(text: Option<&str>) -> Self {
        text.map_or(Value::Null, Value::string)
    }

    /// An object from `(key, value)` pairs; the pairs' order does not matter.
    pub(crate) fn object<'a>(entries: impl IntoIterator<Item = (&'a str, Value)>) -> Self {
        Value::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        )
    }

    /// The canonical rendering, ending with a newline.
    pub(crate) fn render(&self) -> String {
        let mut out = String::new();
        write(self, 0, &mut out);
        out.push('\n');
        out
    }
}

fn write(value: &Value, indent: usize, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Integer(number) => out.push_str(&number.to_string()),
        Value::String(text) => write_string(text, out),
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                separate(index, indent + 2, out);
                write(item, indent + 2, out);
            }
            close(indent, out);
            out.push(']');
        }
        Value::Object(entries) if entries.is_empty() => out.push_str("{}"),
        Value::Object(entries) => {
            out.push('{');
            for (index, (key, item)) in entries.iter().enumerate() {
                separate(index, indent + 2, out);
                write_string(key, out);
                out.push_str(": ");
                write(item, indent + 2, out);
            }
            close(indent, out);
            out.push('}');
        }
    }
}

fn separate(index: usize, indent: usize, out: &mut String) {
    if index > 0 {
        out.push(',');
    }
    out.push('\n');
    out.extend(std::iter::repeat_n(' ', indent));
}

fn close(indent: usize, out: &mut String) {
    out.push('\n');
    out.extend(std::iter::repeat_n(' ', indent));
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

    #[test]
    fn strings_escape_quotes_backslashes_and_every_control_character_below_space() {
        let every_low: String = (0u8..0x20).map(char::from).collect();
        let rendered = Value::string(format!("\"\\{every_low}\u{7f}\u{2028}é")).render();
        assert_eq!(
            rendered,
            "\"\\\"\\\\\\u0000\\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\b\\t\\n\\u000b\\f\\r\
             \\u000e\\u000f\\u0010\\u0011\\u0012\\u0013\\u0014\\u0015\\u0016\\u0017\\u0018\\u0019\
             \\u001a\\u001b\\u001c\\u001d\\u001e\\u001f\u{7f}\u{2028}é\"\n"
        );
    }

    #[test]
    fn object_keys_render_in_code_point_order_whatever_order_they_are_given_in() {
        let forward = Value::object([
            ("a", Value::Null),
            ("B", Value::Integer(1)),
            ("é", Value::Array(Vec::new())),
        ]);
        let backward = Value::object([
            ("é", Value::Array(Vec::new())),
            ("B", Value::Integer(1)),
            ("a", Value::Null),
        ]);
        assert_eq!(forward.render(), backward.render());
        assert_eq!(
            forward.render(),
            "{\n  \"B\": 1,\n  \"a\": null,\n  \"é\": []\n}\n"
        );
    }
}

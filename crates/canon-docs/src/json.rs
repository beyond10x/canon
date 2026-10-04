//! A small JSON value with one pretty serialization, for the generated JSON Schema.

/// A JSON value. Object members keep the order they are written in.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Bool(bool),
    Int(u64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

pub fn str(text: impl Into<String>) -> Json {
    Json::Str(text.into())
}

pub fn obj<const N: usize>(members: [(&str, Json); N]) -> Json {
    Json::Obj(
        members
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}

impl Json {
    /// Two-space indented, one trailing newline.
    pub fn pretty(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0);
        out.push('\n');
        out
    }

    fn write(&self, out: &mut String, depth: usize) {
        let indent = |out: &mut String, depth: usize| out.push_str(&"  ".repeat(depth));
        match self {
            Json::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
            Json::Int(value) => out.push_str(&value.to_string()),
            Json::Str(text) => escape(text, out),
            Json::Arr(items) if items.is_empty() => out.push_str("[]"),
            Json::Arr(items) => {
                out.push_str("[\n");
                for (index, item) in items.iter().enumerate() {
                    indent(out, depth + 1);
                    item.write(out, depth + 1);
                    out.push_str(if index + 1 < items.len() { ",\n" } else { "\n" });
                }
                indent(out, depth);
                out.push(']');
            }
            Json::Obj(members) if members.is_empty() => out.push_str("{}"),
            Json::Obj(members) => {
                out.push_str("{\n");
                for (index, (key, value)) in members.iter().enumerate() {
                    indent(out, depth + 1);
                    escape(key, out);
                    out.push_str(": ");
                    value.write(out, depth + 1);
                    out.push_str(if index + 1 < members.len() {
                        ",\n"
                    } else {
                        "\n"
                    });
                }
                indent(out, depth);
                out.push('}');
            }
        }
    }
}

fn escape(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_nested_values_with_escapes() {
        let value = obj([
            ("a", Json::Arr(vec![Json::Int(1), Json::Bool(true)])),
            ("b", str("x\"\\\n")),
            ("c", Json::Obj(Vec::new())),
        ]);
        assert_eq!(
            value.pretty(),
            "{\n  \"a\": [\n    1,\n    true\n  ],\n  \"b\": \"x\\\"\\\\\\n\",\n  \"c\": {}\n}\n"
        );
    }
}

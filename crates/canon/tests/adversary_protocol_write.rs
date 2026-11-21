//! Adversary pass 1 on story:protocol-writer: cases against `b10x_canon::model::write`.
//!
//! The first group holds protocols `parse` gives whose written text `parse` refuses: a mapping key
//! the source spells within YAML's simple key limit, or spells as an explicit `?` key, and whose
//! JSON spelling exceeds it. The limit counts bytes of the written text, quotes and escapes
//! included, and a quoted key leaves 1022 of them. Decided in review, D1: the round trip holds up
//! to that limit, and for a longer key `write` returns JSON that `serde_json` reads with the key
//! intact, which `parse` refuses. Each case pins that.
//!
//! The second group holds a model built in Rust nested deeper than `parse` reads. Decided in
//! review, D2: `write` recurses once per level and returns for every model whose nesting the
//! calling thread's stack holds; the case writes a predicate 500 deep on a 2 MiB stack.
//!
//! The third group pins what the writer documents and no other case observes, so a mutant of the
//! writer that changes it turns a case red: `Some` of empty arguments, the order of keys inside
//! `arguments`, the indent of every nesting level, and an empty description or result.

use b10x_canon::model::{self, Claim, ClaimId, Declarations, Predicate, Protocol, write};

const HEADER: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n";

/// `parse` reads `source`, and reads the text `write` gives for that protocol back to it.
fn reads_back(source: &str) -> Protocol {
    let protocol =
        model::parse(source).unwrap_or_else(|error| panic!("parse refuses the source: {error}"));
    let text = write(&protocol);
    match model::parse(&text) {
        Ok(read) => assert!(read == protocol, "the written text reads back unequal"),
        Err(error) => {
            panic!("parse gives this protocol and refuses the text write gives for it: {error}")
        }
    }
    protocol
}

/// `parse` gives a protocol from `source`; the text `write` gives for it is JSON that `serde_json`
/// reads, which holds `key` as a key of `section`, and `parse` refuses that text.
fn written_text_is_json_with_the_key_and_parse_refuses_it(source: &str, section: &str, key: &str) {
    let protocol =
        model::parse(source).unwrap_or_else(|error| panic!("parse refuses the source: {error}"));
    let text = write(&protocol);
    let value: serde_json::Value =
        serde_json::from_str(&text).expect("serde_json reads the written text");
    assert!(
        value[section]
            .as_object()
            .is_some_and(|section| section.contains_key(key)),
        "the key is intact in the written JSON"
    );
    assert!(text.ends_with("}\n"));
    assert!(
        model::parse(&text).is_err(),
        "parse refuses the text for a key this long"
    );
}

// ---------------------------------------------------------------------------------------------
// Keys `parse` gives that the written text cannot hold.

// Decided in review, D1
#[test]
fn a_plain_key_of_1023_bytes_is_given_by_parse_and_its_written_text_is_refused() {
    let id = "k".repeat(1023);
    written_text_is_json_with_the_key_and_parse_refuses_it(
        &format!("{HEADER}claims:\n  {id}: {{true_when: {{all: []}}}}\n"),
        "claims",
        &id,
    );
}

// Decided in review, D1
#[test]
fn a_plain_key_of_341_characters_is_given_by_parse_and_its_written_text_is_refused() {
    // 341 characters of three bytes each: 1023 bytes, far below 1024 characters; the limit is in
    // bytes.
    let id = "\u{65e5}".repeat(341);
    written_text_is_json_with_the_key_and_parse_refuses_it(
        &format!("{HEADER}claims:\n  {id}: {{true_when: {{all: []}}}}\n"),
        "claims",
        &id,
    );
}

// Decided in review, D1
#[test]
fn an_explicit_key_of_2000_bytes_is_given_by_parse_and_its_written_text_is_refused() {
    let id = "k".repeat(2000);
    written_text_is_json_with_the_key_and_parse_refuses_it(
        &format!("{HEADER}claims:\n  ? {id}\n  : {{true_when: {{all: []}}}}\n"),
        "claims",
        &id,
    );
}

// Decided in review, D1
#[test]
fn an_explicit_long_property_name_inside_arguments_is_given_by_parse_and_its_text_is_refused() {
    let name = "x".repeat(1100);
    let source = format!(
        "{HEADER}actions:\n  a:\n    arguments:\n      type: object\n      properties:\n        ? {name}\n        : {{type: string}}\n"
    );
    let protocol = model::parse(&source).expect("parse gives the protocol");
    let text = write(&protocol);
    let value: serde_json::Value =
        serde_json::from_str(&text).expect("serde_json reads the written text");
    assert!(
        value["actions"]["a"]["arguments"]["properties"][&name].is_object(),
        "the property name is intact in the written JSON"
    );
    assert!(model::parse(&text).is_err());
}

// Decided in review, D1
#[test]
fn a_plain_key_of_600_backslashes_is_given_by_parse_and_its_written_text_is_refused() {
    // 600 bytes in the source; each backslash is written `\\`, so the written key is 1200 bytes.
    let id = "\\".repeat(600);
    written_text_is_json_with_the_key_and_parse_refuses_it(
        &format!("{HEADER}claims:\n  {id}: {{true_when: {{all: []}}}}\n"),
        "claims",
        &id,
    );
}

// ---------------------------------------------------------------------------------------------
// A model built in Rust, nested deeper than `parse` reads.

// Decided in review, D2 and D4
#[test]
fn a_predicate_nested_500_deep_is_written_on_a_thread_with_a_2_mib_stack() {
    let written = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut predicate = Predicate::All(Vec::new());
            for _ in 0..500 {
                predicate = Predicate::Not(Box::new(predicate));
            }
            let mut protocol = model::parse(HEADER).expect("the header parses");
            protocol.claims = Declarations::new(vec![(
                ClaimId::new("c"),
                Claim {
                    description: None,
                    true_when: predicate,
                },
            )]);
            let text = write(&protocol);
            // The model is not dropped here: the case is about `write`, not about `Drop`.
            std::mem::forget(protocol);
            text
        })
        .expect("the thread starts")
        .join()
        .expect("write returns on the 2 MiB stack");
    assert!(written.ends_with("}\n"));
    assert_eq!(written.matches("\"not\"").count(), 500);
    // Each `not` is one object deeper: the innermost is 500 objects below the claim's.
    let deepest = written
        .lines()
        .map(|line| line.len() - line.trim_start().len())
        .max();
    assert!(deepest.is_some_and(|indent| indent >= 2 * 500));
    assert!(
        model::parse(&written).is_err(),
        "parse reads at most 123 levels"
    );
}

// ---------------------------------------------------------------------------------------------
// What the writer documents and no other case observes.

#[test]
fn empty_arguments_are_written_and_read_back() {
    let protocol = reads_back(&format!(
        "{HEADER}actions: {{a: {{arguments: {{}}}}, b: {{arguments: []}}}}\n"
    ));
    let text = write(&protocol);
    assert!(text.contains("\"arguments\": {}"), "{text}");
    assert!(text.contains("\"arguments\": []"), "{text}");
}

#[test]
fn keys_inside_arguments_are_written_sorted() {
    let protocol = reads_back(&format!(
        "{HEADER}actions: {{a: {{arguments: {{zeta: 1, alpha: {{mid: 2, beta: 3}}}}}}}}\n"
    ));
    let text = write(&protocol);
    let at = |key: &str| {
        text.find(&format!("\"{key}\""))
            .unwrap_or_else(|| panic!("{key} is written:\n{text}"))
    };
    assert!(at("alpha") < at("zeta"), "{text}");
    assert!(at("beta") < at("mid"), "{text}");
}

/// Every line is indented two spaces for each object or list open at its start, and a line that
/// closes one is indented as the line that opened it.
fn assert_indent_follows_nesting(text: &str) {
    let mut depth: usize = 0;
    for (number, line) in text.lines().enumerate() {
        let body = line.trim_start_matches(' ');
        let indent = line.len() - body.len();
        let level = if body.starts_with('}') || body.starts_with(']') {
            depth.checked_sub(1).expect("a close has an open")
        } else {
            depth
        };
        assert_eq!(indent, 2 * level, "line {}: `{line}`\n{text}", number + 1);
        let (mut in_string, mut escaped) = (false, false);
        for c in body.chars() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                }
                continue;
            }
            match c {
                '"' => in_string = true,
                '{' | '[' => depth += 1,
                '}' | ']' => depth -= 1,
                _ => {}
            }
        }
    }
    assert_eq!(depth, 0, "{text}");
}

#[test]
fn every_line_is_indented_two_spaces_for_each_level_of_nesting() {
    let protocol = reads_back(
        "format: protocol/1\n\
        protocol: {id: p, revision: 3, description: d}\n\
        artifacts: {art: {description: d}}\n\
        evidence_kinds: {ev: {description: d, max_age: 5m, supplied_by: other}}\n\
        claims:\n\
        \x20 c: {true_when: {evidence: {kind: ev, result: r, subject: art, independent_on: [dim, dim2]}}}\n\
        \x20 f: {true_when: {first: {kind: ev, result: r, subject: art}}}\n\
        \x20 b: {true_when: {before: {first: ev, then: ev, subject: art}}}\n\
        \x20 af: {true_when: {after: {first: ev, then: {kind: ev, result: r}, subject: art}}}\n\
        \x20 s: {true_when: {since: {last: ev, then: {kind: ev, result: r}, subject: art}}}\n\
        \x20 ch: {true_when: {children: {in: [x, y], revision: current, subject: art}}}\n\
        \x20 i: {true_when: {input: i, in: [a, b]}}\n\
        \x20 n: {true_when: {not: {any: [{all: [{claim: c}, {claim: f, is: unknown}]}, {all: []}]}}}\n\
        obligations: {ob: {discharged_when: {claim: c}, applicable_when: {input: i, in: [a]}}}\n\
        actions: {a: {precondition: {claim: c}, requires: [{capability: cap, scope: {k: v, l: w}}], effect: e, may_produce: [{evidence: ev, decided_by: program}], arguments: {type: object, properties: {x: {enum: [1, [2, {}], [], {}, null]}}}, case_open: {protocol: q, subject: art}, author: judgement, revises: [art, art2]}}\n\
        outcomes: {o: {requires: {claim: c}}, held: {requires: {decision: hold}}}\n\
        invalidation: {r: {upstream: art, invalidates: [c, f]}}\n\
        case: {inputs: {i: {values: [a, b]}}}\n\
        independence_dimensions: {dim: {}, dim2: {description: d}}\n\
        imports: {q: {revision: 2, digest: 'sha256:00', refines: [x, y]}}\n",
    );
    assert_indent_follows_nesting(&write(&protocol));
}

#[test]
fn an_empty_description_and_an_empty_result_are_written_and_read_back() {
    let protocol = reads_back(
        "format: protocol/1\n\
        protocol: {id: p, revision: 1, description: ''}\n\
        artifacts: {art: {description: ''}}\n\
        evidence_kinds: {ev: {description: ''}}\n\
        claims: {c: {description: '', true_when: {evidence: {kind: ev, result: ''}}}}\n\
        obligations: {ob: {description: '', discharged_when: {claim: c}}}\n\
        actions: {a: {description: ''}}\n\
        outcomes: {o: {description: '', requires: {claim: c}}}\n\
        invalidation: {r: {description: '', upstream: art, invalidates: [c]}}\n\
        case: {inputs: {i: {description: '', values: [a]}}}\n\
        independence_dimensions: {dim: {description: ''}}\n",
    );
    let text = write(&protocol);
    assert_eq!(text.matches("\"description\": \"\"").count(), 10, "{text}");
    assert_eq!(text.matches("\"result\": \"\"").count(), 1, "{text}");
}

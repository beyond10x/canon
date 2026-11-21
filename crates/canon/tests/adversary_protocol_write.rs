//! Adversary pass 1 on story:protocol-writer: cases against `b10x_canon::model::write`.
//!
//! The first group holds protocols `parse` gives whose written text `parse` refuses: a mapping key
//! the source spells within YAML's simple key limit, or spells as an explicit `?` key, and whose
//! JSON spelling exceeds it. The limit counts bytes of the written text, quotes and escapes
//! included, and a quoted key leaves 1022 of them.
//!
//! The second group holds a model built in Rust that the module documentation says is still
//! written: a predicate nested deeper than `parse` reads.
//!
//! The third group pins what the writer documents and no other case observes, so a mutant of the
//! writer that changes it turns a case red: `Some` of empty arguments, the order of keys inside
//! `arguments`, the indent of every nesting level, and an empty description or result.

use std::process::Command;

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

// ---------------------------------------------------------------------------------------------
// Keys `parse` gives that the written text cannot hold.

#[test]
fn a_plain_key_of_1023_bytes_parse_reads_reads_back() {
    let id = "k".repeat(1023);
    reads_back(&format!(
        "{HEADER}claims:\n  {id}: {{true_when: {{all: []}}}}\n"
    ));
}

#[test]
fn a_plain_key_of_341_characters_parse_reads_reads_back() {
    // 341 characters of three bytes each: 1023 bytes, far below the 1024 characters the module
    // documentation names as the limit.
    let id = "\u{65e5}".repeat(341);
    reads_back(&format!(
        "{HEADER}claims:\n  {id}: {{true_when: {{all: []}}}}\n"
    ));
}

#[test]
fn an_explicit_key_of_2000_bytes_parse_reads_reads_back() {
    let id = "k".repeat(2000);
    reads_back(&format!(
        "{HEADER}claims:\n  ? {id}\n  : {{true_when: {{all: []}}}}\n"
    ));
}

#[test]
fn an_explicit_long_property_name_inside_arguments_reads_back() {
    let name = "x".repeat(1100);
    reads_back(&format!(
        "{HEADER}actions:\n  a:\n    arguments:\n      type: object\n      properties:\n        ? {name}\n        : {{type: string}}\n"
    ));
}

#[test]
fn a_plain_key_of_600_backslashes_parse_reads_reads_back() {
    // 600 bytes in the source; each backslash is written `\\`, so the written key is 1202 bytes.
    let id = "\\".repeat(600);
    reads_back(&format!(
        "{HEADER}claims:\n  {id}: {{true_when: {{all: []}}}}\n"
    ));
}

// ---------------------------------------------------------------------------------------------
// A model built in Rust: `write` is documented total, and a value nested deeper than `parse`
// reads "is still written".

const DEEP_CHILD: &str = "ADVERSARY_PROTOCOL_WRITE_DEEP_CHILD";
const DEEP_TEST: &str =
    "a_predicate_nested_5000_deep_is_written_on_a_thread_with_the_default_stack";

#[test]
fn a_predicate_nested_5000_deep_is_written_on_a_thread_with_the_default_stack() {
    if std::env::var_os(DEEP_CHILD).is_some() {
        let written = std::thread::spawn(|| {
            let mut predicate = Predicate::All(Vec::new());
            for _ in 0..5000 {
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
            text.matches("\"not\"").count()
        })
        .join()
        .expect("the writing thread returns");
        assert_eq!(written, 5000);
        return;
    }
    // A stack overflow aborts the process, so the write runs in a child test process.
    let output = Command::new(std::env::current_exe().expect("the test binary"))
        .args(["--exact", DEEP_TEST, "--test-threads=1", "--nocapture"])
        .env(DEEP_CHILD, "1")
        .env_remove("RUST_MIN_STACK")
        .output()
        .expect("the child test process runs");
    assert!(
        output.status.success(),
        "write does not return for a predicate nested 5000 deep: {}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
            .lines()
            .rfind(|line| !line.trim().is_empty())
            .unwrap_or("")
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

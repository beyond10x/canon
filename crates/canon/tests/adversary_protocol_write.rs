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
//! review, D2: `write` recurses per level and returns for every model whose nesting the
//! calling thread's stack holds; the case writes a predicate 400 deep on a 2 MiB stack.
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

// Decided in review, D2 and D5
#[test]
fn a_predicate_nested_400_deep_is_written_on_a_thread_with_a_2_mib_stack() {
    let written = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut predicate = Predicate::All(Vec::new());
            for _ in 0..400 {
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
    assert_eq!(written.matches("\"not\"").count(), 400);
    // Each `not` is one object deeper: the innermost is 400 objects below the claim's.
    let deepest = written
        .lines()
        .map(|line| line.len() - line.trim_start().len())
        .max();
    assert!(deepest.is_some_and(|indent| indent >= 2 * 400));
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

// ---------------------------------------------------------------------------------------------
// Adversary pass 2: the module docs' "Keys" and "Depth" sections, read as the specification.

/// A protocol whose only claim is `c` and holds `true_when`.
fn with_claim(id: &str, true_when: Predicate) -> Protocol {
    let mut protocol = model::parse(HEADER).expect("the header parses");
    protocol.claims = Declarations::new(vec![(
        ClaimId::new(id),
        Claim {
            description: None,
            true_when,
        },
    )]);
    protocol
}

/// "Keys": a `\`, a `"`, a line feed, carriage return, tab, backspace or form feed take two bytes,
/// every other escaped character six, any other character its length in UTF-8, and a key of at
/// most 1022 bytes as written reads back. Each form, grown to 1022 bytes, reads back spelled as
/// the docs say ("Spelling"); grown to 1023 it does not.
#[test]
fn a_key_grown_by_each_escape_form_reads_back_at_1022_bytes_as_written_and_not_at_1023() {
    let forms: [(char, &str); 12] = [
        ('"', "\\\""),
        ('\\', "\\\\"),
        ('\n', "\\n"),
        ('\r', "\\r"),
        ('\t', "\\t"),
        ('\u{8}', "\\b"),
        ('\u{c}', "\\f"),
        ('\u{1}', "\\u0001"),
        ('\u{85}', "\\u0085"),
        ('\u{2028}', "\\u2028"),
        ('\u{feff}', "\\uFEFF"),
        ('\u{1f600}', "\u{1f600}"),
    ];
    for (c, spelling) in forms {
        for (n, reads) in [(1022, true), (1023, false)] {
            let units = n / spelling.len();
            let pad = "k".repeat(n - units * spelling.len());
            let key = format!("{}{pad}", c.to_string().repeat(units));
            let written = format!("\"{}{pad}\": ", spelling.repeat(units));
            let protocol = with_claim(&key, Predicate::All(Vec::new()));
            let text = write(&protocol);
            assert!(
                text.contains(&written),
                "U+{:04X} is written {spelling}",
                u32::from(c)
            );
            assert_eq!(
                model::parse(&text).is_ok_and(|read| read == protocol),
                reads,
                "U+{:04X}: a key of {n} bytes as written",
                u32::from(c)
            );
        }
    }
}

/// "Keys": the limit holds for an identifier and for a property name inside `arguments`, at every
/// place the model keys by text, however deep the key is indented. Each source spells the key as
/// an explicit `?` key, which `parse` reads at any length.
#[test]
fn a_key_reads_back_at_1022_bytes_and_not_at_1023_in_every_map_the_model_keys_by_text() {
    let nested = |k: &str| {
        let mut value = format!("{{? {k} : 1}}");
        for _ in 0..25 {
            value = format!("{{n: {value}}}");
        }
        format!("actions:\n  a:\n    arguments: {value}\n")
    };
    type Place<'a> = (&'a str, &'a dyn Fn(&str) -> String);
    let places: [Place<'_>; 13] = [
        ("artifacts", &|k| format!("artifacts:\n  ? {k}\n  : {{}}\n")),
        ("evidence_kinds", &|k| {
            format!("evidence_kinds:\n  ? {k}\n  : {{}}\n")
        }),
        ("claims", &|k| {
            format!("claims:\n  ? {k}\n  : {{true_when: {{all: []}}}}\n")
        }),
        ("obligations", &|k| {
            format!("obligations:\n  ? {k}\n  : {{discharged_when: {{all: []}}}}\n")
        }),
        ("actions", &|k| format!("actions:\n  ? {k}\n  : {{}}\n")),
        ("outcomes", &|k| {
            format!("outcomes:\n  ? {k}\n  : {{requires: {{all: []}}}}\n")
        }),
        ("invalidation", &|k| {
            format!("invalidation:\n  ? {k}\n  : {{upstream: a, invalidates: []}}\n")
        }),
        ("independence_dimensions", &|k| {
            format!("independence_dimensions:\n  ? {k}\n  : {{}}\n")
        }),
        ("imports", &|k| {
            format!("imports:\n  ? {k}\n  : {{revision: 1, digest: d}}\n")
        }),
        ("case inputs", &|k| {
            format!("case:\n  inputs:\n    ? {k}\n    : {{values: [a]}}\n")
        }),
        ("requires scope", &|k| {
            format!(
                "actions:\n  a:\n    requires:\n      - capability: c\n        scope:\n          ? {k}\n          : v\n"
            )
        }),
        ("arguments properties", &|k| {
            format!(
                "actions:\n  a:\n    arguments:\n      type: object\n      properties:\n        ? {k}\n        : {{type: string}}\n"
            )
        }),
        ("arguments 25 objects deep", &nested),
    ];
    for (place, body) in places {
        for (n, reads) in [(1022, true), (1023, false)] {
            let key = "k".repeat(n);
            let protocol = model::parse(&format!("{HEADER}{}", body(&key)))
                .unwrap_or_else(|error| panic!("{place}: parse refuses the source: {error}"));
            let text = write(&protocol);
            assert!(text.contains(&format!("\"{key}\": ")), "{place}: {n}");
            assert_eq!(
                model::parse(&text).is_ok_and(|read| read == protocol),
                reads,
                "{place}: a key of {n} bytes as written"
            );
        }
    }
}

/// "Depth": a predicate 400 deep is written on a 2 MiB stack in a debug build. The case above
/// nests `not`; `all` and `any` take more stack per level (`predicate`, then `predicates`), so
/// they are the shape that overflows first. Measured on 2026-11-21, debug build, rustc 1.99.1:
/// `all` holds 808 levels and overflows at 809, `not` holds 1152.
// Decided in review, D2 and D5
#[test]
fn a_predicate_of_all_nested_400_deep_is_written_on_a_thread_with_a_2_mib_stack() {
    let written = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut predicate = Predicate::All(Vec::new());
            for _ in 0..400 {
                predicate = Predicate::All(vec![predicate]);
            }
            let protocol = with_claim("c", predicate);
            let text = write(&protocol);
            // The model is not dropped here: the case is about `write`, not about `Drop`.
            std::mem::forget(protocol);
            text
        })
        .expect("the thread starts")
        .join()
        .expect("write returns on the 2 MiB stack");
    assert!(written.ends_with("}\n"));
    assert_eq!(written.matches("\"all\"").count(), 401);
}

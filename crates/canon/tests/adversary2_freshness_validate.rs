//! Adversary pass 2 for story:evidence-freshness: the `invalid-max-age` problem against its own
//! documentation.
//!
//! `validate::Problem::InvalidMaxAge` is documented (`validate/mod.rs`) as covering two cases: a
//! `max_age` that "is not a whole number without leading zeros, followed by `s`, `m`, `h` or `d`,
//! or is too long to count in seconds", and the generated protocol reference says "Canon also
//! refuses an age too long to count in seconds". The problem's message is what an author reads.

use b10x_canon::{model, validate};

fn problems(max_age: &str) -> Vec<(String, String)> {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
         evidence_kinds: {{k: {{max_age: {max_age}}}}}\n"
    );
    let protocol = model::parse(&source).expect("protocol parses");
    match validate::validate(&protocol) {
        Ok(()) => Vec::new(),
        Err(problems) => problems
            .iter()
            .map(|problem| (problem.code().to_owned(), problem.to_string()))
            .collect(),
    }
}

/// `99999999999999999999s` IS a whole number without leading zeros followed by `s`; it is refused
/// only because it is too long to count in seconds. The message must not tell the author the
/// opposite of what is wrong, and must name the reason the docs give.
#[test]
fn an_age_too_long_to_count_is_refused_for_being_too_long_not_for_its_form() {
    let found = problems("99999999999999999999s");
    assert_eq!(found.len(), 1, "{found:?}");
    let (code, message) = &found[0];
    assert_eq!(code, "invalid-max-age");
    assert!(
        message.contains("too long"),
        "the message names the actual reason (too long to count in seconds): {message}"
    );
}

/// The well-formed half, for contrast: a malformed age keeps the form message. Green today.
#[test]
fn a_malformed_age_is_refused_for_its_form() {
    assert_eq!(
        problems("5 min"),
        [(
            "invalid-max-age".to_owned(),
            "evidence kind `k` has max_age `5 min`, which is not a whole number without leading \
             zeros, followed by s, m, h or d"
                .to_owned()
        )]
    );
}

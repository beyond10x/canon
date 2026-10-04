//! The `protocol/1` source model and its validator, through the public library API.

use b10x_canon::model::{
    self, ActionId, ClaimId, ClaimTest, EvidenceKindId, EvidenceMatch, OutcomeId, Predicate, Truth,
};
use b10x_canon::validate::{Problem, Referrer, Section, validate};

const BASE: &str = include_str!("../../../fixtures/investigation/protocol.yaml");

fn problems(source: &str) -> Vec<Problem> {
    let protocol = model::parse(source).expect("parses");
    match validate(&protocol) {
        Ok(()) => Vec::new(),
        Err(problems) => problems,
    }
}

#[test]
fn base_fixture_parses_into_the_model() {
    let protocol = model::parse(BASE).expect("base fixture parses");
    assert_eq!(protocol.format, "protocol/1");
    assert_eq!(protocol.protocol.id.as_str(), "investigation");
    assert_eq!(protocol.protocol.revision, 1);
    assert_eq!(
        protocol
            .artifacts
            .ids()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["explanation"]
    );
    assert_eq!(
        protocol
            .evidence_kinds
            .ids()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["supporting_observation", "falsification_attempt"]
    );
    assert_eq!(
        protocol
            .actions
            .ids()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["inspect", "attempt_falsification"]
    );
    assert!(protocol.obligations.is_empty());
    let claim = protocol
        .claims
        .get(&ClaimId::from("explanation.supported"))
        .expect("claim declared");
    assert_eq!(
        claim.true_when,
        Predicate::All(vec![
            Predicate::Evidence(EvidenceMatch {
                kind: EvidenceKindId::from("supporting_observation"),
                result: None,
            }),
            Predicate::Evidence(EvidenceMatch {
                kind: EvidenceKindId::from("falsification_attempt"),
                result: Some("survived".to_owned()),
            }),
        ])
    );
    let outcome = protocol
        .outcomes
        .get(&OutcomeId::from("supported"))
        .expect("outcome declared");
    assert_eq!(
        outcome.requires,
        Predicate::Claim(ClaimTest {
            claim: ClaimId::from("explanation.supported"),
            is: Truth::True,
        })
    );
}

#[test]
fn base_fixture_is_valid() {
    assert_eq!(problems(BASE), Vec::<Problem>::new());
}

#[test]
fn action_fields_parse() {
    let source = r#"
format: protocol/1
protocol: {id: p, revision: 2}
evidence_kinds: {note: {}}
claims:
  ready: {true_when: {evidence: {kind: note}}}
actions:
  publish:
    description: Publish.
    precondition: {not: {claim: ready, is: unknown}}
    requires:
      - capability: finding.publish
    effect: write
    may_produce:
      - evidence: note
obligations:
  review: {description: Someone reviews it., discharged_when: {claim: ready}}
"#;
    let protocol = model::parse(source).expect("parses");
    let action = protocol
        .actions
        .get(&ActionId::from("publish"))
        .expect("declared");
    assert_eq!(
        action.precondition,
        Some(Predicate::Not(Box::new(Predicate::Claim(ClaimTest {
            claim: ClaimId::from("ready"),
            is: Truth::Unknown,
        }))))
    );
    assert_eq!(action.requires[0].capability.as_str(), "finding.publish");
    assert_eq!(action.effect.as_ref().map(|e| e.as_str()), Some("write"));
    assert_eq!(action.may_produce[0].evidence.as_str(), "note");
    assert_eq!(protocol.obligations.len(), 1);
    assert_eq!(problems(source), Vec::<Problem>::new());
}

#[test]
fn claim_test_values() {
    for (written, expected) in [
        ("true", Truth::True),
        ("false", Truth::False),
        ("unknown", Truth::Unknown),
    ] {
        let source = format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nclaims:\n  a: {{true_when: {{any: []}}}}\noutcomes:\n  o: {{requires: {{claim: a, is: {written}}}}}\n"
        );
        let protocol = model::parse(&source).expect("parses");
        let outcome = protocol
            .outcomes
            .get(&OutcomeId::from("o"))
            .expect("declared");
        assert_eq!(
            outcome.requires,
            Predicate::Claim(ClaimTest {
                claim: ClaimId::from("a"),
                is: expected,
            })
        );
    }
}

#[test]
fn malformed_predicates_are_parse_errors() {
    for predicate in [
        "{claim: a, is: maybe}",
        "{claim: a, evidence: {kind: k}}",
        "{evidence: {kind: k}, is: true}",
        "{}",
        "{sometimes: a}",
        "{evidence: {kind: k, freshness: 3}}",
    ] {
        let source = format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nclaims:\n  a: {{true_when: {predicate}}}\n"
        );
        assert!(model::parse(&source).is_err(), "{predicate}");
    }
}

#[test]
fn undeclared_fields_are_parse_errors() {
    for source in [
        "format: protocol/1\nprotocol: {id: p, revision: 1}\ncase: {inputs: {}}\n",
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nobligations:\n  o: {discharged_when: {all: []}, due: soon}\n",
        "format: protocol/1\nprotocol: {id: p, revision: 1, owner: x}\n",
    ] {
        assert!(model::parse(source).is_err(), "{source}");
    }
}

#[test]
fn parse_errors_are_deterministic() {
    let source = "format: protocol/1\nprotocol: {id: p}\n";
    let first = model::parse(source)
        .expect_err("revision missing")
        .to_string();
    for _ in 0..4 {
        assert_eq!(
            model::parse(source)
                .expect_err("revision missing")
                .to_string(),
            first
        );
    }
    assert!(first.contains("revision"), "{first}");
}

#[test]
fn unsupported_format_is_rejected() {
    assert_eq!(
        problems("format: protocol/2\nprotocol: {id: p, revision: 1}\n"),
        [Problem::UnsupportedFormat {
            found: "protocol/2".to_owned()
        }]
    );
}

#[test]
fn duplicates_are_reported_in_every_section() {
    let source = r#"
format: protocol/1
protocol: {id: p, revision: 1}
artifacts: {x: {}, x: {}}
evidence_kinds: {k: {}, k: {}}
claims:
  c: {true_when: {evidence: {kind: k}}}
  c: {true_when: {evidence: {kind: k}}}
obligations: {o: {discharged_when: {claim: c}}, o: {discharged_when: {claim: c}}}
actions: {a: {}, a: {}}
outcomes:
  w: {requires: {claim: c}}
  w: {requires: {claim: c}}
"#;
    let duplicate = |section, id: &str| Problem::DuplicateIdentifier {
        section,
        id: id.to_owned(),
    };
    assert_eq!(
        problems(source),
        [
            duplicate(Section::Artifact, "x"),
            duplicate(Section::EvidenceKind, "k"),
            duplicate(Section::Claim, "c"),
            duplicate(Section::Obligation, "o"),
            duplicate(Section::Action, "a"),
            duplicate(Section::Outcome, "w"),
        ]
    );
}

#[test]
fn every_reference_is_resolved() {
    let source = r#"
format: protocol/1
protocol: {id: p, revision: 1}
evidence_kinds: {k: {}}
claims:
  c:
    true_when:
      all:
        - evidence: {kind: missing_kind}
        - not: {claim: missing_from_claim}
actions:
  act:
    precondition: {any: [{claim: missing_from_action}]}
    may_produce:
      - evidence: k
      - evidence: produced_kind
outcomes:
  w: {requires: {claim: missing_from_outcome}}
"#;
    assert_eq!(
        problems(source),
        [
            Problem::UndeclaredEvidenceKind {
                referrer: Referrer::Claim(ClaimId::from("c")),
                kind: EvidenceKindId::from("missing_kind"),
            },
            Problem::UndeclaredClaim {
                referrer: Referrer::Claim(ClaimId::from("c")),
                claim: ClaimId::from("missing_from_claim"),
            },
            Problem::UndeclaredClaim {
                referrer: Referrer::Action(ActionId::from("act")),
                claim: ClaimId::from("missing_from_action"),
            },
            Problem::UndeclaredEvidenceKind {
                referrer: Referrer::Action(ActionId::from("act")),
                kind: EvidenceKindId::from("produced_kind"),
            },
            Problem::UndeclaredClaim {
                referrer: Referrer::Outcome(OutcomeId::from("w")),
                claim: ClaimId::from("missing_from_outcome"),
            },
        ]
    );
}

#[test]
fn claims_testing_each_other_in_a_cycle_are_rejected() {
    let source = r#"
format: protocol/1
protocol: {id: p, revision: 1}
claims:
  a: {true_when: {claim: b}}
  b: {true_when: {not: {claim: a}}}
  self: {true_when: {claim: self, is: false}}
  base: {true_when: {all: []}}
  derived: {true_when: {claim: base}}
"#;
    let ids = |names: &[&str]| names.iter().map(|n| ClaimId::from(*n)).collect::<Vec<_>>();
    assert_eq!(
        problems(source),
        [
            Problem::ClaimCycle {
                claims: ids(&["a", "b", "a"])
            },
            Problem::ClaimCycle {
                claims: ids(&["self", "self"])
            },
        ]
    );
}

#[test]
fn problems_come_in_a_stable_order() {
    let source = r#"
format: protocol/0
protocol: {id: p, revision: 1}
claims:
  z: {true_when: {claim: z}}
  y: {true_when: {evidence: {kind: nope}}}
  y: {true_when: {any: []}}
outcomes:
  w: {requires: {claim: gone}}
"#;
    let first = problems(source);
    assert_eq!(
        first.iter().map(Problem::code).collect::<Vec<_>>(),
        [
            "unsupported-format",
            "duplicate-identifier",
            "undeclared-evidence-kind",
            "undeclared-claim",
            "claim-cycle",
        ]
    );
    for _ in 0..4 {
        assert_eq!(problems(source), first);
    }
}

/// Every optional field and every defaulted section, written with no value. Leaving a key out
/// takes its default; writing the key with an explicit null is a parse error.
const NULLABLE: &[(&str, &str)] = &[
    (
        "protocol description",
        "protocol: {id: p, revision: 1, description: NULL}\n",
    ),
    ("artifacts", "artifacts: NULL\n"),
    (
        "artifact description",
        "artifacts: {a: {description: NULL}}\n",
    ),
    ("evidence_kinds", "evidence_kinds: NULL\n"),
    (
        "evidence kind description",
        "evidence_kinds: {k: {description: NULL}}\n",
    ),
    ("claims", "claims: NULL\n"),
    (
        "claim description",
        "claims: {c: {description: NULL, true_when: {all: []}}}\n",
    ),
    (
        "evidence match result",
        "evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k, result: NULL}}}}\n",
    ),
    (
        "claim test is",
        "claims: {c: {true_when: {all: []}}, d: {true_when: {claim: c, is: NULL}}}\n",
    ),
    ("obligations", "obligations: NULL\n"),
    (
        "obligation description",
        "obligations: {o: {description: NULL, discharged_when: {all: []}}}\n",
    ),
    ("actions", "actions: NULL\n"),
    ("action description", "actions: {a: {description: NULL}}\n"),
    (
        "action precondition",
        "actions: {a: {precondition: NULL}}\n",
    ),
    ("action requires", "actions: {a: {requires: NULL}}\n"),
    ("action effect", "actions: {a: {effect: NULL}}\n"),
    ("action may_produce", "actions: {a: {may_produce: NULL}}\n"),
    ("outcomes", "outcomes: NULL\n"),
    (
        "outcome description",
        "claims: {c: {true_when: {all: []}}}\noutcomes: {w: {description: NULL, requires: {claim: c}}}\n",
    ),
];

#[test]
fn an_explicit_null_is_a_parse_error_for_every_optional_field() {
    let mut accepted = Vec::new();
    for (field, body) in NULLABLE {
        for null in ["", "~", "null"] {
            let source = format!(
                "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n{}",
                body.replace("NULL", null)
            )
            .replacen("protocol: {id: p, revision: 1}\nprotocol:", "protocol:", 1);
            if model::parse(&source).is_ok() {
                accepted.push(format!("{field} = `{null}`"));
            }
        }
    }
    assert_eq!(accepted, Vec::<String>::new());
}

#[test]
fn omitted_optional_fields_take_their_defaults() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {x: {}}\nevidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\nobligations: {o: {discharged_when: {claim: c}}}\nactions: {a: {}}\noutcomes: {w: {requires: {claim: c}}}\n";
    assert_eq!(problems(source), Vec::<Problem>::new());
    assert_eq!(
        problems("format: protocol/1\nprotocol: {id: p, revision: 1}\n"),
        Vec::<Problem>::new()
    );
    let protocol = model::parse(source).expect("parses");
    let action = protocol
        .actions
        .get(&ActionId::from("a"))
        .expect("declared");
    assert_eq!(
        (
            action.precondition.as_ref(),
            action.requires.len(),
            action.effect.as_ref(),
            action.may_produce.len()
        ),
        (None, 0, None, 0)
    );
    assert_eq!(
        protocol
            .outcomes
            .get(&OutcomeId::from("w"))
            .expect("declared")
            .requires,
        Predicate::Claim(ClaimTest {
            claim: ClaimId::from("c"),
            is: Truth::True
        })
    );
    let claim = protocol.claims.get(&ClaimId::from("c")).expect("declared");
    assert_eq!(
        claim.true_when,
        Predicate::Evidence(EvidenceMatch {
            kind: EvidenceKindId::from("k"),
            result: None
        })
    );
}

#[test]
fn identifiers_must_be_non_empty_and_free_of_whitespace_and_control_characters() {
    let source = r#"
format: protocol/1
protocol: {id: "", revision: 1}
artifacts: {"": {}, "two words": {}}
evidence_kinds: {"tab\there": {}}
claims:
  "line\nbreak": {true_when: {all: []}}
  fine.claim: {true_when: {all: []}}
obligations: {"\u0007bell": {discharged_when: {all: []}}}
actions:
  " lead": {}
  act:
    requires: [{capability: ""}, {capability: "ok.cap"}]
    effect: "read only"
outcomes: {"nbsp here": {requires: {all: []}}}
"#;
    let invalid = |section, id: &str| Problem::InvalidIdentifier {
        section,
        id: id.to_owned(),
    };
    assert_eq!(
        problems(source),
        [
            invalid(Section::Protocol, ""),
            invalid(Section::Artifact, ""),
            invalid(Section::Artifact, "two words"),
            invalid(Section::EvidenceKind, "tab\there"),
            invalid(Section::Claim, "line\nbreak"),
            invalid(Section::Obligation, "\u{7}bell"),
            invalid(Section::Action, " lead"),
            invalid(Section::Outcome, "nbsp\u{a0}here"),
            invalid(Section::Capability, ""),
            invalid(Section::EffectClass, "read only"),
        ]
    );
}

#[test]
fn every_problem_and_parse_error_renders_on_one_line() {
    let forged = "x\nerror[duplicate-identifier]: action `inspect` is declared more than once";
    let source = format!(
        "format: \"protocol/1\\n\"\nprotocol: {{id: p, revision: 1}}\nclaims:\n  {forged:?}: {{true_when: {{claim: {forged:?}}}}}\n  {forged:?}: {{true_when: {{evidence: {{kind: \"k\\r\\n\"}}}}}}\noutcomes:\n  w: {{requires: {{claim: \"gone\\u2028away\"}}}}\n"
    );
    let found = problems(&source);
    let codes: Vec<&str> = found.iter().map(Problem::code).collect();
    assert_eq!(
        codes,
        [
            "unsupported-format",
            "invalid-identifier",
            "invalid-identifier",
            "duplicate-identifier",
            "undeclared-evidence-kind",
            "undeclared-claim",
            "claim-cycle",
        ]
    );
    for problem in &found {
        let line = problem.to_string();
        assert!(
            !line.chars().any(|c| c.is_control() || c == '\u{2028}'),
            "{line:?}"
        );
    }
    assert_eq!(
        found[1].to_string(),
        "claim identifier `x\\nerror[duplicate-identifier]: action `inspect` is declared more than once` is empty or contains whitespace or a control character"
    );

    let unparseable = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\"own\\ner\": 1\n";
    let message = model::parse(unparseable)
        .expect_err("unknown field")
        .to_string();
    assert!(!message.chars().any(char::is_control), "{message:?}");
    assert!(message.contains("own\\ner"), "{message}");
}

/// The class behind the explicit-null cases: no field in the model may take a serde default
/// without going through `present`, which refuses an explicit null. A new optional field added
/// with a bare `#[serde(default)]` fails here, not in a later review.
#[test]
fn every_defaulted_model_field_refuses_an_explicit_null() {
    let model = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/model");
    let mut bare = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&model)
        .expect("model directory")
        .map(|entry| entry.expect("entry").path())
        .collect();
    entries.sort();
    for path in entries {
        let source = std::fs::read_to_string(&path).expect("source file");
        let mut previous = String::new();
        // Whether the struct being read derives `Deserialize`: only a struct serde reads from
        // text can read an explicit null as `None`. The decision is written, never read.
        let (mut derives, mut deserialized) = (false, false);
        for (number, line) in source.lines().enumerate() {
            let item = line.trim_start();
            if item.starts_with("#[derive(") {
                derives = item.contains("Deserialize");
            } else if item.starts_with("pub struct ") || item.starts_with("struct ") {
                deserialized = derives;
                derives = false;
            }
            let bare_default = line.contains("serde(default") && !line.contains("present::");
            if bare_default || (deserialized && is_unguarded_option_field(&previous, line)) {
                bare.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    number + 1,
                    line.trim()
                ));
            }
            previous = line.to_owned();
        }
    }
    assert_eq!(bare, Vec::<String>::new());
}

/// A struct field of type `Option<…>` — serde reads an explicit null in it as `None` even without
/// `#[serde(default)]` — is unguarded unless the attribute line above it names `present::`.
fn is_unguarded_option_field(previous: &str, line: &str) -> bool {
    let field = line.trim_start();
    let field = field.strip_prefix("pub ").unwrap_or(field);
    let Some((name, ty)) = field.split_once(": ") else {
        return false;
    };
    let is_field = !name.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && ty.starts_with("Option<")
        && ty.ends_with(',');
    is_field && !previous.contains("present::")
}

#[test]
fn the_guard_recognizes_an_unguarded_option_field() {
    assert!(is_unguarded_option_field(
        "    #[serde(default)]",
        "    pub description: Option<String>,"
    ));
    assert!(is_unguarded_option_field(
        "    pub id: ProtocolId,",
        "    note: Option<String>,"
    ));
    assert!(!is_unguarded_option_field(
        "    #[serde(default, deserialize_with = \"present::optional\")]",
        "    pub description: Option<String>,"
    ));
    assert!(!is_unguarded_option_field(
        "",
        "    let found: Option<String> = None;"
    ));
}

/// A null where an identifier or other required text is expected is the same omission as a null
/// in an optional field: it is refused at parse, never read as the text `~` or `null`.
#[test]
fn a_null_where_required_text_is_expected_is_a_parse_error() {
    let cases = [
        ("format", "format: NULL\nprotocol: {id: p, revision: 1}\n"),
        (
            "protocol id",
            "format: protocol/1\nprotocol: {id: NULL, revision: 1}\n",
        ),
        (
            "declaration key",
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {NULL: {}}\n",
        ),
        (
            "capability",
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nactions: {a: {requires: [{capability: NULL}]}}\n",
        ),
        (
            "may_produce evidence",
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nactions: {a: {may_produce: [{evidence: NULL}]}}\n",
        ),
        (
            "evidence match kind",
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nclaims: {c: {true_when: {evidence: {kind: NULL}}}}\n",
        ),
    ];
    let mut accepted = Vec::new();
    for (field, template) in cases {
        for null in ["~", "null"] {
            if model::parse(&template.replace("NULL", null)).is_ok() {
                accepted.push(format!("{field} = `{null}`"));
            }
        }
    }
    assert_eq!(accepted, Vec::<String>::new());
    let quoted = model::parse("format: protocol/1\nprotocol: {id: \"null\", revision: 1}\n")
        .expect("a quoted identifier spelled null is text");
    assert_eq!(quoted.protocol.id.as_str(), "null");
}

#[test]
fn a_declaration_written_with_no_value_is_a_parse_error_in_every_section() {
    let mut accepted = Vec::new();
    for section in [
        "artifacts",
        "evidence_kinds",
        "claims",
        "obligations",
        "actions",
        "outcomes",
    ] {
        let source =
            format!("format: protocol/1\nprotocol: {{id: p, revision: 1}}\n{section}:\n  x:\n");
        if model::parse(&source).is_ok() {
            accepted.push(section);
        }
    }
    assert_eq!(accepted, Vec::<&str>::new());
}

#[test]
fn identifiers_with_unicode_format_characters_are_rejected_and_shown_escaped() {
    let source = "format: protocol/1\nprotocol: {id: \"p\\u202Eq\", revision: 1}\nartifacts: {\"soft\\u00ADhyphen\": {}, \"zero\\u200Bwidth\": {}, \"\\uFEFFbom\": {}}\n";
    let found = problems(source);
    let rendered: Vec<String> = found.iter().map(Problem::to_string).collect();
    assert_eq!(
        rendered,
        [
            "protocol identifier `p\\u{202e}q` is empty or contains whitespace or a control character",
            "artifact identifier `soft\\u{ad}hyphen` is empty or contains whitespace or a control character",
            "artifact identifier `zero\\u{200b}width` is empty or contains whitespace or a control character",
            "artifact identifier `\\u{feff}bom` is empty or contains whitespace or a control character",
        ]
    );
}

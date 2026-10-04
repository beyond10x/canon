//! Reading the compiled protocol, and the YAML helpers every input reader shares. Pure: the caller
//! supplies the text. The case snapshot and evidence readers are in `case.rs` and `evidence.rs`.
//!
//! - [`read_ir`] accepts exactly the bytes `canon compile` prints: it parses the text as JSON,
//!   rebuilds the protocol the `canon-ir/1` text describes, compiles it, and refuses the text
//!   unless compiling gives the same bytes back. So an IR that is not canonical, or that describes
//!   a protocol the validator rejects (an undeclared reference, a claim cycle), is refused before
//!   evaluation.
//! - [`read_case`](super::read_case) and [`read_evidence`](super::read_evidence) read one
//!   `canon-case/1` or `canon-evidence/1` document; [`case_from_value`](super::case_from_value)
//!   and [`evidence_from_value`](super::evidence_from_value) read one already parsed as YAML, as
//!   a conformance scenario holds it. The text readers parse the text as YAML and then read the
//!   value exactly as the value readers do, so one document gets one answer on both paths. JSON is
//!   YAML, so either document may be written as JSON. Every identifier, revision and result is a
//!   string: one written as a number or a boolean (`id: 18`, `revision: 1`) is refused as
//!   `malformed-input`.

use serde_json::{Map, Value as Json};
use serde_yaml_ng::Value;

use super::Refusal;
use crate::ir::{self, Ir};
use crate::model::{
    self, Action, Age, Artifact, CapabilityId, CapabilityRequirement, Claim, ClaimId, ClaimTest,
    Declarations, EffectClass, EvidenceKind, EvidenceKindId, EvidenceMatch, EvidenceProduction,
    Obligation, Outcome, Predicate, Protocol, ProtocolHeader, ProtocolId, Truth, one_line,
};

/// A `malformed-input` refusal: `what` is not a document of its format, and why.
pub(super) fn malformed(what: &str, why: impl std::fmt::Display) -> Refusal {
    Refusal::new(
        "malformed-input",
        format!("{what}: {}", one_line(&why.to_string())),
    )
}

/// Text parsed as YAML first, so the text readers and the value readers see the same values: a
/// scalar YAML reads as a number (`18`, `1.0`) or a boolean is one on either path, and an
/// identifier, revision or result written as one is refused.
pub(super) fn yaml(text: &str, what: &str) -> Result<Value, Refusal> {
    serde_yaml_ng::from_str(text).map_err(|error| malformed(what, error))
}

/// Reads `canon-ir/1` text as `canon compile` prints it.
pub fn read_ir(text: &str) -> Result<Ir, Refusal> {
    let not_ir = |why: &str| malformed("not canon-ir/1", why);
    if nesting_depth(text) > MAX_IR_DEPTH {
        return Err(not_ir(&format!(
            "arrays and objects are nested deeper than {MAX_IR_DEPTH}"
        )));
    }
    super::on_deep_stack(|| rebuild(text))
}

/// Parses, rebuilds, compiles and compares; [`read_ir`] runs it on the deep stack.
fn rebuild(text: &str) -> Result<Ir, Refusal> {
    let not_ir = |why: &str| malformed("not canon-ir/1", why);
    let value = json(text).map_err(|_| not_ir("not well-formed JSON"))?;
    let format = field(&value, "format").and_then(Json::as_str);
    if format != Some(ir::FORMAT) {
        return Err(not_ir(&format!("format is not `{}`", ir::FORMAT)));
    }
    let protocol = protocol(&value).map_err(|why| not_ir(&why))?;
    let compiled = ir::compile(&protocol).map_err(|problems| {
        let problems: Vec<String> = problems
            .iter()
            .map(|problem| format!("{}: {problem}", problem.code()))
            .collect();
        not_ir(&format!(
            "describes an invalid protocol: {}",
            problems.join("; ")
        ))
    })?;
    if compiled.canonical_json() != text {
        return Err(not_ir("not the canonical text `canon compile` prints"));
    }
    Ok(compiled)
}

/// The deepest nesting of arrays and objects [`read_ir`] reads. The compiler's own input bounds
/// what it prints far below this; the bound is what keeps every recursive step after parsing
/// (rebuilding, compiling, rendering, evaluating) within the stack those steps run on.
pub const MAX_IR_DEPTH: usize = 4096;

/// The deepest nesting of `[` and `{` in `text`, outside string literals. Counted on the text, so
/// it bounds the parse before the parse recurses.
fn nesting_depth(text: &str) -> usize {
    let (mut depth, mut deepest) = (0usize, 0usize);
    let (mut in_string, mut escaped) = (false, false);
    for byte in text.bytes() {
        if in_string {
            match byte {
                _ if escaped => escaped = false,
                b'\\' => escaped = true,
                b'"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'[' | b'{' => {
                depth += 1;
                deepest = deepest.max(depth);
            }
            b']' | b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    deepest
}

/// Parses JSON with `serde_json`'s own recursion limit (128) lifted: [`nesting_depth`] has
/// already bounded the depth, so a syntax error is the only refusal left.
fn json(text: &str) -> serde_json::Result<Json> {
    use serde::Deserialize;
    let mut deserializer = serde_json::Deserializer::from_str(text);
    deserializer.disable_recursion_limit();
    let value = Json::deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

type Shape<T> = Result<T, String>;

fn field<'a>(value: &'a Json, key: &str) -> Option<&'a Json> {
    value.as_object().and_then(|map| map.get(key))
}

fn required<'a>(value: &'a Json, key: &str) -> Shape<&'a Json> {
    field(value, key).ok_or_else(|| format!("`{key}` is missing"))
}

fn text(value: &Json, key: &str) -> Shape<String> {
    required(value, key)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("`{key}` is not a string"))
}

fn optional_text(value: &Json, key: &str) -> Shape<Option<String>> {
    match required(value, key)? {
        Json::Null => Ok(None),
        Json::String(text) => Ok(Some(text.clone())),
        _ => Err(format!("`{key}` is neither null nor a string")),
    }
}

/// A field the IR writes only when it has a value, so absent is `None` and present is a string.
fn absent_or_text(value: &Json, key: &str) -> Shape<Option<String>> {
    match field(value, key) {
        None => Ok(None),
        Some(Json::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(format!("`{key}` is not a string")),
    }
}

fn section<'a>(value: &'a Json, key: &str) -> Shape<&'a Map<String, Json>> {
    required(value, key)?
        .as_object()
        .ok_or_else(|| format!("`{key}` is not an object"))
}

fn list<'a>(value: &'a Json, key: &str) -> Shape<&'a Vec<Json>> {
    required(value, key)?
        .as_array()
        .ok_or_else(|| format!("`{key}` is not an array"))
}

fn declarations<K: PartialEq, V>(
    value: &Json,
    key: &str,
    id: impl Fn(String) -> K,
    read: impl Fn(&Json) -> Shape<V>,
) -> Shape<Declarations<K, V>> {
    let mut entries = Vec::new();
    for (name, entry) in section(value, key)? {
        entries.push((id(name.clone()), read(entry)?));
    }
    Ok(Declarations::new(entries))
}

/// The `protocol/1` model the IR describes, with every default the IR writes out made explicit.
fn protocol(value: &Json) -> Shape<Protocol> {
    let header = required(value, "protocol")?;
    let revision = required(header, "revision")?
        .as_u64()
        .ok_or("`revision` is not a non-negative integer")?;
    Ok(Protocol {
        format: model::FORMAT.to_owned(),
        protocol: ProtocolHeader {
            id: ProtocolId::new(text(header, "id")?),
            revision,
            description: optional_text(header, "description")?,
        },
        artifacts: declarations(value, "artifacts", model::ArtifactId::new, |entry| {
            Ok(Artifact {
                description: optional_text(entry, "description")?,
            })
        })?,
        evidence_kinds: declarations(value, "evidence_kinds", EvidenceKindId::new, |entry| {
            Ok(EvidenceKind {
                description: optional_text(entry, "description")?,
                max_age: absent_or_text(entry, "max_age")?.map(Age::new),
            })
        })?,
        claims: declarations(value, "claims", ClaimId::new, |entry| {
            Ok(Claim {
                description: optional_text(entry, "description")?,
                true_when: predicate(required(entry, "true_when")?)?,
            })
        })?,
        obligations: declarations(value, "obligations", model::ObligationId::new, |entry| {
            Ok(Obligation {
                description: optional_text(entry, "description")?,
                discharged_when: predicate(required(entry, "discharged_when")?)?,
            })
        })?,
        actions: declarations(value, "actions", model::ActionId::new, action)?,
        outcomes: declarations(value, "outcomes", model::OutcomeId::new, |entry| {
            Ok(Outcome {
                description: optional_text(entry, "description")?,
                requires: predicate(required(entry, "requires")?)?,
            })
        })?,
    })
}

fn action(entry: &Json) -> Shape<Action> {
    let requires = list(entry, "requires")?
        .iter()
        .map(|item| {
            Ok(CapabilityRequirement {
                capability: CapabilityId::new(text(item, "capability")?),
            })
        })
        .collect::<Shape<_>>()?;
    let may_produce = list(entry, "may_produce")?
        .iter()
        .map(|item| {
            Ok(EvidenceProduction {
                evidence: EvidenceKindId::new(text(item, "evidence")?),
            })
        })
        .collect::<Shape<_>>()?;
    Ok(Action {
        description: optional_text(entry, "description")?,
        precondition: Some(predicate(required(entry, "precondition")?)?),
        requires,
        effect: optional_text(entry, "effect")?.map(EffectClass::new),
        may_produce,
    })
}

fn predicate(value: &Json) -> Shape<Predicate> {
    let map = value
        .as_object()
        .filter(|map| map.len() == 1)
        .ok_or("a predicate is not an object with exactly one key")?;
    let (form, inner) = map.iter().next().expect("one entry");
    let members = || {
        inner
            .as_array()
            .ok_or_else(|| "`all` or `any` is not an array".to_owned())?
            .iter()
            .map(predicate)
            .collect::<Shape<Vec<_>>>()
    };
    match form.as_str() {
        "all" => Ok(Predicate::All(members()?)),
        "any" => Ok(Predicate::Any(members()?)),
        "not" => Ok(Predicate::Not(Box::new(predicate(inner)?))),
        "evidence" => Ok(Predicate::Evidence(EvidenceMatch {
            kind: EvidenceKindId::new(text(inner, "kind")?),
            result: optional_text(inner, "result")?,
        })),
        "claim" => {
            let is = match text(inner, "is")?.as_str() {
                "true" => Truth::True,
                "false" => Truth::False,
                "unknown" => Truth::Unknown,
                _ => return Err("`is` is not true, false or unknown".to_owned()),
            };
            Ok(Predicate::Claim(ClaimTest {
                claim: ClaimId::new(text(inner, "id")?),
                is,
            }))
        }
        _ => Err("a predicate is not one of all, any, not, evidence or claim".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{case_from_value, evidence_from_value, read_case, read_evidence};

    const PROTOCOL: &str = "format: protocol/1\n\
        protocol: {id: p, revision: 7, description: \"Quote \\\" and tab\\t.\"}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {description: kind}, l: {}}\n\
        claims:\n\
          \x20\x20c: {true_when: {any: [{evidence: {kind: k, result: pass}}, {not: {evidence: {kind: l}}}]}}\n\
          \x20\x20d: {true_when: {all: [{claim: c, is: unknown}, {claim: c}]}}\n\
        obligations: {o: {discharged_when: {claim: d}}}\n\
        actions:\n\
          \x20\x20act: {precondition: {claim: d}, requires: [{capability: cap}], effect: write, may_produce: [{evidence: k}]}\n\
          \x20\x20idle: {}\n\
        outcomes: {done: {requires: {claim: d, is: false}}}\n";

    fn compiled() -> Ir {
        ir::compile(&model::parse(PROTOCOL).expect("parses")).expect("compiles")
    }

    #[test]
    fn the_ir_canon_compile_prints_reads_back_as_the_same_ir() {
        let ir = compiled();
        assert_eq!(read_ir(&ir.canonical_json()), Ok(ir));
    }

    /// A maximum age reads back as written; one that is not a string, or an explicit null the
    /// compiler never writes, is refused.
    #[test]
    fn a_maximum_age_reads_back_and_any_other_value_is_refused() {
        let ir = ir::compile(
            &model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 evidence_kinds: {k: {max_age: 5m}, l: {}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let text = ir.canonical_json();
        assert!(text.contains("\"max_age\": \"5m\""), "{text}");
        assert_eq!(read_ir(&text), Ok(ir));
        for replacement in ["\"max_age\": 300", "\"max_age\": null"] {
            let refusal =
                read_ir(&text.replace("\"max_age\": \"5m\"", replacement)).expect_err(replacement);
            assert_eq!(refusal.code(), "malformed-input", "{refusal}");
            assert_eq!(
                refusal.to_string(),
                "not canon-ir/1: `max_age` is not a string"
            );
        }
        let refusal = read_ir(&text.replace("\"5m\"", "\"5 min\"")).expect_err("invalid age");
        assert!(refusal.to_string().contains("invalid-max-age"), "{refusal}");
    }

    /// The IR of a claim whose predicate is `not` nested `depth` times around an evidence match.
    fn deep_ir(depth: usize) -> Ir {
        let mut protocol = model::parse(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
             evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k, result: pass}}}}\n",
        )
        .expect("parses");
        let (_, claim) = protocol.claims.iter().next().expect("c");
        let mut predicate = claim.true_when.clone();
        for _ in 0..depth {
            predicate = Predicate::Not(Box::new(predicate));
        }
        protocol.claims = Declarations::new(vec![(
            ClaimId::new("c"),
            Claim {
                description: None,
                true_when: predicate,
            },
        )]);
        ir::compile(&protocol).expect("compiles")
    }

    /// Nesting up to the bound reads back and evaluates, on a test thread's default stack; one
    /// level more is refused naming the bound, not as a syntax error.
    #[test]
    fn ir_nested_to_the_bound_reads_back_and_one_level_more_is_refused() {
        // Built and printed on a deep stack: the compiler's own recursion is not what is tested.
        let built = |depth: usize| {
            std::thread::Builder::new()
                .stack_size(256 << 20)
                .spawn(move || {
                    let base = nesting_depth(&deep_ir(0).canonical_json());
                    let ir = deep_ir(depth - base);
                    let text = ir.canonical_json();
                    (ir, text)
                })
                .expect("thread")
                .join()
                .expect("built")
        };
        // Read and evaluated from this test thread, whose stack is the default 2 MiB.
        let (deepest, text) = built(MAX_IR_DEPTH);
        assert_eq!(nesting_depth(&text), MAX_IR_DEPTH);
        let read = read_ir(&text).expect("reads back");
        let case =
            read_case("format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r}}\n")
                .expect("case");
        let decision = super::super::evaluate(&read, &case, &[]).expect("decides");
        assert_eq!(decision.claims.len(), 1);
        drop(read);
        assert_eq!(read_ir(&text), Ok(deepest));

        let (_, text) = built(MAX_IR_DEPTH + 1);
        assert_eq!(nesting_depth(&text), MAX_IR_DEPTH + 1);
        let refusal = read_ir(&text).expect_err("too deep");
        assert_eq!(refusal.code(), "malformed-input");
        assert_eq!(
            refusal.to_string(),
            "not canon-ir/1: arrays and objects are nested deeper than 4096"
        );
        let broken = read_ir("{\"format\": ").expect_err("syntax");
        assert_eq!(broken.to_string(), "not canon-ir/1: not well-formed JSON");
    }

    /// Brackets inside strings, escaped quotes included, are text, not nesting.
    #[test]
    fn nesting_is_counted_outside_strings_only() {
        assert_eq!(nesting_depth("{\"a\": [1, {\"b\": []}]}"), 4);
        assert_eq!(nesting_depth("{\"a\": \"[[[{{{\"}"), 1);
        assert_eq!(nesting_depth("{\"a\": \"\\\"[[[\", \"b\": [[]]}"), 3);
        assert_eq!(nesting_depth("{\"a\": \"\\\\\", \"b\": [[]]}"), 3);
    }

    /// Free text reaches the IR as written, so every character the compiler can print must read
    /// back: every code point below U+0800, the Unicode line and paragraph separators, the
    /// byte-order mark, noncharacters and the last code point.
    #[test]
    fn ir_free_text_reads_back_whatever_characters_it_holds() {
        let mut text: String = (0u32..0x800).filter_map(char::from_u32).collect();
        text.extend([
            '\u{2028}',
            '\u{2029}',
            '\u{feff}',
            '\u{fffe}',
            '\u{ffff}',
            '\u{10ffff}',
        ]);
        let mut protocol = model::parse(PROTOCOL).expect("parses");
        protocol.protocol.description = Some(text.clone());
        let ir = ir::compile(&protocol).expect("compiles");
        assert_eq!(read_ir(&ir.canonical_json()), Ok(ir));
    }

    /// One rule on both paths: an identifier, revision or result written as a number or a boolean
    /// is refused, whether the document is read from text or from a value.
    #[test]
    fn a_number_or_boolean_where_a_string_belongs_is_refused_on_both_paths() {
        let case = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";
        let evidence = "format: canon-evidence/1\nid: e\nkind: k\nresult: pass\nsubject: a\nsubject_revision: r1\n";
        assert!(read_case(case).is_ok() && read_evidence(evidence).is_ok());
        for bad in [
            case.replace("id: C-1", "id: 18"),
            case.replace("protocol: p", "protocol: 7"),
            case.replace("revision: r1", "revision: 1"),
            case.replace("revision: r1", "revision: true"),
            case.replace("{a: {", "{1: {"),
        ] {
            let value: Value = serde_yaml_ng::from_str(&bad).expect("yaml");
            for refusal in [
                read_case(&bad).expect_err(&bad),
                case_from_value(&value).expect_err(&bad),
            ] {
                assert_eq!(refusal.code(), "malformed-input", "{bad}: {refusal}");
            }
        }
        for bad in [
            evidence.replace("id: e", "id: 1"),
            evidence.replace("kind: k", "kind: 2.5"),
            evidence.replace("result: pass", "result: 1"),
            evidence.replace("result: pass", "result: false"),
            evidence.replace("subject: a", "subject: 3"),
            evidence.replace("subject_revision: r1", "subject_revision: 2"),
        ] {
            let value: Value = serde_yaml_ng::from_str(&bad).expect("yaml");
            for refusal in [
                read_evidence(&bad).expect_err(&bad),
                evidence_from_value(&value).expect_err(&bad),
            ] {
                assert_eq!(refusal.code(), "malformed-input", "{bad}: {refusal}");
            }
        }
    }

    #[test]
    fn ir_text_that_is_not_what_canon_compile_prints_is_refused() {
        let text = compiled().canonical_json();
        let refused = |text: &str| read_ir(text).expect_err("refused");
        for (changed, why) in [
            (text.trim_end().to_owned(), "not the canonical text"),
            (text.replace("  ", "   "), "not the canonical text"),
            (
                text.replacen("canon-ir/1", "canon-ir/2", 1),
                "format is not `canon-ir/1`",
            ),
            (
                text.replacen("\"id\": \"c\"", "\"id\": \"zzz\"", 1),
                "undeclared-claim",
            ),
            (
                text.replacen("\"is\": \"unknown\"", "\"is\": \"maybe\"", 1),
                "`is` is not true, false or unknown",
            ),
            ("[".to_owned(), "not well-formed JSON"),
        ] {
            let refusal = refused(&changed);
            assert_eq!(refusal.code(), "malformed-input");
            assert!(refusal.to_string().contains(why), "{why}: {refusal}");
        }
    }

    #[test]
    fn case_and_evidence_documents_read_from_text_and_from_values_alike() {
        let case_text =
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts:\n  a: {revision: r1}\n";
        let from_text = read_case(case_text).expect("case reads");
        let value: Value = serde_yaml_ng::from_str(case_text).expect("yaml");
        assert_eq!(case_from_value(&value), Ok(from_text.clone()));
        assert_eq!(from_text.id.as_str(), "C-1");
        assert_eq!(from_text.artifacts.len(), 1);

        let evidence_text = "{\"format\": \"canon-evidence/1\", \"id\": \"e\", \"kind\": \"k\", \"subject\": \"a\", \"subject_revision\": \"r1\"}";
        let record = read_evidence(evidence_text).expect("JSON evidence reads");
        assert_eq!(record.result, None);
        let value: Value = serde_yaml_ng::from_str(evidence_text).expect("yaml");
        assert_eq!(evidence_from_value(&value), Ok(record));

        for (text, what) in [
            (format!("{case_text}extra: 1\n"), "case"),
            (
                "format: canon-case/1\nid: C-1\nprotocol: p\n".to_owned(),
                "case",
            ),
            (case_text.replace("id: C-1", "id:"), "case"),
        ] {
            let refusal = read_case(&text).expect_err(what);
            assert_eq!(refusal.code(), "malformed-input");
            assert!(
                refusal
                    .to_string()
                    .starts_with("case is not a canon-case/1 document: ")
            );
        }
        let refusal = read_evidence(
            "format: canon-evidence/1\nid: e\nkind: k\nresult:\nsubject: a\nsubject_revision: r1\n",
        )
        .expect_err("an explicit null result is refused");
        assert!(
            refusal
                .to_string()
                .starts_with("evidence is not a canon-evidence/1 document: ")
        );
    }
}

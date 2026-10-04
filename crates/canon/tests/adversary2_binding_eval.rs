//! Adversary pass 2 for story:evidence-revision-binding, against the evaluator library: the
//! ground pass 1 did not cover.
//!
//! - `exclusion_listing_*`: the module docs of `eval/mod.rs` (rendered as
//!   `website/docs/reference/evaluation.md`) end "with the records bound to another revision
//!   listed as excluded", and `website/docs/status/where-this-stands.md` says "a record bound to
//!   another revision is listed as excluded". Expected red: a stale record of a kind no claim
//!   reaches is listed nowhere in the decision.
//! - the rest are probes (expected green): revision values of another YAML type, evidence about
//!   the case itself, a declared artifact the case leaves out, and the order of the
//!   supplied-input and binding refusals.

use b10x_canon::eval::{self, Refusal, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Decision, Truth};

/// `k` is reached by a claim; `m` is declared (an action may produce it) and no claim reaches it.
const PROTOCOL: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {a: {}, b: {}}\n\
    evidence_kinds: {k: {}, m: {}}\n\
    claims:\n\
      \x20\x20base: {true_when: {evidence: {kind: k, result: pass}}}\n\
    actions:\n\
      \x20\x20probe: {may_produce: [{evidence: m}]}\n";

fn compiled() -> ir::Ir {
    ir::compile(&model::parse(PROTOCOL).expect("parses")).expect("compiles")
}

fn case_text(a: &str, b: &str) -> String {
    format!(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts:\n  a: {{revision: {a}}}\n  b: {{revision: {b}}}\n"
    )
}

fn record(id: &str, kind: &str, subject: &str, revision: &str) -> String {
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nresult: pass\nsubject: {subject}\nsubject_revision: {revision}\n"
    )
}

fn decide_with(
    case: &str,
    evidence: &[String],
    supplied: Supplied<'_>,
) -> Result<Decision, Refusal> {
    let case = eval::read_case(case)?;
    let evidence = evidence
        .iter()
        .map(|text| eval::read_evidence(text))
        .collect::<Result<Vec<_>, _>>()?;
    eval::evaluate_with(&compiled(), &case, &evidence, supplied)
}

fn decide(case: &str, evidence: &[String]) -> Result<Decision, Refusal> {
    decide_with(case, evidence, Supplied::default())
}

fn value(decision: &Decision, claim: &str) -> Truth {
    decision
        .claims
        .iter()
        .find(|(id, _)| id.as_str() == claim)
        .map(|(_, entry)| entry.value)
        .unwrap_or_else(|| panic!("claim `{claim}` decided"))
}

/// The docs say every record bound to another revision is listed as excluded. `stale-m` is bound
/// to `r1` while the case has `a` at `r2`; its kind `m` is one no claim reaches.
#[test]
fn exclusion_listing_names_every_record_bound_to_another_revision() {
    let decision = decide(
        &case_text("r2", "r1"),
        &[
            record("stale-m", "m", "a", "r1"),
            record("stale-k", "k", "a", "r1"),
        ],
    )
    .expect("decides");
    let listed: Vec<String> = decision
        .claims
        .iter()
        .flat_map(|(_, entry)| entry.excluded_evidence.iter())
        .map(|exclusion| exclusion.evidence.as_str().to_owned())
        .collect();
    assert!(
        listed.iter().any(|id| id == "stale-k"),
        "precondition: a stale record of a reached kind is listed: {listed:?}"
    );
    // Coordinator decision on adversary pass 2, finding F1: the behaviour is right and the
    // unqualified docs were wrong. A record bound to another revision is listed as excluded under
    // each claim that reaches its kind, so `stale-m`, of a kind no claim reaches, appears under no
    // claim, and both pages state the qualified rule.
    let rendered = eval::render(&decision);
    assert!(
        !rendered.contains("stale-m"),
        "`stale-m` is of a kind no claim reaches and is listed under no claim:\n{rendered}"
    );
    let root = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("run this test through cargo"),
    )
    .join("../..");
    for page in [
        "website/docs/reference/evaluation.md",
        "website/docs/status/where-this-stands.md",
    ] {
        let text = std::fs::read_to_string(root.join(page))
            .unwrap_or_else(|error| panic!("{page}: {error}"));
        let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            flat.contains("under each claim that reaches its kind"),
            "{page} states the qualified rule"
        );
    }
}

/// Probe: the schema and the documents page type every revision as an identifier (text). A
/// revision written as a YAML integer is refused on either side, so an integer `2` and the text
/// `"2"` never silently meet.
#[test]
fn probe_a_revision_written_as_an_integer_is_refused_on_either_side() {
    let integer_case = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts:\n  a: {revision: 2}\n  b: {revision: r1}\n";
    let refusal = eval::read_case(integer_case).expect_err("an integer case revision is refused");
    assert_eq!(refusal.code(), "malformed-input", "{refusal}");

    let integer_record = "format: canon-evidence/1\nid: e1\nkind: k\nresult: pass\nsubject: a\nsubject_revision: 2\n";
    let refusal =
        eval::read_evidence(integer_record).expect_err("an integer subject revision is refused");
    assert_eq!(refusal.code(), "malformed-input", "{refusal}");

    // Quoted on both sides they are the same text and the record applies.
    let decision = decide(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts:\n  a: {revision: '2'}\n  b: {revision: r1}\n",
        &[record("e1", "k", "a", "\"2\"")],
    )
    .expect("decides");
    assert_eq!(value(&decision, "base"), Truth::True);

    // Text is compared as written: case and format differences exclude.
    for revision in ["R1", "'r1 '", "r01", "sha256:r1"] {
        let decision = decide(&case_text("r1", "r1"), &[record("e1", "k", "a", revision)]);
        match decision {
            Ok(decision) => assert_eq!(value(&decision, "base"), Truth::Unknown, "{revision}"),
            Err(refusal) => assert_eq!(refusal.code(), "invalid-identifier", "{revision}"),
        }
    }
}

/// Probe: a record about the case itself, rather than an artifact, is refused naming that
/// subject, even when it is the only record.
#[test]
fn probe_evidence_about_the_case_itself_is_refused_naming_it() {
    let refusal =
        decide(&case_text("r1", "r1"), &[record("e1", "k", "C-1", "r1")]).expect_err("refused");
    assert_eq!(refusal.code(), "undeclared-artifact", "{refusal}");
    assert_eq!(
        refusal.to_string(),
        "evidence `e1` is about artifact `C-1`, which the protocol does not declare"
    );
}

/// Probe: a declared artifact the case snapshot leaves out is refused as `missing-artifact`
/// before binding runs, whatever the records bound to it.
#[test]
fn probe_a_declared_artifact_missing_from_the_case_is_refused_before_binding() {
    let case = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts:\n  a: {revision: r1}\n";
    for evidence in [
        vec![record("e1", "k", "b", "r1")],
        vec![record("e1", "k", "a", "r0")],
        vec![record("e1", "k", "nowhere", "r1")],
    ] {
        let refusal = decide(case, &evidence).expect_err("refused");
        assert_eq!(refusal.code(), "missing-artifact", "{refusal}");
        assert!(refusal.to_string().contains("`b`"), "{refusal}");
    }
}

/// Probe: the documented order puts the supplied inputs before the exclusion stages, so an
/// unsupported `--at` is the refusal even when a record names an undeclared subject.
#[test]
fn probe_a_supplied_input_is_refused_before_an_undeclared_subject() {
    let refusal = decide_with(
        &case_text("r1", "r1"),
        &[record("e1", "k", "nowhere", "r1")],
        Supplied {
            at: Some("2026-10-04T00:00:00Z"),
            ..Supplied::default()
        },
    )
    .expect_err("refused");
    assert_eq!(refusal.code(), "unsupported-input", "{refusal}");
}

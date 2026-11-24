//! story:evidence-digest-binds-approvals. A `ground_evidence` entry of `canon-authority/1` or
//! `canon-decisions/1` may be a map, `{id: <evidence id>, digest: sha256:<64 lowercase hex>}`,
//! the digest optional. The digest is the SHA-256 of the record's canonical form, the JSON
//! Canonicalization Scheme (RFC 8785) of its fields as Canon reads them. An approval resting on a
//! digest holds while the record under the id is the one hashed, and lapses as `content_changed`
//! once other contents are stored under the same id. The digests here are computed in the test
//! from the canonical text written out by hand, not by Canon, so the definition is pinned.

use std::path::PathBuf;

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, EvidenceRecord};
use sha2::{Digest, Sha256};

/// Read at run time, so a test binary reused from another build directory reads this tree.
fn fixture() -> ir::Ir {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/investigation/decision-grounds.yaml");
    let text = std::fs::read_to_string(&path).expect("the fixture reads");
    ir::compile(&model::parse(&text).expect("the fixture parses")).expect("the fixture compiles")
}

const CASE: &str = "format: canon-case/1\nid: G-1\nprotocol: grounds\nrevision: c2\n\
                    artifacts: {spec: {revision: r2}, up: {revision: u2}}\n";

/// The record `e1` as it was when the approvals were taken.
const ORIGINAL: &str = "format: canon-evidence/1\nid: e1\nkind: k\nresult: pass\nsubject: spec\n\
                        subject_revision: r2\nobserved_at: 2026-11-22T10:00:00Z\n\
                        upstream_revisions: {up: u2}\nproducer: alice\n";

/// [`ORIGINAL`]'s canonical form: its fields under their keys, sorted, no whitespace.
const ORIGINAL_CANONICAL: &str = r#"{"format":"canon-evidence/1","id":"e1","kind":"k","observed_at":"2026-11-22T10:00:00Z","producer":"alice","result":"pass","subject":"spec","subject_revision":"r2","upstream_revisions":{"up":"u2"}}"#;

/// [`ORIGINAL`] as JSON, its keys in another order and its whitespace another.
const ORIGINAL_AS_JSON: &str = "{ \"upstream_revisions\" : { \"up\" : \"u2\" },\n  \
                                \"producer\": \"alice\", \"observed_at\": \"2026-11-22T10:00:00Z\",\n  \
                                \"subject_revision\": \"r2\", \"subject\": \"spec\", \"result\": \"pass\",\n  \
                                \"kind\": \"k\", \"id\": \"e1\", \"format\": \"canon-evidence/1\" }\n";

/// Other contents stored under the same id later: another result, another producer, another
/// observation instant, still applicable.
const REPLACED: &str = "format: canon-evidence/1\nid: e1\nkind: k\nresult: fail\nsubject: spec\n\
                        subject_revision: r2\nobserved_at: 2026-11-22T10:30:00Z\n\
                        upstream_revisions: {up: u2}\nproducer: mallory\n";

fn digest_of(canonical: &str) -> String {
    let mut text = String::from("sha256:");
    for byte in Sha256::digest(canonical.as_bytes()) {
        text.push_str(&format!("{byte:02x}"));
    }
    text
}

fn record(text: &str) -> EvidenceRecord {
    eval::read_evidence(text).expect("evidence reads")
}

fn evaluate(
    evidence: &[EvidenceRecord],
    authority: Option<&str>,
    decisions: Option<&str>,
) -> Result<model::Decision, eval::Refusal> {
    let case = eval::read_case(CASE).expect("case reads");
    eval::evaluate_with(
        &fixture(),
        &case,
        evidence,
        Supplied {
            authority,
            decisions,
            ..Supplied::default()
        },
    )
}

fn decide(evidence: &[EvidenceRecord], authority: &str, decisions: &str) -> model::Decision {
    evaluate(evidence, Some(authority), Some(decisions)).expect("decides")
}

/// The authority and decisions documents citing `e1` with `digest`.
fn approvals(digest: &str) -> (String, String) {
    (
        format!("- {{capability: x, decision: granted, principal: p, ground_evidence: [{{id: e1, digest: '{digest}'}}]}}\n"),
        format!("- {{decision: d, outcome: o, principal: q, case_revision: c2, ground_evidence: [{{id: e1, digest: '{digest}'}}]}}\n"),
    )
}

/// Acceptance 2: an approval with a digest holds while the record's content is the one hashed,
/// and lapses with `content_changed` after the content under the same id changes; the grant and
/// the explicit decision alike.
#[test]
fn an_approval_with_a_digest_lapses_as_content_changed_when_the_contents_under_its_id_change() {
    let (authority, decisions) = approvals(&digest_of(ORIGINAL_CANONICAL));

    let before = decide(&[record(ORIGINAL)], &authority, &decisions);
    assert_eq!(
        before.actions.as_ref().expect("actions")["act"],
        serde_json::json!({"status": "admissible"}),
        "the grant holds with the contents it hashed"
    );
    assert_eq!(
        before.outcomes.as_ref().expect("outcomes")["o"],
        serde_json::json!({"status": "legitimate",
                           "decided_by": {"decision": "d", "principals": ["q"]}}),
        "the decision holds with the contents it hashed"
    );

    let after = decide(&[record(REPLACED)], &authority, &decisions);
    let content_changed = serde_json::json!([{"evidence": "e1", "reason": "content_changed"}]);
    let act = &after.actions.as_ref().expect("actions")["act"];
    assert_eq!(act["status"], "approval-required", "{act}");
    assert_eq!(act["reasons"][0]["stale_grounds"], content_changed, "{act}");
    let o = &after.outcomes.as_ref().expect("outcomes")["o"];
    assert_eq!(o["status"], "blocked", "{o}");
    assert_eq!(o["reasons"][0]["stale_grounds"], content_changed, "{o}");
}

/// The digest depends on neither the key order nor the whitespace of the file, nor on YAML
/// against JSON: the record written as JSON in another order holds the grant hashed from the
/// canonical text.
#[test]
fn key_order_and_whitespace_do_not_change_the_digest() {
    let (authority, decisions) = approvals(&digest_of(ORIGINAL_CANONICAL));
    let decided = decide(&[record(ORIGINAL_AS_JSON)], &authority, &decisions);
    assert_eq!(
        decided.actions.as_ref().expect("actions")["act"],
        serde_json::json!({"status": "admissible"})
    );
    assert_eq!(
        decided.outcomes.as_ref().expect("outcomes")["o"]["status"],
        "legitimate"
    );
}

/// A map without a digest is the bare id: with the replaced contents both still hold, and the
/// decision, explanation included, is the same byte for byte.
#[test]
fn a_map_without_a_digest_is_the_bare_id() {
    let bare = decide(
        &[record(REPLACED)],
        "- {capability: x, decision: granted, principal: p, ground_evidence: [e1]}\n",
        "- {decision: d, outcome: o, principal: q, case_revision: c2, ground_evidence: [e1]}\n",
    );
    let map = decide(
        &[record(REPLACED)],
        "- {capability: x, decision: granted, principal: p, ground_evidence: [{id: e1}]}\n",
        "- {decision: d, outcome: o, principal: q, case_revision: c2, ground_evidence: [{id: e1}]}\n",
    );
    assert_eq!(
        map.actions.as_ref().expect("actions")["act"],
        serde_json::json!({"status": "admissible"})
    );
    assert_eq!(eval::render(&map), eval::render(&bare));
}

/// `computed_from` records a ground with a digest as `{"digest", "id"}` and one without as its id,
/// in id order, the bare form first, each once.
#[test]
fn computed_from_records_the_digest_beside_the_id() {
    let digest = digest_of(ORIGINAL_CANONICAL);
    let authority = format!(
        "- {{capability: x, decision: granted, principal: p, \
         ground_evidence: [e9, {{id: e1, digest: '{digest}'}}, e1, {{id: e9}}]}}\n"
    );
    let decision = evaluate(&[record(ORIGINAL)], Some(&authority), None).expect("decides");
    let from = &decision.explanation.as_ref().expect("explained")["computed_from"];
    assert_eq!(
        from["authority"][0]["ground_evidence"],
        serde_json::json!(["e1", {"digest": digest, "id": "e1"}, "e9"])
    );
}

/// A digest not written `sha256:` and 64 lowercase hexadecimal digits is refused as
/// malformed-input, naming the entry and the digest.
#[test]
fn a_malformed_digest_is_refused_naming_the_entry() {
    let digest = digest_of(ORIGINAL_CANONICAL);
    for bad in [
        digest.to_uppercase(),
        digest.replace("sha256:", "sha512:"),
        digest[..digest.len() - 1].to_owned(),
        format!("{digest}0"),
        digest.replace("sha256:", ""),
    ] {
        let authority = format!(
            "- {{capability: y, decision: granted}}\n\
             - {{capability: x, decision: granted, ground_evidence: [{{id: e1, digest: '{bad}'}}]}}\n"
        );
        let refused = evaluate(&[], Some(&authority), None).expect_err(&bad);
        let message = refused.to_string();
        assert_eq!(refused.code(), "malformed-input", "{bad}: {message}");
        assert!(
            message.starts_with("`--authority` is not a canon-authority/1 document: entry 2: "),
            "{message}"
        );
        assert!(message.contains(&format!("`{bad}`")), "{message}");
        assert!(
            message.contains("64 lowercase hexadecimal digits"),
            "{message}"
        );

        let decisions = format!(
            "- {{decision: d, outcome: o, principal: q, case_revision: c2, \
             ground_evidence: [{{id: e1, digest: '{bad}'}}]}}\n"
        );
        let refused = evaluate(&[], None, Some(&decisions)).expect_err(&bad);
        assert_eq!(refused.code(), "malformed-input", "{refused}");
        assert!(
            refused
                .to_string()
                .starts_with("`--decisions` is not a canon-decisions/1 document: entry 1: "),
            "{refused}"
        );
    }
}

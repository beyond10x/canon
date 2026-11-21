//! Adversary pass 1 for story:revision-ordering: the `descended` form, `descends_from` on a case
//! artifact and `item` on an evidence record, driven from the `ess/` comments the unit wrote
//! (`ess/domains/protocol.yaml`, `canon.protocol.CaseArtifact.descends_from`,
//! `canon.protocol.EvidenceRecord.item`) and from the schemas it regenerated.

use std::path::{Path, PathBuf};

use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model;
use serde_json::Value;

const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}}\nevidence_kinds: {k: {}}\n\
    claims:\n  c: {true_when: {descended: {first: {kind: k, result: failed}, \
    then: {kind: k, result: passed}, subject: a}}}\n";

fn compiled() -> ir::Ir {
    ir::compile(&model::parse(PROTOCOL).expect("protocol parses")).expect("protocol compiles")
}

fn refusal(artifact: &str) -> eval::Refusal {
    let case = eval::read_case(&format!(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {{a: {artifact}}}\n"
    ))
    .expect("a canon-case/1 snapshot");
    eval::evaluate(&compiled(), &case, &[]).expect_err(artifact)
}

/// `ess/domains/protocol.yaml`, `CaseArtifact.descends_from`: "An evaluation is refused, naming
/// the case, the artifact and the revision, when an entry is not an identifier
/// (invalid-identifier), is listed twice (duplicate-identifier), or is the current revision
/// itself (self-descent)." The same sentence is in the docs of `eval/case.rs::descent` and of
/// `eval/mod.rs`. The two other refusals name all three; the `invalid-identifier` one names only
/// the revision, so a case with many artifacts does not say which artifact lists it.
#[test]
fn an_ancestor_that_is_not_an_identifier_is_refused_naming_the_case_and_the_artifact() {
    for (artifact, code) in [
        (
            "{revision: r2, descends_from: [r1, r1]}",
            "duplicate-identifier",
        ),
        ("{revision: r2, descends_from: [r2]}", "self-descent"),
        (
            "{revision: r2, descends_from: ['r 1']}",
            "invalid-identifier",
        ),
        ("{revision: r2, descends_from: ['']}", "invalid-identifier"),
    ] {
        let refused = refusal(artifact);
        assert_eq!(refused.code(), code, "{artifact}");
        let message = refused.to_string();
        assert!(
            message.contains("case `C-1`") && message.contains("artifact `a`"),
            "{artifact}: the refusal names the case and the artifact, as the ess/ comment of \
             descends_from says; it reads: {message}"
        );
    }
}

/// Read at run time: a test binary reused from a shared build directory must read this tree.
fn repository_root() -> PathBuf {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    Path::new(&manifest)
        .join("../..")
        .canonicalize()
        .expect("repository root resolves")
}

fn schema(name: &str) -> Value {
    let path = repository_root().join("website/static/schemas").join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_str(&text).expect("schema is JSON")
}

/// The schema of `property` of the `$defs` entry `def`, with a local `$ref` resolved.
fn property<'a>(root: &'a Value, def: &str, property: &str) -> &'a Value {
    let declared = &root["$defs"][def]["properties"][property];
    assert!(!declared.is_null(), "{def} declares no `{property}`");
    match declared["$ref"].as_str() {
        Some(reference) => {
            let name = reference
                .strip_prefix("#/$defs/")
                .unwrap_or_else(|| panic!("non-local $ref {reference}"));
            &root["$defs"][name]
        }
        None => declared,
    }
}

/// `ess/domains/protocol.yaml`, `EvidenceRecord.item`: "The evaluator refuses one that is not an
/// identifier (invalid-identifier)", and the schema's own description of `item` says "opaque text
/// without whitespace or a control character". Every other field the evaluator checks as an
/// identifier (`id`, `kind`, `subject`, `subject_revision`, `producer`, `child_case`, and each
/// entry of `descends_from` in `case-1.schema.json`) carries the identifier rule in the published
/// schema; `item` is a bare string, so `item: ""` and `item: "t 1"` validate against
/// `evidence-1.schema.json` and are refused by `canon evaluate`.
#[test]
fn the_published_schema_of_item_holds_the_identifier_rule_the_evaluator_applies() {
    let evidence = schema("evidence-1.schema.json");
    let compared = property(&evidence, "EvidenceRecord", "child_case");
    assert_eq!(compared["pattern"], "^\\S+$", "{compared}");
    let item = property(&evidence, "EvidenceRecord", "item");
    assert_eq!(
        (&item["pattern"], &item["minLength"]),
        (&compared["pattern"], &compared["minLength"]),
        "the schema of `item` accepts what the evaluator refuses as invalid-identifier: {item}"
    );
    let empty = model::EvidenceRecord {
        item: Some(model::ItemId::new("")),
        ..record()
    };
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r2}}\n",
    )
    .expect("a canon-case/1 snapshot");
    let refused = eval::evaluate(&compiled(), &case, &[empty]).expect_err("an empty item");
    assert_eq!(refused.code(), "invalid-identifier");
}

/// The value of claim `c` for `records`, each `(result, revision, item, observed_at)` about `a`,
/// with `a` at `r2` descending from `r1` and `r0`.
fn value(records: &[(&str, &str, &str, Option<&str>)]) -> String {
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\n\
         artifacts: {a: {revision: r2, descends_from: [r1, r0]}}\n",
    )
    .expect("a canon-case/1 snapshot");
    let evidence: Vec<model::EvidenceRecord> = records
        .iter()
        .enumerate()
        .map(|(n, (result, revision, item, at))| {
            let at = at.map_or(String::new(), |at| format!("observed_at: '{at}'\n"));
            eval::read_evidence(&format!(
                "format: canon-evidence/1\nid: e{n}\nkind: k\nresult: {result}\nsubject: a\n\
                 subject_revision: {revision}\nitem: {item}\n{at}"
            ))
            .expect("a canon-evidence/1 record")
        })
        .collect();
    let decision = eval::evaluate(&compiled(), &case, &evidence).expect("decides");
    let (_, entry) = decision
        .claims
        .iter()
        .find(|(id, _)| id.as_str() == "c")
        .expect("claim c");
    entry.value.to_string()
}

/// `ess/domains/protocol.yaml`, `EvidenceDescended`: "A later record is a record of K2 with R2
/// ... An earlier record for it is a record of K1 with R1"; table row "item T: R1 on an ancestor,
/// none of R2 on the current revision: unknown". The unit's table tests always write R1 on the
/// ancestor and R2 on the current revision, so a form that ignores either result is not caught by
/// them: here the ancestor holds R2 (`passed`) and the current revision R1 (`failed`).
#[test]
fn descended_pairs_only_records_of_the_results_it_names() {
    assert_eq!(
        value(&[("passed", "r1", "t", None), ("passed", "r2", "t", None)]),
        "false",
        "a pass on an ancestor is not an earlier record of `failed`"
    );
    assert_eq!(
        value(&[("failed", "r1", "t", None), ("failed", "r2", "t", None)]),
        "unknown",
        "a failure on the current revision is not a later record of `passed`"
    );
    assert_eq!(
        value(&[("failed", "r1", "t", None), ("passed", "r2", "t", None)]),
        "true"
    );
}

/// `EvidenceDescended`: "It reads the history ... and never observed_at: descent is the order."
/// A pass observed before the failure, at the same instant, or with no instant on one side gives
/// the value the records give without any instant.
#[test]
fn descended_reads_no_observed_at() {
    for (failed_at, passed_at) in [
        (Some("2026-10-04T12:00:00Z"), Some("2026-10-04T11:00:00Z")),
        (Some("2026-10-04T12:00:00Z"), Some("2026-10-04T12:00:00Z")),
        (Some("2026-10-04T12:00:00Z"), None),
        (None, Some("2026-10-04T12:00:00Z")),
    ] {
        assert_eq!(
            value(&[
                ("failed", "r1", "t", failed_at),
                ("passed", "r2", "t", passed_at)
            ]),
            "true",
            "{failed_at:?} then {passed_at:?}"
        );
    }
}

fn record() -> model::EvidenceRecord {
    eval::read_evidence(
        "format: canon-evidence/1\nid: e1\nkind: k\nresult: failed\nsubject: a\n\
         subject_revision: r1\n",
    )
    .expect("a canon-evidence/1 record")
}

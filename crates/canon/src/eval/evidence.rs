//! `canon-evidence/1` records as the evaluator reads and checks them. The type is
//! [`crate::model::EvidenceRecord`]; this file reads a record from text or a YAML value and checks
//! the set against the compiled protocol. Which records apply is decided afterwards, by the
//! exclusion stages (`binding.rs`, `freshness.rs`, `invalidation.rs`).
//!
//! A record may be given more than once, as a runner that delivers at least once gives it
//! (story:evidence-redelivery). The copies count once when each equals the first, and the set
//! [`check`] returns holds the first copy in the order given, so every later stage reads each id
//! once; a copy that differs is refused as `duplicate-identifier`. [`same_record`] is the equality
//! and lives here alone.
//!
//! Each upstream artifact a record names is one the protocol declares, except on a record of a
//! kind that declares `supplied_by` (story:supplied-evidence): copied unchanged from a case under
//! another protocol, it may name artifacts only that protocol declares. Those entries are carried
//! and neither checked nor read; every upstream artifact is still named once and its revision is
//! still an identifier.

use std::collections::{BTreeMap, BTreeSet};

use serde_yaml_ng::Value;

use super::freshness::invalid_instant;
use super::read::{malformed, yaml};
use super::{Refusal, identifier, unsupported_format};
use crate::ir::Ir;
use crate::model::{
    ArtifactId, Declarations, EVIDENCE_FORMAT, EvidenceId, EvidenceRecord, Instant, Revision,
    one_line,
};

/// Reads one `canon-evidence/1` document: parsed as YAML, then read as [`evidence_from_value`]
/// reads it.
pub fn read_evidence(text: &str) -> Result<EvidenceRecord, Refusal> {
    evidence_from_value(&yaml(text, "evidence is not a canon-evidence/1 document")?)
}

/// Reads one `canon-evidence/1` document already parsed as YAML. A document that is not one is
/// refused naming the record when its `id` is a string (`` evidence `e1` is not a
/// canon-evidence/1 document: missing field `subject` ``).
pub fn evidence_from_value(value: &Value) -> Result<EvidenceRecord, Refusal> {
    serde_yaml_ng::from_value(value.clone()).map_err(|error| {
        let what = match value.get("id").and_then(Value::as_str) {
            Some(id) => format!(
                "evidence `{}` is not a canon-evidence/1 document",
                one_line(id)
            ),
            None => "evidence is not a canon-evidence/1 document".to_owned(),
        };
        malformed(&what, error)
    })
}

/// Checks each record in the order given, as the module docs of [`super`] describe, and returns the
/// distinct records: a record given more than once counts once, as its first copy, when every
/// copy equals it ([`same_record`]); a copy that differs is refused as `duplicate-identifier`.
/// Every later stage reads the returned records, so each id is read once. Every refusal names the
/// record by its id.
pub(super) fn check(ir: &Ir, evidence: &[EvidenceRecord]) -> Result<Vec<EvidenceRecord>, Refusal> {
    let mut seen = BTreeMap::new();
    let mut distinct = Vec::with_capacity(evidence.len());
    for record in evidence {
        let first =
            check_record(ir, record, &mut seen).map_err(|refusal| refusal.citing(&record.id))?;
        if first {
            distinct.push(record.clone());
        }
    }
    super::composition::one_record_per_child(&distinct)?;
    Ok(distinct)
}

/// Whether two records are equal for [`check`]: every field as written, `observed_at` as text
/// (`12:00:00Z` and `12:00:00.000Z` differ, since precision sets the order resolution, as
/// `order.rs` explains), and `upstream_revisions` as the same pairs in any order. The record is
/// destructured without `..`, so a field added to it does not compile until this reads it.
fn same_record(first: &EvidenceRecord, copy: &EvidenceRecord) -> bool {
    let EvidenceRecord {
        format,
        id,
        kind,
        result,
        subject,
        subject_revision,
        observed_at,
        upstream_revisions,
        producer,
        child_case,
        item,
    } = first;
    let EvidenceRecord {
        format: copy_format,
        id: copy_id,
        kind: copy_kind,
        result: copy_result,
        subject: copy_subject,
        subject_revision: copy_subject_revision,
        observed_at: copy_observed_at,
        upstream_revisions: copy_upstream_revisions,
        producer: copy_producer,
        child_case: copy_child_case,
        item: copy_item,
    } = copy;
    fn pairs(upstream: &Declarations<ArtifactId, Revision>) -> Vec<(&ArtifactId, &Revision)> {
        let mut pairs: Vec<_> = upstream.iter().collect();
        pairs.sort();
        pairs
    }
    format == copy_format
        && id == copy_id
        && kind == copy_kind
        && result == copy_result
        && subject == copy_subject
        && subject_revision == copy_subject_revision
        && observed_at.as_ref().map(Instant::as_str)
            == copy_observed_at.as_ref().map(Instant::as_str)
        && pairs(upstream_revisions) == pairs(copy_upstream_revisions)
        && producer == copy_producer
        && child_case == copy_child_case
        && item == copy_item
}

/// One record of [`check`], refused without its citation, which [`check`] adds. Returns whether
/// the record is the first of its id, to be counted: a later copy equal to the first is not, and
/// is not checked again, since the first passed every check this one would.
fn check_record<'a>(
    ir: &Ir,
    record: &'a EvidenceRecord,
    seen: &mut BTreeMap<&'a EvidenceId, &'a EvidenceRecord>,
) -> Result<bool, Refusal> {
    let named = format!("evidence `{}`", one_line(record.id.as_str()));
    if record.format != EVIDENCE_FORMAT {
        let refusal = unsupported_format(&record.format, EVIDENCE_FORMAT);
        return Err(Refusal::new(refusal.code(), format!("{named} {refusal}")));
    }
    identifier("evidence identifier", record.id.as_str())?;
    identifier(&format!("{named} kind identifier"), record.kind.as_str())?;
    identifier(
        &format!("{named} subject identifier"),
        record.subject.as_str(),
    )?;
    identifier(
        &format!("{named} subject revision"),
        record.subject_revision.as_str(),
    )?;
    for (artifact, revision) in record.upstream_revisions.iter() {
        identifier(
            &format!("{named} upstream artifact identifier"),
            artifact.as_str(),
        )?;
        identifier(&format!("{named} upstream revision"), revision.as_str())?;
    }
    if let Some(producer) = &record.producer {
        identifier(&format!("{named} producer identifier"), producer.as_str())?;
    }
    if let Some(child) = &record.child_case {
        identifier(&format!("{named} child case identifier"), child.as_str())?;
    }
    if let Some(item) = &record.item {
        identifier(&format!("{named} item"), item.as_str())?;
    }
    if let Some(observed_at) = &record.observed_at
        && observed_at.seconds().is_none()
    {
        return Err(invalid_instant(
            &format!("evidence `{}` observed_at", one_line(record.id.as_str())),
            observed_at.as_str(),
        ));
    }
    if let Some(first) = seen.get(&record.id) {
        if same_record(first, record) {
            return Ok(false);
        }
        return Err(Refusal::new(
            "duplicate-identifier",
            format!(
                "evidence `{}` is given more than once with different content",
                one_line(record.id.as_str())
            ),
        ));
    }
    seen.insert(&record.id, record);
    let Some(kind) = ir.evidence_kinds.get(&record.kind) else {
        return Err(Refusal::new(
            "undeclared-evidence-kind",
            format!(
                "evidence `{}` is of kind `{}`, which the protocol does not declare",
                one_line(record.id.as_str()),
                one_line(record.kind.as_str())
            ),
        ));
    };
    // A record of a supplied kind is copied unchanged from a case under its supplier, which may
    // declare artifacts this protocol does not (story:supplied-evidence, option B): such an entry
    // is carried. Nothing reads it: an invalidation rule watches a declared artifact only.
    let supplied = kind.supplied_by.is_some();
    let mut upstream = BTreeSet::new();
    for artifact in record.upstream_revisions.ids() {
        if !supplied && !ir.artifacts.contains_key(artifact) {
            return Err(Refusal::new(
                "undeclared-artifact",
                format!(
                    "evidence `{}` records a revision of upstream artifact `{}`, which the protocol does not declare",
                    one_line(record.id.as_str()),
                    one_line(artifact.as_str())
                ),
            ));
        }
        if !upstream.insert(artifact) {
            return Err(Refusal::new(
                "duplicate-identifier",
                format!(
                    "evidence `{}` records a revision of upstream artifact `{}` more than once",
                    one_line(record.id.as_str()),
                    one_line(artifact.as_str())
                ),
            ));
        }
    }
    if super::composition::is_case_outcome(record) {
        if record.child_case.is_none() {
            return Err(Refusal::new(
                "missing-child-case",
                format!("{named} is a case_outcome record and names no child case"),
            ));
        }
        if record.result.is_none() {
            return Err(Refusal::new(
                "missing-child-outcome",
                format!("{named} is a case_outcome record and gives no outcome"),
            ));
        }
    } else if record.child_case.is_some() {
        return Err(Refusal::new(
            "misplaced-child-case",
            format!(
                "{named} names a child case, which only a case_outcome record does; it is of kind `{}`",
                one_line(record.kind.as_str())
            ),
        ));
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ir() -> Ir {
        crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {k: {}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles")
    }

    fn checked(observed_at: &str) -> Result<(), Refusal> {
        let record = read_evidence(&format!(
            "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n\
             observed_at: {observed_at}\n"
        ))?;
        check(&ir(), &[record]).map(|_| ())
    }

    /// Each upstream artifact a record records a revision of is one the protocol declares, named
    /// once; one that is not is refused, naming the record and the artifact.
    #[test]
    fn upstream_revisions_name_declared_artifacts_once() {
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let checked = |upstream: &str| -> Result<(), Refusal> {
            let record = read_evidence(&format!(
                "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n\
                 upstream_revisions: {upstream}\n"
            ))?;
            check(&ir, &[record]).map(|_| ())
        };
        assert_eq!(checked("{up: u1}"), Ok(()));
        let undeclared = checked("{up: u1, gone: g1}").expect_err("refused");
        assert_eq!(
            (undeclared.code(), undeclared.to_string().as_str()),
            (
                "undeclared-artifact",
                "evidence `e1` records a revision of upstream artifact `gone`, which the protocol does not declare"
            )
        );
        // YAML itself refuses a repeated key, so a record read from text never repeats one; a
        // record built in Rust can, and `check` refuses it as `duplicate-identifier`.
        assert_eq!(
            checked("{up: u1, up: u2}").expect_err("refused").code(),
            "malformed-input"
        );
        let mut built = read_evidence(
            "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n",
        )
        .expect("reads");
        built.upstream_revisions = crate::model::Declarations::new(vec![
            ("up".into(), "u1".into()),
            ("up".into(), "u2".into()),
        ]);
        assert_eq!(
            check(&ir, &[built]).expect_err("refused").to_string(),
            "evidence `e1` records a revision of upstream artifact `up` more than once"
        );
        assert_eq!(
            checked("{up: ' '}").expect_err("refused").code(),
            "invalid-identifier"
        );
        assert_eq!(
            checked("~").expect_err("null refused").code(),
            "malformed-input"
        );
    }

    /// A record of a supplied kind may name upstream artifacts the protocol does not declare, each
    /// still once and each revision still an identifier; a record of a kind without a supplier
    /// may not, as before.
    #[test]
    fn a_supplied_record_may_name_undeclared_upstream_artifacts() {
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}, up: {}}\nevidence_kinds: {s: {supplied_by: q}, k: {}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let checked = |kind: &str, upstream: &str| -> Result<(), Refusal> {
            let record = read_evidence(&format!(
                "format: canon-evidence/1\nid: e1\nkind: {kind}\nsubject: a\nsubject_revision: r1\n\
                 upstream_revisions: {upstream}\n"
            ))?;
            check(&ir, &[record]).map(|_| ())
        };
        assert_eq!(checked("s", "{up: u1, gone: g1}"), Ok(()));
        assert_eq!(checked("s", "{gone: g1}"), Ok(()));
        assert_eq!(
            checked("k", "{up: u1, gone: g1}")
                .expect_err("refused")
                .to_string(),
            "evidence `e1` records a revision of upstream artifact `gone`, which the protocol does not declare"
        );
        assert_eq!(
            checked("s", "{gone: ' '}").expect_err("refused").code(),
            "invalid-identifier"
        );
        let mut built = read_evidence(
            "format: canon-evidence/1\nid: e1\nkind: s\nsubject: a\nsubject_revision: r1\n",
        )
        .expect("reads");
        built.upstream_revisions = crate::model::Declarations::new(vec![
            ("gone".into(), "g1".into()),
            ("gone".into(), "g2".into()),
        ]);
        assert_eq!(
            check(&ir, &[built]).expect_err("refused").to_string(),
            "evidence `e1` records a revision of upstream artifact `gone` more than once"
        );
    }

    /// `observed_at` is optional, and when given it is an instant; an explicit null or any other
    /// text is refused.
    #[test]
    fn observed_at_is_an_instant_when_given() {
        assert_eq!(checked("2026-10-04T12:00:00Z"), Ok(()));
        assert_eq!(checked("2026-10-04T12:00:00.5Z"), Ok(()));
        assert_eq!(checked("2026-10-04T12:00:00.123456789Z"), Ok(()));
        for refused in [
            "2026-10-04T12:00:00.Z",
            "2026-10-04T12:00:00.1234567890Z",
            "2026-10-04T12:00:00,5Z",
        ] {
            assert_eq!(
                checked(refused).expect_err(refused).code(),
                "invalid-instant"
            );
        }
        let refusal = checked("2026-10-04T12:00:00+02:00").expect_err("refused");
        assert_eq!(refusal.code(), "invalid-instant");
        assert_eq!(
            refusal.to_string(),
            "evidence `e1` observed_at `2026-10-04T12:00:00+02:00` is not an instant in UTC written YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DDTHH:MM:SS.fffZ (one to nine fraction digits)"
        );
        assert_eq!(
            checked("~").expect_err("null refused").code(),
            "malformed-input"
        );
    }

    /// Every refusal `check` gives names the record it refuses: among `e1` and a bad `e2`, each
    /// message names `e2` (an invalid id names the id it found, `e 2`; a repeated id with different
    /// content is `e1`).
    #[test]
    fn every_evidence_check_refusal_names_the_record() {
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let text = |id: &str| {
            format!(
                "format: canon-evidence/1\nid: {id}\nkind: k\nsubject: a\nsubject_revision: r1\n\
                 observed_at: 2026-10-04T12:00:00Z\nupstream_revisions: {{up: u1}}\n"
            )
        };
        let e2 = text("e2");
        let cases = [
            (
                "format",
                e2.replace("canon-evidence/1", "canon-evidence/2"),
                "`e2`",
            ),
            ("id", e2.replace("id: e2", "id: 'e 2'"), "`e 2`"),
            ("kind", e2.replace("kind: k", "kind: 'k k'"), "`e2`"),
            (
                "subject",
                e2.replace("subject: a", "subject: 'a a'"),
                "`e2`",
            ),
            (
                "subject revision",
                e2.replace("subject_revision: r1", "subject_revision: 'r 1'"),
                "`e2`",
            ),
            (
                "upstream artifact",
                e2.replace("{up: u1}", "{'u p': u1}"),
                "`e2`",
            ),
            (
                "upstream revision",
                e2.replace("{up: u1}", "{up: 'u 1'}"),
                "`e2`",
            ),
            (
                "observed_at",
                e2.replace("2026-10-04T12:00:00Z", "yesterday"),
                "`e2`",
            ),
            ("item", format!("{e2}item: 't 1'\n"), "`e2`"),
            (
                "repeated id",
                text("e1").replace("12:00:00Z", "12:00:00.000Z"),
                "`e1`",
            ),
            ("undeclared kind", e2.replace("kind: k", "kind: z"), "`e2`"),
            (
                "undeclared upstream artifact",
                e2.replace("{up: u1}", "{gone: u1}"),
                "`e2`",
            ),
        ];
        for (what, bad, names) in cases {
            let records = [
                read_evidence(&text("e1")).expect("e1 reads"),
                read_evidence(&bad).unwrap_or_else(|refusal| panic!("{what}: {refusal}")),
            ];
            let refusal = check(&ir, &records).expect_err(what);
            let message = refusal.to_string();
            assert!(
                message.starts_with("evidence ") && message.contains(names),
                "{what}: error[{}]: {message}",
                refusal.code()
            );
            // And it cites the record it names, so a caller can say which file it came from.
            let cited: Vec<String> = refusal
                .evidence()
                .iter()
                .map(|id| format!("`{id}`"))
                .collect();
            assert_eq!(cited, [names], "{what}: {message}");
        }
    }

    const REDELIVERED: &str = "format: canon-evidence/1\nid: e1\nkind: k\nresult: pass\nsubject: a\n\
         subject_revision: r1\nobserved_at: 2026-10-04T12:00:00Z\n\
         upstream_revisions: {up: u1, down: d1}\nproducer: p1\n";

    const OUTCOME: &str = "format: canon-evidence/1\nid: o1\nkind: case_outcome\nresult: open\n\
         subject: a\nsubject_revision: r1\nchild_case: CHG-1\n";

    fn redelivery_ir() -> Ir {
        crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}, up: {}, down: {}}\n\
                 evidence_kinds: {k: {}, k2: {}, case_outcome: {}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles")
    }

    fn checked_set(texts: &[&str]) -> Result<Vec<EvidenceRecord>, Refusal> {
        let records = texts
            .iter()
            .map(|text| read_evidence(text))
            .collect::<Result<Vec<_>, _>>()?;
        check(&redelivery_ir(), &records)
    }

    /// Copies equal to the first count once, and the first copy in the order given is the one
    /// kept; the keys of `upstream_revisions` may be written in any order.
    #[test]
    fn copies_equal_to_the_first_count_once_as_the_first() {
        let reordered = REDELIVERED.replace("{up: u1, down: d1}", "{down: d1, up: u1}");
        let kept = checked_set(&[REDELIVERED, &reordered, REDELIVERED]).expect("equal copies");
        assert_eq!(kept.len(), 1);
        let order = |record: &EvidenceRecord| -> Vec<String> {
            record
                .upstream_revisions
                .ids()
                .map(|id| id.as_str().to_owned())
                .collect()
        };
        assert_eq!(order(&kept[0]), ["up", "down"]);
        let kept = checked_set(&[&reordered, REDELIVERED]).expect("equal copies");
        assert_eq!(kept.len(), 1);
        assert_eq!(order(&kept[0]), ["down", "up"]);
        let other = REDELIVERED.replace("id: e1", "id: e2");
        let kept = checked_set(&[REDELIVERED, &other, REDELIVERED, &other]).expect("distinct");
        let ids: Vec<&str> = kept.iter().map(|record| record.id.as_str()).collect();
        assert_eq!(ids, ["e1", "e2"]);
        let outcome = checked_set(&[OUTCOME, OUTCOME]).expect("an equal copy is one record");
        assert_eq!(outcome.len(), 1);
    }

    /// A copy that differs from the first in any one field is refused as `duplicate-identifier`
    /// naming and citing the record; `observed_at` is compared as written, so the same instant at
    /// another precision differs, and a field left out differs from one given.
    #[test]
    fn a_copy_that_differs_in_any_field_is_refused() {
        let message = "evidence `e1` is given more than once with different content";
        let outcome = OUTCOME.replace("id: o1", "id: e1");
        let with_item = format!("{REDELIVERED}item: t1\n");
        let cases: Vec<(&str, &str, String)> = vec![
            ("item", &with_item, with_item.replace("t1", "t2")),
            ("item left out", &with_item, REDELIVERED.to_owned()),
            ("result", REDELIVERED, REDELIVERED.replace("pass", "fail")),
            (
                "result left out",
                REDELIVERED,
                REDELIVERED.replace("result: pass\n", ""),
            ),
            (
                "kind",
                REDELIVERED,
                REDELIVERED.replace("kind: k\n", "kind: k2\n"),
            ),
            (
                "subject",
                REDELIVERED,
                REDELIVERED.replace("subject: a", "subject: up"),
            ),
            (
                "subject revision",
                REDELIVERED,
                REDELIVERED.replace("subject_revision: r1", "subject_revision: r2"),
            ),
            (
                "observed_at precision",
                REDELIVERED,
                REDELIVERED.replace("12:00:00Z", "12:00:00.000Z"),
            ),
            (
                "observed_at instant",
                REDELIVERED,
                REDELIVERED.replace("12:00:00Z", "12:00:01Z"),
            ),
            (
                "observed_at left out",
                REDELIVERED,
                REDELIVERED.replace("observed_at: 2026-10-04T12:00:00Z\n", ""),
            ),
            (
                "upstream revision",
                REDELIVERED,
                REDELIVERED.replace("down: d1", "down: d2"),
            ),
            (
                "upstream artifact left out",
                REDELIVERED,
                REDELIVERED.replace("{up: u1, down: d1}", "{up: u1}"),
            ),
            (
                "upstream revisions left out",
                REDELIVERED,
                REDELIVERED.replace("upstream_revisions: {up: u1, down: d1}\n", ""),
            ),
            ("producer", REDELIVERED, REDELIVERED.replace("p1", "p2")),
            (
                "producer left out",
                REDELIVERED,
                REDELIVERED.replace("producer: p1\n", ""),
            ),
            ("child case", &outcome, outcome.replace("CHG-1", "CHG-2")),
            (
                "child outcome",
                &outcome,
                outcome.replace("open", "accepted"),
            ),
        ];
        for (what, first, copy) in &cases {
            for texts in [[*first, copy.as_str()], [copy.as_str(), *first]] {
                let refusal = checked_set(&texts).expect_err(what);
                assert_eq!(
                    (refusal.code(), refusal.to_string().as_str()),
                    ("duplicate-identifier", message),
                    "{what}"
                );
                let cited: Vec<String> =
                    refusal.evidence().iter().map(|id| id.to_string()).collect();
                assert_eq!(cited, ["e1"], "{what}");
            }
        }
    }

    /// A copy that differs is refused at the place the id is checked: a record refused for an
    /// earlier reason is still refused for it, and so is any record before it in the order given.
    #[test]
    fn a_differing_copy_is_refused_where_the_id_is_checked() {
        let differing = REDELIVERED.replace("pass", "fail");
        let code = |texts: &[&str]| -> (&'static str, String) {
            let refusal = checked_set(texts).expect_err("refused");
            (refusal.code(), refusal.to_string())
        };
        let message = "evidence `e1` is given more than once with different content".to_owned();
        let bad_instant = differing.replace("2026-10-04T12:00:00Z", "yesterday");
        assert_eq!(code(&[REDELIVERED, &bad_instant]).0, "invalid-instant");
        let bad_kind = differing.replace("kind: k\n", "kind: z\n");
        assert_eq!(
            code(&[REDELIVERED, &bad_kind]),
            ("duplicate-identifier", message.clone()),
            "the id is checked before the kind is declared"
        );
        assert_eq!(
            code(&[&bad_kind, REDELIVERED]).0,
            "undeclared-evidence-kind"
        );
        let other = REDELIVERED
            .replace("id: e1", "id: e2")
            .replace("kind: k\n", "kind: z\n");
        assert_eq!(
            code(&[REDELIVERED, &other, &differing]).0,
            "undeclared-evidence-kind",
            "e2 comes before the copy of e1"
        );
        assert_eq!(
            code(&[REDELIVERED, &differing, &other]),
            ("duplicate-identifier", message),
            "the copy of e1 comes before e2"
        );
    }
}

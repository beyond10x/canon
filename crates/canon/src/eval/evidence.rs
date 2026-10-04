//! `canon-evidence/1` records as the evaluator reads and checks them. The type is
//! [`crate::model::EvidenceRecord`]; this file reads a record from text or a YAML value and checks
//! the set against the compiled protocol. Which records apply is decided afterwards, by the
//! exclusion stages (`binding.rs`, `freshness.rs`, `invalidation.rs`).

use std::collections::BTreeSet;

use serde_yaml_ng::Value;

use super::freshness::invalid_instant;
use super::read::{malformed, yaml};
use super::{Refusal, identifier, unsupported_format};
use crate::ir::Ir;
use crate::model::{EVIDENCE_FORMAT, EvidenceId, EvidenceRecord, one_line};

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

/// Checks each record in the order given, as the module docs of [`super`] describe. Every refusal
/// names the record by its id.
pub(super) fn check(ir: &Ir, evidence: &[EvidenceRecord]) -> Result<(), Refusal> {
    let mut seen = BTreeSet::new();
    for record in evidence {
        check_record(ir, record, &mut seen).map_err(|refusal| refusal.citing(&record.id))?;
    }
    Ok(())
}

/// One record of [`check`], refused without its citation, which [`check`] adds.
fn check_record<'a>(
    ir: &Ir,
    record: &'a EvidenceRecord,
    seen: &mut BTreeSet<&'a EvidenceId>,
) -> Result<(), Refusal> {
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
    if let Some(observed_at) = &record.observed_at
        && observed_at.seconds().is_none()
    {
        return Err(invalid_instant(
            &format!("evidence `{}` observed_at", one_line(record.id.as_str())),
            observed_at.as_str(),
        ));
    }
    if !seen.insert(&record.id) {
        return Err(Refusal::new(
            "duplicate-identifier",
            format!(
                "evidence `{}` is given more than once",
                one_line(record.id.as_str())
            ),
        ));
    }
    if !ir.evidence_kinds.contains_key(&record.kind) {
        return Err(Refusal::new(
            "undeclared-evidence-kind",
            format!(
                "evidence `{}` is of kind `{}`, which the protocol does not declare",
                one_line(record.id.as_str()),
                one_line(record.kind.as_str())
            ),
        ));
    }
    let mut upstream = BTreeSet::new();
    for artifact in record.upstream_revisions.ids() {
        if !ir.artifacts.contains_key(artifact) {
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
    Ok(())
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
        check(&ir(), &[record])
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
            check(&ir, &[record])
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

    /// `observed_at` is optional, and when given it is an instant; an explicit null or any other
    /// text is refused.
    #[test]
    fn observed_at_is_an_instant_when_given() {
        assert_eq!(checked("2026-10-04T12:00:00Z"), Ok(()));
        let refusal = checked("2026-10-04T12:00:00+02:00").expect_err("refused");
        assert_eq!(refusal.code(), "invalid-instant");
        assert_eq!(
            refusal.to_string(),
            "evidence `e1` observed_at `2026-10-04T12:00:00+02:00` is not an instant in UTC written YYYY-MM-DDTHH:MM:SSZ"
        );
        assert_eq!(
            checked("~").expect_err("null refused").code(),
            "malformed-input"
        );
    }

    /// Every refusal `check` gives names the record it refuses: among `e1` and a bad `e2`, each
    /// message names `e2` (an invalid id names the id it found, `e 2`; a repeated id is `e1`).
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
            ("repeated id", text("e1"), "`e1`"),
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
}

//! Adversary cases for story:conformance-suite: the `canon-requirements/1` catalogue and the checks
//! `run_registry_with_requirements` adds to the runner, driven through the library's public API.

use b10x_canon::conform::{
    ScenarioFile, parse_catalogue, run_registry, run_registry_with_requirements,
};

/// The fixture protocol of the unit's own requirement fixtures, read at run time.
fn protocol() -> String {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::read_to_string(root.join("fixtures/conformance/requirements/protocol.yaml"))
        .expect("fixture protocol reads")
}

fn file(name: &str, text: String) -> ScenarioFile {
    ScenarioFile {
        path: format!("dir/{name}.yaml"),
        text: Ok(text),
    }
}

fn catalogue(entries: &[(&str, &str)]) -> b10x_canon::model::RequirementCatalogue {
    let mut text = String::from("format: canon-requirements/1\nrequirements:\n");
    if entries.is_empty() {
        text = String::from("format: canon-requirements/1\nrequirements: {}\n");
    }
    for (id, status) in entries {
        text.push_str(&format!(
            "  {id}:\n    status: {status}\n    statement: s\n    source: d\n"
        ));
    }
    parse_catalogue(&text).expect("catalogue parses")
}

const CASE: &str = "      case:\n        format: canon-case/1\n        id: CHANGE-1\n        protocol: requirements-fixture\n        artifacts:\n          change: {revision: r1}\n";

const REVIEWED: &str = "    expect:\n      decision: |\n        {\n          \"claims\": {\n            \"change.reviewed\": {\n              \"value\": \"true\"\n            }\n          }\n        }\n";

fn review(id: &str) -> String {
    format!(
        "        - {{format: canon-evidence/1, id: {id}, kind: review, subject: change, subject_revision: r1}}\n"
    )
}

/// A scenario with one passing evaluate step (the unit's `covered/reviewed.yaml`, inlined).
fn passing(id: &str, covers: &str) -> String {
    format!(
        "format: canon-conformance/1\nid: {id}\ncovers: [{covers}]\nfixture: p.yaml\nsteps:\n  - id: reviewed\n    evaluate:\n{CASE}      evidence:\n{}{REVIEWED}",
        review("review-1")
    )
}

/// The same scenario with a second step whose expectation is wrong: the first step passes, the
/// scenario fails.
fn failing_late(id: &str, covers: &str) -> String {
    format!(
        "{}  - id: wrong\n    evaluate:\n{CASE}      evidence: []\n    expect:\n      refusal: not-what-happens\n",
        passing(id, covers)
    )
}

fn run(files: &[ScenarioFile], entries: &[(&str, &str)]) -> (String, bool) {
    let protocol = protocol();
    let report =
        run_registry_with_requirements(files, |_| Ok(protocol.clone()), &catalogue(entries));
    (report.render(), report.all_passed())
}

// ---------------------------------------------------------------------------------------------
// Catalogue parsing
// ---------------------------------------------------------------------------------------------

/// The module docs: a catalogue that does not parse "exits 1 with `error[catalogue]: <path>:
/// <why>`, the catalogue reported by kind and position only". A file given to `--requirements`
/// that is not a catalogue must not have its values echoed into the message.
#[test]
fn a_refused_catalogue_does_not_echo_its_values() {
    for source in [
        "format: SECRET-FORMAT-VALUE\nrequirements: {}\n",
        "format: canon-requirements/1\nrequirements:\n  SECRET-KEY-VALUE: {status: implemented, statement: s, source: d}\n",
    ] {
        let message = parse_catalogue(source).expect_err(source).to_string();
        assert!(
            !message.contains("SECRET"),
            "the refusal quotes the file's content: {message}"
        );
    }
}

#[test]
fn an_unknown_key_inside_a_requirement_is_refused() {
    let source = "format: canon-requirements/1\nrequirements:\n  CANON-A-001:\n    status: implemented\n    statement: s\n    source: d\n    owner: someone\n";
    let message = parse_catalogue(source).expect_err("unknown key refused").to_string();
    assert!(!message.contains("owner"), "{message}");
}

#[test]
fn malformed_ids_are_refused() {
    for id in ["'CANON-'", "'CANON-\u{200b}'", "canon-A-001", "' CANON-A-001'"] {
        let source = format!(
            "format: canon-requirements/1\nrequirements:\n  {id}: {{status: implemented, statement: s, source: d}}\n"
        );
        assert!(parse_catalogue(&source).is_err(), "{id} accepted");
    }
}

#[test]
fn a_duplicate_id_spelled_two_ways_is_refused() {
    let source = "format: canon-requirements/1\nrequirements:\n  CANON-A-001: {status: implemented, statement: s, source: d}\n  \"CANON-A-001\": {status: planned, statement: s, source: d}\n";
    let message = parse_catalogue(source).expect_err("duplicate refused").to_string();
    assert_eq!(message, "requirement `CANON-A-001` is listed more than once");
}

#[test]
fn null_or_missing_fields_are_refused() {
    for body in [
        "{status: implemented, statement: ~, source: d}",
        "{status: implemented, statement: s, source: ~}",
        "{status: ~, statement: s, source: d}",
        "{status: implemented, statement: s}",
        "{status: implemented, statement: '\t\n', source: d}",
    ] {
        let source =
            format!("format: canon-requirements/1\nrequirements:\n  CANON-A-001: {body}\n");
        assert!(parse_catalogue(&source).is_err(), "{body} accepted");
    }
    assert!(parse_catalogue("format: canon-requirements/1\nrequirements: ~\n").is_err());
    assert!(parse_catalogue("").is_err());
}

// ---------------------------------------------------------------------------------------------
// Coverage
// ---------------------------------------------------------------------------------------------

#[test]
fn a_scenario_failing_after_a_passing_step_covers_nothing() {
    let (text, passed) = run(
        &[file("a", failing_late("S", "CANON-A-001"))],
        &[("CANON-A-001", "implemented")],
    );
    assert!(!passed);
    assert_eq!(
        text,
        "failed: scenario `S` step `wrong`: expected refusal `not-what-happens`, the evaluation gave a decision\n\
         uncovered: requirement `CANON-A-001`\n\
         conform: 0 passed, 1 failed, 0 unreadable\n\
         requirements: 1 implemented, 0 covered, 0 planned\n"
    );
}

#[test]
fn a_planned_requirement_only_a_failing_scenario_covers_is_not_reported() {
    let (text, _) = run(
        &[file("a", failing_late("S", "CANON-A-001"))],
        &[("CANON-A-001", "planned")],
    );
    assert!(!text.contains("planned requirement"), "{text}");
    assert!(text.ends_with("requirements: 0 implemented, 0 covered, 1 planned\n"), "{text}");
}

#[test]
fn a_failing_scenario_covering_an_unlisted_requirement_is_reported() {
    let (text, passed) = run(&[file("a", failing_late("S", "CANON-Z-009"))], &[]);
    assert!(!passed);
    assert!(
        text.contains("unlisted: requirement `CANON-Z-009` covered by scenario `S`\n"),
        "{text}"
    );
}

#[test]
fn check_lines_follow_catalogue_then_registry_order() {
    let files = [
        file("a", passing("A", "CANON-P-001, CANON-U-002")),
        file("b", passing("B", "CANON-U-001")),
    ];
    let (text, passed) = run(
        &files,
        &[
            ("CANON-X-002", "implemented"),
            ("CANON-P-001", "planned"),
            ("CANON-X-001", "implemented"),
        ],
    );
    assert!(!passed);
    assert_eq!(
        text,
        "passed: scenario `A`\n\
         passed: scenario `B`\n\
         uncovered: requirement `CANON-X-002`\n\
         planned requirement `CANON-P-001` is covered\n\
         uncovered: requirement `CANON-X-001`\n\
         unlisted: requirement `CANON-U-002` covered by scenario `A`\n\
         unlisted: requirement `CANON-U-001` covered by scenario `B`\n\
         conform: 2 passed, 0 failed, 0 unreadable\n\
         requirements: 2 implemented, 0 covered, 1 planned\n"
    );
}

#[test]
fn a_passing_registry_with_the_flag_is_the_plain_report_plus_one_line() {
    let files = [file("a", passing("S", "CANON-A-001"))];
    let protocol = protocol();
    let plain = run_registry(&files, |_| Ok(protocol.clone())).render();
    assert_eq!(plain, "passed: scenario `S`\nconform: 1 passed, 0 failed, 0 unreadable\n");
    let (text, passed) = run(&files, &[("CANON-A-001", "implemented")]);
    assert!(passed);
    assert_eq!(
        text,
        format!("{plain}requirements: 1 implemented, 1 covered, 0 planned\n")
    );
}

// ---------------------------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------------------------

/// Two reviews, either order: one decision, nothing reported.
#[test]
fn reversing_equivalent_evidence_is_not_reported() {
    let text = format!(
        "format: canon-conformance/1\nid: S\ncovers: [CANON-A-001]\nfixture: p.yaml\nsteps:\n  - id: two\n    evaluate:\n{CASE}      evidence:\n{}{}{REVIEWED}",
        review("review-1"),
        review("review-2")
    );
    let (report, passed) = run(&[file("a", text)], &[("CANON-A-001", "implemented")]);
    assert!(passed, "{report}");
}

/// The unit's nondeterministic fixture, as a library input: the refusal code differs reversed.
#[test]
fn a_refusal_code_that_changes_when_reversed_is_reported_even_in_a_failing_scenario() {
    let text = format!(
        "format: canon-conformance/1\nid: S\ncovers: [CANON-A-001]\nfixture: p.yaml\nsteps:\n  - id: two\n    evaluate:\n{CASE}      evidence:\n        - {{format: canon-evidence/1, id: o, kind: opinion, subject: change, subject_revision: r1}}\n        - {{format: canon-evidence/1, id: r, kind: review, subject: change, subject_revision: r1, upstream_revisions: {{dataset: d1}}}}\n    expect:\n      refusal: something-else\n"
    );
    let (report, passed) = run(&[file("a", text)], &[("CANON-A-001", "implemented")]);
    assert!(!passed);
    assert!(
        report.contains("nondeterministic: scenario `S` step `two`: permuted differs at line 1\n"),
        "{report}"
    );
}

/// An `authority` list one entry of which cannot be written back as YAML is never evaluated, as
/// written or reversed: the step fails before the evaluator runs. There is no decision and no
/// refusal code to compare, so the determinism check has nothing to report; the reversed list
/// must not be reported as a different output just because the bad entry moved position.
#[test]
fn an_unevaluated_step_is_not_reported_nondeterministic() {
    let text = format!(
        "format: canon-conformance/1\nid: S\ncovers: [CANON-A-001]\nfixture: p.yaml\nsteps:\n  - id: auth\n    evaluate:\n{CASE}      evidence: []\n      authority:\n        - ? {{a: 1}}\n          : \"x\\x85y\"\n        - plain\n    expect:\n      refusal: malformed-input\n"
    );
    let (report, _) = run(&[file("a", text)], &[("CANON-A-001", "implemented")]);
    assert!(
        report.contains("cannot be serialized"),
        "probe precondition: the step must fail as unserializable\n{report}"
    );
    assert!(!report.contains("nondeterministic:"), "{report}");
}

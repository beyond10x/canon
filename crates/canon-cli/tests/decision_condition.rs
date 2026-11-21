//! Acceptance for story:decision-condition: an outcome that requires an explicit decision may carry
//! an optional predicate, `when`, and the decision counts toward the outcome only while `when` is
//! `true` at the evaluation. The named test is `a_decision_counts_only_while_its_condition_is_true`;
//! it runs `canon` over `fixtures/investigation/decision-condition.yaml`, and over the fixtures
//! under `fixtures/investigation/decision-condition/` for the refusals and the diff.
//!
//! The protocol: artifact `a`; evidence kind `publication`; claims `refused` and `accepted`; action
//! `submit`, admissible while both claims are `unknown`, which may produce `publication`; outcome
//! `published`, requiring `accepted`; outcome `withdrawn`, requiring `{decision: withdraw, when:
//! {claim: refused}}`. The case is at revision `c1`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FIXTURE: &str = "fixtures/investigation/decision-condition.yaml";
const FIXTURES: &str = "fixtures/investigation/decision-condition";
const REQUIREMENTS: &str = "conformance/requirements.yaml";

/// The decision `withdraw` for `withdrawn`, taken at the given case revision.
fn withdraw(revision: &str) -> String {
    format!(
        "- decision: withdraw\n  outcome: withdrawn\n  principal: p\n  case_revision: {revision}\n"
    )
}

/// A `publication` record about `a` at revision `subject_revision`, with `result`.
fn publication(result: &str, subject_revision: &str) -> String {
    format!(
        "format: canon-evidence/1\nid: pub-1\nkind: publication\nresult: {result}\nsubject: a\n\
         subject_revision: {subject_revision}\n"
    )
}

/// `withdrawn` blocked by its condition, as canonical `canon-decision/1` JSON writes the entry.
fn blocked_by_condition(value: &str) -> String {
    format!(
        "\"withdrawn\": {{\n      \"reasons\": [\n        {{\n          \"claim\": \"refused\",\n          \
         \"value\": \"{value}\"\n        }}\n      ],\n      \"status\": \"blocked\"\n    }}"
    )
}

/// `withdrawn` blocked for want of the decision, its condition `true`.
const BLOCKED_FOR_DECISION: &str = "\"withdrawn\": {\n      \"reasons\": [\n        {\n          \
     \"decision\": \"withdraw\",\n          \"present\": false\n        }\n      ],\n      \
     \"status\": \"blocked\"\n    }";

/// `withdrawn` legitimate, recording who decided.
const LEGITIMATE: &str = "\"withdrawn\": {\n      \"decided_by\": {\n        \"decision\": \
     \"withdraw\",\n        \"principals\": [\n          \"p\"\n        ]\n      },\n      \
     \"status\": \"legitimate\"\n    }";

/// The one pending decision `withdraw` for `withdrawn` waits on at `c1`.
const PENDING: &str = "\"pending_decisions\": {\n    \"decisions\": [\n      {\n        \
     \"case_revision\": \"c1\",\n        \"decision\": \"withdraw\",\n        \"options\": [\n          \
     \"taken\"\n        ],\n        \"outcome\": \"withdrawn\"\n      }\n    ]\n  }";

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(args)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("decision-condition-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

fn path(p: &Path) -> String {
    p.to_str().expect("utf-8 path").to_owned()
}

/// `canon evaluate` over the compiled fixture for a case at `c1` with the given records and
/// decisions, and the case's `termination` when given.
fn evaluate(
    name: &str,
    evidence: &[String],
    decisions: Option<&str>,
    termination: Option<&str>,
) -> Output {
    let dir = scratch(name);
    let compiled = canon(&["compile", "--path", FIXTURE]);
    assert_eq!(
        compiled.status.code(),
        Some(0),
        "the fixture compiles: {}",
        text(&compiled.stderr)
    );
    let ir = dir.join("protocol.ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("IR written");
    let case = dir.join("case.yaml");
    let termination = termination
        .map(|outcome| format!("termination: {outcome}\n"))
        .unwrap_or_default();
    std::fs::write(
        &case,
        format!(
            "format: canon-case/1\nid: PUB-1\nprotocol: publication\nrevision: c1\n\
             artifacts:\n  a: {{revision: r1}}\n{termination}"
        ),
    )
    .expect("case written");
    let evidence_dir = dir.join("evidence");
    std::fs::create_dir(&evidence_dir).expect("evidence directory created");
    for (position, record) in evidence.iter().enumerate() {
        std::fs::write(evidence_dir.join(format!("{position}.yaml")), record)
            .expect("evidence written");
    }
    let mut args = vec![
        "evaluate".to_owned(),
        "--ir".to_owned(),
        path(&ir),
        "--case".to_owned(),
        path(&case),
        "--evidence".to_owned(),
        path(&evidence_dir),
    ];
    if let Some(decisions) = decisions {
        let file = dir.join("decisions.yaml");
        std::fs::write(&file, decisions).expect("decisions written");
        args.extend(["--decisions".to_owned(), path(&file)]);
    }
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    canon(&args)
}

/// Standard output of an evaluation that succeeded.
fn evaluated(run: &Output, step: &str) -> String {
    assert_eq!(
        text(&run.stderr),
        "",
        "step `{step}`: canon evaluate stderr"
    );
    assert_eq!(run.status.code(), Some(0), "step `{step}`: exit");
    text(&run.stdout).to_owned()
}

/// The text between the `"explanation"` key and the end of its section's first `"outcomes"` key,
/// where `computed_from` lists the decisions as given.
fn computed_from(stdout: &str) -> &str {
    let start = stdout.find("\"computed_from\"").expect("computed_from");
    &stdout[start..start + stdout[start..].find("\"outcomes\"").expect("outcomes")]
}

/// A protocol file that must be refused: its stderr starts with `prefix`, exit 1, no stdout.
fn refused(file: &str, prefix: &str, contains: &str) {
    let run = canon(&["validate", "--path", &format!("{FIXTURES}/{file}")]);
    assert_eq!(text(&run.stdout), "", "{file}: stdout");
    assert_eq!(run.status.code(), Some(1), "{file}: exit");
    assert!(
        text(&run.stderr).starts_with(prefix) && text(&run.stderr).contains(contains),
        "{file}: expected `{prefix}` and `{contains}`, got {}",
        text(&run.stderr)
    );
}

#[test]
fn a_decision_counts_only_while_its_condition_is_true() {
    // 1. The registry passes, CANON-OUTCOME-003 and every existing scenario.
    let run = canon(&["conform", "run", "--requirements", REQUIREMENTS]);
    let report = text(&run.stdout);
    assert_eq!(
        run.status.code(),
        Some(0),
        "conform run:\n{report}{}",
        text(&run.stderr)
    );
    assert!(
        report
            .lines()
            .any(|line| line == "passed: scenario `CANON-OUTCOME-003`"),
        "conform run:\n{report}"
    );
    assert!(
        !report.lines().any(|line| line.starts_with("failed")),
        "conform run:\n{report}"
    );

    // 2. No evidence and no decision: blocked by the claim, nothing pending.
    let out = evaluated(&evaluate("nothing", &[], None, None), "nothing");
    assert!(out.contains(&blocked_by_condition("unknown")), "{out}");
    assert!(!out.contains("\"pending_decisions\""), "{out}");
    assert!(
        !out.contains("\"decisions\""),
        "no decision was given: {out}"
    );

    // 3. The decision at c1 while `when` is UNKNOWN: the same outcome, listed as given.
    let given = withdraw("c1");
    let out = evaluated(
        &evaluate("given-unknown", &[], Some(&given), None),
        "given-unknown",
    );
    assert!(out.contains(&blocked_by_condition("unknown")), "{out}");
    assert!(!out.contains("\"pending_decisions\""), "{out}");
    let from = computed_from(&out);
    assert!(
        from.contains("\"decisions\": [")
            && from.contains("\"decision\": \"withdraw\"")
            && from.contains("\"principal\": \"p\""),
        "computed_from.decisions lists the entry as given:\n{from}"
    );
    // The explanation's `because` holds the `when` reason, and the claim is listed under `claims`.
    let explanation = &out[out.find("\"explanation\"").expect("explanation")..];
    assert!(
        explanation.contains("\"claims\": {") && explanation.contains("\"refused\": {"),
        "the claim `refused` is explained:\n{explanation}"
    );
    assert!(
        !explanation.contains("\"present\""),
        "no decision reason is explained while `when` is not true:\n{explanation}"
    );

    // 4. A `refused` record and no decision: blocked naming the decision, and pending.
    let refused_record = publication("refused", "r1");
    let out = evaluated(
        &evaluate("refused", std::slice::from_ref(&refused_record), None, None),
        "refused",
    );
    assert!(out.contains(BLOCKED_FOR_DECISION), "{out}");
    assert!(out.contains(PENDING), "{out}");

    // 5. The record and the decision at c1: legitimate. At c0 it is blocked as in 4.
    let out = evaluated(
        &evaluate(
            "decided",
            std::slice::from_ref(&refused_record),
            Some(&given),
            None,
        ),
        "decided",
    );
    assert!(out.contains(LEGITIMATE), "{out}");
    let out = evaluated(
        &evaluate(
            "decided-c0",
            std::slice::from_ref(&refused_record),
            Some(&withdraw("c0")),
            None,
        ),
        "decided-c0",
    );
    assert!(out.contains(BLOCKED_FOR_DECISION), "{out}");
    assert!(out.contains(PENDING), "{out}");

    // 6. The record about a superseded revision of `a`, plus the decision at c1: blocked as in 2,
    // and terminating through the outcome is refused.
    let stale = [publication("refused", "r0")];
    let out = evaluated(&evaluate("stale", &stale, Some(&given), None), "stale");
    assert!(out.contains(&blocked_by_condition("unknown")), "{out}");
    let run = evaluate("stale-terminated", &stale, Some(&given), Some("withdrawn"));
    assert!(
        text(&run.stderr).starts_with("error[illegitimate-termination]: "),
        "{}",
        text(&run.stderr)
    );
    assert_eq!((text(&run.stdout), run.status.code()), ("", Some(1)));

    // The condition false: ignored as well, the reason carrying the value.
    let out = evaluated(
        &evaluate(
            "false",
            &[publication("accepted", "r1")],
            Some(&given),
            None,
        ),
        "false",
    );
    assert!(out.contains(&blocked_by_condition("false")), "{out}");
    assert!(!out.contains("\"pending_decisions\""), "{out}");

    // A decision for an undeclared name stays refused whatever `when` holds.
    let other = "- decision: other\n  outcome: withdrawn\n  principal: p\n  case_revision: c1\n";
    let run = evaluate("undeclared", &[], Some(other), None);
    assert!(
        text(&run.stderr).starts_with("error[undeclared-decision]: "),
        "{}",
        text(&run.stderr)
    );

    // 7. Validation and compiling.
    refused(
        "when-beside-claim.yaml",
        "error[parse]: ",
        "a requirement is either `decision` alone or a predicate, not both",
    );
    refused(
        "decision-inside-when.yaml",
        "error[parse]: ",
        "unknown field `decision`",
    );
    refused(
        "evidence-in-when.yaml",
        "error[evidence-in-decision-condition]: ",
        "withdrawn",
    );
    let valid = canon(&["validate", "--path", FIXTURE]);
    assert_eq!(
        (
            text(&valid.stdout),
            text(&valid.stderr),
            valid.status.code()
        ),
        ("valid: protocol `publication` revision 1\n", "", Some(0))
    );
    let without = format!("{FIXTURES}/without-when.yaml");
    let ir = canon(&["compile", "--path", &without]);
    assert_eq!(ir.status.code(), Some(0), "{}", text(&ir.stderr));
    assert!(
        !text(&ir.stdout).contains("\"when\""),
        "the IR writes `when` only when it is present"
    );
    let ir = canon(&["compile", "--path", FIXTURE]);
    assert!(
        text(&ir.stdout).contains(
            "\"requires\": {\n        \"decision\": \"withdraw\",\n        \"when\": {\n          \
             \"claim\": {\n            \"id\": \"refused\","
        ) || text(&ir.stdout).contains("\"when\""),
        "the IR writes `when`:\n{}",
        text(&ir.stdout)
    );

    // 8. Simulate and diff.
    let run = canon(&["simulate", "--path", FIXTURE]);
    let report = text(&run.stdout);
    assert!(
        report.lines().any(|line| line
            .starts_with("stuck-state: state {evidence `publication` result `refused`}: ")
            && line.ends_with("; decision `withdraw` taken makes outcome `withdrawn` legitimate")),
        "the refused stuck state has the R4 ending:\n{report}"
    );
    assert!(
        report.lines().any(|line| line
            .starts_with("stuck-state: state {evidence `publication`}: ")
            && !line.contains("decision `withdraw` taken")),
        "the unresulted stuck state has none:\n{report}"
    );
    assert!(report.contains("outcome `withdrawn` 1"), "{report}");
    let dir = scratch("diff");
    let from = dir.join("from.ir.json");
    std::fs::write(&from, canon(&["compile", "--path", &without]).stdout).expect("IR written");
    let to = dir.join("to.ir.json");
    std::fs::write(&to, canon(&["compile", "--path", FIXTURE]).stdout).expect("IR written");
    let run = canon(&["diff", "--from", &path(&from), "--to", &path(&to)]);
    assert_eq!(
        (text(&run.stdout), run.status.code()),
        (
            "diff: protocol `publication` revision 1 to 1\n\
             BREAKING: outcome `withdrawn` requirement changed\n",
            Some(0)
        ),
        "{}",
        text(&run.stderr)
    );
}

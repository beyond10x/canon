use super::*;

const HEAD: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n";

fn protocol(body: &str) -> Protocol {
    model::parse(&format!("{HEAD}{body}")).expect("parses")
}

fn sites(body: &str) -> Vec<String> {
    mutants(&protocol(body))
        .into_iter()
        .map(|mutant| format!("{} {}", mutant.operator, mutant.site))
        .collect()
}

/// The expected `outcomes` section of a decision in which outcome `o` is legitimate.
const LEGITIMATE: &str =
    "{\n  \"outcomes\": {\n    \"o\": {\n      \"status\": \"legitimate\"\n    }\n  }\n}\n";

/// The expected `outcomes` section of a decision in which outcome `o` is blocked by claim `c`
/// being unknown.
const BLOCKED: &str = "{\n  \"outcomes\": {\n    \"o\": {\n      \"reasons\": [\n        {\n          \
    \"claim\": \"c\",\n          \"value\": \"unknown\"\n        }\n      ],\n      \
    \"status\": \"blocked\"\n    }\n  }\n}\n";

/// A scenario file reading `p.yaml` with one evaluate step that expects `decision`, canonical JSON.
fn scenario(id: &str, evidence: &str, decision: &str) -> ScenarioFile {
    let indented: String = decision
        .lines()
        .map(|line| format!("        {line}\n"))
        .collect();
    let text = format!(
        "format: canon-conformance/1\nid: {id}\ncovers: []\nfixture: p.yaml\nsteps:\n  \
         - id: s\n    evaluate:\n      case: {{format: canon-case/1, id: c, protocol: p, \
         artifacts: {{a: {{revision: r1}}}}}}\n      evidence: [{evidence}]\n    expect:\n      \
         decision: |\n{indented}"
    );
    ScenarioFile {
        path: format!("{id}.yaml"),
        text: Ok(text),
    }
}

/// A scenario file whose fixture is `q.yaml`, importing `p.yaml`: it reads the protocol only as an
/// import. The test reader cannot read `q.yaml`, so the scenario fails wherever it runs.
fn importer(id: &str) -> ScenarioFile {
    ScenarioFile {
        path: format!("{id}.yaml"),
        text: Ok(format!(
            "format: canon-conformance/1\nid: {id}\ncovers: []\nfixture: q.yaml\n\
             imports: [p.yaml]\nsteps:\n  - id: compile\n    compile: {{}}\n    expect:\n      \
             canon-ir: x\n"
        )),
    }
}

fn record(kind: &str, result: &str) -> String {
    format!(
        "{{format: canon-evidence/1, id: {kind}-{result}, kind: {kind}, result: {result}, \
         subject: a, subject_revision: r1}}"
    )
}

/// Runs [`mutate`] with `p.yaml` the protocol file. The protocol file is read through the reader
/// too, which may refuse it; its text is never used, so the reader returns an empty one.
fn run(source: &str, files: &[ScenarioFile]) -> Result<Report, Refusal> {
    mutate(
        source,
        files,
        |path| path == "p.yaml",
        |path| match path {
            "p.yaml" => Ok(String::new()),
            _ => Err(FixtureError::Unreadable(format!("{path} is not here"))),
        },
    )
}

#[test]
fn a_protocol_file_the_reader_refuses_kills_nothing_and_is_refused() {
    let files = [scenario("both", "", LEGITIMATE)];
    let refusal = mutate(
        SOURCE,
        &files,
        |path| path == "p.yaml",
        |_| Err(FixtureError::Outside),
    )
    .expect_err("refused");
    assert_eq!(
        refusal.lines(),
        [(
            "scenario-fails",
            "the scenarios do not pass on the protocol itself: unreadable: both.yaml: fixture \
             `p.yaml` resolves outside the working directory"
                .to_owned()
        )]
    );
}

#[test]
fn sites_come_by_operator_then_identifier_then_position() {
    let body = "evidence_kinds: {k: {}, l: {}}\n\
        claims:\n\
        \x20 z: {true_when: {all: [{not: {all: [{evidence: {kind: k, result: x}}, {evidence: {kind: l}}]}}, {evidence: {kind: k, result: y}}]}}\n\
        \x20 a: {true_when: {evidence: {kind: k, result: y}}}\n\
        actions: {act: {precondition: {claim: a}, requires: [{capability: one}, {capability: two}]}}\n\
        outcomes: {o: {requires: {all: [{claim: z}]}}, d: {requires: {decision: n}}}\n";
    assert_eq!(
        sites(body),
        [
            "drop-conjunct claim `z` true_when all[0]",
            "drop-conjunct claim `z` true_when all[0].not.all[0]",
            "drop-conjunct claim `z` true_when all[0].not.all[1]",
            "drop-conjunct claim `z` true_when all[1]",
            "drop-conjunct outcome `o` requires all[0]",
            "all-to-any claim `z` true_when",
            "all-to-any claim `z` true_when all[0].not",
            "all-to-any outcome `o` requires",
            "remove-precondition action `act` precondition",
            "remove-capability action `act` requires[0] capability `one`",
            "remove-capability action `act` requires[1] capability `two`",
            "change-result claim `a` true_when result `y` to `x`",
            "change-result claim `z` true_when all[0].not.all[0] result `x` to `y`",
            "change-result claim `z` true_when all[1] result `y` to `x`",
        ]
    );
}

#[test]
fn obligations_and_order_forms() {
    // Order forms, children forms, input tests and claim tests are walked past; an obligation's
    // two predicates are its sites, discharge first.
    let body = "evidence_kinds: {k: {}}\n\
        claims: {c: {true_when: {all: [{first: {kind: k, result: x}}, {evidence: {kind: k, result: y}}]}}}\n\
        obligations: {ob: {discharged_when: {all: [{claim: c}]}, applicable_when: {all: [{claim: c}]}}}\n";
    assert_eq!(
        sites(body),
        [
            "drop-conjunct claim `c` true_when all[0]",
            "drop-conjunct claim `c` true_when all[1]",
            "drop-conjunct obligation `ob` discharged_when all[0]",
            "drop-conjunct obligation `ob` applicable_when all[0]",
            "all-to-any claim `c` true_when",
            "all-to-any obligation `ob` discharged_when",
            "all-to-any obligation `ob` applicable_when",
        ]
    );
}

#[test]
fn nothing_inside_a_descended_form_is_mutated() {
    // The `descended` form is walked past like every order form: its results are neither changed
    // nor counted among the results `change-result` offers, and the `all` members beside it are
    // still sites.
    let body = "artifacts: {a: {}}\nevidence_kinds: {k: {}}\n\
        claims: {c: {true_when: {all: [\
        {descended: {first: {kind: k, result: x}, then: {kind: k, result: z}, subject: a}}, \
        {evidence: {kind: k, result: y}}, {evidence: {kind: k, result: w}}]}}}\n";
    assert_eq!(
        sites(body),
        [
            "drop-conjunct claim `c` true_when all[0]",
            "drop-conjunct claim `c` true_when all[1]",
            "drop-conjunct claim `c` true_when all[2]",
            "all-to-any claim `c` true_when",
            "change-result claim `c` true_when all[1] result `y` to `w`",
            "change-result claim `c` true_when all[2] result `w` to `y`",
        ]
    );
}

#[test]
fn a_protocol_with_nothing_to_mutate_has_no_mutant() {
    assert!(
        sites("evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\n")
            .is_empty()
    );
}

#[test]
fn every_mutant_differs_from_the_protocol_and_is_written_as_a_protocol() {
    let body = "evidence_kinds: {k: {}}\n\
        claims: {c: {true_when: {all: [{evidence: {kind: k, result: x}}]}}, \
        d: {true_when: {evidence: {kind: k, result: y}}}}\n";
    let original = protocol(body);
    let found = mutants(&original);
    assert_eq!(found.len(), 4);
    for mutant in found {
        assert_ne!(mutant.protocol, original, "{}", mutant.site);
        assert_eq!(
            model::parse(&model::write(&mutant.protocol)).as_ref(),
            Ok(&mutant.protocol)
        );
    }
}

const SOURCE: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}}\nevidence_kinds: {k: {}, l: {}}\n\
    claims: {c: {true_when: {all: [{evidence: {kind: k, result: x}}, {evidence: {kind: l, result: x}}]}}}\n\
    outcomes: {o: {requires: {claim: c}}}\n";

#[test]
fn a_weakening_survivor_is_a_weak_point_and_a_witness_kills_it() {
    let legitimate = scenario(
        "both",
        &format!("{}, {}", record("k", "x"), record("l", "x")),
        LEGITIMATE,
    );
    let lacking = run(SOURCE, std::slice::from_ref(&legitimate)).expect("runs");
    assert_eq!(
        lacking.render(),
        "weak point: drop-conjunct claim `c` true_when all[0]: RELAXATION (weakening); no scenario fails\n\
         weak point: drop-conjunct claim `c` true_when all[1]: RELAXATION (weakening); no scenario fails\n\
         weak point: all-to-any claim `c` true_when: RELAXATION (weakening); no scenario fails\n\
         mutate: 3 mutants, 0 killed, 3 survived (3 weak points), 0 invalid\n"
    );
    assert_eq!(lacking.weak_points(), 3);
    let complete = run(
        SOURCE,
        &[
            legitimate,
            scenario("only-k", &record("k", "x"), BLOCKED),
            scenario("only-l", &record("l", "x"), BLOCKED),
        ],
    )
    .expect("runs");
    assert_eq!(
        complete.render(),
        "killed: drop-conjunct claim `c` true_when all[0]: RELAXATION (weakening); scenario `only-l` fails\n\
         killed: drop-conjunct claim `c` true_when all[1]: RELAXATION (weakening); scenario `only-k` fails\n\
         killed: all-to-any claim `c` true_when: RELAXATION (weakening); scenario `only-k` fails\n\
         mutate: 3 mutants, 3 killed, 0 survived (0 weak points), 0 invalid\n"
    );
    assert_eq!(complete.weak_points(), 0);
}

#[test]
fn a_mutant_the_validator_refuses_is_invalid_with_its_code() {
    // Dropping the one evidence match leaves `all: []`, which compiles; the invalidation rule on
    // `c` then names a claim that reads no evidence, which the validator refuses.
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\nevidence_kinds: {k: {}}\n\
        claims: {c: {true_when: {all: [{evidence: {kind: k}}]}}}\n\
        outcomes: {o: {requires: {claim: c}}}\n\
        invalidation: {r: {upstream: a, invalidates: [c]}}\n";
    let both = scenario(
        "k",
        "{format: canon-evidence/1, id: e, kind: k, subject: a, subject_revision: r1}",
        LEGITIMATE,
    );
    let report = run(source, &[both]).expect("runs");
    assert_eq!(
        report.render(),
        "invalid: drop-conjunct claim `c` true_when all[0]: inert-invalidation\n\
         survived: all-to-any claim `c` true_when: NO SEMANTIC CHANGE; no scenario fails\n\
         mutate: 2 mutants, 0 killed, 1 survived (0 weak points), 1 invalid\n"
    );
}

#[test]
fn an_emptied_all_compiles() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\nevidence_kinds: {k: {}}\n\
        claims: {c: {true_when: {all: [{evidence: {kind: k}}]}}}\n\
        outcomes: {o: {requires: {claim: c}}}\n";
    let empty = scenario("none", "", BLOCKED);
    let report = run(source, &[empty]).expect("runs");
    assert_eq!(
        report.render(),
        "killed: drop-conjunct claim `c` true_when all[0]: RELAXATION (weakening); scenario `none` fails\n\
         survived: all-to-any claim `c` true_when: NO SEMANTIC CHANGE; no scenario fails\n\
         mutate: 2 mutants, 1 killed, 1 survived (0 weak points), 0 invalid\n"
    );
}

/// Rule 3 of the correction after adversary pass 1: a set in which the protocol is only ever an
/// import is refused as `no-scenario`, and only a scenario's fixture is asked about.
#[test]
fn a_protocol_named_only_as_an_import_is_refused() {
    let mut asked = Vec::new();
    let refusal = mutate(
        SOURCE,
        &[importer("imports-p")],
        |path| {
            asked.push(path.to_owned());
            path == "p.yaml"
        },
        |_| Err(FixtureError::Unreadable("not read".to_owned())),
    )
    .expect_err("refused");
    assert_eq!(refusal, Refusal::NoScenario);
    assert_eq!(asked, ["q.yaml"]);
}

/// Rule 2: a scenario that reads the protocol only as an import runs neither in the baseline nor
/// against a mutant, so it neither refuses the set nor kills a mutant: the weak points stay.
#[test]
fn a_scenario_importing_the_protocol_does_not_run() {
    let legitimate = scenario(
        "both",
        &format!("{}, {}", record("k", "x"), record("l", "x")),
        LEGITIMATE,
    );
    let alone = run(SOURCE, std::slice::from_ref(&legitimate)).expect("runs");
    let with_importer = run(SOURCE, &[importer("a-importer"), legitimate]).expect("runs");
    assert_eq!(with_importer, alone);
    assert_eq!(with_importer.weak_points(), 3);
}

#[test]
fn refusals_name_their_cause() {
    let code = |result: Result<Report, Refusal>| result.expect_err("refused").lines()[0].0;
    assert_eq!(code(run("format: [", &[])), "parse");
    assert_eq!(code(run(SOURCE, &[])), "no-scenario");
    let unreadable = ScenarioFile {
        path: "x.yaml".to_owned(),
        text: Err("gone".to_owned()),
    };
    assert_eq!(code(run(SOURCE, &[unreadable])), "scenario-unreadable");
    let failing = scenario("wrong", "", LEGITIMATE);
    let refusal = run(SOURCE, &[failing]).expect_err("refused");
    assert_eq!(
        refusal.lines(),
        [(
            "scenario-fails",
            "the scenarios do not pass on the protocol itself: failed: scenario `wrong` step `s`: \
             canon-decision/1 section `outcomes` differs from the expectation at line 4"
                .to_owned()
        )]
    );
    let invalid = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        claims: {c: {true_when: {claim: missing}}}\n";
    let named = scenario("n", "", LEGITIMATE);
    assert_eq!(code(run(invalid, &[named])), "undeclared-claim");
}

#[test]
fn two_runs_give_one_report() {
    let files = [scenario(
        "both",
        &format!("{}, {}", record("k", "x"), record("l", "x")),
        LEGITIMATE,
    )];
    assert_eq!(run(SOURCE, &files), run(SOURCE, &files));
}

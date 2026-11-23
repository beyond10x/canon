//! The one integration-test binary of this crate (`[[test]] main` in its `Cargo.toml`,
//! `autotests = false`): every file `tests/<name>.rs` is the module `<name>` here, so a test
//! is named `<name>::<test>`. A file under `tests/` that is not declared below never runs;
//! `every_test_file_is_a_module_of_this_binary` fails when one is missing.

mod action_arguments;
mod adversary2_admissibility_authority;
mod adversary2_admissibility_reasons;
mod adversary2_binding_eval;
mod adversary2_check;
mod adversary2_conform_lines;
mod adversary2_decisions_docs;
mod adversary2_decisions_eval;
mod adversary2_decisions_messages;
mod adversary2_ess_gate;
mod adversary2_ess_model;
mod adversary2_explain;
mod adversary2_freshness_conform;
mod adversary2_freshness_eval;
mod adversary2_freshness_ir;
mod adversary2_freshness_validate;
mod adversary2_invalidation;
mod adversary2_ir_order;
mod adversary2_ir_text;
mod adversary2_obligations_ir;
mod adversary2_outcomes_polarity;
mod adversary2_review_hardening;
mod adversary2_skel_authority;
mod adversary2_skel_conform;
mod adversary2_skel_validate;
mod adversary2_source_model;
mod adversary2_subject_docs;
mod adversary2_subject_e2e;
mod adversary2_tvc_eval;
mod adversary3_simulate_recovery;
mod adversary_action_arguments;
mod adversary_admissibility_docs;
mod adversary_admissibility_reasons;
mod adversary_argument_validation;
mod adversary_authority_binding;
mod adversary_binding_docs;
mod adversary_binding_eval;
mod adversary_case_composition;
mod adversary_case_inputs;
mod adversary_check_slices;
mod adversary_check_space;
mod adversary_check_subject;
mod adversary_child_outcome_revision;
mod adversary_child_outcome_subject;
mod adversary_conform_steps;
mod adversary_conformance_suite;
mod adversary_decision_grounds;
mod adversary_decisions_docs;
mod adversary_decisions_eval;
mod adversary_descended;
mod adversary_ess_model_mutants;
mod adversary_eval_worker;
mod adversary_evidence_order;
mod adversary_evidence_order_after;
mod adversary_evidence_order_since;
mod adversary_evidence_order_subsecond;
mod adversary_explain_trace;
mod adversary_freshness_eval;
mod adversary_freshness_schema;
mod adversary_freshness_time;
mod adversary_gate_lanes;
mod adversary_independence_check;
mod adversary_independence_diff;
mod adversary_independence_probes;
mod adversary_invalidation;
mod adversary_ir_canonical;
mod adversary_missing_evidence;
mod adversary_obligations_docs;
mod adversary_obligations_order;
mod adversary_outcomes_reasons;
mod adversary_pending_decisions;
mod adversary_protocol_imports;
mod adversary_protocol_write;
mod adversary_review_hardening;
mod adversary_scenario_generation;
mod adversary_simulate;
mod adversary_simulate_child_outcome;
mod adversary_simulate_revise;
mod adversary_skel_lib;
mod adversary_source_model;
mod adversary_subject_probes;
mod adversary_subject_reasons;
mod adversary_supplied_evidence;
mod adversary_tvc_eval;
mod approvals_cite_evidence_by_id;
mod approvals_cite_evidence_replaced_contents;
mod argument_validation;
mod case_composition;
mod check_independence;
mod child_outcome_subject;
mod claim_direction;
mod decision_grounds;
mod ess_gate;
mod ess_model_matches;
mod evaluate_many_small_cases;
mod evidence_independence;
mod explain_subject;
mod invalidation_evaluation_cost;
mod judgement_marking;
mod missing_keys;
mod pending_decisions;
mod planning_only_classifier;
mod protocol_write;
mod review_depth_beyond_bound;
mod review_outcome_reasons_docs;
mod review_properties;
mod review_refusal_naming;
mod simulate_open_record_without_demand;
mod source_model;
mod strict_freshness;
mod strict_order_forms_read_trusted_records;
mod unsafe_confined;

/// Helpers shared by the modules above, holding no test of their own.
mod support {
    pub mod schemas;
}

/// The files under `tests` that this binary does not build, given the text of its `main.rs`: a
/// `tests/<name>.rs` not declared as `mod <name>;`, a `.rs` file in a subdirectory not declared as
/// `pub mod <name>;` in a block of `main.rs`, and a `main.rs` in a subdirectory.
fn undeclared_files(tests: &std::path::Path, main: &str) -> Vec<String> {
    let declared: std::collections::BTreeSet<&str> = main
        .lines()
        .filter_map(|line| line.strip_prefix("mod ")?.strip_suffix(';'))
        .collect();
    let nested: std::collections::BTreeSet<&str> = main
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod ")?.strip_suffix(';'))
        .collect();
    let mut undeclared = Vec::new();
    for entry in std::fs::read_dir(tests).expect("tests/ is readable") {
        let path = entry.expect("an entry of tests/ is readable").path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("a UTF-8 file name")
            .to_owned();
        if path.is_dir() {
            let mut pending = vec![path];
            while let Some(directory) = pending.pop() {
                for inner in std::fs::read_dir(&directory).expect("a subdirectory is readable") {
                    let inner = inner.expect("an entry is readable").path();
                    if inner.is_dir() {
                        pending.push(inner);
                        continue;
                    }
                    let file = inner.file_name().and_then(|file| file.to_str());
                    let module = file.and_then(|file| file.strip_suffix(".rs"));
                    if module.is_some_and(|module| module == "main" || !nested.contains(module)) {
                        let shown = inner.strip_prefix(tests).expect("an entry is under tests/");
                        undeclared.push(shown.display().to_string());
                    }
                }
            }
        } else if let Some(module) = name.strip_suffix(".rs")
            && module != "main"
            && !declared.contains(module)
        {
            undeclared.push(name);
        }
    }
    undeclared.sort();
    undeclared
}

/// Every `tests/<name>.rs` beside this file is declared above as `mod <name>;`, and no directory
/// under `tests/` holds a `main.rs` of its own.
#[test]
fn every_test_file_is_a_module_of_this_binary() {
    let tests = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let undeclared = undeclared_files(&tests, include_str!("main.rs"));
    assert!(
        undeclared.is_empty(),
        "these files under {} are not modules of the test binary `main` and never run; declare \
         each in tests/main.rs: {undeclared:?}",
        tests.display()
    );
}

/// A `.rs` file in a subdirectory of `tests/` builds only as a nested module, `pub mod <name>;`
/// in a block of this file; one that is not declared never builds (G6 of the wave 9 review).
#[test]
fn a_file_in_a_subdirectory_is_a_module_too() {
    let tests = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("layout-fixture");
    let _ = std::fs::remove_dir_all(&tests);
    std::fs::create_dir_all(tests.join("support/deep")).expect("the fixture directory is made");
    for file in [
        "top.rs",
        "support/declared.rs",
        "support/orphan.rs",
        "support/deep/buried.rs",
        "support/main.rs",
    ] {
        std::fs::write(tests.join(file), "").expect("the fixture file is written");
    }
    let main = "mod top;\nmod support {\n    pub mod declared;\n    pub mod buried;\n}\n";
    assert_eq!(
        undeclared_files(&tests, main),
        ["support/main.rs", "support/orphan.rs"]
    );
}

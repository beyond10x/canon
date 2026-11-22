//! The one integration-test binary of this crate (`[[test]] main` in its `Cargo.toml`,
//! `autotests = false`): every file `tests/<name>.rs` is the module `<name>` here, so a test
//! is named `<name>::<test>`. A file under `tests/` that is not declared below never runs;
//! `every_test_file_is_a_module_of_this_binary` fails when one is missing.

mod adversary2_compile;
mod adversary2_conform_exit;
mod adversary2_conform_paths;
mod adversary2_ess_commands;
mod adversary2_review_hardening;
mod adversary2_scenario_generation;
mod adversary2_skel_conform_cli;
mod adversary2_tvc_cli;
mod adversary2_validate;
mod adversary3_simulate_cli;
mod adversary_case_composition_cli;
mod adversary_check_slices_cli;
mod adversary_compile_bom;
mod adversary_conform_registry;
mod adversary_conformance_suite_cli;
mod adversary_ess_commands;
mod adversary_floor;
mod adversary_gate_ci_watch;
mod adversary_missing_evidence_cli;
mod adversary_mutate;
mod adversary_record_groups_cli;
mod adversary_review_hardening;
mod adversary_scenario_generation;
mod adversary_simulate_child_outcome_cli;
mod adversary_simulate_revise_cli;
mod adversary_skel_cli;
mod adversary_tvc_cli;
mod adversary_validate;
mod canon_check;
mod canon_mutate;
mod canon_simulate;
mod case_composition;
mod case_inputs;
mod check_history_modes;
mod ci_watch;
mod compile;
mod conform_registry;
mod conform_requirements;
mod conform_run;
mod decision_condition;
mod decision_outcomes;
mod ess_commands_match;
mod evaluator_skeleton;
mod evidence_file_named;
mod evidence_freshness;
mod evidence_revision_binding;
mod explanation;
mod floor;
mod imports;
mod invalidation_rules;
mod review_cli_contract;
mod revised_by;
mod scenario_generation;
mod semantic_diff;
mod simulate_child_outcome;
mod simulate_record_groups;
mod subject_bound_evidence_match;
mod supplied_evidence;
mod three_valued_claims;
mod usage_exit;
mod validate;

/// Every `tests/<name>.rs` beside this file is declared above as `mod <name>;`, and no directory
/// under `tests/` holds a `main.rs` of its own.
#[test]
fn every_test_file_is_a_module_of_this_binary() {
    let tests = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let declared: std::collections::BTreeSet<&str> = include_str!("main.rs")
        .lines()
        .filter_map(|line| line.strip_prefix("mod ")?.strip_suffix(';'))
        .collect();
    let mut undeclared = Vec::new();
    for entry in std::fs::read_dir(&tests).expect("tests/ is readable") {
        let path = entry.expect("an entry of tests/ is readable").path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("a UTF-8 file name")
            .to_owned();
        if path.is_dir() {
            if path.join("main.rs").exists() {
                undeclared.push(format!("{name}/main.rs"));
            }
        } else if let Some(module) = name.strip_suffix(".rs")
            && module != "main"
            && !declared.contains(module)
        {
            undeclared.push(name);
        }
    }
    undeclared.sort();
    assert!(
        undeclared.is_empty(),
        "these files under {} are not modules of the test binary `main` and never run; declare \
         each in tests/main.rs: {undeclared:?}",
        tests.display()
    );
}

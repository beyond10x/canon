//! The one integration-test binary of this crate (`[[test]] main` in its `Cargo.toml`,
//! `autotests = false`): every file `tests/<name>.rs` is the module `<name>` here, so a test
//! is named `<name>::<test>`. A file under `tests/` that is not declared below never runs;
//! `every_test_file_is_a_module_of_this_binary` fails when one is missing.

mod adversary;
mod conformance;
mod model_matches;

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

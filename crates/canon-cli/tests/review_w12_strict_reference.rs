//! Review of wave 2026-11-22-w12, story:freshness-needs-trusted-timestamps: the generated CLI
//! reference (`website/docs/reference/cli.md`, `task docs-generate`) writes each option of
//! `canon evaluate` in its options table. `--strict` takes an optional value (`num_args = 0..=1`),
//! which `canon evaluate --help` writes `--strict [<SKEW>]`; the table must not tell a reader the
//! value is required.

use std::path::PathBuf;

fn reference() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::read_to_string(root.join("website/docs/reference/cli.md")).expect("the page reads")
}

#[test]
fn the_reference_table_writes_the_strict_value_as_optional_as_the_help_does() {
    let page = reference();
    assert!(
        page.contains("      --strict [<SKEW>]\n"),
        "the help block of the page writes the optional value"
    );
    let rows: Vec<&str> = page
        .lines()
        .filter(|line| line.starts_with("| `--strict"))
        .collect();
    assert_eq!(
        rows.len(),
        1,
        "one table row for --strict: {rows:?}"
    );
    assert!(
        rows[0].starts_with("| `--strict [<SKEW>]` |"),
        "the options table writes a required value where the help writes an optional one: {}",
        rows[0]
    );
}

//! The generated CLI reference (`website/docs/reference/cli.md`, `task docs-generate`) writes each
//! option of a command twice: in its options table and in the help block below it. The table
//! writes an option's value as the help does (review of wave 2026-11-22-w12,
//! story:freshness-needs-trusted-timestamps): `--strict` of `canon evaluate` takes an optional
//! value (`num_args = 0..=1`), which the help writes `--strict [<SKEW>]`, so the table must not
//! tell a reader the value is required. Every row of every command is held to its help line.

use std::path::PathBuf;

fn reference() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::read_to_string(root.join("website/docs/reference/cli.md")).expect("the page reads")
}

/// `(command section, written option)` of every options-table row the help block of the same
/// section does not write as an option line (`      --x <X>` or `  -x, --x <X>`).
fn rows_unlike_the_help(page: &str) -> (usize, Vec<(String, String)>) {
    let mut rows = 0;
    let mut unlike = Vec::new();
    for section in page.split("\n## ").skip(1) {
        let title = section.lines().next().unwrap_or_default().to_owned();
        let help: Vec<&str> = section
            .split("```text\n")
            .nth(1)
            .unwrap_or_default()
            .lines()
            .map(str::trim_start)
            .collect();
        // Only the rows of an options table, the one under `| Option | Required | ...`.
        let mut options = false;
        for line in section.lines() {
            if line.starts_with("| Option |") {
                options = true;
            }
            if !line.starts_with('|') {
                options = false;
            }
            let Some(rest) = line.strip_prefix("| `").filter(|_| options) else {
                continue;
            };
            let Some((written, _)) = rest.split_once("` |") else {
                continue;
            };
            rows += 1;
            let shown = help.iter().any(|option| {
                *option == written
                    || option
                        .split_once(", ")
                        .is_some_and(|(short, long)| short.starts_with('-') && long == written)
            });
            if !shown {
                unlike.push((title.clone(), written.to_owned()));
            }
        }
    }
    (rows, unlike)
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
    assert_eq!(rows.len(), 1, "one table row for --strict: {rows:?}");
    assert!(
        rows[0].starts_with("| `--strict [<SKEW>]` |"),
        "the options table writes a required value where the help writes an optional one: {}",
        rows[0]
    );
}

#[test]
fn every_reference_table_row_writes_its_option_as_the_help_does() {
    let (rows, unlike) = rows_unlike_the_help(&reference());
    assert!(rows > 20, "the page has its options tables: {rows} rows");
    assert!(
        unlike.is_empty(),
        "options-table rows the help block of their command writes otherwise: {unlike:?}"
    );
}

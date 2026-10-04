#![forbid(unsafe_code)]

//! `canon-docs`: generates every page of the Canon documentation site that can be derived from
//! the repository, as Markdown under `website/docs/reference/` (and the `protocol/1` JSON Schema
//! under `website/static/schemas/`). The site itself is built by Docusaurus in `website/`; nothing
//! here builds or publishes it. The landing inputs under `website/data/` include the
//! `b10x-status/1` status document, built from `website/status.yaml` and the AEP story statuses.
//! `generate --check` writes nothing and fails when any generated
//! file differs from what the repository would generate now.

mod json;
mod landing;
mod md;
mod pages;
mod source;
mod status;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "canon-docs",
    about = "Generate the derived pages of the Canon documentation site"
)]
struct Cli {
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Write every generated page; with --check, write nothing and fail on any difference.
    Generate {
        /// The repository root.
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Compare instead of writing; exit 1 when a generated file is missing, differs or is stale.
        #[arg(long)]
        check: bool,
        /// The `canon` binary whose runs are recorded; defaults to the `canon` built beside this
        /// binary (`cargo build -p canon-cli -p canon-docs`).
        #[arg(long)]
        canon: Option<PathBuf>,
    },
    /// Write `.well-known/b10x-site.json` into a built site, binding it to the commit it was
    /// built from, for the project-site publisher.
    SiteManifest {
        /// The built site directory.
        #[arg(long)]
        out: PathBuf,
        /// The full Git revision the site was built from.
        #[arg(long)]
        commit: String,
    },
}

/// The path the site is served under.
const BASE_URL: &str = "/canon/";

/// A nonzero full Git revision.
fn valid_commit(commit: &str) -> bool {
    commit.len() == 40
        && commit.bytes().all(|b| b.is_ascii_hexdigit())
        && commit.bytes().any(|b| b != b'0')
}

/// The publication provenance the project-site publisher refuses a mismatch on.
fn site_manifest(commit: &str) -> String {
    json::obj([
        ("schema", json::str("b10x-project-site/v1")),
        ("repository", json::str("canon")),
        ("commit", json::str(commit)),
        ("baseUrl", json::str(BASE_URL)),
    ])
    .pretty()
}

fn write_site_manifest(out: &Path, commit: &str) -> Result<(), String> {
    if !valid_commit(commit) {
        return Err("commit must be a nonzero full Git revision".to_owned());
    }
    if !out.join("index.html").is_file() {
        return Err(format!("{} holds no built site", out.display()));
    }
    let dir = out.join(".well-known");
    fs::create_dir_all(&dir).map_err(|error| format!("creating {}: {error}", dir.display()))?;
    let path = dir.join("b10x-site.json");
    fs::write(&path, site_manifest(commit))
        .map_err(|error| format!("writing {}: {error}", path.display()))?;
    println!("canon-docs: wrote {}", path.display());
    Ok(())
}

/// Why a file on disk does not match the generation.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Drift {
    Missing(String),
    Differs(String),
    /// A file that carries the marker but is no longer generated.
    Stale(String),
}

impl std::fmt::Display for Drift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Drift::Missing(path) => write!(f, "missing: {path}"),
            Drift::Differs(path) => write!(f, "differs: {path}"),
            Drift::Stale(path) => write!(f, "stale: {path}"),
        }
    }
}

/// The generated files currently on disk, keyed by path relative to the root: every file of the
/// landing data directory, which `canon-docs` owns whole, and every file in the other generated
/// directories whose text carries the marker.
fn generated_on_disk(root: &Path) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    if let Ok(entries) = fs::read_dir(root.join(landing::DATA_DIR)) {
        for entry in entries.flatten() {
            let relative = format!(
                "{}/{}",
                landing::DATA_DIR,
                entry.file_name().to_string_lossy()
            );
            found.insert(
                relative,
                fs::read_to_string(entry.path()).unwrap_or_default(),
            );
        }
    }
    for dir in [pages::REFERENCE, pages::SCHEMA_DIR] {
        let Ok(entries) = fs::read_dir(root.join(dir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let relative = format!("{dir}/{}", entry.file_name().to_string_lossy());
            if let Ok(text) = fs::read_to_string(entry.path())
                && text.contains(md::MARKER)
            {
                found.insert(relative, text);
            }
        }
    }
    found
}

/// Whether a line opens an admonition with its title written after a space (`:::note Title`),
/// the form `^:::[a-z]+ +\S` that MDX renders as raw text; the title belongs in brackets.
fn is_raw_admonition(line: &str) -> bool {
    let Some(rest) = line.strip_prefix(":::") else {
        return false;
    };
    let keyword = rest.bytes().take_while(u8::is_ascii_lowercase).count();
    let after = &rest[keyword..];
    let spaces = after.bytes().take_while(|b| *b == b' ').count();
    keyword > 0
        && spaces > 0
        && after[spaces..]
            .chars()
            .next()
            .is_some_and(|c| !c.is_whitespace())
}

/// Every raw admonition line in the documentation sources, as `path:line: text`, outside fenced
/// code, in sorted path order.
fn raw_admonitions(root: &Path) -> Vec<String> {
    fn files(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files(&path, out);
            } else if path
                .extension()
                .is_some_and(|ext| ext == "md" || ext == "mdx")
            {
                out.push(path);
            }
        }
    }
    let mut paths = Vec::new();
    files(&root.join("website/docs"), &mut paths);
    paths.sort();
    let mut found = Vec::new();
    for path in paths {
        let text = fs::read_to_string(&path).unwrap_or_default();
        let shown = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let mut in_fence = false;
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("```") {
                in_fence = !in_fence;
            } else if !in_fence && is_raw_admonition(line) {
                found.push(format!("{shown}:{}: {line}", index + 1));
            }
        }
    }
    found
}

/// Compares what would be generated with what is on disk.
fn drift(
    expected: &BTreeMap<String, String>,
    on_disk: &BTreeMap<String, String>,
    read: impl Fn(&str) -> Option<String>,
) -> Vec<Drift> {
    let mut found = Vec::new();
    for (path, text) in expected {
        match read(path) {
            None => found.push(Drift::Missing(path.clone())),
            Some(actual) if actual != *text => found.push(Drift::Differs(path.clone())),
            Some(_) => {}
        }
    }
    for path in on_disk.keys() {
        if !expected.contains_key(path) {
            found.push(Drift::Stale(path.clone()));
        }
    }
    found
}

/// Every generated file: the pages, and the landing inputs, recorded with the `canon` binary.
fn expected(root: &Path, canon: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut files = pages::all(root)?;
    files.insert(
        format!("{}/{}", landing::DATA_DIR, landing::GRAPH_FILE),
        landing::investigation_graph(root)?,
    );
    files.insert(
        format!("{}/{}", landing::DATA_DIR, status::FILE),
        status::generate(root)?,
    );
    let version = landing::run_canon(canon, root, &["--version"])?.output;
    files.insert(
        format!("{}/{}", landing::DATA_DIR, landing::TERMINAL_FILE),
        landing::terminal(|args| landing::run_canon(canon, root, args), &version)?,
    );
    Ok(files)
}

/// The `canon` binary built beside this one.
fn sibling_canon() -> Result<PathBuf, String> {
    let me = std::env::current_exe().map_err(|error| format!("locating canon-docs: {error}"))?;
    let canon = me.with_file_name(format!("canon{}", std::env::consts::EXE_SUFFIX));
    if canon.is_file() {
        Ok(canon)
    } else {
        Err(format!(
            "no canon binary at {}; build it with `cargo build -p canon-cli` or pass --canon",
            canon.display()
        ))
    }
}

fn generate(root: &Path, check: bool, canon: Option<PathBuf>) -> Result<bool, String> {
    let canon = match canon {
        Some(canon) => canon,
        None => sibling_canon()?,
    };
    let expected = expected(root, &canon)?;
    let on_disk = generated_on_disk(root);
    let read = |path: &str| fs::read_to_string(root.join(path)).ok();
    if check {
        let found = drift(&expected, &on_disk, read);
        for problem in &found {
            eprintln!("canon-docs: {problem}");
        }
        if found.is_empty() {
            println!("canon-docs: {} generated files are current", expected.len());
        } else {
            eprintln!("canon-docs: run `canon-docs generate` and commit the result");
        }
        let raw = raw_admonitions(root);
        for line in &raw {
            eprintln!(
                "canon-docs: raw admonition, write the title in brackets (`:::note[Title]`): {line}"
            );
        }
        return Ok(found.is_empty() && raw.is_empty());
    }
    for (path, text) in &expected {
        let path = root.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("creating {}: {error}", parent.display()))?;
        }
        if fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
            fs::write(&path, text)
                .map_err(|error| format!("writing {}: {error}", path.display()))?;
        }
    }
    for path in on_disk.keys().filter(|path| !expected.contains_key(*path)) {
        fs::remove_file(root.join(path)).map_err(|error| format!("removing {path}: {error}"))?;
    }
    println!("canon-docs: wrote {} generated files", expected.len());
    Ok(true)
}

fn main() -> ExitCode {
    let result = match Cli::parse().command {
        Action::Generate { root, check, canon } => generate(&root, check, canon),
        Action::SiteManifest { out, commit } => write_site_manifest(&out, &commit).map(|()| true),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(problem) => {
            eprintln!("canon-docs: error: {problem}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Read at run time, not compile time: a test binary reused from a shared build directory must
    /// still read this tree.
    fn repository_root() -> PathBuf {
        let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
        PathBuf::from(manifest_dir)
            .join("../..")
            .canonicalize()
            .expect("repository root exists")
    }

    fn generated() -> BTreeMap<String, String> {
        pages::all(&repository_root()).expect("the repository generates")
    }

    #[test]
    fn every_generated_file_carries_the_marker_near_its_top() {
        for (path, text) in generated() {
            let head: String = text.lines().take(3).collect::<Vec<_>>().join("\n");
            assert!(head.contains(md::MARKER), "{path}: {head}");
        }
    }

    #[test]
    fn the_committed_pages_are_current() {
        let root = repository_root();
        // The landing inputs need a built `canon`; `generate --check` covers them.
        let on_disk: BTreeMap<String, String> = generated_on_disk(&root)
            .into_iter()
            .filter(|(path, _)| !path.starts_with(landing::DATA_DIR))
            .collect();
        let found = drift(&generated(), &on_disk, |path| {
            fs::read_to_string(root.join(path)).ok()
        });
        assert!(found.is_empty(), "run `task docs-generate`: {found:?}");
    }

    #[test]
    fn the_site_manifest_names_the_project_site_and_its_commit() {
        let commit = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(
            site_manifest(commit),
            format!(
                "{{\n  \"schema\": \"b10x-project-site/v1\",\n  \"repository\": \"canon\",\n  \
                 \"commit\": \"{commit}\",\n  \"baseUrl\": \"/canon/\"\n}}\n"
            )
        );
        assert!(valid_commit(commit));
        assert!(!valid_commit(&"0".repeat(40)));
        assert!(!valid_commit("abc123"));
        assert!(!valid_commit(&"g".repeat(40)));
    }

    #[test]
    fn raw_admonitions_are_the_keyword_space_title_form_only() {
        for raw in [":::note Title", ":::caution  Two spaces", ":::shipped Done"] {
            assert!(is_raw_admonition(raw), "{raw}");
        }
        for fine in [
            ":::note[Title]",
            ":::shipped[Claim evaluation]",
            ":::",
            ":::note",
            ":::note ",
            "::: note Title",
            ":::Note Title",
            "text :::note Title",
        ] {
            assert!(!is_raw_admonition(fine), "{fine}");
        }
    }

    #[test]
    fn the_documentation_has_no_raw_admonition() {
        assert_eq!(raw_admonitions(&repository_root()), Vec::<String>::new());
    }

    #[test]
    fn drift_names_missing_changed_and_stale_files() {
        let expected = BTreeMap::from([
            ("a.md".to_owned(), "a".to_owned()),
            ("b.md".to_owned(), "b".to_owned()),
            ("c.md".to_owned(), "c".to_owned()),
        ]);
        let disk = BTreeMap::from([
            ("b.md".to_owned(), "b, edited by hand".to_owned()),
            ("c.md".to_owned(), "c".to_owned()),
            ("old.md".to_owned(), "old".to_owned()),
        ]);
        let found = drift(&expected, &disk, |path| disk.get(path).cloned());
        assert_eq!(
            found,
            vec![
                Drift::Missing("a.md".to_owned()),
                Drift::Differs("b.md".to_owned()),
                Drift::Stale("old.md".to_owned()),
            ]
        );
    }

    #[test]
    fn the_cli_reference_lists_every_command_from_the_definition() {
        let cli = &generated()["website/docs/reference/cli.md"];
        for command in [
            "canon validate",
            "canon compile",
            "canon conform",
            "canon conform run",
        ] {
            assert!(cli.contains(&format!("## `{command}`")), "{command}");
        }
        assert!(cli.contains("`--scenarios <SCENARIOS>`"));
        assert!(cli.contains("`conformance/scenarios`"));
    }

    #[test]
    fn the_protocol_reference_covers_every_section_of_the_model() {
        let protocol = &generated()["website/docs/reference/protocol.md"];
        for key in [
            "format",
            "protocol",
            "artifacts",
            "evidence_kinds",
            "claims",
            "obligations",
            "actions",
            "outcomes",
            "precondition",
            "may_produce",
            "true_when",
        ] {
            assert!(protocol.contains(&format!("| `{key}` |")), "{key}");
        }
        for key in ["all", "any", "not", "evidence", "claim", "is"] {
            assert!(
                protocol.contains(&format!("| `{key}` |")),
                "predicate key {key}"
            );
        }
    }

    #[test]
    fn the_schema_requires_the_format_and_reads_predicates_as_one_of_five_forms() {
        let schema = &generated()["website/static/schemas/protocol-1.schema.json"];
        assert!(schema.contains("\"const\": \"protocol/1\""));
        let compact: String = schema.chars().filter(|c| !c.is_whitespace()).collect();
        for key in ["all", "any", "not", "evidence", "claim"] {
            assert!(
                compact.contains(&format!("\"required\":[\"{key}\"]")),
                "{key}"
            );
        }
        assert!(compact.contains("\"is\":{\"$ref\":\"#/$defs/Truth\"}"));
        assert!(schema.contains("\"$ref\": \"#/$defs/Predicate\""));
    }

    /// `canon-authority/1` has a section and a schema, and both say what Canon refuses as given
    /// twice: a capability decided twice, whatever the decision, which `uniqueItems` cannot say.
    #[test]
    fn the_authority_document_says_a_capability_decided_twice_is_refused() {
        let files = generated();
        let documents = &files["website/docs/reference/documents.md"];
        let rule = "A capability decided twice is refused, whatever the decision; the JSON Schema \
                    cannot express that, and refuses only an entry repeated exactly.";
        assert!(documents.contains("\n## `canon-authority/1`\n"));
        assert!(documents.contains(rule));
        assert!(!documents.contains("an entry given twice is refused"));
        let schema = &files["website/static/schemas/authority-1.schema.json"];
        assert!(schema.contains(rule));
        assert!(schema.contains("\"enum\": [\n        \"granted\",\n        \"denied\"\n      ]"));
    }

    #[test]
    fn the_properties_document_has_a_section_and_a_schema() {
        let files = generated();
        let documents = &files["website/docs/reference/documents.md"];
        assert!(documents.contains("\n## `canon-properties/1`\n"));
        assert!(documents.contains("Read by `canon check --properties`."));
        for key in ["subject", "independent_of", "action", "outcome", "claim"] {
            assert!(documents.contains(&format!("| `{key}` |")), "{key}");
        }
        let schema = &files["website/static/schemas/properties-1.schema.json"];
        assert!(schema.contains("\"const\": \"canon-properties/1\""));
        let compact: String = schema.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(compact.contains("\"required\":[\"format\",\"protocol\"]"));
        assert!(compact.contains("\"required\":[\"subject\",\"independent_of\"]"));
        for key in ["action", "outcome", "claim"] {
            assert!(
                compact.contains(&format!("\"required\":[\"{key}\"]")),
                "{key}"
            );
        }
    }

    #[test]
    fn the_example_is_compiled_by_canon_itself() {
        let example = &generated()["website/docs/reference/investigation-example.md"];
        assert!(example.contains("\"format\": \"canon-ir/1\""));
        assert!(example.contains("`duplicate-identifier`"));
        assert!(example.contains("`undeclared-claim`"));
    }
}

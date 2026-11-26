//! story:trigger-rule-publication, `canon trigger check-set --path <rule> ... --published <index>
//! --protocols <file>` and `canon trigger digest --path <rule>` against the command line:
//! `stale@1` and `stale@2` in one set pass and print the set's index, whatever the order of the
//! paths; `stale@1` with its cap edited, against an index holding `stale@1`, fails
//! `key-republished` naming the key; the unedited document, and the same rule written another way,
//! pass; a rule in `auto` over a guarded protocol fails `guarded-auto`; the digest is the one the
//! index holds, the same for the rule written another way, another for the edited rule. What each
//! failure means is the conformance scenario `conformance/scenarios/trigger-rule-publication.yaml`.

use std::path::PathBuf;
use std::process::{Command, Output};

/// Read at run time: a test binary reused from a shared build directory must read this tree.
fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

const DIR: &str = "fixtures/trigger/publication";

/// The digest of `stale-1.yaml`: `sha256:` and the SHA-256 of the RFC 8785 JSON of its
/// canonical form.
const STALE_1: &str = "sha256:2785572303ce15cfe7a3c5f58c846746fc1466b4c78b3980edde1944335c62af";
/// The digest of `stale-2.yaml`.
const STALE_2: &str = "sha256:6c9ce26ad1108a304b9ea7201dfa7ccd305a7d9550a4dd624bd6706cac64dbc8";
/// The digest of `stale-1-edited.yaml`, `stale-1.yaml` with the cap 20.
const STALE_1_EDITED: &str =
    "sha256:0c89665900fe25561fdca2fa498c32028fdbf9d408ae03de2534a821bfdeb850";

fn canon(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(arguments)
        .output()
        .expect("canon runs")
}

/// `canon trigger check-set` over the rule files, the index and `protocols.json`, each in
/// [`DIR`].
fn check_set(rules: &[&str], published: &str) -> Output {
    let mut arguments = vec!["trigger".to_owned(), "check-set".to_owned()];
    for rule in rules {
        arguments.push("--path".to_owned());
        arguments.push(format!("{DIR}/{rule}"));
    }
    arguments.push("--published".to_owned());
    arguments.push(format!("{DIR}/{published}"));
    arguments.push("--protocols".to_owned());
    arguments.push(format!("{DIR}/protocols.json"));
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    canon(&arguments)
}

/// The `canon-trigger-index/1` document of the keys and digests, in key order.
fn index(entries: &[(&str, &str)]) -> String {
    let rules: Vec<String> = entries
        .iter()
        .map(|(key, digest)| format!("    \"{key}\": \"{digest}\""))
        .collect();
    format!(
        "{{\n  \"format\": \"canon-trigger-index/1\",\n  \"rules\": {{\n{}\n  }}\n}}\n",
        rules.join(",\n")
    )
}

#[test]
fn two_versions_of_one_rule_pass_in_one_set_and_print_its_index() {
    let first = check_set(&["stale-1.yaml", "stale-2.yaml"], "nothing-published.yaml");
    let reversed = check_set(&["stale-2.yaml", "stale-1.yaml"], "nothing-published.yaml");
    for output in [&first, &reversed] {
        assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
        assert_eq!(text(&output.stderr), "");
    }
    assert_eq!(
        text(&first.stdout),
        index(&[("stale@1", STALE_1), ("stale@2", STALE_2)])
    );
    assert_eq!(first.stdout, reversed.stdout);
}

#[test]
fn an_edited_published_rule_fails_key_republished_naming_the_key() {
    let output = check_set(
        &["stale-1-edited.yaml", "stale-2.yaml"],
        "stale-1-published.yaml",
    );
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stdout));
    assert_eq!(text(&output.stdout), "");
    let stderr = text(&output.stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines.len(), 1, "{stderr}");
    assert!(
        lines[0].starts_with("error[key-republished]: stale@1: "),
        "{stderr}"
    );
    assert!(lines[0].contains(STALE_1), "{stderr}");
    assert!(lines[0].contains(STALE_1_EDITED), "{stderr}");
}

#[test]
fn the_unedited_rule_and_the_rule_written_another_way_pass_against_the_index() {
    for rule in ["stale-1.yaml", "stale-1-reformatted.yaml"] {
        let output = check_set(&[rule], "stale-1-published.yaml");
        assert_eq!(
            output.status.code(),
            Some(0),
            "{rule}: {}",
            text(&output.stderr)
        );
        assert_eq!(
            text(&output.stdout),
            index(&[("stale@1", STALE_1)]),
            "{rule}"
        );
    }
}

#[test]
fn an_auto_rule_over_a_guarded_protocol_fails_guarded_auto() {
    let output = check_set(
        &["publish-auto.yaml", "stale-1.yaml"],
        "nothing-published.yaml",
    );
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stdout));
    let stderr = text(&output.stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines.len(), 1, "{stderr}");
    assert!(
        lines[0].starts_with("error[guarded-auto]: publish-auto@1: "),
        "{stderr}"
    );
}

#[test]
fn the_digest_is_of_the_rule_not_of_its_text() {
    for (rule, digest) in [
        ("stale-1.yaml", STALE_1),
        ("stale-1-reformatted.yaml", STALE_1),
        ("stale-2.yaml", STALE_2),
        ("stale-1-edited.yaml", STALE_1_EDITED),
    ] {
        let output = canon(&["trigger", "digest", "--path", &format!("{DIR}/{rule}")]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{rule}: {}",
            text(&output.stderr)
        );
        assert_eq!(text(&output.stdout), format!("{digest}\n"), "{rule}");
    }
}

#[test]
fn an_invalid_rule_and_an_unreadable_file_are_refused() {
    let invalid = canon(&[
        "trigger",
        "digest",
        "--path",
        "fixtures/trigger/invalid/mode-sometimes.yaml",
    ]);
    assert_eq!(invalid.status.code(), Some(1));
    assert!(
        text(&invalid.stderr).starts_with("error[invalid-value]: "),
        "{}",
        text(&invalid.stderr)
    );
    let missing = check_set(&["no-such-rule.yaml"], "nothing-published.yaml");
    assert_eq!(missing.status.code(), Some(2));
    assert!(
        text(&missing.stderr).starts_with("error[unreadable]: "),
        "{}",
        text(&missing.stderr)
    );
}

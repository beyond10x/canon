//! The crate's one `unsafe` stays in `src/eval/worker.rs` (story:evaluate-without-thread-per-call).
//!
//! `src/lib.rs` denies `unsafe_code` for the crate and `worker.rs` allows it for itself. Nothing
//! else keeps that claim true, so this scan does: it reads every file under `src` and fails when
//! a file other than `eval/worker.rs` allows `unsafe_code` or holds the word `unsafe` in code, and
//! when `eval/worker.rs` does not hold it, so that the scan cannot pass by finding nothing.
//!
//! A line scan, not a parse: a line is cut at its first `//` and a line that starts with `*` (inside
//! a block comment) is skipped, and text between double quotes on a line is dropped. A line that
//! cuts a string at a `//` in it is cut early, which can only hide an `unsafe` after it; none is
//! written that way here.

use std::fs;
use std::path::{Path, PathBuf};

/// The one file that may hold `unsafe`, relative to `src`.
const ALLOWED: &str = "eval/worker.rs";

fn source_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()))
        .map(|entry| entry.expect("directory entry reads").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            source_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

/// `line` without its `//` comment and without what is between double quotes.
fn code_of(line: &str) -> String {
    let line = line.trim_start();
    if line.starts_with('*') {
        return String::new();
    }
    let line = line.split("//").next().unwrap_or("");
    let mut code = String::new();
    let mut in_string = false;
    for character in line.chars() {
        if character == '"' {
            in_string = !in_string;
        } else if !in_string {
            code.push(character);
        }
    }
    code
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// What in `text` allows or uses `unsafe`: one entry per offending line, with its number.
fn unsafe_uses(text: &str) -> Vec<String> {
    let mut uses = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let code = code_of(line);
        let squeezed: String = code.split_whitespace().collect();
        let allows = ["allow", "expect"]
            .iter()
            .any(|lint| squeezed.contains(&format!("{lint}(unsafe_code")));
        let word = code.match_indices("unsafe").any(|(at, found)| {
            let before = code[..at].chars().next_back();
            let after = code[at + found.len()..].chars().next();
            !before.is_some_and(is_word_character) && !after.is_some_and(is_word_character)
        });
        if allows || word {
            uses.push(format!("{}: {}", index + 1, line.trim()));
        }
    }
    uses
}

#[test]
fn the_scan_tells_unsafe_code_from_text_about_it() {
    for flagged in [
        "unsafe { f() }",
        "    let x = unsafe { g() };",
        "unsafe fn h() {}",
        "unsafe impl Send for T {}",
        "#![allow(unsafe_code)]",
        "#[allow( unsafe_code )]",
        "#![expect(unsafe_code)]",
        "pub unsafe extern \"C\" fn k() {}",
    ] {
        assert_eq!(unsafe_uses(flagged).len(), 1, "not flagged: {flagged}");
    }
    for clean in [
        "#![deny(unsafe_code)]",
        "#![forbid(unsafe_code)]",
        "// unsafe { f() }",
        "//! the one `unsafe` of the crate",
        "    // SAFETY: the job is over",
        " * unsafe in a block comment",
        "let text = \"unsafe\";",
        "let not_unsafe_at_all = 1;",
        "let unsafely = 1;",
        "let x = 1; // unsafe",
    ] {
        assert_eq!(unsafe_uses(clean), Vec::<String>::new(), "flagged: {clean}");
    }
}

#[test]
fn unsafe_is_confined_to_the_eval_worker() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    source_files(&src, &mut files);
    assert!(
        files.len() > 50,
        "the scan found {} files under {}: it is not reading the crate",
        files.len(),
        src.display()
    );
    let mut elsewhere = Vec::new();
    let mut in_worker = Vec::new();
    for path in &files {
        let relative = path
            .strip_prefix(&src)
            .expect("under src")
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let uses = unsafe_uses(&text);
        if relative == ALLOWED {
            in_worker = uses;
        } else {
            elsewhere.extend(uses.into_iter().map(|line| format!("{relative}:{line}")));
        }
    }
    assert!(
        elsewhere.is_empty(),
        "`unsafe` outside src/{ALLOWED}, where the crate keeps its one use of it:\n{}",
        elsewhere.join("\n")
    );
    assert!(
        in_worker.iter().any(|line| line.contains("allow(unsafe_code)")),
        "src/{ALLOWED} does not allow `unsafe_code` or is not there; the scan has nothing to confine"
    );
    assert!(
        in_worker.len() >= 2,
        "src/{ALLOWED} holds no `unsafe` block: the claim of the scan is stale"
    );
    let lib = fs::read_to_string(src.join("lib.rs")).expect("lib.rs reads");
    assert!(
        lib.lines().any(|line| line.trim() == "#![deny(unsafe_code)]"),
        "src/lib.rs does not deny `unsafe_code` for the crate"
    );
}

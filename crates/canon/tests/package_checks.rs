//! story:linux-release-archives: the two binary checks of `scripts/package.sh`, run on binaries
//! the test has at hand, so the negative controls need no release build. `check-glibc` must
//! refuse a binary linked against a glibc above the 2.17 floor and name the symbol; `check-static`
//! must refuse a binary with a program interpreter. The host-linked binary of this very test is the
//! dynamically linked control; tiny ELF files written here are the static ones. The workflow and
//! the task both run the one script.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("run this test through cargo"))
        .join("../..")
}

fn package(args: &[&str]) -> Output {
    Command::new("sh")
        .arg(root().join("scripts/package.sh"))
        .args(args)
        .current_dir(root())
        .stdin(Stdio::null())
        .output()
        .expect("sh runs scripts/package.sh")
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// A dynamically linked glibc binary built on this host: the test binary itself.
fn host_binary() -> String {
    std::env::current_exe()
        .expect("the test binary has a path")
        .to_string_lossy()
        .into_owned()
}

/// The `GLIBC_2.<minor>` versions named in a check's output, as minor numbers.
fn named_minors(output: &str) -> Vec<u32> {
    output
        .split("GLIBC_2.")
        .skip(1)
        .filter_map(|rest| {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            digits.parse().ok()
        })
        .collect()
}

/// A minimal x86_64 ELF executable: one `PT_LOAD`, and a `PT_INTERP` when `interpreter` is given.
fn tiny_elf(interpreter: Option<&str>) -> Vec<u8> {
    let headers: u16 = if interpreter.is_some() { 2 } else { 1 };
    let base: u64 = 0x40_0000;
    let data_offset = 64 + 56 * u64::from(headers);
    let interp = interpreter.map(|path| {
        let mut bytes = path.as_bytes().to_vec();
        bytes.push(0);
        bytes
    });
    let total = data_offset + interp.as_ref().map_or(0, |bytes| bytes.len() as u64) + 1;
    let mut elf = Vec::new();
    elf.extend_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    elf.extend_from_slice(&2u16.to_le_bytes()); // ET_EXEC
    elf.extend_from_slice(&0x3eu16.to_le_bytes()); // EM_X86_64
    elf.extend_from_slice(&1u32.to_le_bytes());
    elf.extend_from_slice(&(base + total - 1).to_le_bytes()); // entry: the last byte, a hlt
    elf.extend_from_slice(&64u64.to_le_bytes()); // program headers right after this header
    elf.extend_from_slice(&0u64.to_le_bytes()); // no section headers
    elf.extend_from_slice(&0u32.to_le_bytes());
    elf.extend_from_slice(&64u16.to_le_bytes());
    elf.extend_from_slice(&56u16.to_le_bytes());
    elf.extend_from_slice(&headers.to_le_bytes());
    elf.extend_from_slice(&64u16.to_le_bytes());
    elf.extend_from_slice(&0u16.to_le_bytes());
    elf.extend_from_slice(&0u16.to_le_bytes());
    let mut program_header = |kind: u32, flags: u32, offset: u64, size: u64, align: u64| {
        elf.extend_from_slice(&kind.to_le_bytes());
        elf.extend_from_slice(&flags.to_le_bytes());
        elf.extend_from_slice(&offset.to_le_bytes());
        elf.extend_from_slice(&(base + offset).to_le_bytes());
        elf.extend_from_slice(&(base + offset).to_le_bytes());
        elf.extend_from_slice(&size.to_le_bytes());
        elf.extend_from_slice(&size.to_le_bytes());
        elf.extend_from_slice(&align.to_le_bytes());
    };
    if let Some(bytes) = &interp {
        program_header(3, 4, data_offset, bytes.len() as u64, 1); // PT_INTERP
    }
    program_header(1, 5, 0, total, 0x1000); // PT_LOAD
    if let Some(bytes) = &interp {
        elf.extend_from_slice(bytes);
    }
    elf.push(0xf4);
    assert_eq!(elf.len() as u64, total);
    elf
}

fn scratch(name: &str) -> PathBuf {
    let dir = root().join("target/package-checks-test");
    fs::create_dir_all(&dir).expect("target/package-checks-test is created");
    dir.join(format!("{name}-{}", std::process::id()))
}

fn write(path: &Path, bytes: &[u8]) -> String {
    fs::write(path, bytes).expect("the fixture is written");
    path.to_string_lossy().into_owned()
}

/// Negative control: a binary linked on a current host needs a glibc above 2.17, and the check
/// says which symbol needs which version.
#[test]
fn glibc_check_refuses_a_host_binary_naming_a_symbol_above_2_17() {
    let out = package(&["check-glibc", &host_binary()]);
    let said = text(&out);
    assert!(
        !out.status.success(),
        "check-glibc passes a binary built against this host's glibc:\n{said}"
    );
    let above: Vec<u32> = named_minors(&said)
        .into_iter()
        .filter(|minor| *minor > 17)
        .collect();
    assert!(
        !above.is_empty(),
        "check-glibc names no GLIBC_ version above 2.17:\n{said}"
    );
    assert!(
        said.lines()
            .any(|line| line.contains(&format!("GLIBC_2.{}", above[0]))
                && line.contains("symbol ")),
        "check-glibc names the symbol beside the version:\n{said}"
    );
}

/// Versions compare as numbers, not text: `GLIBC_2.2.5`, present in every x86_64 binary, is
/// below 2.17. A floor equal to the highest version the binary needs passes, one below fails.
#[test]
fn glibc_check_compares_versions_as_numbers() {
    let refused = text(&package(&["check-glibc", &host_binary()]));
    let highest = named_minors(&refused)
        .into_iter()
        .max()
        .expect("the refusal names a version");
    let at = package(&["check-glibc", &host_binary(), &format!("2.{highest}")]);
    assert!(
        at.status.success(),
        "a floor of 2.{highest}, the highest version named, passes:\n{}",
        text(&at)
    );
    let below = package(&["check-glibc", &host_binary(), &format!("2.{}", highest - 1)]);
    assert!(
        !below.status.success(),
        "a floor of 2.{} fails:\n{}",
        highest - 1,
        text(&below)
    );
}

/// Negative control: a dynamically linked binary has a program interpreter and is not static.
#[test]
fn static_check_refuses_a_dynamically_linked_binary() {
    let out = package(&["check-static", &host_binary()]);
    let said = text(&out);
    assert!(
        !out.status.success(),
        "check-static passes a dynamically linked binary:\n{said}"
    );
    assert!(
        said.contains("INTERP"),
        "check-static names INTERP:\n{said}"
    );

    let fixture = write(
        &scratch("interp.elf"),
        &tiny_elf(Some("/lib64/ld-linux-x86-64.so.2")),
    );
    let out = package(&["check-static", &fixture]);
    assert!(
        !out.status.success(),
        "check-static passes an ELF with a PT_INTERP:\n{}",
        text(&out)
    );
}

/// The static control passes: an ELF with program headers and no `PT_INTERP`.
#[test]
fn static_check_passes_an_elf_without_an_interpreter() {
    let fixture = write(&scratch("static.elf"), &tiny_elf(None));
    let out = package(&["check-static", &fixture]);
    assert!(
        out.status.success(),
        "check-static refuses an ELF without PT_INTERP:\n{}",
        text(&out)
    );
}

/// A file that is not a dynamically linked ELF passes neither check: an empty answer from
/// `objdump` or `readelf` is not a pass.
#[test]
fn both_checks_refuse_what_they_cannot_read() {
    let not_elf = root().join("Cargo.toml").to_string_lossy().into_owned();
    let missing = root()
        .join("target/no-such-binary-for-the-package-test")
        .to_string_lossy()
        .into_owned();
    let static_elf = write(&scratch("no-glibc.elf"), &tiny_elf(None));
    for (check, file) in [
        ("check-glibc", &not_elf),
        ("check-glibc", &missing),
        ("check-glibc", &static_elf),
        ("check-static", &not_elf),
        ("check-static", &missing),
    ] {
        let out = package(&[check, file]);
        assert!(
            !out.status.success(),
            "{check} passes {file}:\n{}",
            text(&out)
        );
    }
}

/// An unknown target is refused before anything is downloaded or built.
#[test]
fn an_unknown_target_is_refused() {
    let out = package(&["aarch64-apple-darwin"]);
    assert!(!out.status.success());
    let said = text(&out);
    assert!(
        said.contains("x86_64-unknown-linux-gnu") && said.contains("x86_64-unknown-linux-musl"),
        "the refusal names the two targets:\n{said}"
    );
}

/// The workflow builds both targets with the one script, writes nothing but artifacts, and pins
/// every action by commit; the task runs the same script.
#[test]
fn the_workflow_and_the_task_run_the_one_script() {
    let workflow = fs::read_to_string(root().join(".github/workflows/package.yml"))
        .expect(".github/workflows/package.yml reads");
    for target in ["x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl"] {
        assert!(
            workflow.contains(&format!("scripts/package.sh {target}")),
            "package.yml runs scripts/package.sh {target}"
        );
    }
    assert!(workflow.contains("permissions:\n  contents: read\n"));
    assert!(
        !workflow.contains(": write"),
        "package.yml asks for no write permission"
    );
    for line in workflow.lines().filter(|line| line.contains("uses:")) {
        let pin = line.rsplit('@').next().unwrap_or_default();
        let pin = pin.split_whitespace().next().unwrap_or_default();
        assert!(
            pin.len() == 40 && pin.chars().all(|c| c.is_ascii_hexdigit()),
            "an action pinned by commit: {line}"
        );
    }
    assert!(workflow.contains("actions/upload-artifact@"));
    assert!(workflow.contains("workflow_dispatch"));
    assert!(workflow.contains("tags:"));

    let taskfile = fs::read_to_string(root().join("Taskfile.yml")).expect("Taskfile.yml reads");
    assert!(
        taskfile.contains("sh scripts/package.sh {{.CLI_ARGS}}"),
        "task package runs scripts/package.sh"
    );

    let script = fs::read_to_string(root().join("scripts/package.sh")).expect("package.sh reads");
    for pin in ["ZIG_SHA256=", "ZIGBUILD_SHA256="] {
        let value = script
            .lines()
            .find_map(|line| line.strip_prefix(pin))
            .unwrap_or_else(|| panic!("package.sh sets {pin}"));
        assert!(
            value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit()),
            "{pin} is a SHA-256: {value}"
        );
    }
    assert!(script.contains("--target x86_64-unknown-linux-gnu.2.17"));
}

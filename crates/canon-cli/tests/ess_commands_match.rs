//! The `canon` command surface and its ESS declaration stay equal.
//!
//! `canon_cli::Cli` is the clap definition of the `canon` command line; `ess/domains/cli.yaml`
//! declares the same surface as the commands of the domain `canon.cli` (Atlas ADR 0076). This test
//! walks the clap definition through `clap::CommandFactory`, compiles the specification
//! (`ess specify compile --format json`) and compares the two in both directions. A subcommand or
//! flag added to one and not the other fails here, naming what differs.
//!
//! How the clap definition maps onto the specification:
//!
//! * The clap definition is read built (`clap::Command::build`), as clap parses it: global flags
//!   are propagated, and clap's own `help` subcommand, `--help` and `--version` are not compared.
//! * Every leaf subcommand is one ESS command of `canon.cli` whose `naming.wire` is the clap path
//!   joined by `-`: `canon validate` is `validate`, `canon conform run` is `conform-run`. A
//!   subcommand that has subcommands of its own (`conform`) is not a command. Two ESS commands
//!   with one wire are reported, since only one of them could be compared.
//! * The flags of a leaf are its own long flags, the global ones and those of every command above
//!   it (`canon`, `canon conform`). Each long flag `--name` is the input `name` of its command,
//!   with every `-` of the flag an `_` of the input (ESS refuses `-` in a field name):
//!   `--max-states` is `max_states`. A long flag holding `_` is reported, because that translation
//!   would give it the input of another flag. An argument with no long flag is reported.
//! * A flag clap requires is a required input; one it does not (an `Option<_>`, or one with a
//!   default value) is an `Optional<…>` input.
//! * A flag clap takes more than once (`ArgAction::Append`, or more than one value per
//!   occurrence) is a `List<…>` input, inside the `Optional<…>` when it is not required; a flag it
//!   takes once is not a list. The element type is not compared: every value on a command line is
//!   text. An input of any other kind (a `Map`, a struct) is reported.
//! * A command and a flag have one name each: an alias, short form or `ArgAction::Count` flag is
//!   reported.
//!
//! It also synthesizes the specification's conformance suite (`ess verify conform synthesize`)
//! and counts its scenarios per command: every `canon.cli` command has at least one, which
//! `crates/canon/tests/ess_gate.rs` does not check.

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Arg, ArgAction, CommandFactory};
use serde_yaml_ng::Value;

/// The ESS domain whose commands are the `canon` command line.
const DOMAIN: &str = "canon.cli";

/// The tree under test, read at run time: a test binary reused from a shared target directory
/// would otherwise check the tree it was built from.
fn repo_root() -> PathBuf {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect(
        "CARGO_MANIFEST_DIR is unset: run this test through cargo, which sets it to the \
         crates/canon-cli directory of the tree under test",
    );
    Path::new(&manifest)
        .join("../..")
        .canonicalize()
        .expect("repository root resolves")
}

/// A fresh directory under `CARGO_TARGET_TMPDIR` for one case, distinct per tree under test.
fn scratch(case: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    repo_root().hash(&mut hasher);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("canon-ess-commands")
        .join(format!("{:016x}", hasher.finish()))
        .join(case);
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("remove the previous scratch copy");
    }
    fs::create_dir_all(&dir).expect("create scratch");
    dir
}

/// Runs `ess <args>` in `dir` and returns its stdout. Fails, rather than skips, when `ess` is not
/// on `PATH` or exits non-zero.
fn ess(dir: &Path, args: &[&str]) -> Vec<u8> {
    let output = match Command::new("ess").args(args).current_dir(dir).output() {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => panic!(
            "`ess` is not on PATH: comparing the command surface with ess/ needs ess 0.77.0, the \
             release ess/ess-inputs.yaml pins"
        ),
        Err(error) => panic!("cannot run `ess {}`: {error}", args.join(" ")),
    };
    assert!(
        output.status.success(),
        "`ess {}` in {} exited {}:\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        dir.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

// ---------------------------------------------------------------------------------------------
// One shape for both sides: wire name -> (input name -> input).

/// One input of a command, as either side declares it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Input {
    /// How the side names it in a difference: `--max-states` for clap, `max_states` for ESS.
    shown: String,
    required: bool,
    list: bool,
    /// An ESS input's kind, after one `Optional`, when it is neither a single value nor a `List`.
    other: Option<String>,
}

/// A command's inputs, keyed by the ESS input name (`max_states` for `--max-states`).
type Inputs = BTreeMap<String, Input>;

#[derive(Debug, Default)]
struct Surface {
    /// Wire name (`conform-run`) to the command as its side names it and its inputs.
    commands: BTreeMap<String, (String, Inputs)>,
    /// What the mapping cannot compare, each a difference already: an argument that is not a long
    /// flag, a flag holding `_`, a second ESS command with one wire.
    unmapped: Vec<String>,
}

/// The ESS input name of the long flag `--<long>`: every `-` is an `_`.
fn input_name(long: &str) -> String {
    long.replace('-', "_")
}

/// Whether clap takes the flag more than once: appended occurrences, or several values in one.
fn repeatable(arg: &Arg) -> bool {
    matches!(arg.get_action(), ArgAction::Append)
        || arg
            .get_num_args()
            .is_some_and(|range| range.max_values() > 1)
}

/// clap's own `--help` and `--version`, which no command declares.
fn clap_own(arg: &Arg) -> bool {
    matches!(
        arg.get_action(),
        ArgAction::Help | ArgAction::HelpShort | ArgAction::HelpLong | ArgAction::Version
    )
}

/// The clap side: every leaf subcommand of `command`, built as clap parses it, with the flags of
/// every command above it.
fn clap_surface(command: &clap::Command) -> Surface {
    let mut command = command.clone();
    command.build();
    let mut surface = Surface::default();
    collect_leaves(&command, &[], &Inputs::new(), &mut surface);
    surface
}

fn collect_leaves(
    command: &clap::Command,
    path: &[&str],
    inherited: &Inputs,
    surface: &mut Surface,
) {
    let display = if path.is_empty() {
        "canon".to_owned()
    } else {
        format!("canon {}", path.join(" "))
    };
    let mut inputs = inherited.clone();
    for arg in command.get_arguments().filter(|arg| !clap_own(arg)) {
        let Some(long) = arg.get_long() else {
            surface.unmapped.push(format!(
                "`{display}` argument `{}` is not a long flag, so no ESS input can declare it",
                arg.get_id().as_str()
            ));
            continue;
        };
        if long.contains('_') {
            surface.unmapped.push(format!(
                "`{display} --{long}` holds `_`, which the mapping reads as the `-` of another \
                 flag, so no ESS input is its own"
            ));
        }
        let mut others: Vec<String> = arg
            .get_all_aliases()
            .unwrap_or_default()
            .iter()
            .map(|alias| format!("--{alias}"))
            .collect();
        others.extend(
            arg.get_short()
                .into_iter()
                .chain(arg.get_all_short_aliases().unwrap_or_default())
                .map(|short| format!("-{short}")),
        );
        if !others.is_empty() {
            surface.unmapped.push(format!(
                "`{display} --{long}` is also named {}, and an ESS input has one name",
                others.join(", ")
            ));
        }
        if matches!(arg.get_action(), ArgAction::Count) {
            surface.unmapped.push(format!(
                "`{display} --{long}` counts its occurrences, a number the surface does not declare"
            ));
        }
        inputs.insert(
            input_name(long),
            Input {
                shown: format!("--{long}"),
                required: arg.is_required_set(),
                list: repeatable(arg),
                other: None,
            },
        );
    }
    if !command.has_subcommands() {
        if !path.is_empty() {
            surface.commands.insert(path.join("-"), (display, inputs));
        }
        return;
    }
    // `build` gives every command with subcommands clap's own `help` subcommand.
    for sub in command
        .get_subcommands()
        .filter(|sub| sub.get_name() != "help")
    {
        let mut below = path.to_vec();
        below.push(sub.get_name());
        let others: Vec<String> = sub
            .get_all_aliases()
            .map(str::to_owned)
            .chain(
                sub.get_short_flag()
                    .into_iter()
                    .chain(sub.get_all_short_flag_aliases())
                    .map(|short| format!("-{short}")),
            )
            .chain(
                sub.get_long_flag()
                    .into_iter()
                    .chain(sub.get_all_long_flag_aliases())
                    .map(|long| format!("--{long}")),
            )
            .collect();
        if !others.is_empty() {
            surface.unmapped.push(format!(
                "`canon {}` is also named {}, and an ESS command has one wire",
                below.join(" "),
                others.join(", ")
            ));
        }
        collect_leaves(sub, &below, &inputs, surface);
    }
}

/// The kind of the compiled type `held`, when it is neither a single value (a primitive, or a
/// declared newtype or enum) nor a `List`: `map`, `optional`, `struct`, `union`.
fn other_kind(ir: &Value, held: &Value) -> Option<String> {
    let kind = match held["kind"].as_str() {
        Some("primitive" | "list") => return None,
        Some("declared") => {
            let name = held["name"].as_str().unwrap_or_default();
            ir["types"][name]["body"]["kind"].as_str()
        }
        kind => kind,
    };
    match kind {
        Some("newtype" | "enum") => None,
        kind => Some(kind.unwrap_or("unknown").to_owned()),
    }
}

/// The specification side: every command of [`DOMAIN`] in the compiled IR of `root`/ess.
fn ess_surface(root: &Path) -> Surface {
    let ir: Value = serde_yaml_ng::from_slice(&ess(
        root,
        &["specify", "compile", "--path", "ess", "--format", "json"],
    ))
    .expect("compiled IR is JSON");
    let mut surface = Surface::default();
    let commands = ir["commands"]
        .as_mapping()
        .expect("compiled IR has a commands map");
    for (name, command) in commands {
        if command["domain"].as_str() != Some(DOMAIN) {
            continue;
        }
        let name = name.as_str().expect("command name").to_owned();
        let wire = command["naming"]["wire"]
            .as_str()
            .unwrap_or_else(|| panic!("ESS command `{name}` declares no naming.wire"))
            .to_owned();
        let mut inputs = Inputs::new();
        for input in command["input"].as_sequence().into_iter().flatten() {
            let field = input["name"].as_str().expect("input name").to_owned();
            let type_ref = &input["type_ref"];
            let optional = type_ref["kind"].as_str() == Some("optional");
            let held = if optional { &type_ref["of"] } else { type_ref };
            inputs.insert(
                field.clone(),
                Input {
                    shown: field,
                    required: !optional,
                    list: held["kind"].as_str() == Some("list"),
                    other: other_kind(&ir, held),
                },
            );
        }
        if let Some((first, _)) = surface.commands.get(&wire) {
            surface.unmapped.push(format!(
                "ESS commands `{first}` and `{name}` both have wire `{wire}`, which names one \
                 canon subcommand, so `{name}` is compared with nothing"
            ));
            continue;
        }
        surface.commands.insert(wire, (name, inputs));
    }
    surface
}

/// Every difference between the clap and ESS surfaces, in a stable order.
fn differences(clap: &Surface, spec: &Surface) -> Vec<String> {
    let mut out: Vec<String> = clap
        .unmapped
        .iter()
        .chain(&spec.unmapped)
        .cloned()
        .collect();
    for (wire, (display, clap_inputs)) in &clap.commands {
        let Some((name, spec_inputs)) = spec.commands.get(wire) else {
            out.push(format!(
                "`{display}` has no ESS command with wire `{wire}` in domain {DOMAIN}"
            ));
            continue;
        };
        for (field, flag) in clap_inputs {
            let shown = &flag.shown;
            let Some(declared) = spec_inputs.get(field) else {
                out.push(format!(
                    "`{display} {shown}` has no input `{field}` in ESS command `{name}`"
                ));
                continue;
            };
            if let Some(kind) = &declared.other {
                out.push(format!(
                    "`{display} {shown}`: ESS command `{name}` input `{field}` is a {kind}, which \
                     no flag is: an input is a single value or a List"
                ));
            }
            if declared.required != flag.required {
                out.push(format!(
                    "`{display} {shown}`: clap says {}, ESS command `{name}` says {}",
                    requiredness(flag.required),
                    requiredness(declared.required)
                ));
            }
            if declared.list != flag.list {
                out.push(format!(
                    "`{display} {shown}`: clap takes it {}, ESS command `{name}` input `{field}` \
                     is {}",
                    if flag.list { "more than once" } else { "once" },
                    if declared.list {
                        "a List"
                    } else {
                        "a single value"
                    }
                ));
            }
        }
        for field in spec_inputs.keys() {
            if !clap_inputs.contains_key(field) {
                out.push(format!(
                    "ESS command `{name}` input `{field}` is not a flag of `{display}`"
                ));
            }
        }
    }
    for (wire, (name, _)) in &spec.commands {
        if !clap.commands.contains_key(wire) {
            out.push(format!(
                "ESS command `{name}` (wire `{wire}`) is not a canon subcommand"
            ));
        }
    }
    out
}

fn requiredness(required: bool) -> &'static str {
    if required { "required" } else { "optional" }
}

fn assert_equal(clap: &Surface, spec: &Surface) {
    let found = differences(clap, spec);
    assert!(
        found.is_empty(),
        "canon_cli::Cli and the {DOMAIN} commands of ess/ differ; the first difference is:\n  {}\n\
         all {}:\n  {}",
        found[0],
        found.len(),
        found.join("\n  ")
    );
}

fn assert_named(clap: &Surface, spec: &Surface, needles: &[&str]) {
    let found = differences(clap, spec);
    assert!(
        found
            .first()
            .is_some_and(|first| needles.iter().all(|needle| first.contains(needle))),
        "expected the first difference to name {needles:?}, found {found:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// The scenarios `ess verify conform synthesize` writes, per command.

/// Scenarios per command of [`DOMAIN`] in the suite synthesized from `ess/`.
fn scenarios_per_command() -> BTreeMap<String, usize> {
    let suite = scratch("suite").join("suite.json");
    let out = suite.to_str().expect("suite path is UTF-8");
    ess(
        &repo_root(),
        &[
            "verify",
            "conform",
            "synthesize",
            "--path",
            "ess",
            "--out",
            out,
        ],
    );
    let suite: Value =
        serde_yaml_ng::from_str(&fs::read_to_string(&suite).expect("the synthesized suite reads"))
            .expect("the synthesized suite is JSON");
    let mut counts = BTreeMap::new();
    let scenarios = suite["scenarios"]
        .as_mapping()
        .expect("suite has a scenarios map");
    for scenario in scenarios.values() {
        for source in scenario["source"].as_sequence().into_iter().flatten() {
            if source["kind"].as_str() == Some("command")
                && let Some(name) = source["name"].as_str()
                && name.starts_with(&format!("{DOMAIN}."))
            {
                *counts.entry(name.to_owned()).or_default() += 1;
            }
        }
    }
    counts
}

// ---------------------------------------------------------------------------------------------

/// The story's acceptance: every leaf subcommand and long flag of `canon_cli::Cli` is a command
/// and an input of the compiled `canon.cli` domain, and the reverse, and every command has at
/// least one synthesized scenario.
#[test]
fn every_canon_subcommand_is_declared_in_ess() {
    let clap = clap_surface(&canon_cli::Cli::command());
    let spec = ess_surface(&repo_root());
    assert!(
        !clap.commands.is_empty(),
        "canon_cli::Cli has no leaf subcommand, so nothing was compared"
    );
    assert_equal(&clap, &spec);

    let counts = scenarios_per_command();
    let rows: Vec<String> = spec
        .commands
        .values()
        .map(|(name, _)| format!("{name}: {}", counts.get(name).copied().unwrap_or(0)))
        .collect();
    println!(
        "synthesized scenarios per command:\n  {}",
        rows.join("\n  ")
    );
    let without: Vec<&String> = spec
        .commands
        .values()
        .map(|(name, _)| name)
        .filter(|name| counts.get(*name).copied().unwrap_or(0) == 0)
        .collect();
    assert!(
        without.is_empty(),
        "`ess verify conform synthesize --path ess` writes no scenario for {without:?}; \
         scenarios per command:\n  {}",
        rows.join("\n  ")
    );
}

/// A flag added to a subcommand in clap and not declared in ESS fails, naming both.
#[test]
fn a_flag_only_in_clap_is_named() {
    let spec = ess_surface(&repo_root());
    let command = canon_cli::Cli::command().mut_subcommand("validate", |validate| {
        validate.arg(Arg::new("probe_flag").long("probe-flag"))
    });
    assert_named(
        &clap_surface(&command),
        &spec,
        &[
            "canon validate --probe-flag",
            "input `probe_flag`",
            "canon.cli.Validate",
        ],
    );
}

/// The specification of a copy of `ess/` whose `domains/cli.yaml` commands `edit` changed,
/// compiled, so the reading of the compiled IR is what is checked.
fn ess_surface_edited(case: &str, edit: impl FnOnce(&mut Vec<Value>)) -> Surface {
    let root = scratch(case);
    copy_dir(&repo_root().join("ess"), &root.join("ess"));
    let file = root.join("ess/domains/cli.yaml");
    let mut domain: Value = serde_yaml_ng::from_str(
        &fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("{} reads: {error}", file.display())),
    )
    .expect("ess/domains/cli.yaml is YAML");
    edit(
        domain["commands"]
            .as_sequence_mut()
            .expect("ess/domains/cli.yaml declares commands"),
    );
    fs::write(
        &file,
        serde_yaml_ng::to_string(&domain).expect("the domain serializes"),
    )
    .expect("the edited copy is written");
    ess_surface(&root)
}

fn yaml(text: &str) -> Value {
    serde_yaml_ng::from_str(text).expect("the probe is YAML")
}

/// The command `name` of a parsed `domains/cli.yaml` commands list.
fn declared<'a>(commands: &'a mut [Value], name: &str) -> &'a mut Value {
    commands
        .iter_mut()
        .find(|command| command["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("ess/domains/cli.yaml declares no {name}"))
}

/// The inputs list of the command `name`.
fn inputs_of<'a>(commands: &'a mut [Value], name: &str) -> &'a mut Vec<Value> {
    declared(commands, name)["input"]
        .as_sequence_mut()
        .unwrap_or_else(|| panic!("{name} declares inputs"))
}

/// A probe command with its own `wire`, which is no `canon` subcommand.
fn probe_command(name: &str, wire: &str) -> Value {
    yaml(&format!(
        "name: {name}
naming:
  wire: {wire}
  display: Probe
input:
  - name: probe_input
    type: String
response:
  - name: output
    type: String
outcomes:
  - name: done
    returns: true
"
    ))
}

/// A command declared in ESS that is no `canon` subcommand fails, naming it.
#[test]
fn a_command_only_in_ess_is_named() {
    let spec = ess_surface_edited("command-only-in-ess", |commands| {
        commands.push(probe_command("canon.cli.Probe", "probe"));
    });
    assert_named(
        &clap_surface(&canon_cli::Cli::command()),
        &spec,
        &["canon.cli.Probe", "`probe`", "not a canon subcommand"],
    );
}

/// A second ESS command with the wire of a subcommand fails, naming both commands and the wire,
/// although the first one matches clap exactly.
#[test]
fn a_second_command_with_one_wire_is_named() {
    let spec = ess_surface_edited("shared-wire", |commands| {
        commands.push(probe_command("canon.cli.Probe", "validate"));
    });
    assert_named(
        &clap_surface(&canon_cli::Cli::command()),
        &spec,
        &["canon.cli.Probe", "canon.cli.Validate", "wire `validate`"],
    );
}

/// A flag clap requires that ESS declares optional fails, naming the flag and both sides.
#[test]
fn a_changed_requiredness_is_named() {
    let spec = ess_surface(&repo_root());
    let command = canon_cli::Cli::command().mut_subcommand("evaluate", |evaluate| {
        evaluate.mut_arg("authority", |authority| authority.required(true))
    });
    assert_named(
        &clap_surface(&command),
        &spec,
        &[
            "canon evaluate --authority",
            "clap says required",
            "canon.cli.Evaluate",
            "says optional",
        ],
    );
}

/// A global flag on `canon` is a flag of every leaf subcommand: undeclared, it is named once per
/// command.
#[test]
fn a_global_flag_is_named_on_every_command() {
    let spec = ess_surface(&repo_root());
    let command =
        canon_cli::Cli::command().arg(Arg::new("probe_global").long("probe-global").global(true));
    let clap = clap_surface(&command);
    assert_named(
        &clap,
        &spec,
        &["canon check --probe-global", "input `probe_global`"],
    );
    let named = differences(&clap, &spec)
        .iter()
        .filter(|difference| difference.contains("--probe-global"))
        .count();
    assert_eq!(
        named,
        clap.commands.len(),
        "the global flag is a flag of each of the {} commands",
        clap.commands.len()
    );
}

/// A flag of a command above a leaf, `canon conform` or `canon` itself, not marked global, is still
/// a flag of the leaf's command line, and is named.
#[test]
fn a_flag_of_a_parent_command_is_named() {
    let spec = ess_surface(&repo_root());
    let conform = canon_cli::Cli::command().mut_subcommand("conform", |conform| {
        conform.arg(Arg::new("probe_parent").long("probe-parent"))
    });
    assert_named(
        &clap_surface(&conform),
        &spec,
        &[
            "canon conform run --probe-parent",
            "input `probe_parent`",
            "canon.cli.ConformRun",
        ],
    );
    let root = canon_cli::Cli::command().arg(Arg::new("probe_root").long("probe-root"));
    assert_named(
        &clap_surface(&root),
        &spec,
        &["canon check --probe-root", "input `probe_root`"],
    );
}

/// `--probe-flag` is the input `probe_flag`: declared so, nothing differs. A flag holding `_`
/// would take another flag's input, and is named.
#[test]
fn a_hyphenated_flag_is_its_underscored_input() {
    let spec = ess_surface_edited("hyphenated-flag", |commands| {
        inputs_of(commands, "canon.cli.Validate")
            .push(yaml("name: probe_flag\ntype: Optional<String>\n"));
    });
    let hyphen = canon_cli::Cli::command().mut_subcommand("validate", |validate| {
        validate.arg(Arg::new("probe_flag").long("probe-flag"))
    });
    let found = differences(&clap_surface(&hyphen), &spec);
    assert!(
        found.is_empty(),
        "`--probe-flag` and the input `probe_flag` differ: {found:?}"
    );
    let underscore = canon_cli::Cli::command().mut_subcommand("validate", |validate| {
        validate.arg(Arg::new("probe_flag").long("probe_flag"))
    });
    assert_named(
        &clap_surface(&underscore),
        &spec,
        &["canon validate --probe_flag", "holds `_`"],
    );
}

/// A flag clap takes more than once is a List input: declared so, nothing differs; declared as a
/// single value, it is named.
#[test]
fn a_repeatable_flag_declared_single_is_named() {
    let command = canon_cli::Cli::command().mut_subcommand("validate", |validate| {
        validate.arg(Arg::new("probe").long("probe").action(ArgAction::Append))
    });
    let clap = clap_surface(&command);
    let as_list = ess_surface_edited("repeatable-as-list", |commands| {
        inputs_of(commands, "canon.cli.Validate")
            .push(yaml("name: probe\ntype: Optional<List<String>>\n"));
    });
    let found = differences(&clap, &as_list);
    assert!(
        found.is_empty(),
        "a repeatable `--probe` and an Optional<List<String>> input differ: {found:?}"
    );
    let single = ess_surface_edited("repeatable-as-single", |commands| {
        inputs_of(commands, "canon.cli.Validate")
            .push(yaml("name: probe\ntype: Optional<String>\n"));
    });
    assert_named(
        &clap,
        &single,
        &[
            "canon validate --probe",
            "more than once",
            "canon.cli.Validate",
            "a single value",
        ],
    );
}

/// A flag clap takes once that ESS declares a List fails, naming the flag and both sides.
#[test]
fn a_single_valued_flag_declared_as_a_list_is_named() {
    let spec = ess_surface_edited("single-as-list", |commands| {
        for input in inputs_of(commands, "canon.cli.Evaluate") {
            if input["name"].as_str() == Some("authority") {
                input["type"] = Value::from("Optional<List<String>>");
            }
        }
    });
    assert_named(
        &clap_surface(&canon_cli::Cli::command()),
        &spec,
        &[
            "canon evaluate --authority",
            "takes it once",
            "canon.cli.Evaluate",
            "a List",
        ],
    );
}

/// An input that is neither a single value nor a `List`, such as a `Map`, fails, naming the flag.
#[test]
fn a_single_valued_flag_declared_as_a_map_is_named() {
    let spec = ess_surface_edited("single-as-map", |commands| {
        for input in inputs_of(commands, "canon.cli.Evaluate") {
            if input["name"].as_str() == Some("authority") {
                input["type"] = Value::from("Optional<Map<String, String>>");
            }
        }
    });
    assert_named(
        &clap_surface(&canon_cli::Cli::command()),
        &spec,
        &[
            "canon evaluate --authority",
            "canon.cli.Evaluate",
            "is a map",
        ],
    );
}

/// A second name for a flag or a subcommand, visible or hidden, long or short, fails, naming it.
#[test]
fn an_alias_is_named() {
    let spec = ess_surface(&repo_root());
    let cases: [(clap::Command, &[&str]); 4] = [
        (
            canon_cli::Cli::command().mut_subcommand("validate", |validate| {
                validate.mut_arg("path", |path| path.visible_alias("file"))
            }),
            &["canon validate --path", "--file"],
        ),
        (
            canon_cli::Cli::command().mut_subcommand("validate", |validate| {
                validate.mut_arg("path", |path| path.alias("hidden-file"))
            }),
            &["canon validate --path", "--hidden-file"],
        ),
        (
            canon_cli::Cli::command().mut_subcommand("validate", |validate| {
                validate.mut_arg("path", |path| path.short('p'))
            }),
            &["canon validate --path", "-p"],
        ),
        (
            canon_cli::Cli::command().mut_subcommand("diff", |diff| diff.alias("compare")),
            &["canon diff", "compare", "one wire"],
        ),
    ];
    for (command, needles) in cases {
        assert_named(&clap_surface(&command), &spec, needles);
    }
}

/// A flag that counts its occurrences fails, naming it: the surface declares no such number.
#[test]
fn a_count_flag_is_named() {
    let spec = ess_surface(&repo_root());
    let command = canon_cli::Cli::command().mut_subcommand("validate", |validate| {
        validate.arg(Arg::new("verbose").long("verbose").action(ArgAction::Count))
    });
    assert_named(
        &clap_surface(&command),
        &spec,
        &["canon validate --verbose", "counts its occurrences"],
    );
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|error| panic!("create {}: {error}", to.display()));
    for entry in
        fs::read_dir(from).unwrap_or_else(|error| panic!("read {}: {error}", from.display()))
    {
        let path = entry.expect("directory entry").path();
        let target = to.join(path.file_name().expect("entry has a name"));
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            fs::copy(&path, &target)
                .unwrap_or_else(|error| panic!("copy {}: {error}", path.display()));
        }
    }
}

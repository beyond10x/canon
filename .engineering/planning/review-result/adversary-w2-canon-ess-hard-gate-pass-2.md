---
format: aep.planning-md/3
id: review-result:adversary-w2-canon-ess-hard-gate-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w2 adversary, canon story:ess-hard-gate, pass 2
relations:
- reviews: story:ess-hard-gate
revision: 1
---
unit: canon/ess-hard-gate, working tree on base dac4265 (uncommitted unit changes plus pass-1 adversary file)
verdict: NEEDS-CHANGE
cases: executed 66→72, red 5
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths (part 6)
needs-coordinator: none

**1. Diff**

```
 .github/workflows/check.yml | 16 ++++++++++++++++
 AGENTS.md                   | 19 ++++++++++++++++---
 Taskfile.yml                |  8 +++++++-
?? crates/canon/tests/adversary2_ess_gate.rs     <- mine
?? crates/canon/tests/adversary2_ess_model.rs    <- mine
?? (implementor's ess/, ess_gate.rs, ess_model_matches.rs; pass-1 adversary_ess_model_mutants.rs)
```

I added only those two test files. The tracked changes are the implementor's.

**2. Cases added (each run alone before the suite; all red now)**

How `adversary2_ess_model.rs` works: it copies the unit's `ess_model_matches.rs` unchanged at run time and builds it once into its own target directory. It then runs that one binary with `CARGO_MANIFEST_DIR` pointed at copies of `ess/` and the model, each copy with one change. A control case on an unchanged copy passes, so the setup is faithful.

| case | the unit's `specification_and_model_are_equal` |
|---|---|
| `a_field_retyped_to_a_foreign_type_of_the_same_name_is_caught`: `claims: Declarations<ClaimId, crate::ir::Claim>` | `test result: ok. 1 passed` |
| `a_field_compiled_out_by_a_feature_is_caught`: `#[cfg(feature = "effects")]` on `Action.effect` | `ok. 1 passed` |
| `a_map_key_changed_in_the_compiled_spec_is_caught_beside_a_stale_copy`: compiled spec says `Map<OutcomeId, Claim>`, an unlisted `protocol.yaml.orig.7` holds the old text | `ok. 1 passed`. The same change without the stale copy does fail, and the case checks that first. `ess validate --strict-requires` accepts the copy. |
| `toolchain_which_from_the_root_names_the_pin` (gate file) | `reason: self: no ess-inputs.yaml in …/canon-w2-ess-hard-gate or any directory above it` |
| `agents_md_command_surface_claim_is_backed_by_ess` (gate file) | `the compiled specification declares 0 command(s) while crates/canon-cli/src/main.rs defines a subcommand enum: true` |

The first map-key run went red for a harness reason: no stale-copy file name sorted after `protocol.yaml` in directory order. I fixed the harness by trying more names and reran the case alone; it then went red for the claimed reason. Logs are in the scratch dir.

**3. Suite (after the cases existed)**

- `cargo fmt --all --check`: exit 0, after `rustfmt` on my two files.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0.
- `cargo test --workspace --locked --no-fail-fast`: exit 101.
  - Red: `adversary2_ess_gate` 0 passed / 2 failed; `adversary2_ess_model` 1 passed / 3 failed.
  - Green: everything else, including `ess_gate` 11 passed, `ess_model_matches` 13 passed, pass-1 `adversary_ess_model_mutants` 7 passed.

**4. Findings (cover the working tree above)**

| # | file:line | what was measured | what reaches it | verdict |
|---|---|---|---|---|
| F1 | AGENTS.md:32 | Says "ESS specifies Canon's data model and command surface". The compiled spec has `commands: {}`, while `canon validate` exists. ADR 0076 § Canon requires the command surface; the story moved it out of scope. | Any agent reading § ESS before changing the CLI. Fix: say only the protocol/1 source model is specified, and that the command surface, case snapshot, evidence and decision documents are still owed. | NEEDS-CHANGE, warning |
| F2 | crates/canon/tests/ess_gate.rs:393-399 | From the repository root, `ess specify toolchain which` reports the `ess` on `PATH` (`reason: self`). The ess dispatcher searches the working directory and its parents, never `ess/`. The test reads only the first line, so it passes whatever `requires:` says. Only `--strict-requires` in step 1 ties the running ess to the pin. | Every gate run, but `pin_agrees_across_inputs_and_ci` and step 1 together still keep the version honest. | CONFIRMED, note |
| F3 | crates/canon/tests/ess_model_matches.rs:569-571, 588 | `last_segment` matches types by name only, so `crate::ir::Claim` is compared as the model's `Claim`. That contradicts the file's own doc at :18 ("Any other Rust type matches nothing"). story:canon-ir adds `ir::{Claim, Action, Outcome}` in this wave. | Nothing found that writes such a path in `model/`. | INFEASIBLE, note |
| F4 | crates/canon/tests/ess_model_matches.rs:431-441, 669 | Field attributes are stripped, so a field gated by `cfg(feature)` counts as model. Only module-level `#[cfg(test)]` is excluded. | b10x-canon declares no features. | INFEASIBLE, note |
| F5 | crates/canon/tests/ess_model_matches.rs:137-139 | `authored_map_keys` reads every file in `ess/domains/`; ess compiles only the files `ess-inputs.yaml` lists. A stale copy can supply the key type, and the last file read wins. Fix: read only the `specification:` list. | `.orig`, `~` and `.bak` files are not gitignored; none exists today. | INFEASIBLE, note |

**5. Attacked and could not break silently**

- **Masker:**
  - Nested block comments, char literals vs lifetimes (`'\''`, `'{'`, `b'"'`) and doc attributes all handled.
  - `r#"…"#` is handled. `br"\"` and `cr"…"` are mishandled, but that only causes loud false reds, and a model file has none.
- **cfg and attributes:**
  - `#[cfg(test)]` on a field or variant gives a loud panic.
  - Two same-named items under `cfg(feature)` / `cfg(not(feature))` trip the duplicate-name panic.
- **Type handling:**
  - Generics, `where` clauses, tuple structs, type aliases and `pub use` from outside `model/` all fail loudly.
  - `Box<Predicate>` and `Vec<Predicate>` map to the union correctly.
- **Map keys:**
  - ess itself refuses an undeclared key type ("a map key must be a primitive, or a newtype of one").
  - Optional or nested maps give a loud false red.
- **Gate step 3:** in ess 0.52.0 (`main.rs:4113`) the summary is always the last line printed, after the `refused:`, `outside:` and `note:` lines, so the parse is sound.
- **Other checks:**
  - All `# read from:` citations land on the declared lines.
  - The CI install step is byte-identical to commission's.
  - AGENTS.md agrees with ADR 0067: ess/ holds no protocol semantics and synthesizes 0 scenarios.

**6. Paths written outside the worktree**

- `~/.cache/ga-wave-2026-10-04-w2/canon-ess-hard-gate/scratch/adv2/` (`red-model.log`, `red-mapkey.log`, `suite.log`)
- `~/.cache/b10x-target/canon-w2-ess-hard-gate/tmp/adversary2-ess-model/`: 44M probe build plus copies. I deleted it, but every run of `adversary2_ess_model` recreates it.
- My scratch probe copies `p1/` and `p2/`: deleted.

```findings
- file: AGENTS.md
  line: 32
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "AGENTS.md says ESS specifies Canon's command surface, but the compiled spec declares 0 commands while canon-cli defines one, and ADR 0076 still owes it"
- file: crates/canon/tests/ess_gate.rs
  line: 393
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "toolchain_is_the_pinned_release observes the PATH binary (reason self), not the pin, because the ess dispatcher never finds ess/ess-inputs.yaml from the repository root"
- file: crates/canon/tests/ess_model_matches.rs
  line: 569
  category: mutant
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "last_segment matching lets a foreign same-named type such as crate::ir::Claim pass as the model type, contrary to the file's own rule that other Rust types match nothing"
- file: crates/canon/tests/ess_model_matches.rs
  line: 431
  category: mutant
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "a field gated by cfg(feature) is counted as model because field attributes are stripped, so a field absent from the default build still matches ess/"
- file: crates/canon/tests/ess_model_matches.rs
  line: 137
  category: mutant
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "authored_map_keys reads unlisted files in ess/domains, so a stale copy can mask a map-key change in the compiled specification"
```

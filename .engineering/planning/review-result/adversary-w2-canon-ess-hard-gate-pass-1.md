---
format: aep.planning-md/3
id: review-result:adversary-w2-canon-ess-hard-gate-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w2 adversary, canon story:ess-hard-gate, pass 1
relations:
- reviews: story:ess-hard-gate
revision: 1
---
unit: canon/ess-hard-gate, uncommitted working tree on base dac4265 in ~/.local/state/worktree/trees/b10x/canon/canon-w2-ess-hard-gate
verdict: NEEDS-CHANGE (broken: 3 red cases)
cases: executed 51→58, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths
needs-coordinator: none

**1. Diff stat**

`git --no-pager diff --stat` (the implementor's tracked edits; I changed none of them):
```
 .github/workflows/check.yml | 16 ++++++++++++++++
 AGENTS.md                   | 16 +++++++++++++---
 Taskfile.yml                |  8 +++++++-
 3 files changed, 36 insertions(+), 4 deletions(-)
```
My only path is the untracked file `crates/canon/tests/adversary_ess_model_mutants.rs`, which is a test file. I edited nothing outside test files.

**2. Cases added** in `~/.local/state/worktree/trees/b10x/canon/canon-w2-ess-hard-gate/crates/canon/tests/adversary_ess_model_mutants.rs`

Each mutant case copies `ess/` and `src/model/` into `CARGO_TARGET_TMPDIR`, edits the copy, and runs the unit's own `ess_model_matches.rs` unchanged on it, from a build directory that belongs to that mutant. The case then asserts that the unit's test fails.

| case | now |
|---|---|
| `control_the_unmutated_copy_passes` (shows the harness is faithful) | green |
| `a_changed_identifier_representation_is_caught` | **red** |
| `a_field_added_beside_a_same_named_fixture_is_caught` | **red** |
| `the_specification_bounds_revision_as_the_model_does` | **red** |
| `a_multiline_attributed_type_change_in_a_union_payload_is_caught` | green (holds) |
| `a_renamed_struct_is_caught` | green (holds) |
| `a_changed_union_payload_is_caught` | green (holds) |

My first run of this file alone was invalid: the probes shared one build directory and ran each other's binaries. I deleted those probe directories, gave each mutant its own, and added an assertion that the binary that ran came from that mutant's build. The valid run of the file alone (`scratch/adv/red-run-2.log`), verbatim excerpts:
```
---- the_specification_bounds_revision_as_the_model_does stdout ----
panicked at crates/canon/tests/adversary_ess_model_mutants.rs:221:5:
canon.protocol.Protocol declares no invariant on revision; ESS Integer admits negative revisions that the Rust model (u64) refuses. Invariants: []
---- a_field_added_beside_a_same_named_fixture_is_caught stdout ----
mutant shadowed-artifact changes the Rust model and ess_model_matches::specification_and_model_are_equal still passes:
test specification_and_model_are_equal ... ok
---- a_changed_identifier_representation_is_caught stdout ----
mutant identifier-wraps-u64 changes the Rust model and ess_model_matches::specification_and_model_are_equal still passes:
test specification_and_model_are_equal ... ok
test result: FAILED. 4 passed; 3 failed
```

**3. Suite**, run after the cases existed:
- `cargo fmt --all --check`: exit 0, after I ran `rustfmt` on my own file only.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0.
- `cargo test --workspace --locked --no-fail-fast`: exit 101.
  - My file: 4 passed, 3 failed (the 3 above).
  - Unit's tests: `ess_gate` 7 passed, `ess_model_matches` 9 passed.
  - Other targets: `source_model` 21, `validate` 6, `adversary2_source_model` 4, `adversary2_validate` 2, `adversary_source_model` 1, `adversary_validate` 1, all passed.
  - 58 cases ran. The 51 before is this run minus my 7.

**4. Findings**

| file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|
| `ess/domains/protocol.yaml:208-209`; `crates/canon/tests/ess_model_matches.rs:439` | `revision: Integer` has no bound. ESS Integer covers `[i64::MIN, i64::MAX]` (ess `docs/design/review-primitive-semantics.md:34`), while the Rust field is `u64`. So the spec admits negative revisions the model refuses, and the test maps `u64` to `integer` as if the two were equal. Fix, tested on a scratch copy: add `invariants: [revision >= 0]` to `Protocol`. With it, validate exits 0, compile exits 0 and synthesize reports 0 refusals. Values above i64::MAX cannot be expressed in ESS. | NEEDS-CHANGE | introduced | Nothing consumes the range today: 0 scenarios, and no generated code is in scope. |
| `ess_model_matches.rs:479-484` | Every `identifier!` type is recorded as `newtype of string` without reading what the macro wraps. Changing `pub struct $name(String)` to `u64` passes. | INFEASIBLE | introduced | Nothing found. `ids.rs` says identifiers wrap text as written. |
| `ess_model_matches.rs:498`, `:518` | Structs and enums from all model files go into one map by bare name, and the last one wins. With a `mod tests { struct Artifact {..} }` in `predicate.rs`, a field added to the real `Artifact` passes. | INFEASIBLE | introduced | No duplicate names or test modules in `model/` today. Function-local helper types already exist (`predicate.rs:35` `enum Written`). |

**5. Attacked and held**
- Spec against model: all 12 structs, 9 newtypes, `Truth` and the five `Predicate` variants match by hand read.
- Union tag `kind` against `EvidenceMatch.kind`: no collision, because ESS stores a union's payload separately from its tag.
- Parser edits that are caught:
  - multi-line type with a doc comment and a serde attribute, in a payload reached only through the union
  - renamed struct
  - changed union payload
  - header field added (the unit's own test)
- Parser edits that fail loudly, not silently: type alias, block comment, tuple struct, `pub(super)`, struct-like variant.
- `#[serde(rename)]` and `#[serde(default)]` change only the document text. The story leaves that text out of scope.
- `ess_gate` refusal parse holds against the real output `0 scenario(s) (0 authored), 0 refusal(s), written to …`.
- Every gate step checks the exit status. A missing `ess` panics. `toolchain which` reports the binary actually running, because there is no `ess-inputs.yaml` at the root. The marker scan recurses and includes hidden and non-YAML files.
- CI install step:
  - The pinned SHA matches the release's SHA256SUMS line and the downloaded tarball.
  - SHA256SUMS lists that asset twice, identically, and `sha256sum --check` handles that.
  - The tarball unpacks to the directory that is added to `GITHUB_PATH`.
  - The step is identical to commission's, whose check passes.
  - It runs before `task check`.
- Taskfile: `ess-gate` is the first command in `check`. The `--test` target names must exist, so a typo errors rather than running nothing. Both targets also run again in `cargo test --workspace`: a duplicate run, not a defect.

**6. Written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w2/canon-ess-hard-gate/scratch/adv/` (24M): logs, IR, suite JSON, the ess tarball, SHA256SUMS, the `ess-fix/` probe.
- `~/.cache/b10x-target/canon-w2-ess-hard-gate/tmp/adversary-ess/` (260M): mutant copies, each with its own build directory. My test file recreates it on every run.

**7. Findings block**
```findings
- file: ess/domains/protocol.yaml
  line: 209
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "revision is an unbounded ESS Integer while the model's u64 refuses negatives, and ess_model_matches.rs:439 equates u64 with integer; an invariant revision >= 0 closes it with 0 refusals"
- file: crates/canon/tests/ess_model_matches.rs
  line: 481
  category: mutant
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "identifier! newtypes are hard-coded as wrapping string, so changing the macro's wrapped type to u64 leaves the test green"
- file: crates/canon/tests/ess_model_matches.rs
  line: 498
  category: mutant
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "model types are keyed by bare name with last definition winning, so a same-named fixture in a later file hides a field added to the real struct"
```

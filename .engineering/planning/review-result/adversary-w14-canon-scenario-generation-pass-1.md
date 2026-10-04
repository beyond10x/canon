---
format: aep.planning-md/3
id: review-result:adversary-w14-canon-scenario-generation-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w14 adversary, canon story:scenario-generation, pass 1
relations:
- reviews: story:scenario-generation
revision: 1
---
unit: canon/scenario-generation, working tree = f5c070f (phase 1) + phase 2 uncommitted
verdict: CONFIRMED (4 red cases, all warning or note; no blocker)
cases: executed 488→494, red 4
origin: introduced 7 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (part 6)
needs-coordinator: yes. Should a protocol that yields 0 scenarios be refused? Separately, the implementor's question about the scope of `crates/canon/src/lib.rs` needs no decision: the story's scope already lists that file.

**1. Diff and scope**

`git --no-pager diff --stat` lists only the implementor's 8 tracked phase-2 files (66 insertions, 22 deletions). I changed none of them. My additions are 2 new untracked test files:
- `crates/canon/tests/adversary_scenario_generation.rs`
- `crates/canon-cli/tests/adversary_scenario_generation.rs`

I touched no non-test path and made no `git add`.

**2. Cases** (each run alone, before the suite)

| case | asserts | now |
|---|---|---|
| `a_scenario_that_cannot_be_written_leaves_no_partial_output` | if a write fails, `--out` stays empty | RED: `canon generate exited Some(2) (error[unreadable]: …: File name too long (os error 36)) and left ["outcome.a.legitimate.yaml"] in --out` |
| `a_protocol_path_through_a_symbolic_link_out_of_the_working_directory_runs_or_is_refused` | `generate` refuses the path, or `conform run` from the same directory passes | RED: `unreadable: out/…: fixture `link.yaml` resolves outside the working directory` … `conform: 0 passed, 0 failed, 3 unreadable` |
| `a_protocol_that_yields_no_scenario_is_refused_or_leaves_a_runnable_registry` | same, for `subject-bound-evidence-match.yaml` | RED: `canon generate exited 0 (… 0 scenarios), but canon conform run refuses what it left: no scenario ran` |
| `scenario_file_names_do_not_collide_on_a_case_or_normalisation_insensitive_filesystem` | no two file names fold to the same name | RED: `[["outcome.A.legitimate.yaml", "outcome.a.legitimate.yaml"], ["outcome.e\u{301}.legitimate.yaml", "outcome.é.legitimate.yaml"]]` |
| `a_non_empty_out_is_refused_and_left_as_it_was` | `out-not-empty`, exit 1, existing file untouched | green; kills mutant M1 |
| `every_generated_witness_passes_and_is_minimal_over_every_investigation_protocol` | across all 20 generating fixtures plus an inline subject-bound protocol: every scenario passes, and removing any one evidence, authority or decision entry makes it fail | green; kills mutant M2 |

Mutants, each in a scratch copy with its own target dir, both deleted:

| mutant | change | existing suite | my case |
|---|---|---|---|
| M1 | `if false && entries.next().is_some()` in `generate.rs:50` | green (`scenario_generation` 1 passed, `generate` unit tests 9 passed) | red (`generate.rs:150`) |
| M2 | outcome witness `legitimate.last()` | red | red, 19+ `still passes without …` lines, including authority dimensions |

**3. Gate** (unit build dir, run after the cases existed)

| cmd | exit | summary |
|---|---|---|
| fmt | 1, then 0 | the first run flagged only my 2 files; I ran `rustfmt` on them and re-ran |
| clippy | 0 | `Finished` |
| test `--no-fail-fast` | 101 | 84 result lines, passed=490 failed=4. `error: 2 targets failed: -p b10x-canon --test adversary_scenario_generation`, `-p canon-cli --test adversary_scenario_generation` |
| ess | 0 | `canon v1 — 3 file(s), valid`; `0 scenario(s) (0 authored), 0 refusal(s)` |
| canon-docs | 0 | `canon-docs: 15 generated files are current` |
| conform run | 0 | `conform: 9 passed, 0 failed, 0 unreadable` |

`-- --list` shows all 6 cases in this tree.

**4. Judgement findings** (no case written)
- **Shared decision** (`generate/mod.rs:222`, introduced): if outcomes `o1` and `o2` both require decision `d`, `o1`'s witness also carries the `d` entry for `o2`. Removing that entry leaves `o1` legitimate. The scenario still fails, but only because `o2`'s entry in the section changes. This meets the story's evidence-only definition of minimal. A probe confirmed it.
- **Windows names** (`generate/mod.rs:190`, introduced): `\` is refused "for Windows", but `: * ? < > |` are not; all of them are illegal in Windows file names. I checked on Linux only.

**5. Attacked, not broken**
- One scenario per outcome and per blocked action, on all 20 fixtures that generate.
- `conform run` passes on every one of them, including decisions, authority and subject-bound matches.
- Least-weight ties: `min_by_key` over a fixed index order, so deterministic.
- YAML round-trip of `" \ # & * ! % @ ` | > { [ , ? - '`, U+E000 and U+1FFFE through `conform`. U+FEFF, U+FFFE, U+200B and U+00AD are rejected earlier, by the parser or the identifier check.
- `..`, absolute and drive-letter paths are refused.
- The `check/` diff against 8d1599e changes visibility only.
- The CLI reference, the status row and `product.json` are accurate.

**6. Written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w14/canon-scenario-generation/scratch/adversary-p1/gate/` (gate logs)
- `~/.cache/b10x-target/canon-w14-scenario-generation/tmp/adversary-generate-{long,symlink,empty,non-empty}`, created by my tests in the unit's build dir
- Probe and mutant copies, and their target dir, are already deleted.

**7. Findings**

```findings
- file: crates/canon-cli/src/generate.rs
  line: 65
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "a scenario file that cannot be created (id over ~230 bytes, ENAMETOOLONG) exits 2 after earlier files are written, leaving a partial --out that every re-run refuses as out-not-empty"
- file: crates/canon/src/generate/mod.rs
  line: 133
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "a protocol path confined as written but symlinked outside the working directory is accepted, and canon conform run from that directory reports every generated scenario unreadable, contrary to the command's help text"
- file: crates/canon-cli/src/generate.rs
  line: 40
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "a protocol with no outcome and no blocked action (fixtures/investigation/subject-bound-evidence-match.yaml) generates 0 scenarios with exit 0, leaving a registry canon conform run refuses"
- file: crates/canon/src/generate/mod.rs
  line: 356
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "outcomes A and a, or NFC and NFD forms of one name, get file names that are one file on case- or normalisation-insensitive filesystems (macOS, Windows defaults), so one scenario silently overwrites another; measured as a name collision on Linux, overwrite inferred"
- file: crates/canon-cli/src/generate.rs
  line: 50
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "removing the out-not-empty check left the existing suite green; a_non_empty_out_is_refused_and_left_as_it_was now kills it"
- file: crates/canon/src/generate/mod.rs
  line: 222
  category: judgement
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "an outcome's witness carries the shared decision's entries for every other outcome requiring it, which the subject does not need"
- file: crates/canon/src/generate/mod.rs
  line: 190
  category: judgement
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "backslash is refused as a Windows path separator while the other Windows-illegal file-name characters are accepted"
```

---
format: aep.planning-md/3
id: review-result:adversary-w6-canon-evaluator-skeleton-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w6 adversary, canon story:evaluator-skeleton, pass 2
relations:
- reviews: story:evaluator-skeleton
revision: 1
---
unit: canon/evaluator-skeleton, uncommitted working tree on 962ac65 (phase 2 plus pass-1 fixes), worktree ~/.local/state/worktree/trees/b10x/canon/canon-w6-evaluator-skeleton
verdict: INFEASIBLE
cases: executed 235→239, red 3
origin: introduced 4 / pre-existing 1 / undecided 0
wrote-outside-worktree: 6 paths (part 6)
needs-coordinator: the ESS slot comments conflict with the wave-7 story bodies (J1)

**1. Diff stat.** `git --no-pager diff --stat` lists 15 tracked files. They are all the implementor's, and I edited none of them. My additions are four new untracked test files and nothing else:
- `crates/canon/tests/adversary2_skel_conform.rs`
- `crates/canon/tests/adversary2_skel_authority.rs`
- `crates/canon/tests/adversary2_skel_validate.rs`
- `crates/canon-cli/tests/adversary2_skel_conform_cli.rs`

**2. Cases added (each run alone before the suite)**

| File / test | Asserts | Now |
|---|---|---|
| `adversary2_skel_conform.rs` `a_carriage_return_in_a_listed_section_is_a_difference` | A listed section with `\r` before `\n` is not the decision's bytes, so the step fails. A control without `\r` passes. | red |
| `adversary2_skel_authority.rs` `any_authority_the_scenario_accepts_reaches_the_evaluator` | Every authority list the scenario parser accepts reaches the evaluator and is refused as `unsupported-input`, with no panic. | red |
| `adversary2_skel_conform_cli.rs` `an_authority_with_a_mapping_key_does_not_abort_the_registry` | `canon conform run` exits 0 and reports both scenarios. | red |
| `adversary2_skel_validate.rs` `each_discharge_predicate_is_followed_by_its_own_evidence_matches` | With two obligations, each one's evidence matches come right after its own references (validate docs item 4). | green pin |

The green pin catches a mutant the unit's tests miss: a second loop over obligations would put both obligations' evidence matches after both sets of references. I checked this by reading the code, not by building the mutant.

Red output, verbatim:
```
panicked at crates/canon/tests/adversary2_skel_conform.rs:35:5:
an expectation that is not the decision's bytes passed: Passed
```
```
panicked at crates/canon/src/conform/mod.rs:489:45:
YAML values serialize back to YAML: Error { kind: EMITTER, problem: "expected SCALAR, SEQUENCE-START, MAPPING-START, or ALIAS" }
authority [{{x: 1}: c}]: Err("panicked")
```
```
  left: Some(101)
 right: Some(0)
stdout:
(empty)
```

**3. Suite run.** Command: `CARGO_TARGET_DIR=~/.cache/b10x-target/canon-w6-evaluator-skeleton CARGO_INCREMENTAL=0 cargo test --workspace --locked --no-fail-fast`. Exit 101. Summed over the 39 `test result` lines: 236 passed, 3 failed, 239 ran. The failures are exactly the three red cases above. `cargo clippy --workspace --all-targets --locked -- -D warnings` is clean, and the new files pass `rustfmt --check`.

**4. Findings**

| # | file:line | Finding | Verdict | Origin | What reaches it |
|---|---|---|---|---|---|
| F1 | `crates/canon/src/conform/mod.rs:489` | When an authority entry has a mapping as a key, re-serializing it panics. `canon conform run` then exits 101 with no report, so every other scenario's verdict is lost. The base returned before serializing. Fix: turn the serialization error into a step failure. This has to land here, because wave-7 stories cannot edit `conform/`. | INFEASIBLE | introduced | Nothing found: no scenario writes mapping keys. |
| F2 | `crates/canon/src/conform/mod.rs:639` (also `:609`) | `members()` reads lines with `str::lines`, which drops a `\r` before `\n`, and JSON parsing treats `\r` as whitespace. So an expectation that differs from the decision by a `\r` passes, although the docs at `:88` promise a byte-for-byte comparison. The base's whole-text comparison failed it. Fix: split with `split_inclusive('\n')` or refuse `\r`. | INFEASIBLE | introduced | Only an explicit `\r` escape in a double-quoted YAML scalar. Block scalars normalise line breaks. |
| J1 | `ess/domains/protocol.yaml:326-335` | The comments say each slot is "Typed by story:obligations / action-admissibility / outcomes", following the coordinator's decision. The story bodies disagree: obligations "no longer edits `ess/`", and the actions and outcomes sections are "not declared in `ess/` by this story or any other". Typing the slots would mean three parallel stories editing adjacent lines in `ess/`, in `model/decision.rs:27-34` and in `eval/decision.rs` `sections()`. Keeping Json makes the comments false. | CONFIRMED | introduced | Wave-7 dispatch reads both. |
| J2 | `crates/canon/src/eval/mod.rs:33-46` | The "Refusals" section says the checks run "in this order" and that "Then claims are evaluated". It leaves out the supplied-input refusals and the exclusion-stage refusals that now come between those steps. The pipeline section (`:55-79`) does give them. | CONFIRMED | introduced | Any reader of the docs. |
| J3 | `crates/canon/src/conform/mod.rs:157` | A scenario's `authority` must be a YAML list. If `canon-authority/1` gets a `format:` header like every other canon input, story:action-admissibility has to edit `conform/`. The shape is still unsettled. | CONFIRMED | pre-existing (base `:141`) | story:action-admissibility |

**5. What I attacked and could not break**
- **`claims::predicate` vs claim evaluation:** they share `all`/`any`/`not`/`evidence_match`/`tested`. The only difference is the claim lookup, and both treat a missing claim as unknown. That covers empty all/any, nested `not`, unknown propagation and a claim missing from the map.
- **F5 fallback (non-sectioned expectation):** it passes only on an exact match. Duplicate keys, other indentation, a missing trailing newline and escaped keys all still fail. Only `\r` gets through (F2).
- **Evidence-in-discharge refusal:** the match covers every predicate form and is checked by the compiler. The walk is a deterministic depth-first pass in written order, and the order across two obligations holds.
- **F6 transitive walk:** the visited set makes it terminate, and the output is sorted. It has no duplicates because `set_aside` excludes each record once. The cycle guard can't be reached through `evaluate`, because `claims::values` refuses cycles first.
- **Wave-7 independence:** each of the five stories can be built in its own file if the slots stay Json (J1 aside). The binding, freshness and outcomes signatures can each return a refusal, and actions and outcomes receive the evidence left after exclusion.

I acquired no worktree lease, because the brief forbids worktree commands.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w6/canon-evaluator-skeleton/scratch/adv2-conform.log`
- `~/.cache/ga-wave-2026-10-04-w6/canon-evaluator-skeleton/scratch/adv2-authority.log`
- `~/.cache/ga-wave-2026-10-04-w6/canon-evaluator-skeleton/scratch/adv2-conform-cli.log`
- `~/.cache/ga-wave-2026-10-04-w6/canon-evaluator-skeleton/scratch/adv2-validate.log`
- `~/.cache/ga-wave-2026-10-04-w6/canon-evaluator-skeleton/scratch/adv2-suite.log`
- `~/.cache/b10x-target/canon-w6-evaluator-skeleton/` (the assigned build dir), including `tmp/adversary2_skel_conform_cli/mapping-key/`

**7. Findings block**
```findings
- file: crates/canon/src/conform/mod.rs
  line: 489
  category: boundary
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: "An authority entry with a mapping key, which the scenario parser accepts, panics in the runner's YAML re-serialization, so canon conform run exits 101 and prints no report for any scenario."
- file: crates/canon/src/conform/mod.rs
  line: 639
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "members() reads lines with str::lines, which drops a carriage return before the newline, so an expectation that differs from the decision by a \\r passes the comparison the module docs call byte for byte."
- file: ess/domains/protocol.yaml
  line: 326
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "The slot comments say story:obligations, story:action-admissibility and story:outcomes will type their slots in ESS, but those story bodies exclude ess/ and model/, so either the comments are false or three parallel stories must edit the same adjacent lines."
- file: crates/canon/src/eval/mod.rs
  line: 33
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The Refusals section gives the order of input checks and then says claims are evaluated, leaving out the supplied-input and exclusion-stage refusals that now come between them."
- file: crates/canon/src/conform/mod.rs
  line: 157
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "A scenario's authority input must be a YAML list, so if canon-authority/1 becomes a document with a format header, story:action-admissibility has to edit conform/."
```

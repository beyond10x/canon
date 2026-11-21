---
format: aep.planning-md/3
id: review-result:adversary-argument-validator-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:argument-validator (wave 2026-11-20-w5): red, 9 red cases, 1 blocker, 3 warnings and 3 notes, introduced'
relations:
- reviews: story:argument-validator
revision: 1
---
Adversary pass 1 on story:argument-validator, wave 2026-11-20-w5, `aep:adversary` on Opus, on `impl/argument-validator` at `60c1162` (a Sonnet implementor). The report below is as the agent returned it.

unit: story:argument-validator at 60c1162 (cases committed at 0416b2ac)
verdict: red
cases: executed 1028→1042, red 9
origin: introduced 6, pre-existing 0, undecided 0
wrote-outside-worktree: 1 directory, `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w5/argument-validator/adversary-p1/` (full list in part 6)
needs-coordinator: yes

**1. Diff since 60c1162** (`git --no-pager diff --stat 60c1162 HEAD`)
```
 .../canon/tests/adversary_argument_validation.rs   | 426 +++++++++++++++++++++
 1 file changed, 426 insertions(+)
```
Only the one test file changed, so the charter held. Two points to know:
- **Mutation probes:** the brief said to mutate in the worktree and restore. The skill says never touch the file under attack, so I mutated a copy in scratch instead.
- **Temporary harness:** I put an untracked harness file, `crates/canon/tests/zz_adv1_mutants.rs`, in the worktree. It only included the scratch copies. I deleted it before the suite run and the commit. Its binary is still in `target/debug/deps/`.

**2. Cases added** in `crates/canon/tests/adversary_argument_validation.rs`: 14 cases, 9 red, 5 green.
- **Claim 5, red in both build modes:**
  - `validate_still_refuses_the_enum_the_base_refused` (:65)
  - `diff_still_prints_what_the_base_printed_for_an_enum_above_two_to_the_53` (:86)
- **Claim 3, red only when serde_json has `arbitrary_precision`:** :109, :121, :133, :146, :160, :173, :186. Each returns early when that feature is off, because serde_json then cannot hand the value over.
- **Green:**
  - mutant-killers: `pointers_escape_tilde_and_slash`, `a_type_refusal_hides_the_enum_of_that_schema`, `refusals_at_three_depths_come_in_document_order`
  - `a_ten_thousand_deep_document_does_not_overflow`
  - `random_schemas_and_documents_never_panic` (4000 schemas, 20000 documents)

Red output from running this file alone with `cargo test --workspace --locked --test adversary_argument_validation`, verbatim excerpts:
```
  left: []
 right: ["action `act` arguments at `/properties/n/enum/1`: `9007199254740992.0` is listed more than once"]
  left: ["BREAKING: arguments changed"]
 right: ["NO SEMANTIC CHANGE: arguments rewritten"]
thread 'a_number_beyond_the_double_range_is_above_every_maximum' ... panicked at ...:112:5:
  left: []
  left: Err(Arguments([ArgumentRefusal { pointer: "/n", keyword: "minimum", reason: "9007199254740993.0 is below `minimum` 9007199254740993" }]))
 right: Ok(())
test result: FAILED. 5 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s
```
- **First run was partly my error:** the three `1e400` cases first failed on my own check that the number kept its text (serde_json prints `1e+400`). I fixed that check and re-ran. All three now fail on the claim itself (`left: []`, meaning the arguments were accepted).
- **The 10,000-deep case first aborted with a stack overflow in my own test code** (`json!` serialising the value), not in the function. I fixed the test and it now passes.
- **Package mode**, `cargo test -p b10x-canon --test adversary_argument_validation`, run on the 12-case version: `test result: FAILED. 10 passed; 2 failed; ...`. Only the two claim-5 cases are red there.

**3. Suite run, after the cases existed**

`cargo test --workspace --locked --no-fail-fast` exited 101 and ended with:
```
error: 1 target failed:
    `-p b10x-canon --test adversary_argument_validation`
```
Summed over all 144 `test result` lines: 1033 passed, 9 failed, 3 ignored. That is 1042 executed, against the implementor's 1028.
- My file: `test result: FAILED. 5 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s`
- `adversary_action_arguments`: `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.11s` (same count as on the base)

**4. Findings** (they cover 60c1162)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/validate/arguments.rs:389 | contract-drift | warning | NEEDS-CHANGE | introduced | `canon validate` and `canon compile` on a protocol whose enum is `[9007199254740993, 9007199254740992.0]`: refused on the base, valid now. No fixture in the repo has such an enum. |
| crates/canon/src/diff/arguments.rs:172 | contract-drift | warning | NEEDS-CHANGE | introduced | `canon diff` on enum `[9007199254740992.0]`→`[9007199254740993]`: base `NO SEMANTIC CHANGE`, now `BREAKING`. The new output is the correct one: the base was unsound. |
| crates/canon/src/validate/arguments.rs:415 | boundary | blocker | NEEDS-CHANGE | introduced | Under `arbitrary_precision`, `1e400` has no f64, so `double` returns NaN and the value passes every `maximum`, `minimum` and `enum`. `b10x-canon-expr` turns this feature on, so `cargo test --workspace` (the gate) and any build linking both crates run in this mode. |
| crates/canon/src/validate/arguments.rs:404 | acceptance | warning | NEEDS-CHANGE | introduced | Same mode: the exact text is held but compared through `as_f64`. `-9223372036854775809` passes `minimum` i64::MIN; `9007199254740992.5` passes `maximum` 2^53; `9007199254740993.0` fails its equal `minimum` and its `enum`. The module docs (:60-62) say otherwise. |
| crates/canon/tests/argument_validation.rs:128 | mutant | note | CONFIRMED | introduced | Two mutants pass the unit's whole suite: dropping the `return` after a `type` refusal, and dropping RFC 6901 escaping. My green cases kill both. |
| crates/canon/tests/argument_validation.rs:424 | judgement | note | CONFIRMED | introduced | `a_refusal_has_a_pointer_a_keyword_and_a_reason` builds a struct and compares it to its own clone. It calls no code under test and cannot fail. |
| crates/canon/src/validate/arguments.rs:526 | judgement | note | NEEDS-CHANGE | introduced | Every call allocates the absent schema with `json!`, plus a pointer buffer, the sorted name list and the escaped tokens. The code is reasonable, so the story's "no allocation beyond its result" line is the thing to change. |

**What needs your decision (the yes on needs-coordinator):** claim 5 says outputs do not change. The new `same` output is correct per 2020-12, the base output was a defect. Either the story's wording changes and you drop my two claim-5 cases, or `same` goes back to its old behaviour for `check`/`diff` only.
- **Fixes to consider:**
  - Treat a `Number` with no finite f64 as beyond every bound and as equal to nothing.
  - When `arbitrary_precision` is on, compare the decimal text exactly, or state in the docs that the feature is not supported.
- **Base:** I compared outputs against the base binary `canon-6f44325`. `git log 6f44325..619fd03` is empty for `validate/arguments.rs` and `diff/arguments.rs`. I rebuilt the HEAD binary. The protocols I fed both binaries use only actions and arguments.

**5. Attacked and not broken**
- **Move of the oracle:** `accepts` and the generators are the same as on the base apart from `pub`, a module doc, an `allow` and one rustfmt reflow of `roll`. `adversary_action_arguments.rs` lost no assertion.
- **Mutants on a scratch copy, 15 in total:** the unit's suite kills 13; the oracle comparison alone kills 10.
  - Missed by the oracle: maxLength counted in bytes, `same` via f64, member order (all three caught by hand-written cases), plus escaping and the type early return (caught by nothing).
- **Exact comparison in default mode:** i64 against u64 above i64::MAX, `-0.0` against `0`, `1e19` against i64::MAX, a double against u64::MAX, an integer against a fraction on each side, `1e300` as an integer.
- **Keywords against 2020-12:** `required` with a member set to `null`, `properties` with no `additionalProperties`, `items` on an empty array.
  - Also: `true`, `false` and `{}` as schemas; a `$ref` nested in a schema; an absent schema.
- **Order and determinism:** refusals at three depths, two runs, RFC 6901 escaping.
- **No panic:** 4000 fuzz schemas with 20000 documents; a 10,000-deep document.
- **Shared files:** the hunks stay in scope. `protocol.yaml` also gains one `# read from:` line, which the story requires.

**6. Paths written outside the worktree**, all under `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w5/argument-validator/adversary-p1/`:
- `base_aaa.rs`, `base_oracle.rs`, `head_oracle.rs`, `cmp_move.sh`
- `probe/` (4 yaml, `run.sh`, 6 json)
- `mut/` (`mutant.rs`, `av_copy.rs`, `adv_copy.rs`, `make.sh`, `switches.py`, `run.sh`, `run2.sh`, `build.log`, `summary.log`, `summary2.log`, `run-*.log`)
- `build.log`, `alone-ws.log` to `alone-ws4.log`, `alone-p.log`, `suite.log`, `msg-1.txt`, `fuzz.rs`, `splice.py`, `count.sh`

```findings
[{"file": "crates/canon/src/validate/arguments.rs", "line": 389, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "exact same makes canon validate and canon compile accept an enum [9007199254740993, 9007199254740992.0] the base refused, against claim 5"},
 {"file": "crates/canon/src/diff/arguments.rs", "line": 172, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "canon diff of enum [9007199254740992.0] to [9007199254740993] prints BREAKING where the base printed NO SEMANTIC CHANGE, against claim 5 though the new line is the sound one"},
 {"file": "crates/canon/src/validate/arguments.rs", "line": 415, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "with arbitrary_precision on, as in cargo test --workspace, 1e400 becomes NaN and passes every maximum, minimum and enum"},
 {"file": "crates/canon/src/validate/arguments.rs", "line": 404, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "with arbitrary_precision the exact text is compared through as_f64, so -9223372036854775809, 9007199254740992.5 and 9007199254740993.0 get the wrong bound and enum result"},
 {"file": "crates/canon/tests/argument_validation.rs", "line": 128, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the unit's suite stays green with the type early return dropped or with pointer escaping dropped"},
 {"file": "crates/canon/tests/argument_validation.rs", "line": 424, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "a_refusal_has_a_pointer_a_keyword_and_a_reason compares a struct with its clone and calls no code under test"},
 {"file": "crates/canon/src/validate/arguments.rs", "line": 526, "category": "judgement", "severity": "note", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the story's no allocation beyond its result line is not met and its wording should change"}]
```

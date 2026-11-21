---
format: aep.planning-md/3
id: review-result:adversary-argument-validator-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 on story:argument-validator: red, 1 finding to change and 4 notes'
relations:
- reviews: story:argument-validator
revision: 1
---
Second and last adversary pass on the unit of story:argument-validator, after its correction (head 8999964; the adversary's cases at 2b76d32e). Recorded as the adversary returned it.

unit: story:argument-validator at 8999964 (cases committed at 2b76d32e)
verdict: red
cases: executed 1051→1057, red 4
origin: introduced 5, pre-existing 0, undecided 0
wrote-outside-worktree: 1 directory, `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w5/argument-validator/adversary-p2/`
needs-coordinator: yes

**Diff since 8999964:** `crates/canon/tests/adversary_argument_validation.rs | 494 +++++++++++++++++++--`, one test file and nothing else. The mutant harness `crates/canon/tests/zz_adv2_mutants.rs` was in the tree only temporarily and is deleted; its binaries stay in `target/`. All mutation was done on scratch copies.

**What's red:** four new cases, all red only when serde_json holds number text (`cargo test --workspace`) and green with doubles (`cargo test -p b10x-canon`). The D1 cases `:89` and `:102` are re-pinned to the new outputs and pass. My first two runs of the file alone were my own mistakes, not findings: a borrow error that did not compile, then invalid JSON from my generator. Both are fixed; the runs below are after the fixes.

Red output, `cargo test --workspace --locked --test adversary_argument_validation`, verbatim excerpts:
```
thread 'a_yaml_enum_value_a_double_holds_exactly_is_that_integer' ... panicked at ...:467:5:
  left: Err(Arguments([ArgumentRefusal { pointer: "/n", keyword: "enum", reason: "the value is not one of the `enum` values" }]))
 right: Ok(())
thread 'diff_finds_no_change_when_an_enum_value_is_rewritten_as_its_exact_double' ... panicked at ...:488:5:
  left: ["BREAKING: arguments changed"]
 right: ["NO SEMANTIC CHANGE: arguments rewritten"]
test result: FAILED. 16 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.63s
```

**Suite runs**
- **Workspace:** `cargo test --workspace --locked --no-fail-fast` exited 101 with `error: 1 target failed:` / ``    `-p b10x-canon --test adversary_argument_validation` ``. Over 144 summaries: 1053 passed, 4 failed, 3 ignored.
  - argument_validation: `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.59s`
  - my file: `test result: FAILED. 16 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.22s`
  - adversary_action_arguments: `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.11s`
- **Package:** `cargo test -p b10x-canon --no-fail-fast` (the 3 files) exited 0.
  - argument_validation: `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.09s`
  - my file: `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s`
  - adversary_action_arguments: `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.98s`

**Findings** (they cover 8999964)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/validate/arguments.rs:458 | acceptance | warning | NEEDS-CHANGE | introduced | The YAML reader makes `1152921504606846976.0` a double that holds 2^60 exactly. With text held, `compare` reads it as its shortest text `1152921504606847000`. Three wrong answers by 2020-12 follow (cases `:462`, `:473`, `:487`), listed below. Reached by a runner whose serde_json has `arbitrary_precision` (any build with `b10x-canon-expr`; the README offers both libraries) and by a workspace-built `canon`. The shipped binary and the pass-1 code answer right. |
| crates/canon/tests/adversary_argument_validation.rs:501 | acceptance | note | CONFIRMED | introduced | The same literal `9007199254740993.0` in the protocol's `enum` and in the arguments is refused with text held, and accepted with doubles. |
| crates/canon/tests/argument_validation.rs:482 | judgement | note | CONFIRMED | introduced | The unit pins the wrong answer of row 1 as expected: `from_f64(2^60)` refuses the integer 2^60 with text held. It needs re-pinning with any fix. |
| crates/canon/src/validate/arguments.rs:74 | contract-drift | note | CONFIRMED | introduced | `# Numbers` opens with "Two numbers are the same when their values are equal exactly". Its own third bullet at :105 says a number made from a double is compared by another value. |
| crates/canon/src/validate/arguments.rs:1033 | mutant | note | CONFIRMED | introduced | The doc says this test covers "carry and borrow across every digit", but the unit's whole suite stays green in both builds when the borrow in `subtract_digits` is dropped. My `:761` kills that mutant. |

The three wrong answers in row 1:
- `enum: [1152921504606846976.0]` refuses the arguments `1152921504606846976`.
- `canon validate` accepts `[1152921504606846976, 1152921504606846976.0]` as two distinct values.
- `canon diff` prints `BREAKING` for that rewrite.

**Decision needed (the yes on needs-coordinator):** whether the row-1 answers stand as documented build differences or get fixed. One fix in a file outside this unit, `model/present.rs:121`: turn a YAML float that is an exact 64-bit integer into an integer. That repairs `:462`, `:473` and `:487`; `:501` stays, because the YAML parser has already rounded that literal.

**Pass 1 findings**
1. arguments.rs:389 (validate and compile accept the enum): answered by D1; `:89` is re-pinned and green.
2. diff/arguments.rs:172 (`BREAKING`): answered by D1; `:102` is re-pinned and green.
3. arguments.rs:415 (NaN from `1e400`): answered by the correction; `:117`–`:141` are green in both builds.
4. arguments.rs:404 (rounding with text held): answered for numbers read from JSON (`:154`–`:194` green). Numbers read from YAML remain open as new row 1.
5. argument_validation.rs:128 (mutant survivors): half answered. Pointer escaping is now pinned by the rewritten `:712`. The `type` early return is still killed only by my `:228`.
6. argument_validation.rs:424 (self-referential test): answered; it now calls the function and checks its output.
7. The story's "no allocation" line: answered by D3, which withdraws it.

**Which build is which, and where they differ**
- **Shipped binary holds doubles.** It is a source release built with `cargo install --locked --git … canon-cli` (README:30). `cargo tree -p canon-cli -e features -i serde_json` shows only `default`, `float_roundtrip`, `unbounded_depth`. No workflow builds a release binary.
- **Workspace build holds text.** It adds `arbitrary_precision`, so the gate's CLI tests run a different binary from the one users install. D2's "the `canon` binary holds text" is wrong for the installed binary.

Where the two builds give a user different answers:
1. `canon validate` and `canon compile`: an enum holding an integer above 2^53 and its exact double written with a fraction. Shipped refuses it as a duplicate; the workspace build accepts it. Tested.
2. `canon diff` of that rewrite: shipped prints `NO SEMANTIC CHANGE`; the workspace build prints `BREAKING`. Tested.
3. `canon evaluate` and `canon diff` reading a canon-ir/1 that `canon compile` did not write. I found these by reading the code; none was run through the CLI.
   - `enum: [1e400]`: refused as malformed by shipped, read by the workspace build.
   - `minimum: -0`: refused by shipped, accepted by the workspace build.
   - `[9007199254740993, 9007199254740993.0]`: two values in shipped, one duplicate in the workspace build.
   - `[9007199254740993.0]`→`[9007199254740993]`: `BREAKING` in shipped, rewritten in the workspace build.
4. `check_arguments` depends on the runner's own build.
   - Every JSON literal that is not exactly a double: `9007199254740992.5`, `9007199254740993.0`, `-9223372036854775809`, `1e-400`, `1e400`.
   - YAML numbers with a fraction against integers above 2^53: `:462`, `:501`.
   - A `-0` bound.

**Attacked and not broken**
- **Decimal comparison, text held:**
  - 300,000 random pairs (as an enum value and as `type: integer`) and 100,000 against nine 64-bit bounds, checked against my own oracle that aligns digit strings at the point. 0 disagreements.
  - 15 written pairs: exponents beyond 128 bits on both sides, digits differing at the 400th place, `1e+0005`, `100e-2`, `1.0e1`, `0e5`/`-0`, `0.000` against `0e-99999999999999999999`, negative against negative.
- **Run-time detection:** a `OnceLock`, safe under threads, and the feature is the same for the whole process. In a text build every number holds text (JSON, `from_f64`, `json!`, YAML through `from_f64`). Only the shortest text of a double is skewed, which is row 1.
- **`type: integer`:** `1.0`, `1e0`, `10e-1`, `1e400`, `1.5e1`, `-0.0`, `1e-400` match 2020-12 and the docs in both builds, inside the 300k fuzz.
- **Claim 6:** argument_validation runs 18 of 18 in both builds; `support/schemas.rs` is untouched since 0416b2a. The unit's skips happen for the reasons they state (`1e400` is refused without the feature).
- **Mutants on a scratch copy:** 11 in total.
  - With text held, the unit's suite kills 10. It misses the dropped borrow, which my file kills.
  - The mutant that flips the order of two negative scales is killed only by the unit's inner test. It cannot be seen through the public function, because bounds are integers.
  - With doubles, the unit's inner tests kill 9 of 11. The two survivors are the dropped borrow and detection switched off (expected, since that build never takes the text path).
- **`# Numbers`:** every sentence was checked against the code; all hold except row 4.
- **Pass-1 cases:** all 9 are green in both builds. Clippy `-D warnings` on my file: exit 0.

**Paths written outside the worktree**, all under `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w5/argument-validator/adversary-p2/`: `head.rs`, `tail.rs`, `new_head_tail.rs`, `assemble.py`, `alone.sh`, `suite.sh`, `count.sh`, `msg-1.txt`, `alone-ws-{1,2,3}.log`, `alone-p-{1,2,3}.log`, `clippy-{1,2,3}.log`, `suite-ws.log`, `suite-p.log`, and `mut/` (`make.py`, `mutant.rs`, `av_copy.rs`, `adv_copy.rs`, `run.sh`, `all.sh`, `build-ws.log`, `build-p.log`, `summary.log`, `run-ws-*.log`, `run-p-*.log`).

```findings
[{"file": "crates/canon/src/validate/arguments.rs", "line": 458, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "with the text held compare reads a YAML-made double by its shortest text, so enum [1152921504606846976.0] refuses 1152921504606846976 and canon validate and canon diff misjudge that rewrite though the double holds the literal exactly"},
 {"file": "crates/canon/tests/adversary_argument_validation.rs", "line": 501, "category": "acceptance", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the same literal 9007199254740993.0 in the protocol enum and in the arguments is refused where serde_json holds text"},
 {"file": "crates/canon/tests/argument_validation.rs", "line": 482, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the unit pins as expected that from_f64(2^60) refuses the integer 2^60 where the text is held"},
 {"file": "crates/canon/src/validate/arguments.rs", "line": 74, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the Numbers section says numbers compare by exact value and its third bullet says a number made from a double does not"},
 {"file": "crates/canon/src/validate/arguments.rs", "line": 1033, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "dropping the borrow in subtract_digits leaves the unit's suite green in both builds although the test claims to cover borrow"}]
```

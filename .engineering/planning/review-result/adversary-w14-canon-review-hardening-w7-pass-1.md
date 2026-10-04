---
format: aep.planning-md/3
id: review-result:adversary-w14-canon-review-hardening-w7-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w14 adversary, canon story:review-hardening-w7, pass 1
relations:
- reviews: story:review-hardening-w7
revision: 1
---
unit: canon/review-hardening-w7, working tree = 49fe773 plus phase 2 uncommitted, plus my 2 untracked test files
verdict: NEEDS-CHANGE
cases: executed 528→535, red 6
origin: introduced 3 / pre-existing 5 / undecided 0
wrote-outside-worktree: 3 locations (part 6)
needs-coordinator: finding 1. The release `canon evaluate` aborts on a protocol that `canon compile` accepts. It is pre-existing, but it is the same "never aborting" promise this story makes. Hold the unit, or open a new story?

**1. `git --no-pager diff --stat`**: `30 files changed, 622 insertions(+), 232 deletions(-)`. That is the implementor's diff, unchanged. My two files are untracked: `git status` shows `?? crates/canon/tests/adversary_review_hardening.rs` and `?? crates/canon-cli/tests/adversary_review_hardening.rs`. I touched no non-test path. The only edits after the first run were rustfmt and two clippy lint fixes in my own file.

**2. Cases added** (each file run alone first; the red lines below are from that run)

| Case | Asserts | Now |
|---|---|---|
| lib `a_chain_of_claims_each_within_the_bound_does_not_abort` | 64 chained claims, each under 4000 `not`s (every predicate within the bound), evaluate or are refused | red: `ExitStatus(unix_wait_status(134))` … `thread 'canon-eval' has overflowed its stack` |
| lib `check_of_a_caller_built_ir_beyond_the_bound_does_not_abort` | `check::check` refuses a 2,000,000-deep IR as `predicate-too-deep` | red: child `has overflowed its stack`, status 134 |
| lib `depth_in_a_later_member_of_all_or_any_is_held_to_the_bound` | the deep member stands second in an `all`/`any`: refused at 4097 levels, evaluated at 4096 | green. It is red against mutant A (part 4) |
| lib `an_explicit_null_on_a_required_key_does_not_advise_leaving_it_out` | `subject: ~` is not refused with "leave the key out" | red: `…an explicit null is not allowed here; leave the key out to take its default` / leaving it out: `…missing field \`subject\`` |
| lib `a_scenario_holds_the_subject_bound_reason_shape_in_actions_and_outcomes` | the story's last Outcome bullet | red: `left: [("actions", false), ("outcomes", false)]` |
| cli `a_compiled_chain_of_claims_evaluates_without_aborting` | `canon compile` then `canon evaluate` on 1500 claims × 100 `not`s exits 0 or 1 | red: `ExitStatus(unix_wait_status(134)); … 'canon-eval' has overflowed its stack` |
| cli `an_evidence_refusal_after_reading_names_the_file` | a repeated id, an undeclared kind or a bad kind names the file | red: `repeated-id: … error[duplicate-identifier]: evidence \`e1\` is given more than once` (no file) |

**Origin**: I ran both files against a `git archive a0cc691` export in scratch. All 6 red cases are also red there. claims.rs is unchanged since base.

**Release reach**: I built `canon` with `--release`. At 1500 claims × 100 it exits 0. At 5000 × 100 it exits 134, `thread 'canon-eval' has overflowed its stack`; the protocol is about 3.7 MB, the IR 117 MB.

**3. Suite**, run after the cases existed, in the unit's build dir. Logs are in `scratch/adversary-p1/g*.log`.

| Command | Exit | Summary |
|---|---|---|
| `cargo fmt --all --check` | 0 | |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized] target(s) in 0.09s` |
| `cargo test --workspace --locked --no-fail-fast` | 101 | 95 result lines; totals 529 passed, 6 failed, 3 ignored. The two failing lines: `test result: FAILED. 1 passed; 4 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.45s` and `test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.11s`. The 6 failures are exactly my red cases; my test names appear in the run, so this tree ran |
| `ess specify validate … && ess verify conform synthesize …` | 0 | `canon v1 — 3 file(s), valid`, `0 scenario(s) (0 authored), 0 refusal(s)` |
| `canon-docs generate --check` | 0 | `canon-docs: 16 generated files are current` |
| `canon-cli conform run` | 0 | `conform: 10 passed, 0 failed, 0 unreadable` |

The last three ran before my two lint fixes; they read no test file.

**4. Findings** (they cover the working tree above)

| # | file:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|---|
| 1 | `crates/canon/src/eval/claims.rs:85` | NEEDS-CHANGE / pre-existing, warning | Claim evaluation recurses through claim references as well as nesting. `check_depth` bounds one predicate only, so a chain of claims each within the bound overflows the 64 MiB thread. The new text at `eval/mod.rs:105` ("checks the bound itself before any step recurses") reads as a guarantee it does not give | Any protocol author, through `canon compile` and `canon evaluate`, release build included. `canon check` evaluates the same way. Fix options: memoised iterative claim evaluation in dependency order, or a bound on summed depth along reference chains |
| 2 | `crates/canon/src/check/mod.rs:324` | INFEASIBLE / pre-existing, warning | `check::check` walks predicates recursively on the caller's thread before `evaluate_with`'s depth check, and aborts | Only Rust callers that build an `Ir` themselves; this is the same audience as review finding F1, which this story fixed. Fix: call the depth check first |
| 3 | `crates/canon-cli/src/evaluate.rs:80` | NEEDS-CHANGE / pre-existing, warning | Only read-time refusals name the file. A repeated id across two files says `evidence \`e1\``, which is ambiguous | Any evidence directory. The story says "every evidence refusal names the record, and `canon evaluate` names the file" |
| 4 | `conformance/scenarios` | NEEDS-CHANGE / pre-existing, warning | No scenario expects `{evidence, present, subject}` in `actions` or `outcomes` | The story's last Outcome bullet. The implementor disclosed it as not done |
| 5 | `crates/canon/src/model/present.rs:10` | CONFIRMED / pre-existing, note | A null on a required key advises leaving the key out, which then fails with `missing field` | Any author who writes `key: ~` on a required field |
| 6 | `crates/canon/src/eval/read.rs:162` | CONFIRMED / introduced, note | Mutant A, `members.iter().take(1)`: all 93 other binaries stay green. Only my later-member case goes red (`panicked at …:191:59: all: Decision {…}`) | A test gap only; my case closes it |
| 7 | `website/docs/reference/documents.md:106` | CONFIRMED / introduced, note | The page says "an entry given twice is refused", and the schema's `uniqueItems` accepts `[{a, granted}, {a, denied}]`. Canon refuses a capability decided twice, whatever the decision | Readers of the page and users of the schema |
| 8 | `crates/canon-cli/tests/evaluator_skeleton.rs:339` | CONFIRMED / introduced, note | The doc comment still says a usage error is "exit 2" | Readers of the test |

**5. Attacked and held**
- **Depth bound**: 4096 is accepted and 4097 refused in every section. The check itself is iterative, and `read_ir` is stricter than `check_depth`. Mutant A is the only gap.
- **Missing keys**: every model struct has every required identifier annotated. Merge keys are refused as an unknown field `<<`; aliases resolve; the file and record are named.
- **Usage exit 64**: every subcommand goes through `try_parse`. `--help` and `--version` exit 0. The CLI reference has the row. No doc or caller still says 2, apart from finding 8.
- **Outcome reasons**: they now delegate to `actions::unmet`, and the explanation's obligation reasons use the same rule. `not`, `any`, `is:` and subject cases agree with the reference.
- **`canon-authority/1`**: the model, ESS declaration, schema enum and documents page agree, and the library reader uses the model type. Mutant B (an extra `Grant` variant only in Rust) is caught by `ess_model_matches` and by canon-docs ("differs: …authority-1.schema.json").

**6. Written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w14/canon-review-hardening-w7/scratch/adversary-p1/`: 7 logs, 144K, kept. I deleted my scratch copies `base/` and `head/`, their target dirs (`target-base` 281M, `target-head` 84M or more), and the probe dirs, including about 500 MB of probe IR files.
- `~/.cache/b10x-target/canon-w14-review-hardening-w7/`: my test binaries, plus `tmp/adversary-rh-w7-chain` (35M) and `tmp/adversary-rh-w7-repeated-id`. I left them, since build dirs are not mine to clear. The gate also rewrote `suite.json` there.
- `/var/lib/systemd/coredump`: 12 SIGABRT cores of about 1–3.3 MB each, listed by `coredumpctl list`. Each suite run adds three more while cases 1, 2 and 6 stay red. I did not delete them.

**7.**
```findings
- file: crates/canon/src/eval/claims.rs
  line: 85
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: "Claim evaluation recurses through claim references as well as nesting, so a compiled protocol of chained claims each within MAX_IR_DEPTH aborts canon evaluate (release too) although check_depth passes it."
- file: crates/canon/src/check/mod.rs
  line: 324
  category: boundary
  severity: warning
  verdict: INFEASIBLE
  origin: pre-existing
  message: "check::check walks a caller-built IR recursively before evaluate_with's depth check and aborts on one nested beyond the bound instead of refusing it as predicate-too-deep."
- file: crates/canon-cli/src/evaluate.rs
  line: 80
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: "Evidence refusals raised after reading (repeated id, undeclared or invalid kind) do not name the file, so a repeated id across two files cannot be located."
- file: conformance/scenarios
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: "No conformance scenario holds the {evidence, present, subject} reason shape in actions or outcomes, which the story's last Outcome bullet requires."
- file: crates/canon/src/model/present.rs
  line: 10
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "An explicit null on a required key advises leaving the key out to take its default, and following that advice is refused as missing field."
- file: crates/canon/src/eval/read.rs
  line: 162
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "A depth check that walks only the first member of all/any survives every existing test; only the new later-member case catches it."
- file: website/docs/reference/documents.md
  line: 106
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The canon-authority/1 section and schema state only that an identical entry is refused, while Canon refuses any capability decided twice, which the schema's uniqueItems accepts."
- file: crates/canon-cli/tests/evaluator_skeleton.rs
  line: 339
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "A doc comment still says an undeclared flag is a usage error with exit 2; usage errors now exit 64."
```

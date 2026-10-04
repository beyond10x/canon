---
format: aep.planning-md/3
id: review-result:adversary-w7-canon-evidence-freshness-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w7 adversary, canon story:evidence-freshness, pass 2
relations:
- reviews: story:evidence-freshness
revision: 1
---
```
unit: canon/evidence-freshness — working tree of canon-w7-evidence-freshness (7e60184 + uncommitted phase 2 + pass-1 fixes)
verdict: CONFIRMED
cases: executed 280→291, red 2
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths (part 6)
needs-coordinator: yes — the binding.rs test stops compiling once binding is merged (finding 3)
```

**1. Diff stat.** I added four untracked test files and changed nothing tracked. `git --no-pager diff --stat` still shows the 23 tracked files of the implementor's uncommitted phase 2, unchanged by me. Mine:
```
?? crates/canon/tests/adversary2_freshness_conform.rs
?? crates/canon/tests/adversary2_freshness_eval.rs
?? crates/canon/tests/adversary2_freshness_ir.rs
?? crates/canon/tests/adversary2_freshness_validate.rs
```
No path outside the tests was touched. `cargo fmt --all --check` passes (exit 0) and clippy `-D warnings` is clean on these four files.

**2. Cases added.** Test names confirmed with `--list`.

| File::test | Asserts | Now |
|---|---|---|
| validate::`an_age_too_long_to_count_is_refused_for_being_too_long_not_for_its_form` | the `invalid-max-age` message for `99999999999999999999s` names "too long" | **red** |
| validate::`a_malformed_age_is_refused_for_its_form` | the exact message for `5 min` | green |
| eval::`a_caller_built_ir_with_a_malformed_max_age_does_not_silently_disable_expiry` | an IR with `max_age = "1 hour"` set by hand is refused or expires a 10-year-old record | **red** |
| eval::`an_instant_with_no_stamped_record_changes_nothing` | `--at` with no `observed_at` gives the same bytes as no `--at` | green |
| ir::`every_fixture_round_trips_through_read_ir` | every fixture's IR reads back to the same `Ir` and the same bytes | green |
| ir::`the_freshness_ir_is_the_base_ir_plus_one_max_age_member` | the fixture's IR is the base IR plus exactly `"max_age": "1h"` after `description` | green |
| ir::`only_the_canonical_max_age_member_reads_back` | members in the wrong order or an empty `max_age` are refused; `60m` reads as a different IR from `1h` | green |
| conform::4 tests | without `at` nothing expires; a malformed or empty `at`, or an impossible date, is refused as `invalid-instant`; an `at:` with no value or with `~` is refused when the scenario is read; `at: 1791115200` reaches the evaluator as text | green |

Red output, each case run alone before the suite:
```
panicked at crates/canon/tests/adversary2_freshness_validate.rs:35:5:
the message names the actual reason (too long to count in seconds): evidence kind `k` has max_age `99999999999999999999s`, which is not a whole number without leading zeros, followed by s, m, h or d
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out
```
```
a max_age the evaluator cannot read was ignored, so ten-year-old evidence applied:
{ "case": "C-1", "claims": { "c": { "value": "true" }, "d": { "value": "unknown" } }, ... }
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out
```
One assertion of mine was wrong: the null-`at` case checked for "explicit null" in the message, but scenario errors report only a position, by design. I corrected that test; it is not a finding.

**3. Suite.** `cargo test --workspace --locked --no-fail-fast` gave EXIT=101, 289 passed and 2 failed. The failures are the 2 red cases above, in `-p b10x-canon --test adversary2_freshness_eval` and `--test adversary2_freshness_validate`. Log: `scratch/adversary2-suite.log`.

**4. Findings**

| # | file:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|---|
| 1 | crates/canon/src/validate/mod.rs:155 | CONFIRMED / introduced | Red case 1: an age that is too long to count in seconds gets the form message, which wrongly tells the author it is not a whole number. The problem doc at :98 and the published reference both say "too long to count in seconds". Step 4 of the module docs (:11) leaves that case out as well. | `canon validate` or `canon compile` on any protocol with such a `max_age`. Fix: a second message, or a separate variant, for the overflow case. |
| 2 | crates/canon/src/eval/freshness.rs:426 | INFEASIBLE / introduced | Red case 2: `.and_then(\|age\| age.seconds())` turns an age it cannot read into "no maximum age", so expiry is silently switched off. `ir/mod.rs:694` claims "Well-formed" but nothing enforces it on a caller-built `Ir`. `eval/mod.rs` treats caller-built IRs as in scope for claim cycles, which are refused. | The public library API only (`Ir` fields are pub). The CLI and the conformance runner go through `read_ir` or `compile`, which validate. I found no caller in the tree. |
| 3 | crates/canon/src/eval/binding.rs:75 (at 1b14539) | CONFIRMED / introduced | Three-way `git merge-file` in scratch. binding.rs is unchanged by this unit, and its test helper builds an `EvidenceRecord` literal without `observed_at`, so the merge will not compile (E0063). This is read from the source; I did not compile the merge. There are also 2 text conflicts each in `eval/mod.rs` and `website/docs/reference/evaluation.md` (both doc prose). `model/evidence.rs`, `ess/protocol.yaml`, `pages.rs`, `claims.rs` and the evidence schema merge cleanly. | Merging this unit into wave/2026-10-04-w7. Fix at merge: add `observed_at: None` at binding.rs:81. |

Precedence once binding is merged, read from the code and with no conflict:
- A record that is both stale-bound and expired is excluded once, as `revision_mismatch`, because binding runs first and `set_aside` removes the record.
- An expired record with an undeclared subject is refused as `undeclared-artifact`.
- An invalid `--at` together with an undeclared subject gives `invalid-instant`, because `--at` is read before the stages run.
- An invalid `observed_at` together with an undeclared subject also gives `invalid-instant`.

These orders match the two units' module docs.

**5. Attacked, could not break**
- The canon-docs guard fires. With `Age` renamed to `Duration` in a scratch copy, the run exits 2 with "TEXT_FORMATS lists `Age`, which ids.rs does not declare". The unmutated copy exits 0 with "11 generated files are current".
- TEXT_FORMATS changes only `documents.md`, `protocol.md`, `evidence-1` and `protocol-1`. `case-1` and the identifier tables are correct, and each `#text-formats` anchor exists on the page that links to it.
- The IR's canonical form, the order of members within a kind, and the round trip of every fixture all hold (green probes).
- The `--at` help text matches what the code does.
  - `--at yesterday` and `--at ''` exit 1 with `error[invalid-instant]`.
  - `--at` with no record carrying `observed_at` prints the same bytes as without it.
- The conformance `at` field: malformed, empty, null, numeric and missing values are all handled as the module docs say.
- The ESS citations at `design.md:477`, `:505` and story lines 40-53 and 77-78 say what they are cited for.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w7/canon-evidence-freshness/scratch/adv2-merge/` (merge probe)
- `~/.cache/ga-wave-2026-10-04-w7/canon-evidence-freshness/scratch/adv2-cli/` (CLI probe inputs and outputs)
- `~/.cache/ga-wave-2026-10-04-w7/canon-evidence-freshness/scratch/adv2-docs-base/` and `.../adv2-docs-renamed/` (two copies of the tree, about 2.9M each)
- `~/.cache/ga-wave-2026-10-04-w7/canon-evidence-freshness/scratch/adversary2-suite.log`
- Build output went to `~/.cache/b10x-target/canon-w7-evidence-freshness`, the assigned build dir.

**7. Findings block**
```findings
- file: crates/canon/src/validate/mod.rs
  line: 155
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "An invalid-max-age caused by an age too long to count in seconds is reported as 'not a whole number without leading zeros', which contradicts the problem doc and the published reference."
- file: crates/canon/src/eval/freshness.rs
  line: 426
  category: judgement
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: "A caller-built Ir whose max_age Age::seconds cannot read silently disables expiry (fail-open) instead of being refused, unlike the caller-built claim cycle; no in-tree caller builds one."
- file: crates/canon/src/eval/binding.rs
  line: 75
  category: concurrency
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "After merging into wave/2026-10-04-w7 (1b14539), the binding.rs test helper's EvidenceRecord literal lacks observed_at and will not compile; eval/mod.rs and evaluation.md also have two text conflicts each."
```

---
format: aep.planning-md/3
id: review-result:adversary-w7-canon-evidence-freshness-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w7 adversary, canon story:evidence-freshness, pass 1
relations:
- reviews: story:evidence-freshness
revision: 1
---
```
unit: canon/evidence-freshness — working tree on 7e60184 plus uncommitted phase 2, in ~/.local/state/worktree/trees/b10x/canon/canon-w7-evidence-freshness
verdict: CONFIRMED
cases: executed 268→280, red 2
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (listed in part 6)
needs-coordinator: two calls — the schema fix needs crates/canon-docs, which is outside the story's scope; and the unit edited eval/mod.rs and eval/claims.rs, which the brief reserves (finding 5)
```

The freshness logic held under every attack. My only red cases show that the published JSON Schemas promise to accept values the code refuses.

## 1. Diff stat

`git --no-pager diff --stat` changes only the unit's own 21 tracked files (421 insertions, 57 deletions), the same as before I started. I touched no tracked file. My additions are 3 untracked test files:
- `crates/canon/tests/adversary_freshness_time.rs`
- `crates/canon/tests/adversary_freshness_eval.rs`
- `crates/canon/tests/adversary_freshness_schema.rs`

No path outside the tests is mine.

## 2. Cases added

| File | Test | Asserts | Now |
|---|---|---|---|
| `adversary_freshness_schema.rs` | `every_max_age_the_published_protocol_schema_accepts_is_one_canon_accepts` | Any `max_age` that `protocol-1.schema.json` accepts is one `validate` accepts. The schema's `Identifier` description says the only extra refusals are control and format characters. | **red** |
| `adversary_freshness_schema.rs` | `every_observed_at_the_published_evidence_schema_accepts_is_one_canon_accepts` | The same for `observed_at` in `evidence-1.schema.json`, checked with `evaluate_with`. | **red** |
| `adversary_freshness_time.rs` | 3 tests | Every day from 0000-01-01 to 9999-12-31 is counted independently and lands exactly one day after the previous one; the day after each month's end is refused. Also: the epoch values at the range ends, `23:59:60`, lowercase `z`, `-00:00`, fractions, and `Age` at the last value that fits in i64 for each unit. | green |
| `adversary_freshness_eval.rs` | 7 tests | The `0s` boundary, both before and after the instant; exactly `max_age` old vs one second older; the extremes of the calendar with an `i64::MAX` age; an expired refutation gives `UNKNOWN`, not `FALSE`; expired records are listed once per reaching claim, sorted by id, and evidence order doesn't change the decision; `max_age` on an unused kind leaves the output byte-identical; a bad `observed_at` is refused before `--at` is read. | green |

Red output from running those cases alone, before the suite:
```
---- every_max_age_the_published_protocol_schema_accepts_is_one_canon_accepts stdout ----
panicked at crates/canon/tests/adversary_freshness_schema.rs:94:5:
assertion `left == right` failed: protocol-1.schema.json accepts these `max_age` values; canon validate refuses them
  left: ["05m: evidence kind `k` has max_age `05m`, which is not a whole number followed by s, m, h or d", "1H: …", "1w: …", "-1h: …", "+1h: …", "1.5h: …", "5min: …", "PT5M: …"]
 right: []
---- every_observed_at_the_published_evidence_schema_accepts_is_one_canon_accepts stdout ----
panicked at crates/canon/tests/adversary_freshness_schema.rs:139:5:
  left: ["2026-10-04T12:00:00+02:00: invalid-instant: evidence `e` observed_at `2026-10-04T12:00:00+02:00` is not an instant in UTC written YYYY-MM-DDTHH:MM:SSZ", "2026-10-04T12:00:00.5Z: …", "2026-10-04t12:00:00z: …", "2026-02-30T00:00:00Z: …", "2026-10-04: …", "yesterday: …"]
 right: []
test result: FAILED. 0 passed; 2 failed
```

## 3. Suite run, after the cases existed

Command: `cargo test --workspace --locked --no-fail-fast`, with the brief's environment and build dir. Log: `scratch/adversary1-suite.log`.

```
EXIT=101
passed 278 failed 2 ignored 0
error: 1 target failed: `-p b10x-canon --test adversary_freshness_schema`
```
268 before plus my 12 cases makes 280. rustfmt and clippy (`-D warnings`) are clean on my 3 files.

## 4. Findings

1. **`website/static/schemas/protocol-1.schema.json:127`** — CONFIRMED, introduced.
   - **What was measured:** `max_age` points at `#/$defs/Identifier`, which only checks `^\S+$`, so the schema accepts `05m`, `1H`, `PT5M` and similar. `validate` refuses all of them.
   - **Cause:** `model/ids.rs:103-111` declares `Instant` and `Age` with `identifier!`, and `crates/canon-docs/src/pages.rs:729` maps every identifier type to the generic `Identifier`.
   - **What reaches it:** anyone who checks protocol documents against the published schema.
   - **Fix:** give these two types their own schema pattern, e.g. `^(0|[1-9][0-9]*)[smhd]$`. That needs `crates/canon-docs/`, which the story's scope does not include.
2. **`website/static/schemas/evidence-1.schema.json:44`** — CONFIRMED, introduced. Same cause for `observed_at`: the schema accepts offsets, fractions, `2026-02-30…` and `yesterday`, and evaluation refuses them all as `invalid-instant`.
3. **`crates/canon/src/validate/mod.rs:157`** — CONFIRMED, introduced. The refusal message for `05m` says "not a whole number followed by s, m, h or d", but 05 is a whole number. The no-leading-zero rule exists only in `model/time.rs:812`; `ids.rs:109`, `protocol.md:167` and `validation.md` leave it out.
4. **`ess/domains/protocol.yaml:233`** — CONFIRMED, introduced. The spec says "RFC 3339 with whole seconds and the offset Z". RFC 3339 §5.6 also allows lowercase `t`/`z` and second `60`, and the code refuses both (my green time cases pin that). The website gives the strict form, so only the ESS wording is off.
5. **`crates/canon/src/eval/mod.rs:89-93`** — CONFIRMED, introduced (judgement).
   - **What was measured:** the unit edited `eval/mod.rs` (docs and a test literal), `eval/claims.rs:193`, `canon-cli/src/lib.rs` and `canon-cli/tests/evaluator_skeleton.rs`. None is in the story's scope, and the brief reserves `eval/mod.rs` and `claims.rs`.
   - **Why:** the claims.rs and mod.rs test literals had to change once the new `observed_at` field was added, and the doc edits keep the pipeline docs true.
   - **Risk:** the "not built yet" paragraph at `mod.rs:89-93` is text each parallel stage unit would also need to change, so it may conflict at merge.

## 5. Attacked and could not break
- The epoch day count across every day of years 0000 to 9999, including Feb 29 in leap and non-leap years, month 13 and day 0.
- Leap seconds, lowercase `z`, offsets and fractional seconds are all refused, as the docs say.
- `Age`: `0s`; overflow refused at exactly i64::MAX for each unit; missing, uppercase or non-ASCII units; leading zeros; signs.
- A negative age (observed after the instant) never expires.
- At the boundary, exactly `max_age` old still applies.
- Stage order: freshness only sees records that binding left (`mod.rs:185-188`), and `set_aside` lists a record once. I could not test this at runtime because binding is still a stub.
- `max_age` on a kind no record has: output byte-identical.
- IR is byte-stable for protocols without `max_age` (existing compile scenarios stay green).
- `ess_model_matches` maps `identifier!` newtypes generically, so `Instant` and `Age` are covered.
- Freshness doc comments match the behaviour.

## 6. Paths written outside the worktree
- `~/.cache/ga-wave-2026-10-04-w7/canon-evidence-freshness/scratch/adversary1-suite.log`
- `~/.cache/b10x-target/canon-w7-evidence-freshness` (the assigned build dir; I added build output there)

The worktree session lease `adversary-w7-evidence-freshness-p1` was acquired and released.

## 7. Findings block
```findings
- file: website/static/schemas/protocol-1.schema.json
  line: 127
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "The published protocol schema types max_age as the generic Identifier, so it accepts 05m, 1H and PT5M, which canon validate refuses as invalid-max-age."
- file: website/static/schemas/evidence-1.schema.json
  line: 44
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "The published evidence schema types observed_at as the generic Identifier, so it accepts offsets, fractions, impossible dates and free text, which evaluation refuses as invalid-instant."
- file: crates/canon/src/validate/mod.rs
  line: 157
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The invalid-max-age message and the Age docs say 'a whole number followed by s, m, h or d' but 05m is refused for its leading zero, a rule stated only in model/time.rs."
- file: ess/domains/protocol.yaml
  line: 233
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "ESS describes Instant as RFC 3339 with offset Z, but the parser refuses the lowercase t/z and second 60 that RFC 3339 permits."
- file: crates/canon/src/eval/mod.rs
  line: 89
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The unit edited eval/mod.rs, eval/claims.rs and canon-cli files outside its story scope, two of which the brief reserves, and the paragraph at mod.rs:89-93 may conflict with parallel units at merge."
```

---
format: aep.planning-md/3
id: review-result:adversary-protocol-writer-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 on story:protocol-writer: green, 1 warning, 1 note'
relations:
- reviews: story:protocol-writer
revision: 1
---
Adversary pass 2, the last, on story:protocol-writer, wave 2026-11-20-w7, on branch impl/protocol-writer at 3009425f. The report is recorded as the agent returned it. Verdict green: the four findings of pass 1 are closed; one warning (the stack margin of the depth figure for all and any) and one note (two mutants only the pass's cases kill), with three green cases in commit 26b7de25.

unit: story:protocol-writer, branch impl/protocol-writer at 26b7de25 (head 3009425f plus one adversary test commit)
verdict: green (strongest finding: a CONFIRMED warning, the stack margin for `all`/`any`)
cases: executed 1208→1211, red 0
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 directory, `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w7/protocol-writer/adversary-p2`
needs-coordinator: yes. D4 says 500 levels leave a factor of two. That holds for `not` (2.30) but not for `all` or `any` (1.62). Keep 500, or name 400 for `all`/`any`.

**1. Diff, `git diff --stat 3009425f..HEAD`:** `crates/canon/tests/adversary_protocol_write.rs | 153 +` is the only path, and it is a test file. Committed by the bot as 26b7de25 and the tree is clean.
- Departure from the brief: mutants were not applied in the tree. Each ran from a scratch copy through a temporary harness test file.
- A temporary probe file, `crates/canon/tests/adv2_probe.rs`, was also used.
- Both temporary files are deleted, and `write.rs` was never edited.

**2. Cases added.** All three are green now, alone and in both builds; none was red, so there is no red output to quote.
- `a_key_grown_by_each_escape_form_reads_back_at_1022_bytes_as_written_and_not_at_1023` (:299): `\" \\ \n \r \t \b \f \u0001 \u0085 \u2028 \uFEFF` and a raw U+1F600. At 1022 bytes the key reads back spelled as the docs say; at 1023 it does not.
- `a_key_reads_back_at_1022_bytes_and_not_at_1023_in_every_map_the_model_keys_by_text` (:341): 13 places.
- `a_predicate_of_all_nested_500_deep_is_written_on_a_thread_with_a_2_mib_stack` (:410).
- Run alone: `test result: ok. 1 passed` for each case. `--release`: `ok. 2 passed` for both 500-level cases. Number-text build: `ok. 13 passed`.

**3. Suite**, run after the cases existed: `cargo test --workspace --locked --no-fail-fast`. 1211 passed, 0 failed, 3 ignored over 158 summaries; exit 0. `adversary_protocol_write`: `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.
- Clippy from rustc 1.99.1 was then red on my file (`type_complexity`, :349). I added a type alias.
- After that: `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0, and `cargo fmt -p b10x-canon --check` exit 0.
- The file was re-run alone in both builds: 13 passed.
- Clippy from rustc 1.100.0 was not run.

**4. Findings against 26b7de25.**

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/model/write.rs:111 | boundary | warning | CONFIRMED | introduced | Only a model built in Rust. In a debug build on a 2 MiB stack, `all`/`any` hold 808 levels and overflow at 809, a factor of 1.62 over the documented 500. `not` holds 1152 (2.30). The D4 case nests `not` only. `all` takes two frames per level (`predicate`, `predicates`), so "recurses once per level" (:106) undercounts it. Fix: name 400 levels, or give each shape its own figure. |
| crates/canon/tests/protocol_write.rs:1136 | mutant | note | CONFIRMED | introduced | `spelled` covers only `\` and `"`. Mutant A2 (backspace written `\u0008`, 6 bytes against the documented 2) and A3 (2 KiB more stack in `predicates`) pass the unit's cases and pass 1's. Only the pass-2 cases :299 and :410 kill them. |

**5. The seven points of the brief**
1. All four pass-1 findings are closed:
   - F1: write.rs:5-6, "for every model `p` that `parse` gives in which every key, as written with its escapes, is at most 1022 bytes".
   - F2: write.rs:93-94, "drops a simple key whose text runs more than 1024 bytes before its `:` … leaves 1022 bytes of written text, escapes included". The false sentence is gone, and protocol_write.rs:1121 reads `const KEY_LIMIT: usize = 1022;`.
   - F3: write.rs:106-108, "returns for every model whose nesting the calling thread's stack holds". "is still written" is removed.
   - F4: the four green cases are byte for byte as written, and 19 of 19 mutants are killed.
2. **The limit.** 15 places × 9 key forms = 135 combinations, every one with largest-reading 1022 and smallest-failing 1023.
   - Places: the 9 declaration sections, `case.inputs`, `requires[].scope`, `arguments`, `arguments.properties`, a key 25 objects deep and a key 25 arrays deep (both indented 58).
   - Forms: plain, `\"`, `\\`, `\n`, `\u0001`, `\u0085`, `\uFEFF`, a three-byte character, and a raw non-BMP character (4 bytes).
   - Indentation does not count: libyaml takes the key's mark at the opening quote (unsafe-libyaml `scanner.rs:346`).
3. **Below the limit.** Corpus: 128 of 128 read back and 128 of 128 are fixpoints, in both builds. 64 strings (pass 1's list) in 5 places: 320 of 320 read back in both builds.
4. **Beyond the limit.** 135 combinations × 21 lengths (1023 to 1040, 1500, 2000, 4000). `parse` refuses every one; there is 0 `Ok` with a different model, and `serde_json` reads the key intact every time.
5. **Depth.** The 500-level test is green in debug and with `--release`. On a 2 MiB stack in a debug build, holds/overflows:

   | shape | holds | overflows | margin over 500 |
   |---|---|---|---|
   | `not` | 1152 | 1153 | 2.30 |
   | `all`, `any` | 808 | 809 | 1.62 |
   | arguments array | 2209 | 2210 | 4.42 |
   | arguments object | 1945 | 1946 | 3.89 |

   `parse` reads at most 123 levels of `not`, 61 of `all` and 125 of `arguments`, and every one of those models reads back.
6. **Nothing else changed.**
   - `git diff --stat 280c42df..HEAD`: 4 files, 221 insertions, 92 deletions.
   - `write.rs` without its `//!` lines is identical between the two commits (`cmp`, 666 lines).
   - `write` bytes over the corpus, head against the 280c42df writer compiled from a scratch copy: 128 files compared, 0 differ, in both builds.
   - The four green cases are identical (`cmp`, 108 lines).
   - The six rewritten cases keep their input; the depth one went from 5000 to 500, as D3 and D4 decided. Five assert D1 and one asserts D2.
7. **Mutants.** The M00 control passes all 47 cases, and M01-M19 are all killed (19 of 19).
   - A1, a space before the colon (`"key" : value`): killed by 14 cases, 10 of them the unit's.
   - A2: killed only by :299.
   - A3: killed only by :410, through a stack overflow abort. With :410 skipped, 46 pass.

**Attacked and did not break:** `Ok` with a different model beyond the limit; escapes or indentation shifting the limit; a parse shorthand that would make written text nest deeper than its source (there is none); corpus bytes changed by the correction.

**6. Paths written outside the worktree**, all in `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w7/protocol-writer/adversary-p2/`:
- scripts: probe.sh, probe2.sh, cases.sh, mutate.sh, mutate-a3-skip.sh, suite.sh, recheck.sh
- message: msg-1.txt
- logs: probe-probe.log, probe2-doubles.log, probe2-text.log, case-*.log (3), cases-text.log, cases-release.log, mutants.log, fmt.log, fmt2.log, clippy.log, clippy2.log, suite.log, final-doubles.log, final-text.log
- markers: suite-done.txt, recheck-done.txt
- copies: green-280.rs, green-head.rs, write-280-code.rs, write-head-code.rs, write_280.rs
- `mut/`: write_mut.rs, unit_copy.rs, adversary_copy.rs, the M00-M19 and A1-A3 `.diff`/`.log` files, A3-skip.log

The lease `adv2-protocol-writer` is released.

```findings
[{"file": "crates/canon/src/model/write.rs", "line": 111, "category": "boundary", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "A predicate of all or any nested 809 deep overflows a 2 MiB stack in a debug build, so the documented 500 levels leave a factor of 1.62 for that shape, not the factor of two D4 rests on (not holds 1152), and the D4 case nests not only"}, {"file": "crates/canon/tests/protocol_write.rs", "line": 1136, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "A mutant writing a backspace as u0008 and one adding 2 KiB of stack to predicates pass the unit and pass-1 suites, and only the pass-2 escape-form and all-500 cases kill them"}]
```

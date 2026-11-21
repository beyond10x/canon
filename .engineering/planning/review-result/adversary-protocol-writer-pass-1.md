---
format: aep.planning-md/3
id: review-result:adversary-protocol-writer-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:protocol-writer: red, 4 findings'
relations:
- reviews: story:protocol-writer
revision: 1
---
Adversary pass 1 on story:protocol-writer (wave 2026-11-20-w7), on Opus, against impl/protocol-writer at da33207a; the pass added one test commit, 280c42df. The report as it returned:

unit: story:protocol-writer, branch impl/protocol-writer at 280c42df (the head da33207a plus one adversary test commit)
verdict: red
cases: executed 1196→1206, red 6
origin: introduced 4, pre-existing 0, undecided 0
wrote-outside-worktree: 1 directory, `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w7/protocol-writer/adversary-p1` (logs, scripts, `mut/` mutant copies, msg-1.txt)
needs-coordinator: yes. Decide how to resolve finding 1: narrow the documented contract, have `parse` refuse such keys, or let the output stop being pure JSON.

**1. Diff against da33207a**
`crates/canon/tests/adversary_protocol_write.rs | 241 +` (new). It is the only path, and it is a test file. Committed by the bot as 280c42df. The tree is clean.

**2. Cases** (written before anything ran; run alone, doubles build, then again in the number-text build `-p b10x-canon -p b10x-canon-expr`, same result)
Red, all from text that `parse` accepts. Each `parse(&write(&p))` fails:
- `a_plain_key_of_1023_bytes_parse_reads_reads_back` (:39): `parse gives this protocol and refuses the text write gives for it: claims: this key is required and needs a value; an explicit null is not one at line 7 column 13`
- `a_plain_key_of_341_characters_…`: same message. 341 three-byte characters make 1023 bytes.
- `a_plain_key_of_600_backslashes_…`: same message. Each `\` is written `\\`, so the key grows to 1202 bytes.
- `an_explicit_key_of_2000_bytes_…` (`? <key>`): same message.
- `an_explicit_long_property_name_inside_arguments_reads_back`: `did not find expected ',' or '}' at line 11 column 1113, while parsing a flow mapping at line 10 column 23`

Red, model built in Rust:
- `a_predicate_nested_5000_deep_is_written_on_a_thread_with_the_default_stack` (:90): `write does not return for a predicate nested 5000 deep: signal: 6 (SIGABRT) (core dumped)` / `fatal runtime error: stack overflow, aborting`

Green now. Each kills a mutant that the unit's tests let through:
- `empty_arguments_are_written_and_read_back` (:137)
- `keys_inside_arguments_are_written_sorted`
- `every_line_is_indented_two_spaces_for_each_level_of_nesting` (:197)
- `an_empty_description_and_an_empty_result_are_written_and_read_back`

**3. Suite run** (after the cases existed): `cargo test --workspace --locked --no-fail-fast`
- 1200 passed, 6 failed, 3 ignored over 158 summaries; exit 101 (`error: 1 target failed: -p b10x-canon --test adversary_protocol_write`).
- `slot task docs-check`: `canon-docs: 18 generated files are current`, exit 0.
- `cargo fmt -p b10x-canon --check` and clippy (local 1.99.1, `-D warnings`) were red on my own file. I fixed both before committing and both are exit 0 now; clippy 1.100.0 was not available to run.

**4. Findings** (all against 280c42df; `write.rs` does not exist at base 7ef6baa1)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/model/write.rs:159 | acceptance | blocker | NEEDS-CHANGE | introduced | `parse` of a block YAML key of 1023 or 1024 bytes, an explicit `?` key of any length, or a key whose escapes push it past 1022 bytes. This holds for identifiers and for `arguments` property names. Today only tests call `write` (it is a public function). |
| crates/canon/src/model/write.rs:99 | contract-drift | warning | NEEDS-CHANGE | introduced | Anyone reading the docs. The limit is in bytes, not characters. It is 1022 bytes for a quoted key, escapes included. `parse` does give such keys. `protocol_write.rs:1117-1119` states the same false premise. |
| crates/canon/src/model/write.rs:584 | boundary | warning | INFEASIBLE | introduced | Only a model built in Rust (`parse` stops at about 125 levels). Lines 9 and 92-97 of the module docs say such a model "is still written". It aborts at 5000 levels on a 2 MiB stack, because `predicate` and `json_value` recurse. |
| crates/canon/tests/protocol_write.rs:203 | mutant | note | CONFIRMED | introduced | 6 of 19 mutants of a scratch copy survived the unit's suite. They cover `Some` of empty `arguments`, empty description, empty result, `arguments` keys written in reverse, and the indent of `independent_on` and `children.in`. My four green cases kill all six. |

- **Finding 1, the mechanism.** libyaml treats a simple key as stale once the key's text runs more than 1024 bytes before its `:` (`scanner.rs:347`). The quotes count, so a quoted key can hold 1022 bytes.
  - JSON has no other way to spell a key, so the module's statement `parse(&write(&p)) == Ok(p)` for every `p` that `parse` gives (write.rs:4-5) cannot hold for these keys.
  - The fix is the coordinator's choice: narrow the contract and the docs, have `parse` refuse such keys, or write those keys as YAML, which means the output is no longer JSON.
  - The 129-fixture acceptance itself still holds.
- **Finding 3, the fix.** Write with an explicit stack, or document the depth bound. Even a non-recursive writer produces output that grows with the square of the depth (about 20 GB at 100,000 levels), so the brief's 100,000 is not a fair depth for a case.

**5. Attacked and did not break**
- **Probes run in both builds.** These were not committed: round trip, second-write fixpoint and JSON validity over:
  - 42 number spellings: `-0`, `-0.0`, 2^53+1, i64/u64 bounds, 65-128-bit integers (refused by `parse`), >128-bit integers (read as doubles), `1e400` (read as a string), subnormals, `0x`/`0o`/`0b`, `1_000`, `012`.
  - 60 strings, each placed as a description, a key, an explicit key, an input value, an `arguments` key and value, and a scope. They include `null`, `~`, truth words, dates, `\N`, `\L`, `\P`, `\x85`, `\U0001F600`, `!!str`, block scalars and a 100,000-character value.
  - 33 shapes: every predicate form and `is` spelling, both `requires` variants, `children` with `revision` and `subject`, `independent_on: []`, `case: {}`, `case: {inputs: {}}`, empty and scalar `arguments`, nested `null`, aliases, merge keys, `$serde_json::private::Number` as a key, tags, and non-string keys.
- **What `parse` refuses.** `arguments: Some(null)` cannot be produced: `.nan`/`.inf` and non-string keys are refused by `present::optional_json`. A lone surrogate escape is refused too.
- **Mutants.** 13 of 19 were killed by the unit's suite: dropped keys in IndependenceDimension, ChildOutcomes.revision, EvidenceSince.subject, decided_by, refines and effect; swapped `case_open` keys; `null` for an absent description; U+FEFF and U+007F removed from the escape set; keeping only the last repeated id; writing `{}` for an empty section; omitting `is: true`.
- **Claim 6 holds.** Every struct is destructured without `..`, the `Predicate`, `OutcomeRequirement` and `Value` matches have no wildcard arm, and no field is bound and left unused.
- **Claim 7 holds.** The diff touches 4 files, nothing under `ir/`, and there is no `Serialize` derive on the model. The escape set of `generate::quote` is the same as the writer's.
- **The ESS header comment** is accurate.

Mutants were applied only to a scratch copy, through a temporary harness test file that I deleted before committing. `write.rs` itself was never edited, even though the brief allowed in-tree mutation. One flaw in the harness: the `--skip a_predicate_nested` filter also skipped the unit's `a_predicate_nested_as_deep_as_parse_reads_is_written_and_read_back`. The lease is released.

**6. Paths written outside the worktree**
- `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w7/protocol-writer/adversary-p1/`, containing:
  - logs: red-doubles.log, red-text.log, red-final.log, probe-doubles.log, probe-text.log, mutants.log, suite.log, fmt.log, fmt2.log, clippy.log, clippy2.log, docs-check.log
  - scripts and message: run-case.sh, probe.sh, mutate.sh, suite.sh, recheck.sh, msg-1.txt
  - `mut/`: write_mut.rs, unit_copy.rs, adversary_copy.rs, and M00-M19 `.diff` and `.log` files

```findings
[{"file": "crates/canon/src/model/write.rs", "line": 159, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Every key is written as a quoted simple key, which libyaml drops past 1022 bytes, so protocols parse gives with a 1023-byte plain key, an explicit ? key or an escape-grown key do not read back and the documented parse(write(p)) == Ok(p) is false"}, {"file": "crates/canon/src/model/write.rs", "line": 99, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "The module docs put the key limit at 1024 characters and say parse never gives such a key, but the limit is 1022 bytes of written text including escapes and parse does give such keys; protocol_write.rs:1117 repeats the premise"}, {"file": "crates/canon/src/model/write.rs", "line": 584, "category": "boundary", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "write recurses per nesting level and aborts with a stack overflow on a Rust-built predicate 5000 deep on a default 2 MiB thread, while the module docs say a model nested deeper than parse reads is still written"}, {"file": "crates/canon/tests/protocol_write.rs", "line": 203, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Six writer mutants (empty arguments, empty description, empty result left out; arguments keys reversed; independent_on and children in mis-indented) pass the unit suite and are killed only by the adversary green cases"}]
```

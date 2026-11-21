---
format: aep.planning-md/3
id: review-result:adversary-judgement-marking-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:judgement-marking (wave 2026-11-20-w5): green, 0 cases added, 1 note, introduced'
relations:
- reviews: story:judgement-marking
revision: 1
---
Adversary pass 1 on story:judgement-marking, wave 2026-11-20-w5, `aep:adversary` on Opus, on `impl/judgement-marking` at `7c495e5`. The report below is as the agent returned it.

unit: story:judgement-marking
verdict: green
cases: executed 1026→1026, red 0
origin: introduced 1, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w5/judgement-marking/adversary-p1/ (bytes.sh, bytes/, probes/, build.log, unit-tests.log, src.diff, wait.sh)
needs-coordinator: no

No case broke anything, so I added no test file and made no commit. The worktree is clean and the lease is ended.

**Suites run** (head 7c495e5, worktree `target/`)
- `cargo test -p b10x-canon --test judgement_marking`: `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s`
- `cargo test -p b10x-canon --lib judgement`: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 314 filtered out; finished in 0.18s`
- Probe `canon conform run` over `canon generate` output of the unit fixture: `conform: 3 passed, 0 failed, 0 unreadable`. Over my own marked protocol: `conform: 2 passed, 0 failed, 0 unreadable`.

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| website/docs/concepts/protocols.md:248 | docs | note | CONFIRMED | introduced | `canon diff` across an `author` change prints one extra line per production that declares no `decided_by`. Example: the unit fixture, unmarked to marked, prints 5 lines, not the 2 the table suggests. Story § Outcome 5 does not name these lines either. They follow from § Outcome 4 (the IR holds the resolved value) and are in a fixed order (sorted kinds). The diff module doc says so; the concepts table does not. |

**Attacked and not broken**
- **Claim 2, IR bytes: 98 inputs.** I compiled every `protocol/1` document under `fixtures/` and `conformance/` with head and with the oracle `canon-6f44325`:
  - 68 are byte-identical, including `fixtures/investigation/protocol.yaml`; 0 differ.
  - 25 are refused by both. Their error text is the same except `invalid/supplied-by-self.yaml`, where the oracle has no `supplied_by`.
  - 5 the oracle cannot read: `child-outcome-revision`, `judgement-marking`, `missing-evidence`, `supplied-evidence/bypass`, `supplied-evidence/supplied`. The head IR of the 4 that are not the unit's own fixture contains no `author`, `revises` or `decided_by` key.
  - Import compositions are identical by sha256 on head and oracle: root with 3 imports, falsification, review, flattened. The two pinned digests match the root's pins.
- **Resolution of `decided_by`: 12 forms.** Judgement author with `decided_by: program`; program author with `decided_by: judgement`; a kind listed twice, both values, both orders, under either author; `program` then absent and absent then `program` under judgement. Every IR is as claimed, and judgement wins.
  - 7 pairs of documents that compile to the same IR give `NO SEMANTIC CHANGE: no declaration differs`.
- **Claim 5, diff: 6 pairs.**
  - An `author` change with the old value written as `decided_by` on every production prints the `author` line only, in both directions.
  - A kind added under a judgement action is `EXPANSION` only; removed, `BREAKING` only. That is the same as the base.
  - Line order matches the module doc.
- **Claim 3, read-back: 31 hand edits** of a compiled IR through `canon evaluate --ir`.
  - Every non-canonical form is `malformed-input`: `program` on `author` or `decided_by`; `Judgement`; null, number or list; `revises` empty, unsorted, repeated, a string, null, an object, nested, a number, an undeclared or non-ASCII id; a duplicate production; an extra key; a production written as a string.
  - The 7 edits that exit 0 are canonical IRs of other valid protocols.
  - No panic. Production entries were already objects on the base.
- **Source forms: 20 cases.** Null, empty, wrong case, a bool or a list on each of the three fields; `revises: []` and a repeated id are accepted.
  - Claim 4 order holds: the `may_produce` kind, then each `revises` listing, then the `case_open` subject, then `self-opened-case`.
- **Claim 1 beyond the named commands: 8 comparisons.**
  - `canon check` and `canon evaluate` give the same bytes with and without the fields, for the unit fixture and for my protocol. `missing_evidence` is identical for an action with `decided_by` entries and a kind listed twice.
  - `canon generate` output is the same once the fixture name is replaced.
  - `canon diff` of a marked protocol against itself is no change.
  - There is no `canon explain` subcommand.
- **Claim 6, imports: 7 two-import compositions via `canon compile --import`.**
  - A `decided_by`-only difference conflicts, and so does `author: judgement` against `decided_by: judgement`.
  - Same-IR spellings merge, keeping the marks: absent against explicit `judgement`, `program` written out, `revises` order and repeats, `revises: []`.
  - A `revises` difference conflicts. The root-revises-imported and imported-revises-root cases are covered by the unit's tests, and their logic matches.
- **Model and docs: 5 checks.**
  - ESS `Optional<Author>` and `List<ArtifactId>` match the Rust types, and `ess_model_matches` compares enum variants in snake case.
  - The ESS `# read from:` line ranges are correct.
  - The schema refuses `author: llm`.
  - `canon-ir.md` counts "ten lists" and "fifteen exceptions"; both are right.
- **Shared files.** The `model/mod.rs` and `ess/domains/protocol.yaml` hunks sit after `case_open` and leave the `arguments` doc comment alone. `validate/mod.rs` adds no `mod` or `pub use` for `arguments.rs`.

```findings
[{"file": "website/docs/concepts/protocols.md", "line": 248, "category": "docs", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The diff table does not say that an author change also prints one line per production declaring no decided_by (5 lines for the unit fixture unmarked to marked). The order is deterministic and the diff module doc states it. Story Outcome 5 does not name these lines; they follow from Outcome 4's resolved IR."}]
```

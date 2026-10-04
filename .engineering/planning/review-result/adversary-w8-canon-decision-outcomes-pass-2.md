---
format: aep.planning-md/3
id: review-result:adversary-w8-canon-decision-outcomes-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w8 adversary, canon story:decision-outcomes, pass 2
relations:
- reviews: story:decision-outcomes
revision: 1
---
unit: canon/decision-outcomes, working tree at d39d426 plus uncommitted phase 2 and the pass-1 fixes (worktree canon-w8-decision-outcomes)
verdict: NEEDS-CHANGE
cases: executed 388→397, red 4
origin: introduced 5 / pre-existing 1 / undecided 0
wrote-outside-worktree: 8 paths, all under the assigned scratch dir (part 6)
needs-coordinator: whether F4 (pre-existing, a misleading message) is fixed in this unit or gets its own story

**1. Diff.** `git --no-pager diff --stat` is unchanged from the implementer's: `38 files changed, 1066 insertions(+), 226 deletions(-)`. I added three untracked test files and touched nothing else:
- `?? crates/canon/tests/adversary2_decisions_docs.rs`
- `?? crates/canon/tests/adversary2_decisions_eval.rs`
- `?? crates/canon/tests/adversary2_decisions_messages.rs`

**2. Cases added.** Each file was run alone first. Logs are in `scratch/adv2-*-red.log` and `scratch/adv2-eval-run.log`.

| File:test | What it asserts | Now |
|---|---|---|
| `adversary2_decisions_docs.rs` `the_documents_page_names_the_scenario_that_fixes_the_decided_outcome_shape` | the `outcomes` row of `canon-decision/1` names CANON-OUTCOME-002 or `decided_by` | red |
| `…docs.rs` `the_status_row_for_evaluation_documents_names_canon_decisions_1` | the "Evaluation documents" status row names `canon-decisions/1` | red |
| `…docs.rs` `the_decisions_schema_states_that_a_repeated_entry_is_refused` | `decisions-1.schema.json` sets `uniqueItems` or says a repeated entry is refused | red |
| `adversary2_decisions_messages.rs` `a_decision_entry_missing_a_key_is_refused_naming_the_key` | a missing key is refused naming the key | red |
| `adversary2_decisions_eval.rs`, 5 tests | the 9-pair refusal-order matrix; authority, then instant, then decisions read order, and neither input accepted as the other; a decision is not a grant and a grant is not a decision; `decided_by` gives byte-identical output for all 720 orders of 6 entries (including `zoë`, `Zoe`, `ß` and a superseded `AAA`); 14 malformed entry shapes; `yes` and a date read as text | green |

Red output, verbatim:
```
panicked at crates/canon/tests/adversary2_decisions_docs.rs:43:5:
| `outcomes` | any JSON value | when present | Each declared outcome, `legitimate` or `blocked`, with the reasons it is blocked; written when the protocol declares an outcome. Its shape is fixed by CANON-OUTCOME-001. |
panicked at crates/canon/tests/adversary2_decisions_docs.rs:58:5:
| [Evaluation documents](../reference/documents.md) | Shipped | `canon-case/1`, `canon-evidence/1` and `canon-decision/1`. |
panicked at crates/canon/tests/adversary2_decisions_docs.rs:79:5:
uniqueItems = null
panicked at crates/canon/tests/adversary2_decisions_messages.rs:52:9:
`--decisions` is not a canon-decisions/1 document: an explicit null is not allowed here; leave the key out to take its default
```

Proof the green eval cases can fail: mutant M1 moves the `undeclared-decision` check ahead of the termination's `undeclared-outcome` check, in a scratch copy with its own target dir. Every other test binary in canon and canon-cli stayed green. Only my case failed: `left: "undeclared-decision" right: "undeclared-outcome"` at `adversary2_decisions_eval.rs:140`. The unit's own suite does not pin that order; this case now does.

**3. Suite run, after the cases existed.** `cargo test --workspace --locked --no-fail-fast` exited 101 with 393 passed and 4 failed: the four red cases above. `cargo fmt --all --check` gives 0. `cargo clippy -p b10x-canon --tests -D warnings` is clean. `--list` shows all 9 new tests in this tree's binaries.

**4. Findings**

| # | file:line | Verdict / origin | Evidence | What reaches it |
|---|---|---|---|---|
| F1 | `crates/canon/src/model/decision.rs:35`, generated to `website/docs/reference/documents.md:72`; also `ess/domains/protocol.yaml:436` | NEEDS-CHANGE / introduced | The docs say the `outcomes` shape "is fixed by CANON-OUTCOME-001". The evaluator now also writes `decided_by` and the reason `{decision, present}`, which only CANON-OUTCOME-002 holds. | Any consumer reading the documents page as the contract |
| F2 | `website/docs/status/where-this-stands.md:17` | NEEDS-CHANGE / introduced | The hand-written "Evaluation documents" row names 3 formats; the linked page now lists 4 | Readers of the status page |
| F3 | `website/static/schemas/decisions-1.schema.json` (generator `crates/canon-docs/src/pages.rs`) | NEEDS-CHANGE / introduced | A list that repeats an entry exactly passes the schema, and Canon refuses it as `duplicate-identifier`. `uniqueItems: true` states exactly that rule. | Anyone validating `--decisions` input against the published schema |
| F4 | `crates/canon/src/model/ids.rs:21` (`present::required`) | CONFIRMED / pre-existing | Measured with `canon evaluate --decisions` on `[{decision: d, outcome: o, principal: p}]`: exit 1, "leave the key out to take its default". The key was already left out and has no default. `--authority` says "missing field `decision`" in the same situation. | Any `--decisions` file missing a key. A case snapshot without `id` gets the same message, and the mechanism is in `git show 8fc260a:…/ids.rs:21`. |
| F5 | `crates/canon/src/eval/decisions.rs:70` | CONFIRMED / introduced (judgement) | Deterministic: 720 orders give the same bytes, and code-point order equals the `String` order Canon uses elsewhere. It works as a tie-break but loses information as a record: other principals who took the decision at the current revision are dropped, while design § 37 asks evaluations to record which decisions applied. | Any protocol where 2 or more principals decide |
| F6 | `website/docs/concepts/protocols.md:142`; `website/docs/concepts/evaluation.md:3` | NEEDS-CHANGE / introduced (judgement) | "…outcome requirements are written as predicates" is now false for decision requirements. The page description lists the evaluation inputs without explicit decisions. | Readers of the concept pages |

Fixes, not applied:
- **F1:** extend the doc comment on `decision.rs:35` to name CANON-OUTCOME-002 and `decided_by`, regenerate the docs, and change the `ess/` comment in the same commit.
- **F2:** add `canon-decisions/1` to the status row.
- **F3:** have the generator emit `"uniqueItems": true` for list documents read by `--decisions`.
- **F4:** have `present::required` tell a missing field apart from an explicit null.

**5. Attacked and could not break**
- **Refusal order:** all 9 pairs follow the documentation, including malformed vs invalid-identifier vs duplicate entry by entry, and undeclared-outcome vs undeclared-decision vs illegitimate-termination.
- **Schema vs parser:** keys, required fields, `additionalProperties` and identifier strings agree. Number, boolean, null, list, map, a key given twice and a key in the wrong case are all `malformed-input`.
- **A decision for an outcome that needs a predicate:** refused as `undeclared-decision`. `decision` placed beside or inside a predicate does not parse.
- **`read_ir`:** a decision name that is not an identifier (`a b`, `x\u200by`) is refused with exit 1.
- **`--authority`:** a decision is not a grant and a grant is not a decision. Neither document is read as the other, and the read order (authority, then instant, then decisions) holds.
- **CLI exit codes:** decided 0. Malformed, empty file, invalid-identifier, duplicate-identifier, undeclared-outcome, undeclared-decision and illegitimate-termination each exit 1 with nothing on stdout. A missing file or a directory exits 2 (`unreadable`).
- **Generated pages and status row:** the "Decision-based outcomes" row, `cli.md`, `conformance.md` and product.json's "seven ship today" match behaviour (conform ran 7).

**6. Paths written outside the worktree**, all under `~/.cache/ga-wave-2026-10-04-w8/canon-decision-outcomes/scratch/`:
- `adv2-cli/`
- `adv2-docs-red.log`
- `adv2-eval-run.log`
- `adv2-messages-red.log`
- `adv2-mutant/` (3.4M)
- `adv2-mutant-target/` (531M)
- `adv2-mutant-M1.log`
- `adv2-suite.log`

I also built in the assigned dir `~/.cache/b10x-target/canon-w8-decision-outcomes`. The worktree lease (`adversary2-decision-outcomes`) is released.

**7. Findings block**

```findings
- file: crates/canon/src/model/decision.rs
  line: 35
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The canon-decision/1 outcomes docs (and documents.md:72, ess protocol.yaml:436) say the shape is fixed by CANON-OUTCOME-001, but decided outcomes now carry decided_by and {decision, present} reasons fixed by CANON-OUTCOME-002."
- file: website/docs/status/where-this-stands.md
  line: 17
  category: contract-drift
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The shipped Evaluation documents status row names three formats and omits canon-decisions/1, which the linked documents page now lists."
- file: website/static/schemas/decisions-1.schema.json
  category: contract-drift
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The published decisions-1 schema accepts a list repeating an entry exactly, which canon evaluate refuses as duplicate-identifier; uniqueItems would state the rule."
- file: crates/canon/src/model/ids.rs
  line: 21
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "A canon-decisions/1 entry missing a key is refused with 'an explicit null is not allowed here; leave the key out to take its default' instead of naming the missing key."
- file: crates/canon/src/eval/decisions.rs
  line: 70
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "decided_by is deterministic but records only the least principal, dropping every other principal whose decision applied at the current revision, which design section 37 asks an evaluation to record."
- file: website/docs/concepts/protocols.md
  line: 142
  category: judgement
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The concept pages still say outcome requirements are written as predicates and list evaluation inputs without explicit decisions (also concepts/evaluation.md:3)."
```

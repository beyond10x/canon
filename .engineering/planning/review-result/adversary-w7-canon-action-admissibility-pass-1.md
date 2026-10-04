---
format: aep.planning-md/3
id: review-result:adversary-w7-canon-action-admissibility-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w7 adversary, canon story:action-admissibility, pass 1
relations:
- reviews: story:action-admissibility
revision: 1
---
unit: canon/action-admissibility, working tree at 5196c59 plus uncommitted phase 2 (worktree `~/.local/state/worktree/trees/b10x/canon/canon-w7-action-admissibility`)
verdict: NEEDS-CHANGE
cases: executed 260→265, red 5
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (plus the assigned build dir)
needs-coordinator: the fix for F3 is in files outside the unit's scope (`eval/mod.rs`, `canon-cli/src/lib.rs`, `canon-docs/src/pages.rs`)

**1. `git --no-pager diff --stat`** — exactly what I was handed. Every path in it is the unit's own phase-2 change. My two files are untracked test files: `crates/canon/tests/adversary_admissibility_reasons.rs` and `crates/canon/tests/adversary_admissibility_docs.rs`. I changed no implementation file.

**2. Cases added.** I ran these alone first, before the suite. All 5 are red now. `--list` confirms they exist in this tree.

| Test | Asserts | Red output, verbatim |
|---|---|---|
| `equivalent_negations_give_the_same_reason_naming_the_claim` | `{not: {claim: a}}` with a=TRUE gives `[{"claim":"a","value":"true"}]`, the same as `{claim: a, is: false}` | `left: {"reasons": [{"precondition": "false"}], ...}` `right: {"reasons": [{"claim": "a", "value": "true"}], ...}` |
| `a_claim_whose_negation_holds_is_not_a_reason` | `all:[not a, b]` with a=FALSE, b=UNKNOWN names only b | `left: [{"claim":"a","value":"false"},{"claim":"b","value":"unknown"}]` |
| `a_claim_in_a_branch_that_holds_is_not_a_reason` | `all:[a, any:[b,c]]` with a=F, b=T, c=F names only a | `left: [{"claim":"a","value":"false"},{"claim":"c","value":"false"}]` |
| `capability_reasons_are_in_capability_order_for_a_caller_built_ir` | a hand-built IR with `requires = [z, d]` gives reasons in the order d, z | `left: [{"capability":"z",...},{"capability":"d",...}]` |
| `the_reference_does_not_say_authority_is_unsupported_or_actions_unevaluated` | after `--authority` is shown to be read, `cli.md`, `evaluation.md` and `investigation-example.md` do not say it is unsupported or that actions are not evaluated | `cli.md: ... Read and passed through; not supported yet` (×2), `evaluation.md: authority is refused as unsupported-input`, `evaluation.md: each section is absent`, `investigation-example.md: actions are not evaluated` |

**3. Suite**, run after the cases: `cargo test --workspace --locked --no-fail-fast` gave EXIT=101, with 260 passed and 5 failed, summed over every `test result:` line. The only failures are the 5 above. The 260 "before" is the count the implementing state reported. Both new files pass `rustfmt --check` and `clippy -D warnings`; I formatted them before the clippy run.

**4. Findings**

| # | file:line | What was measured | What reaches it | Verdict / origin |
|---|---|---|---|---|
| F1 | `crates/canon/src/eval/actions.rs:100-102` | `unmet` tests each claim on its own and ignores the `not` above it. So `{not: {claim: a}}` with a=TRUE names no claim (it falls back to `{precondition: false}`). And in `all:[not a, b]` a claim that already satisfies the `not` is named. The story's Outcome says the reason names "the claim and its value", and the evaluator docs say the two forms are equivalent. | Any protocol: `canon compile` accepts `not` in a precondition, and my test compiles it. | NEEDS-CHANGE, introduced |
| F2 | `crates/canon/src/eval/actions.rs:100-102` | A claim in an `any` branch that already holds is still named. This breaks the unit's own contract: "only the reasons that decide it" (`conformance/scenarios/action-admissibility.yaml:7`, `actions.rs:5`). The module doc at `actions.rs:7-8` says "each claim … whose test is not true", so the two docs contradict each other. | Any precondition that nests `any` inside `all`. | NEEDS-CHANGE, introduced |
| F3 | `crates/canon-cli/src/lib.rs:72-73`, `crates/canon/src/eval/mod.rs:44-46,87-89`, `crates/canon-docs/src/pages.rs:1240-1242` | The published reference still says `--authority` is "not supported yet" and refused as `unsupported-input`, and that actions are not evaluated. The last of these sits directly under an example JSON that now shows `actions`. `generate --check` passes because the pages faithfully copy stale sources. | `canon evaluate --help` and the website. | NEEDS-CHANGE, introduced (these statements were true at base 4514e81) |
| F4 | `crates/canon/src/eval/actions.rs:20,74` | The doc promises "Capability reasons are in capability order". The code instead relies on the compiler sorting `requires`. `evaluate_with` accepts any `Ir` a caller builds, and `claims.rs` guards against exactly that for cycles. | Nothing found: the CLI and conform paths recompile the IR (`read_ir`). I built this state myself. | INFEASIBLE, introduced |

Fix directions, for the implementor:
- **F1/F2:** compute reasons with the polarity and branch taken into account, or reword the contract.
- **F3:** update the three source strings and regenerate the site.
- **F4:** sort and dedupe in `with`, or drop the promise.

**5. Attacked, not broken**
- **Mixed decisions:** a denial outranks an undecided capability, and the reasons list only the denials (the unit test covers this).
- **Unmet precondition plus a denial:** `blocked` with only the precondition reasons. This is documented ("Authority is not consulted").
- **A precondition over evidence:** gives `{precondition: unknown}`.
- **An excluded evidence kind:** the exclusion stages are still stubs, so I could not exercise it.
- **Authority for a capability no action requires:** ignored, as documented and as the scenario step `no-decision` uses it.
- **Case sensitivity:** capability ids compare exactly, the same rule `is_identifier` applies to protocol ids.
- **Action ordering:** follows the IR's sorted map.
- **No `requires` and no precondition:** `admissible`.
- **`approval-required` → `admissible` without a grant:** cannot happen; every requirement must be `Granted`.
- **Effect class:** design § 10 treats authority and effect class as independent (its read-only example requires `telemetry.read`), so ignoring `effect` is correct.
- **Coordinator test patch:** I found no lost guarantee. `three_valued_claims.rs` now compares only the listed sections, but `adversary_skel_cli.rs` still checks the full output bytes for the no-evidence step. The `adversary2_*` intent (no panic, the expectation is met) holds.
- **Authority refusal order and codes:** match `authority.rs`'s docs.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w7/canon-action-admissibility/scratch/adversary1-suite.log`
- The assigned build dir `~/.cache/b10x-target/canon-w7-action-admissibility` (already existed; I added build output to it)

I acquired no worktree lease, so none was released.

**7.**
```findings
- file: crates/canon/src/eval/actions.rs
  line: 100
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "unmet() ignores the negation above a claim, so {not: {claim: a}} with a TRUE names no claim while the equivalent {claim: a, is: false} does, and a claim that already satisfies its negation is named as a reason"
- file: crates/canon/src/eval/actions.rs
  line: 102
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a claim in an any-branch that already holds is listed as a reason, against the scenario's own 'only the reasons that decide it'"
- file: crates/canon/src/eval/mod.rs
  line: 44
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the generated reference (CLI help at canon-cli/src/lib.rs:72, eval module docs, canon-docs pages.rs:1240) still says --authority is unsupported and actions are not evaluated"
- file: crates/canon/src/eval/actions.rs
  line: 20
  category: property
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "capability reasons follow the IR's requires order, so a caller-built Ir with unsorted requires breaks the documented capability order; no caller found builds one"
```

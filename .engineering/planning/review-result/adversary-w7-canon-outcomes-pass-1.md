---
format: aep.planning-md/3
id: review-result:adversary-w7-canon-outcomes-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w7 adversary, canon story:outcomes, pass 1
relations:
- reviews: story:outcomes
revision: 1
---
unit: canon/outcomes, covering working tree `~/.local/state/worktree/trees/b10x/canon/canon-w7-outcomes` (cf91239 plus the uncommitted phase 2)
verdict: NEEDS-CHANGE
cases: executed 258→261, red 3
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (`~/.cache/ga-wave-2026-10-04-w7/canon-outcomes/scratch/adversary1-suite.log`), plus builds in the assigned dir `~/.cache/b10x-target/canon-w7-outcomes`
needs-coordinator: yes. Two things: should a termination through a declared but blocked outcome be refused (finding 5)? And the stale docs in `eval/mod.rs` and `eval/claims.rs` are outside this unit's scope (finding 4).

**1. `git --no-pager diff --stat`**
```
 crates/canon-cli/tests/adversary_skel_cli.rs    |  46 ++++---
 crates/canon-cli/tests/three_valued_claims.rs   |  45 ++++++-
 crates/canon/src/eval/outcomes.rs               | 154 ++++++++++++++++++++++--
 website/docs/reference/investigation-example.md |   5 +
```
All four are phase 2's uncommitted changes, made by the implementor and the coordinator, not by me. The only file I added is the untracked `crates/canon/tests/adversary_outcomes_reasons.rs`. I touched no implementation file.

**2. Cases added** (`crates/canon/tests/adversary_outcomes_reasons.rs`, all red now, and red on their first run alone)

| Test | What it asserts | Red output, verbatim |
|---|---|---|
| `a_negation_over_a_true_claim_gives_the_reason_its_is_false_test_gives` (:49) | With `c` TRUE, `{not:{claim:c}}` gives the same entry as `{claim:c, is:false}` | `left: {"reasons": [], "status": "blocked"}` / `right: {"reasons": [{"claim":"c","value":"true"}], "status":"blocked"}` |
| `a_negated_claim_whose_negation_is_met_is_not_named_as_a_reason` (:69) | With `c` FALSE and `d` UNKNOWN, `all[not c, d]` gives only `d` as a reason, as `all[c is:false, d]` does | `left: reasons [{c,false},{d,unknown}]` / `right: reasons [{d,unknown}]` |
| `a_blocked_outcome_whose_requirement_is_an_evidence_match_states_a_reason` (:89) | An outcome blocked by `requires: {evidence:{kind:l}}` has a non-empty `reasons` list | `"observed": {"reasons": [], "status": "blocked"}` |

**3. Suite run, after the cases existed**
`cargo test --workspace --locked --no-fail-fast` exited 101. Summed `test result` lines: 258 passed, 3 failed. The only failing target is `adversary_outcomes_reasons`. Full log is in `scratch/adversary1-suite.log`.

**4. Findings** (all against the working tree above)

| # | file:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|---|
| 1 | `crates/canon/src/eval/outcomes.rs:72-84` | NEEDS-CHANGE / introduced | `reasons()` names every claim node whose own test is not TRUE and ignores the `not` around it. So `{not:{claim:c}}` with `c` TRUE is blocked with `reasons: []`, while `{claim:c,is:false}` names `c`. The `eval` module docs (`mod.rs:25-27`) say these two mean the same thing. | The validator and compiler accept `not` in outcome `requires`; the IR tests at `ir/mod.rs:451,521` use it. Any protocol author can write it, though no fixture does today. |
| 2 | `crates/canon/src/eval/outcomes.rs:76-81` | NEEDS-CHANGE / introduced | Same root cause, other direction: in `all[not c, d]`, `c` FALSE is named as a reason even though FALSE is exactly what that member requires. | Same as #1. |
| 3 | `crates/canon/src/eval/outcomes.rs:13-15` | CONFIRMED / introduced | A blocked outcome can have `reasons: []` (evidence-match requirement, or #1). The doc comment admits it ("possibly none"). Design §6 ("not legitimate because …") and §14 (`blocked_outcomes[].because`) expect a reason. The scenario's reason shape is `{claim, value}` only, so there is no way to state an evidence reason. Once binding or freshness exclude records, an outcome blocked by an excluded kind will give no reason at all. | Validator accepts evidence matches in `requires`. No fixture uses one. |
| 4 | `crates/canon/src/eval/mod.rs:56`, `:88-89`; `crates/canon/src/eval/claims.rs:92` | NEEDS-CHANGE / introduced | Stale docs. `mod.rs` still says the outcomes section "refuses nothing yet", "each section is absent" and the decision equals three-valued claim evaluation "byte for byte"; all three are now false. `claims.rs` still says "The section stubs do not evaluate anything yet" on the `allow(dead_code)`. | Readers of the `eval` module docs. Both files are outside the story's scope, so the fix needs a patch from the coordinator. |
| 5 | `crates/canon/src/eval/outcomes.rs:38-49` | CONFIRMED / introduced | A termination through a declared outcome that is blocked is accepted, and the decision does not record the termination at all. CANON-OUTCOME-001 (design :1448) only requires "declared", but design §4.6 defines an outcome as a "declared *legitimate* terminal interpretation". The scenario's `terminated-declared` step only exercises a legitimate outcome. | Any case snapshot with `termination: supported` and no evidence. |

**5. Attacked and could not break**
- **Outcome ordering:** `ir.outcomes` is a BTreeMap and is rendered as a canonical JSON object, so the order is deterministic.
- **Excluded evidence kind:** the outcomes section receives the post-exclusion `applicable` evidence (`mod.rs` around :176). Every exclusion stage in this tree is inert, so no exclusion can be built yet. This becomes live through #3 once evidence-revision-binding and evidence-freshness merge.
- **`three_valued_claims.rs` loosening:** `listed_sections` compares the members `canon conform run` compares (`conform/mod.rs:93-97`): a missing listed section still fails, and extra sections are ignored by design. `adversary_skel_cli.rs` still pins the full document for the skeleton fixture. I found no wrong outcome decision that the old full comparison would have caught and that the outcomes scenario misses.
- **Undeclared termination:** it is refused naming the outcome, and `one_line` sanitises the id. It is refused after claim evaluation, which matches "Last" in `mod.rs:55`.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w7/canon-outcomes/scratch/adversary1-suite.log`
- `~/.cache/b10x-target/canon-w7-outcomes`, the assigned build dir. It was built into, not created by me.

**Fix to name (not applied):** compute the reasons from the claim tests that actually decide the requirement's value. Propagate through `not` (flip which test value counts as unmet) and through `any`/`all`, or normalise `not:{claim}` into `is:` before evaluating. Then either define a reason shape for evidence-only blocks or have validation refuse such outcome requirements.

```findings
- file: crates/canon/src/eval/outcomes.rs
  line: 72
  category: property
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "`{not: {claim: c}}` with c TRUE is blocked with an empty reasons list while the equivalent `{claim: c, is: false}` names c, so two requirements the eval docs define as equal give different decisions"
- file: crates/canon/src/eval/outcomes.rs
  line: 76
  category: property
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "in `all[not c, d]` with c FALSE the claim c is named as a reason although its value satisfies the requirement, because reasons() ignores the enclosing `not`"
- file: crates/canon/src/eval/outcomes.rs
  line: 13
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "a blocked outcome can carry `reasons: []` (evidence-match requirement or a `not` over a true claim), contrary to design § 6 and § 14 where a blocked outcome states why, and the {claim, value} shape cannot express an evidence reason"
- file: crates/canon/src/eval/mod.rs
  line: 56
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "eval module docs still say the outcomes section refuses nothing, every section is absent and the decision equals three-valued claim evaluation byte for byte (also mod.rs:88-89 and claims.rs:92), all false after this change and outside the story scope"
- file: crates/canon/src/eval/outcomes.rs
  line: 38
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "a termination through a declared but blocked outcome is accepted and the decision does not record it, while design § 4.6 defines an outcome as a declared legitimate terminal interpretation"
```

---
format: aep.planning-md/3
id: review-result:adversary-simulate-record-groups-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:simulate-record-groups: green, 3 notes'
relations:
- reviews: story:simulate-record-groups
revision: 1
---
Adversary pass 1 on story:simulate-record-groups, wave 2026-11-20-w7, on branch impl/simulate-record-groups at 41a8dd84. The report is recorded as the agent returned it. Verdict green: nothing in the implementation needs a fix; three mutants of the step order stayed green on the unit's tests and the pass's four cases (commit 7b42b328) kill them.

unit: story:simulate-record-groups, branch impl/simulate-record-groups, findings cover 41a8dd84 (+ adversary commit 7b42b328)
verdict: green
cases: executed 1174→1178, red 0
origin: introduced 3, pre-existing 0, undecided 0
wrote-outside-worktree: 1 directory, `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w7/simulate-record-groups/adversary-p1/`
needs-coordinator: no

Nothing in the implementation needs a fix. I found three gaps in the unit's tests: three mutants of the step order stayed green on them. The four cases I committed now catch all three.

**1. `git --no-pager diff --stat 41a8dd84..HEAD`** (only test files and test fixtures)
```
 .../canon-cli/tests/adversary_record_groups_cli.rs | 155 +++++
 .../record-group-adversary/count/protocol.yaml     |  95 +++
 .../record-group-adversary/fastest/protocol.yaml   |  54 +++
 .../group-order/protocol.yaml                      |  65 +++
 .../single-first/protocol.yaml                     |  56 +++
 5 files changed, 425 insertions(+)
```
Commit 7b42b328 `test: adversary pass 1 on record groups`, made as the bot. The tree is clean and the lease is released.

**Not done as the procedure says.** The brief told me to apply the mutants in the worktree. That goes against the procedure's rule "never mutate a file under attack". I followed the brief. Each mutant was restored with `git checkout -- <file>` before the next one. `git status` showed only my new files at the end.

**2. Cases added** (`crates/canon-cli/tests/adversary_record_groups_cli.rs`)
All four were written before anything ran them. Run alone (`case-alone.log`) they were **green**, not red: `test result: ok. 4 passed; 0 failed`. Each pins one sentence of § Outcome 3 or 4 that none of the unit's fixtures can observe:
- `single_kind_steps_come_before_group_steps`: the stuck state `{a,b,c}` is reached by the path "`b`, then `a` and `c`".
- `groups_step_in_order_of_their_least_kind`: groups written as `[c,b],[d,a]`; the path is "`a` and `d`, then `b` and `c`".
- `the_last_member_of_a_group_varies_fastest`: the cycle path is (x, other) then (other, y); 10 states.
- `a_group_step_has_the_product_of_its_members_successors`: one ungrouped kind (2 successors), a three-kind group (2×2×1) and a two-kind group (2×2). That gives 11 states and 10 stuck states, each a single path line.

**Mutants** (`mutants.log`). The unit's tests are `simulate_record_groups` and the `record_group` lib tests.

| mutant | unit's tests | adversary file |
|---|---|---|
| m1 a grouped kind also offered alone | red (`simulate_records_a_group_in_one_step`, `..._chain_...`) | red 4 |
| m2 group step adds only its first member | red (same two) | red 4 |
| m3 product replaced by a zip | red (same two) | red 2 |
| m4 group steps tried before single-kind steps | **green 7/7, 3/3** | red `single_kind_steps_come_before_group_steps` |
| m5 IR always writes `records_together` | red (`the_ir_writes_groups_sorted...`, `record_groups_read_back...`) | green |
| m6 diff classes a group `BREAKING` | red (`a_record_group_added_removed...`, CLI diff) | green |
| m7 validator accepts a kind outside `may_produce` | red (`invalid/not-produced.yaml`, validate unit test) | green |
| m8 groups tried in reverse order | **green** | red `groups_step_in_order_of_their_least_kind` |
| m9 first member varies fastest | **green** | red `the_last_member_of_a_group_varies_fastest` |

**3. Suite run** (after the cases existed): `cargo test --workspace --no-fail-fast` gave `suite exit 0`, 158 summaries, 1178 passed, 0 failed, 3 ignored (`suite.log`). `rustfmt --check` on the new file exits 0.

**4. Findings**

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/simulate/mod.rs:1024 | mutant | note | CONFIRMED | introduced | single-kind steps before group steps (§ Outcome 4) had no test; m4 stayed green. Reached by any action with a grouped and an ungrouped kind where two equally short paths lead to one printed state. Now covered by 7b42b328. |
| crates/canon/src/simulate/mod.rs:877 | mutant | note | CONFIRMED | introduced | group order by least kind id had no test; m8 stayed green. Reached by any action with two groups. Now covered. |
| crates/canon/src/simulate/mod.rs:889 | mutant | note | CONFIRMED | introduced | "last member varying fastest" had no test; m9 stayed green. Reached by a group whose members have more than one class. Now covered. |

**5. Attacked and not broken**
- **Bytes, base against head** (`bytes.sh`): 137 `protocol/1` files under `fixtures/` and `conformance/`, each through validate, compile, check, simulate and generate (output trees compared with `diff -r`). The base binary accepts 105 of them.
  - Every difference is in the 7 `record-group/` files, which the base refuses with a parse error.
  - The other 130 give identical bytes on all five commands (claims 3 and 6).
- **Claim 2:**
  - 10 grouped protocols (`grouped`, `result`, `chain` and 7 probes), each against a copy with the declaration removed: check and generate (including scenario trees) are identical.
  - Compile differs only by added `records_together` lines; no line is removed.
  - `evaluate` on `result` with my own case (`shape ok`, `fit bad`) gives identical stdout, stderr and exit.
- **Claim 4:**
  - A grouped kind with `supplied_by`: the supplied step adds it alone, which the story allows ("Supplied kinds stay single-record steps"). The stuck state printed then is real for a runner that records the group together.
  - A grouped kind with `max_age`: expiry leaves the other member alone, and the group step re-adds both. That gives mixed classes, which is the documented known limit and true of a rerun.
  - A member already present adds nothing. A kind grouped on two actions is valid.
  - Invalidation never fires in the walk, because no step revises an artifact.
- **Claim 5:** the 3-kind group and 2 groups on one action match my hand count. The print is right for a member with a result, a subject, a producer profile and a child's record on one path.
- **Claim 1** (13 variants):
  - Accepted: `[a,a,b]` (IR holds `[a,b]`), `[]` (no IR key), `[a,A]` as distinct kinds.
  - Refused: `{kinds: []}`, the same group twice, two faults on one action, faults on two actions. The order is stable.
  - The unit refuses only groups naming `case_outcome` on an opening action, not every group there. This matches § Outcome 3; the brief's wording of claim 1 is wider than the story.
- **Claim 3, `canon diff`:** add, remove, change and reorder-only (`no declaration differs`) all behave as claimed. A group change together with a capability change prints `TIGHTENING` and the `NO SEMANTIC CHANGE` lines.
- **Claim 7:**
  - The pinned line is reproduced twice with identical bytes.
  - The fixture is byte-equal to `renamed/chain-grouped.protocol.yaml`.
  - The sort order is kept within every section: artifacts, kinds, claims, obligations, actions, invalidation.
  - The diff carries none of the other project's vocabulary.
- **Docs:** the module docs, the `ess/` comments and the three regenerated pages state nothing the code does not do. The "twelve lists" and "sixteen exceptions" counts check out.

**6. Paths written outside the worktree** (all under `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w7/simulate-record-groups/adversary-p1/`):
- binary: `canon-head`
- scripts: `bytes.sh`, `strip.sh`, `evaluate.sh`, `mutants.sh`
- output directories: `bytes/`, `strip/`, `evaluate/`, `mutants/`, `probes/`
- logs: `build-head.log`, `bytes.log`, `strip.log`, `case-alone.log`, `mutants.log`, `suite.log`
- other: `chain-1.out`, `chain-2.out`, `msg-1.txt`

```findings
[{"file": "crates/canon/src/simulate/mod.rs", "line": 1024, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "No test of the unit observed that single-kind steps precede group steps; trying group steps first stayed green until adversary_record_groups_cli pinned it."}, {"file": "crates/canon/src/simulate/mod.rs", "line": 877, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "No test of the unit observed group steps in order of least kind id; iterating groups in reverse stayed green until adversary_record_groups_cli pinned it."}, {"file": "crates/canon/src/simulate/mod.rs", "line": 889, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "No test of the unit observed the last group member varying fastest; varying the first fastest stayed green until adversary_record_groups_cli pinned it."}]
```

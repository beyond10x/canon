---
format: aep.planning-md/3
id: review-result:adversary-simulate-revise-step-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on the unit of story:simulate-revise-step: green, three notes'
relations:
- reviews: story:simulate-revise-step
revision: 1
---
Adversary pass 1 on the unit of story:simulate-revise-step (wave 2026-11-20-w8), run on Opus against `5182c2cf`, recorded as returned.

unit: story:simulate-revise-step, findings cover 5182c2cf, cases committed as f159b8ac
verdict: green
cases: executed 1233→1235, red 0
origin: introduced 3, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w8/simulate-revise-step/adversary-p1/ (enumerator, fuzz logs, probes, mutant copies and logs, binary canon-head, out-base/, out-head/, suite logs, msg-1.txt)
needs-coordinator: no

**1. Diff (5182c2cf..f159b8ac):** I changed only two new test files.
```
 crates/canon-cli/tests/adversary_simulate_revise_cli.rs | 46 +++
 crates/canon/tests/adversary_simulate_revise.rs         | 64 +++
```
The brief told me to apply mutants in the worktree. I did, and restored with `git checkout -- crates/canon/src/simulate/mod.rs` after each one. Each mutant log ends `restored 0`. This goes against the skill's rule to mutate only copies; I followed the brief because it is the more specific order.

**2. Cases added (both green on head)**
- `crates/canon/tests/adversary_simulate_revise.rs::a_revise_step_adds_its_record_after_the_revision`
  - It reaches the state `{evidence note about doc}` only through `fix revises doc and produces note`. This rests on § Outcome 1: "then the record is added".
  - Run alone on head: `1 passed`.
  - On mutant M3 (record added before the revision): `test a_revise_step_adds_its_record_after_the_revision ... FAILED`, panicked at `adversary_simulate_revise.rs:44:5`, `m03 exit 101`.
- `crates/canon-cli/tests/adversary_simulate_revise_cli.rs::judgement_marking_has_no_finding_once_its_revision_is_walked`
  - Pins exit 0 and the exact summary line of `judgement-marking.yaml`, run twice. This rests on § Outcome 5, class c.
  - The base binary gives exit 1 and a `cycle` (`out-base/`).

**3. Suite (after the cases):** `cargo test --workspace --locked --no-fail-fast` exit 0: 162 summaries, 1235 passed, 0 failed, 3 ignored. `cargo fmt -p b10x-canon -p canon-cli --check` exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0.

**4. Findings**

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/simulate/mod.rs:1333 | mutant | note | CONFIRMED | introduced | M3 passed all 20 lib tests and every test in canon_simulate and simulate_child_outcome; fuzz found 9 of 300 disagreeing. The code is right. The new case now kills M3. |
| fixtures/investigation/judgement-marking.yaml:1 | acceptance | note | CONFIRMED | introduced | Answer (b) holds: before this pass no test ran `canon simulate` on it. Exit 0 with 0 findings is correct by my enumerator (19 states agree). The new case now pins it. |
| crates/canon-cli/src/lib.rs:202 | contract-drift | note | CONFIRMED | introduced | `canon simulate --help` and `website/docs/reference/cli.md` say "an open state in which every step of every admissible action … returns the state is counted as waiting". They leave out "no outcome legitimate, none approval-required". By that text, `revise-child` would count 4 waiting states, but the code counts 2: the terminable `{start, case_outcome accepted/other earlier}` states are excluded. The module docs (mod.rs:199) and `ess/domains/cli.yaml` are correct. |

**Enumerator against the binary.** I wrote an independent Python walk of § Outcome 1–4 and R3–R6 in scratch. It covers bound and unbound kinds, results, `all`/`any`, `is`, capabilities, `revises` and invalidation rules.
- It covers the 5 `revise*` fixtures without children, `judgement-marking`, 2 probes, and 6000 random protocols (seeds 2–7).
- That is 74 604 states, 393 of them cycles with the revision suffix.
- It found 0 disagreements on state count, terminable count, stuck or cycle count, or the set of printed findings with their suffix. Every state that is not open reaches a stuck state or a cycle.
- The first batch found one model gap on my side: a produced kind that nothing reads still has a dimension. I fixed the enumerator, not the binary.
- I checked `revise-child` (9 states, 2 waiting) by hand: it matches.

**5. Attacked, not broken**
- **Mutants:** 11 tried, 10 killed by the unit's tests. Killed:
  - M1 (not-admissible): 2 lib tests and `simulate_revises_declared_artifacts` fail.
  - M2 (only single-artifact subsets): `revise_subsets_come_in_order_of_their_sorted_ids` fails.
  - M4 (current record about `a` kept): 3 lib tests fail.
  - M5 (`earlier` dropped): `a_record_a_children_form_reads_becomes_earlier` and revise-child fail.
  - M6 (upstream change dropped): the upstream lib test and revise-upstream fail.
  - M7 (suffix always): 3 lib tests and 4 CLI tests fail.
  - M8 (suffix never): 2 lib tests and revise-denied fail.
  - M9 (revise steps tried first): `a_record_step_is_tried_before_a_revise_step` fails.
  - M10 (waiting ignores revise): 1 lib test and revise-child fail.
  - M11 (old cycle rule): 2 lib tests and revise-denied fail.
- **Bytes, 147 `protocol/1` files under `fixtures/` and `conformance/`, base binary against head:**
  - `simulate`: 141 equal. 6 differ, and all 6 declare `revises`: judgement-marking, revise, revise-upstream, revise-denied, revise-mixed, revise-child. `revise-undeclared` is equal.
  - `check`: 147 of 147 equal.
  - `compile`: 147 of 147 equal.
  - `generate`: 147 of 147 equal once the output directory in stdout is normalised.
  - The saved base binary was built at 17:12, between 997c3997 (17:03) and the unit's first commit (17:14), which changes no code.
- **Code reading:**
  - Answer (a) holds: `subsets(&[])` returns nothing, `revisers` stays empty, and the old text prints.
  - No exit-0 run prints a path, so acceptance 3 and 6 cannot hold as written. The path and the `earlier` rendering are pinned in `simulate/tests.rs`.
  - Claim 3 holds: `revise` touches only words after `base`.
  - An approval-required or blocked action gets no revise step: the loop runs over admissible actions only.
- **No vocabulary of another project** in the 6 fixtures or the tests.

**6. Written outside the worktree:** only `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w8/simulate-revise-step/adversary-p1/`, holding `enum/` (with `work/` leftovers), `probes/`, `mutants/`, `out-base/`, `out-head/`, `canon-head`, `*.log`, `*.sh` and `msg-1.txt`. My lease is ended and the tree is clean.

```findings
[{"file": "crates/canon/src/simulate/mod.rs", "line": 1333, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Adding a revise step's record before the revision instead of after left every unit test green; the adversary case a_revise_step_adds_its_record_after_the_revision now kills that mutant."}, {"file": "fixtures/investigation/judgement-marking.yaml", "line": 1, "category": "acceptance", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The revise step changed canon simulate on this fixture from a cycle with exit 1 to no finding with exit 0, and no test ran it until the adversary CLI case pinned the bytes."}, {"file": "crates/canon-cli/src/lib.rs", "line": 202, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The help text counts every open state whose admissible actions' steps all return it as waiting, omitting that no outcome is legitimate and no action is approval-required there, so on revise-child it implies 4 waiting states where the walk counts 2."}]
```

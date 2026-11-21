---
format: aep.planning-md/3
id: review-result:adversary-pending-decisions-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:pending-decisions (wave 2026-11-20-w5): green, 7 cases added, no finding'
relations:
- reviews: story:pending-decisions
revision: 1
---
Adversary pass 1 on story:pending-decisions, wave 2026-11-20-w5, `aep:adversary` on Opus, on `impl/pending-decisions` at `d8b5457`. The report below is as the agent returned it.

unit: story:pending-decisions
verdict: green
cases: executed 1023→1030, red 0
origin: introduced 0, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w5/pending-decisions/adversary-p1/ (crosscheck.py, xc-sim/, mutate.py, mutants.sh, mutants.log, check.sh, check.log, adv-test.log, build.log, absent-registry/, msg-1.txt)
needs-coordinator: no

**No defects found.** I added 7 cases and committed them as the bot: `cd2648ff` `test: adversary pass 1 on pending decisions`. The only file is `crates/canon/tests/adversary_pending_decisions.rs`. I added no fixture and no CLI test file. Not pushed. Lease released.

**Findings**

| file:line | category | severity | verdict | origin | reached by |
|---|---|---|---|---|---|
| — | — | — | — | — | none |

**What I attacked and did not break**

- **Claim 4, reading reasons as entries (5 attacks, 0 broken).**
  - An `approval-required` action only ever carries capability reasons (`actions.rs:148-151`), so a precondition reason can never sit beside one.
  - A decision-requiring outcome carries only `{"decision", "present": false}` (`outcomes.rs:149-151`).
  - Nothing called `legitimate_when` exists in `crates/canon/src`, and nothing about obligations writes outcome reasons.
  - Capability, outcome and decision ids spelled `none`, `require_approval`, `denied`, `decision` and `present` come out as plain entries (case 1).
- **Authority edges (6, 0).**
  - One capability at two scopes, one granted: only the other is listed.
  - `require_approval` beside a `granted` entry bound to c2 with the exact scope: the entry stays, `current: require_approval`.
  - `denied` beside `require_approval`: the action is blocked and has no entry.
  - An entry bound to c1 while the snapshot is at c2: `current: none` (scenario step 7).
  - A snapshot without `revision` and entries bound to c2: they lapse, and no entry carries `case_revision`.
  - One of two capabilities denied: the action is blocked, and another action needing the same requirement still lists it.
  - The precedence note in the reference text matches `actions.rs:64-65`.
- **Decisions edges (4, 0).** An entry at the current revision, one at another revision, and a snapshot with no revision (`options: []`, steps 6 and 8). Two outcomes requiring the same decision name, taken at c2 for one and at c1 for the other: only the second is listed (case 2).
- **Scope order (1, 0).** Upper-case, non-ASCII and numeric scope text, with numbers read as text (`10` before `9`), all sort by code point, key before value (case 6).
- **Termination (1, 0).** A case terminated through a legitimate outcome still lists the other outcome and the action waiting on approval, as the story's Scope says (case 7).
- **Claim 3, bytes (131 steps, 0 differences).**
  - I ran every `decision` step of all 25 scenarios through this branch's binary and through `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w5/sdc-run/canon-sim`.
  - Before comparing, I stripped `pending_decisions`, `missing_evidence` and the `revision` key on `children` reasons, since canon-sim lacks the last two.
  - 7 steps of `child-outcome-revision.yaml` were skipped because canon-sim cannot compile that fixture.
  - Result line: `{"steps": 138, "same": 131, "differ": 0, "skipped": 7, "with_pending": 28, "pending_key_absent_ok": 65}`. The 65 decisions with nothing pending are raw byte-identical to canon-sim's output, with no key added.
- **Step 9 is a real comparison.** The implementor's report does not name the base binary. But `fill.py` ran the canon binary it was given on the scenario inputs, and commit 1 changes no Rust code. The canon-sim run reproduces step 9 and the `actions`/`outcomes` bytes of steps 1–8 exactly.
- **Scenario checked by hand (4 steps).** Steps 1, 6, 7 and 8 match the story's § Acceptance items 1, 6, 7 and 8.
- **Mutants of `eval/pending.rs` (9).** I ran each with `conform run --requirements`, then restored the file from git; the clean tree gives `conform: 25 passed, 0 failed, 0 unreadable`.
  - `current` always `none` → `failed: scenario \`CANON-PENDING-001\` step \`require-approval\`: canon-decision/1 section \`pending_decisions\` differs from the expectation at line 129`
  - blocked actions' denied requirements listed → `... step \`denied\`: ... at line 78`
  - authority list reversed → `... step \`nothing-given\`: ... at line 103`
  - scope sorted value before key → `... step \`nothing-given\`: ... at line 122`
  - `options` not emptied without a revision → `... step \`snapshot-without-revision\`: ... at line 198`
  - decisions list reversed → `... step \`nothing-given\`: ... at line 206`
  - Three stayed green (`conform: 25 passed, 0 failed, 0 unreadable`): the status filter dropped, the authority sort removed, the decisions sort removed. All three change no output for any input: blocked actions never carry `none` or `require_approval` reasons, and both source sections already come in action/capability/scope and outcome-id order.
  - Declaration-order scopes would fail step 1, because `compare` declares its five scopes out of order.
- **Requirement sentences (13, 0 unpinned).** Each sentence of CANON-PENDING-001 is pinned by a step. "A list with no entry MUST be left out" is pinned by steps 4 and 6. The section's own absence cannot be a step: a scratch scenario that lists `pending_decisions` on a decision without one gives `failed: ... section \`pending_decisions\` differs from the expectation at line 2`. The unit tests cover that absence.
- **Position and shape (3, 0).** The key sits after `outcomes` and before `protocol`. The explanation does not read the section (`explain/mod.rs:143-144`, `:569`). `canon explain` is not a subcommand; I checked the explanation section instead.
- **Reference text (3, 0).** The `protocol.yaml` comment, `documents.md` and `evaluation.md` state nothing the code does not do. The `# read from: …pending-decisions.md:46-63` lines are the story's § Outcome.

**My suite's summary lines:** `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`; `cargo fmt -p b10x-canon -p canon-cli --check` exit 0; `cargo clippy --locked -p b10x-canon --test adversary_pending_decisions -- -D warnings` exit 0.

```findings
[]
```

---
format: aep.planning-md/3
id: review-result:adversary-authority-revision-binding-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:authority-revision-binding (wave 2026-11-20-w3): green, 1 note, introduced'
relations:
- reviews: story:authority-revision-binding
revision: 1
---
Adversary pass 1 on story:authority-revision-binding, wave 2026-11-20-w3, `aep:adversary` on Opus.
The report below is as the agent returned it.

unit: story:authority-revision-binding, branch impl/authority-revision-binding at e3ce944 (my commit 88b78c0 sits on top)
verdict: green
cases: executed 972→974, red 0
origin: introduced 1, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w3/authority-revision-binding/adversary-p1/ (bytes.py, bytes/, probe.py, probe/, build.log, case.log, suite.log, msg-1.txt)
needs-coordinator: no

Nothing turned red. The one finding is that a test was missing, not that the code is wrong.

On the 429 message: I had already got past the byte-comparison script, so nothing was redone. I checked `git status`, the log and the scratch directory, waited for the suite run that was still going, then committed.

**1. What I touched** (`git --no-pager diff --stat e3ce944..HEAD`)
```
 crates/canon/tests/adversary_authority_binding.rs | 102 ++++++++++++++++++++++
 1 file changed, 102 insertions(+)
```
Only a test file. One commit, `88b78c0 test: adversary pass 1 on authority revision binding`, made through `b10x-gates bot`, with a body.

**2. Cases added** (`crates/canon/tests/adversary_authority_binding.rs`, both green now and green when first run)
- `a_bound_require_approval_applies_only_at_its_case_revision`: a `require_approval` bound to c1 next to an unbound grant. At c1 the result is approval-required. At c2, and with no revision, it is admissible. Alone at c2 the reason is `decision: none`.
- `a_lapsed_entry_takes_no_part_in_scoped_coverage_or_precedence`:
  - A lapsed wide denial with an unbound narrow grant gives admissible at c2 and blocked at c1.
  - A lapsed narrow grant with an unbound wide `require_approval` gives approval-required.
  - Two principals at c2, one granting and one denying, give blocked. At c1 the result is `none`.

First run of that file alone, `cargo test -p b10x-canon --test adversary_authority_binding`, exit 0:
```
test a_bound_require_approval_applies_only_at_its_case_revision ... ok
test a_lapsed_entry_takes_no_part_in_scoped_coverage_or_precedence ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
They pin a claim no other test pins. They do not show a defect.

**3. Suite** (run after the cases existed)
`export PATH=$HOME/.cache/ess/toolchains/0.77.0:$PATH; cargo test --workspace --locked --no-fail-fast` gave exit 0. Totalled from its own `test result:` lines: 137 summaries, 974 passed, 0 failed, 3 ignored. That is the implementor's 972 plus my 2.

`canon conform run --requirements conformance/requirements.yaml` gave exit 0:
```
conform: 21 passed, 0 failed, 0 unreadable
requirements: 22 implemented, 22 covered, 0 planned
```

**4. Findings** (covering e3ce944)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| conformance/scenarios/authority-binding.yaml:15 | mutant | note | CONFIRMED | introduced | No scenario step, unit test or other test in the tree binds a `require_approval` entry (searched across `crates`, `conformance` and `fixtures`). A filter in `applying_at` (`crates/canon/src/eval/authority.rs:123`) that kept a bound `require_approval` at every revision would pass CANON-AUTHORITY-002 and the unit's own tests. My first case would catch it. |

What reaches it: any caller who binds a `require_approval`. Claim 1 ("whatever it decides") covers it, but only `granted` and `denied` are pinned. I did not run the mutant: the rules forbid mutating the files under attack or making a second copy of the tree, so the "would pass" above comes from that search, not from a run. The suggested fix is one scenario step, bound `require_approval` at c1 next to an unbound grant at c2, expecting admissible. That edit is the implementor's or the coordinator's, not mine.

**5. Attacked and not broken** (head binary built from e3ce944)
- **Unchanged bytes:** 45 comparisons, 0 different, of the oracle `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w2/oracle/canon-6f44325` (standing in for the base, as the brief allows) against the head, on stdout, stderr and exit status. Inputs: every conformance step whose authority list has no new fields (capability-scope, case-composition, explanation, action-admissibility), each run as given, at c1, c2, r9 and with no revision. The explanation is part of those bytes.
- **Lapse cases, 8 probes, all as option A says:** a bound `require_approval` at c1 and at c2; a bound grant at c1 with an unbound denial (blocked); scoped wide/narrow in both directions; a bound and an unbound entry with opposite decisions at c1 and at c2; a lone lapsed denial (`none`); no revision with a bound denial and an unbound grant (admissible).
- **Duplicate key, 7 probes, all as the story says:** entries differing only in principal are both read and the denial wins; entries equal in all four fields are refused and the message names scope, case revision and principal; a duplicate is caught before a later invalid entry; an invalid principal is caught before a duplicate; scope key order and an empty versus absent scope are treated as equal.
- **Identifier edges, 13 probes, case snapshot and authority document always agree:**
  - Refused as malformed: number, boolean, null, a list, an unknown key.
  - Refused as an invalid identifier: empty string, zero-width character, non-breaking space.
  - Read: a non-ASCII principal, a 20000-character principal, a JSON document.
  - NFC and NFD spellings of one revision do not match. Revisions are opaque text compared for equality, as for explicit decisions.
- **CANON-AUTHORITY-002 against mutants, 6 caught:** filter removed (step 1); filter applied to grants only (step 4); old two-part duplicate key (step 7a); principal left out of the key (two-principals step); decision added to the key (duplicate step); no revision treated as matching everything (step 6).
- **Help patch:** both sentences are true of the code.
- **`canon check` and `canon generate`:** both already evaluate a case that has a revision and write unbound entries, and their tests are green.
- **Docs:** the generated schema and the reference pages agree with the code.

**6. Paths written outside the worktree**
`$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w3/authority-revision-binding/adversary-p1/`: bytes.py, bytes/ (compiled IRs and per-step inputs), probe.py, probe/, build.log, case.log, suite.log, msg-1.txt.

```findings
[{"file": "conformance/scenarios/authority-binding.yaml", "line": 15, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "No scenario step or unit test binds a require_approval entry, so a filter that kept a bound require_approval at every revision would leave CANON-AUTHORITY-002 and the unit's tests green; the adversary case a_bound_require_approval_applies_only_at_its_case_revision now pins it."}]
```

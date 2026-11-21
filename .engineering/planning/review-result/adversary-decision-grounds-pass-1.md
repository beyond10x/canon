---
format: aep.planning-md/3
id: review-result:adversary-decision-grounds-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:decision-grounds: green, 1 warning to confirm, 4 notes'
relations:
- reviews: story:decision-grounds
revision: 1
---
First adversary pass on the unit of story:decision-grounds (head e6f60f52; the adversary's cases at a3e2497e). Recorded as the adversary returned it.

unit: story:decision-grounds, branch impl/decision-grounds, implementation at e6f60f52, my case at a3e2497e
verdict: green
cases: executed 1064→1070, red 0
origin: introduced 4, pre-existing 0, undecided 0
wrote-outside-worktree: 1 directory, `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w6/decision-grounds/adversary-p1/`
needs-coordinator: yes. Finding 2: someone has to confirm or reverse the unit's rule that an entry set aside by its case revision adds none of its grounds.

The implementation holds up against all six claims. Its suite missed four rules: four mutants left the unit's tests and `conform run` green. My six new cases catch them and pass on the branch.

**1. Diff since e6f60f52** (only a test file; nothing outside test paths is left changed)
```
 crates/canon/tests/adversary_decision_grounds.rs | 188 +++++++++++++++++++++++
```
Commit a3e2497e, `test: adversary pass 1 on decision grounds`, made as the bot. Lease released and `worktree discard-cache` run (6963 MiB).

**Charter conflict, stated plainly.** The brief told me to mutate the source files in the tree and restore them from git. The adversary procedure forbids touching a file under attack, even briefly. I followed the brief, holding lease `adv1-decision-grounds`. Before each mutant I copied the file to `mut/`, and copied it back afterwards. `git diff --quiet` was clean after all 20 runs. No mutated file was committed.

**2. Cases added**, in `crates/canon/tests/adversary_decision_grounds.rs`. Each was run alone (`cargo test -p b10x-canon --test adversary_decision_grounds`): 6 passed on the branch. Under the mutant that survived the unit's gate, each turns red:

| Case | Rule it pins | Mutant | Red line |
|---|---|---|---|
| `a_set_aside_entry_on_another_capability_adds_no_ground` | a set-aside entry on capability `y` adds nothing to `x`'s reason | M11, `authority.rs:144` capability filter dropped | `panicked at …adversary_decision_grounds.rs:73:5` |
| `a_set_aside_entry_at_a_scope_that_does_not_cover_adds_no_ground` | a set-aside entry at `{ns: b}` adds nothing to a requirement at `{ns: a}` | M10, `authority.rs:144` scope filter dropped | `…:96:5` |
| `a_require_approval_reason_names_no_stale_ground` | a `require_approval` reason gets no `stale_grounds`, whether the set-aside entry granted or denied | M9, `actions.rs:150` `(None, …)` changed to `(_, …)` | `…:124:9` |
| `a_decision_for_another_outcome_adds_no_ground_to_this_one` | two outcomes require `d`; a set-aside decision for `first` adds nothing to `second` | M12, `decisions.rs:155` outcome filter dropped | `…:155:5` |
| `a_set_aside_narrower_denial_beside_a_wider_grant_is_admissible` | a narrower denial set aside by its grounds does not override a wider grant | pin only | – |
| `an_evidence_ground_on_a_record_no_claim_reads` | an evidence ground holds on a record of a kind no claim reads (missing-evidence D1); fails as `revision_mismatch` when bound elsewhere | pin only | – |

**3. Suite run, after the cases existed**
- `cargo test --workspace --no-fail-fast`: 1070 passed, 0 failed, 3 ignored, exit 0.
- `canon conform run --requirements conformance/requirements.yaml`: 26 passed, 0 failed; 27 implemented, 27 covered; exit 0.
- `cargo clippy -p b10x-canon --tests -- -D warnings` on local 1.99.1: exit 0. I could not run CI's 1.100.0.

**4. Findings** (tree e6f60f52)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/eval/authority.rs:144 | mutant | note | CONFIRMED | introduced | any `--authority` with two capabilities or scoped entries. Dropping either filter left the unit's gate green; now pinned |
| crates/canon/src/eval/actions.rs:150 | mutant | note | CONFIRMED | introduced | a lapsed entry beside an applying `require_approval`. M9 survived; now pinned |
| crates/canon/src/eval/decisions.rs:155 | mutant | note | CONFIRMED | introduced | two outcomes requiring one decision name. M12 survived; now pinned |
| conformance/requirements.yaml:205 | judgement | warning | CONFIRMED | introduced | see below |
| ess/domains/protocol.yaml:1059 | contract-drift | note | CONFIRMED | introduced | readers of `ess/` |

- **Finding 2 (requirements.yaml:205, the case-revision rule).** "An entry set aside by its case revision adds none of its grounds" is the unit's own rule. The story says nothing like it: it says `stale_grounds` lists "each failed ground of the set-aside entries", and those include entries bound to another case revision. The unit wrote its rule into CANON-GROUND-001 and step `an-entry-at-another-case-revision-adds-no-ground`. The scenario does fail if the rule is reversed (M5 and M13 both fail that step).
- **Finding 5 (protocol.yaml:1059, stale reference).** The `read from: crates/canon/src/eval/authority.rs:1-41` line is out of date: the module doc now runs to line 50. The struct comments of `AuthorityDecision` and `ExplicitDecision` still give the refusal order and the applicability rule without grounds; only the field comments mention grounds.

**5. Attacked and not broken**
- **Claim 4, unchanged bytes without grounds.** I ran base and branch over every evaluate step in the 25 scenarios that have no grounds: 152 of 152 steps byte-identical (stdout, stderr, exit). The same 31 inputs with `ground_revisions: {}` and `ground_evidence: []` added gave 31 of 31 identical to base.
  - Base binary: `$HOME/.cache/scratch/…/w6/simulate-child-outcome-step/canon-base` (0.9.0). Its owner has since deleted it. A second binary, `…/simulate-child-outcome-step/adversary-p1/canon-base`, reports 0.10.0 and refuses grounds; it matched 152/152 once that version line was normalised.
  - The moved claim block contains no fallible call. Grounds refusals come after `decisions::read` and before the exclusion stages, which is the documented order.
- **Refusal pairs:** undeclared-artifact against undeclared-outcome, duplicate, malformed decisions and bad `--at`. All follow the documented order.
- **Claim 2, evidence grounds:** a record of a kind no claim reads holds. A record two claims exclude for different reasons takes the first claim's reason in claim-id order, checked both ways. A record excluded by one claim and counted by another fails. No `max_age` and no `--at` holds. Invalidation through an unwatched upstream holds.
  - Supersession: not on this base; a duplicate record id is refused.
  - An artifact with no current revision cannot happen: the snapshot must list every declared artifact (`missing-artifact`).
- **Claim 1, precedence:** lapsed denied beside require_approval, lapsed granted beside denied, two principals with one lapsed (`decided_by` names only `q`), and an entry both bound elsewhere and with failed grounds. All correct.
- **Claim 3, bytes:** key order, dedup across entries, an id listed twice, and empty grounds reading as absent. All correct.
  - In `computed_from`, `ground_evidence` is written sorted and without repeats, and empty grounds are left out. That matches the module docs; the story's "as given" is looser.
- **Claim 5:** JSON and YAML duplicate ground keys, null, a number as revision, and duplicates with different grounds are all refused.
- **Claim 6:** pending lists such entries without the grounds. No other reason form can pass the filter (it needs `decision` and `present`).
- **Other readers:** `canon check` and `generate` write no grounds and round-trip. `canon simulate` reads no authority document (it calls only `eval::check_depth`). `missing_evidence` never names a record that a ground needs; the story does not ask for that.
- **Mutants M1–M8, M13, M15:** all red in the unit's gate. M1 and M2: `decision_grounds.rs:223` and `:110`. M3: `:110`. M4: `stale-grounds-order`. M5 and M13: `an-entry-at-another-case-revision-adds-no-ground`. M6 and M15: `:275`. M7: `:223`. M8: `explicit-decision-ground-moved` pending section.
- **Mutant M14** (claim reasons read before the binding-stage reason) stays green. It cannot change any output: a claim lists a binding exclusion with the binding stage's reason.
- **Read-from spot-check:** `model/authority.rs:43-45`, `:46-50` and `:53-56` are correct.

**6. Written outside the worktree**
`$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w6/decision-grounds/adversary-p1/`. It holds the `diff.py`, `probe.py`, `probe2.py`, `mutate.py` and `mutate2.py` scripts, plus `suite.sh`, `base2-norm.sh` and `msg-1.txt`. It also holds the run outputs: `diffrun*/`, `probes*/`, `fx/`, `mut/`, `mut2/`, `canon-branch`, `canon-base2` and the `*.log` files.

```findings
[{"file": "crates/canon/src/eval/authority.rs", "line": 144, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Dropping the capability or the scope filter of Authority::stale_grounds left the unit's tests and conform run green; adversary_decision_grounds.rs now pins both."},
 {"file": "crates/canon/src/eval/actions.rs", "line": 150, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Adding stale_grounds to a require_approval reason left the unit's gate green; now pinned by a_require_approval_reason_names_no_stale_ground."},
 {"file": "crates/canon/src/eval/decisions.rs", "line": 155, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Ignoring the outcome in Decisions::stale_grounds left the unit's gate green; now pinned by a_decision_for_another_outcome_adds_no_ground_to_this_one."},
 {"file": "conformance/requirements.yaml", "line": 205, "category": "judgement", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "The rule that an entry set aside by its case revision adds none of its grounds is the unit's own, while the story says stale_grounds lists each failed ground of the set-aside entries; the coordinator should confirm it."},
 {"file": "ess/domains/protocol.yaml", "line": 1059, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The read-from authority.rs:1-41 no longer spans the module doc (now 1-50), and the AuthorityDecision and ExplicitDecision struct comments give applicability and refusal order without grounds."}]
```

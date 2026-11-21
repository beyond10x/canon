---
format: aep.planning-md/3
id: review-result:adversary-evidence-redelivery-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:evidence-redelivery: green, no findings'
relations:
- reviews: story:evidence-redelivery
revision: 1
---
First adversary pass on the unit of story:evidence-redelivery (head 6954bc4d; no case added). Recorded as the adversary returned it.

unit: story:evidence-redelivery, branch impl/evidence-redelivery at 6954bc4d (worktree clean, nothing committed)
verdict: green
cases: executed 1064→1064, red 0
origin: introduced 0, pre-existing 0, undecided 0
wrote-outside-worktree: 11 paths under `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w6/evidence-redelivery/adversary-p1/` (listed in part 6)
needs-coordinator: no. The base binary named in the brief does not exist at that path; see part 4

**1. `git --no-pager diff --stat`:** empty. I added no case and made no commit. The branch head is still `6954bc4d`.

**2. Cases added:** none. Every attack below stayed green, so there was no failing case to write. `executed 1064→1064` is the implementor's count, unchanged; I did not run the suite.

**3. Suite run:** not run, because I added no case.

**Departure from the adversary procedure, done because the brief asks for it:** the five mutants below were applied to `crates/canon/src/eval/evidence.rs` and `crates/canon-cli/src/evaluate.rs` in the worktree while I held the lease, then restored with `git checkout`. `git status` was empty afterwards. The procedure says to mutate copies only. The brief says to run the mutants and restore from git.

**4. Findings:** none.

- **Base binary.** `$HOME/.cache/scratch/.../w6/simulate-child-outcome-step/canon-base` does not exist. The only `canon-base` in scratch (`.../simulate-child-outcome-step/adversary-p1/canon-base`) reports `canon 0.10.0`, so I did not use it. Instead I used `.../simulate-child-outcome-step/adversary-p1/canon-new` (`canon 0.9.0`, built from `2f53b351`). From `4c35a9e` to `2f53b351` only `crates/canon/src/simulate/{mod,tests}.rs` change, so compile, evaluate and conform match the base.
- **Order of refusals (the implementor's note).** This is the existing rule, not new behaviour.
  - The reference (`website/docs/reference/evaluation.md:87-101`) says inputs are checked "in this order, and the first problem found is the refusal … each evidence record in the order given".
  - The base already refused `[a, a'(bad kind)]` as `duplicate-identifier` and `[a'(bad kind), a]` as `undeclared-evidence-kind`; the branch gives the same.
  - Equal copies of a malformed record (bad `observed_at`, undeclared kind, undeclared upstream artifact, undeclared subject) are refused with the code the single record gets.
  - I found no set the branch refuses depending on order where the base's rule would not predict it. Where it differs from the base, the branch is less order-dependent: the base refused `[a, a, x_bad]` as `duplicate-identifier` and `[x_bad, a, a]` for x; the branch refuses both for x. Claims 5 and 6 hold.

**5. Attacked and not broken**

- **Claim 5, bytes.** All 168 evaluate steps of the 25 compiling scenarios (`imports.yaml` needs its imports supplied), run through base-equivalent and branch `canon evaluate`: 153 identical in exit code, stdout and stderr. The 15 that differ are all in `evidence-redelivery.yaml`: 5 equal copies are now decisions, 10 differing copies have the new message. IR is identical in 25 of 25.
- **Claim 1, equal copies.** On the branch, 681 runs over 107 steps with distinct ids. Each record got a copy placed last, placed first and placed right after itself, plus copies with the `upstream_revisions` keys reversed, placed last and first. Decision bytes were equal in 681 of 681, and the refusal code and message were equal in every step that refuses. This covers binding, freshness with `--at`, invalidation, order forms, children, missing_evidence, pending_decisions, supplied_by and authority/decisions scenarios.
- **"As written", CLI probes (19).**
  - `result: 1` or `1.0`, as JSON or YAML, and `producer` written as a map are refused as `malformed-input`. Every field is a String, so the two `serde_json` builds cannot disagree.
  - `""` and an absent field differ. `" 1"` and `"1"` differ.
  - `upstream_revisions: {}` equals the key left out.
  - `!!str 1` equals `"1"`. A YAML anchor equals the expanded value.
  - A key differing in case (`Result`) is refused by `deny_unknown_fields`. A repeated upstream key is refused by the YAML parser. A multi-document file is refused.
  - A YAML copy and a JSON copy of one record are equal.
- **Kept copy and directory order.** Files are read in sorted file-name order, as the `evaluate.rs` doc comment says. Equal copies differ at most in upstream key order, and that order is read nowhere in the output: invalidation indexes upstream entries in a BTreeMap and `computed_from.evidence` is sorted.
- **Other readers.**
  - `canon check` and `canon simulate` build ids `e{n}`, which are distinct. `conform` reads through `evaluate_with`.
  - CLI refusals still name the file of a dropped equal copy, which is claim 4.
  - decision-grounds (other branch) checks ground records through the applicable set, which is built from the distinct records.
- **Re-pins (6).** Each old expectation refused an equal copy. All six now use a differing copy, so each re-pin is the story's behaviour change. This includes the two files the story did not list.
- **Mutants (5).** Red lines are from `mutants.log`:
  - Ignore `child_case`: red, `a_copy_that_differs_in_any_field_is_refused` and step `differs-in-child-case`.
  - Compare upstream keys only: red, same unit test and step `differs-in-upstream-revision`.
  - Keep the last copy: red, `copies_equal_to_the_first_count_once_as_the_first` only; conform 26/0. No output can show which equal copy was kept.
  - CLI passes only the first record of each id: red, 3 targets, 4 tests (`evidence_file_named` ×2, `an_id_in_three_files…`, `an_evidence_refusal_after_reading_names_the_file`).
  - Ignore `format`: stays green everywhere. It cannot be caught because a format other than `canon-evidence/1` is refused before the id is compared.
- **Requirements and docs.** Each sentence of CANON-EVIDENCE-004 is pinned by a step. The `ess/domains/protocol.yaml` comment, both website pages and the `evaluate.rs` comment match the code. No stale copy of the old message remains outside history.

**6. Paths written outside the worktree** (`$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w6/evidence-redelivery/adversary-p1/`): `build1.log`, `probe.sh`, `diffrun.py`, `diffrun.log`, `mutate.py`, `mutants.sh`, `mutants.log`, `mutants.out`, `mutant-format.sh`, `mutant-format.log`. The probe and diff directories are deleted. `brief.md` was already there. Lease released; `worktree discard-cache` freed 1354 MiB (`target`).

```findings
[]
```

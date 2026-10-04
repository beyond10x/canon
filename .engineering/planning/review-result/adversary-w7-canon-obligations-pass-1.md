---
format: aep.planning-md/3
id: review-result:adversary-w7-canon-obligations-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w7 adversary, canon story:obligations, pass 1
relations:
- reviews: story:obligations
revision: 1
---
unit: canon/obligations, working tree at b38a76e plus the uncommitted phase 2, in ~/.local/state/worktree/trees/b10x/canon/canon-w7-obligations
verdict: NEEDS-CHANGE
cases: executed 257→263, red 3
origin: introduced 2 / pre-existing 1 / undecided 0
wrote-outside-worktree: 2 paths (a log and the build dir I was assigned)
needs-coordinator: yes. Every fix is outside the story's scope: the `eval/mod.rs` module doc, `canon-docs` `pages.rs` and the hand-written website pages.

The acceptance holds and the code matches its own doc comments. All three red cases are documentation that still says obligations are not evaluated, or says something false about the `obligations` key.

**1. Diff stat**
```
 crates/canon-cli/tests/adversary_skel_cli.rs |  41 +++++++---   (the coordinator's, not mine)
 crates/canon/src/eval/obligations.rs         | 113 +++++++++++++++++++++++++--  (the unit's, not mine)
?? crates/canon/tests/adversary_obligations_docs.rs   (mine, new test file)
?? crates/canon/tests/adversary_obligations_order.rs  (mine, new test file)
```
I changed no non-test path.

**2. Cases added** (each file was run on its own before the suite)
- `adversary_obligations_docs.rs`: **3 red.** Each case first checks that the section exists (or is absent), then fails on a document line:
  - `the_evaluation_reference_no_longer_says_every_section_is_absent`: `crates/canon/src/eval/mod.rs:88: //! nothing, each section is absent, …` and `website/docs/reference/evaluation.md:97`.
  - `no_page_still_says_obligations_are_not_evaluated`: `crates/canon-docs/src/pages.rs:1240` ("Only claims are evaluated today: obligations, actions …"), `website/docs/reference/investigation-example.md:250`, `website/docs/status/where-this-stands.md:22` ("| Obligations | Planned |"), `website/docs/concepts/evaluation.md:40`, `website/docs/concepts/protocols.md:55`.
  - `the_documents_reference_says_when_the_obligations_key_is_present`: `website/docs/reference/documents.md:67` says the key is present "always", but the base fixture's decision has no `obligations` key.
- `adversary_obligations_order.rs`: **3 green.** These are not findings, and I did not run them against a mutant:
  - code-point order across case, `é`, U+FF5E and U+1F600;
  - 2000 obligations, one entry each, with the right status;
  - an evidence match a caller puts into a discharge predicate stays `open` even when a record of that kind is supplied.

**3. Suite run** (after the cases existed): `cargo test --workspace --locked --no-fail-fast` gave EXIT=101, 260 passed and 3 failed, all 3 in `adversary_obligations_docs`. The full log is at the scratch path in part 6.

**4. Findings** (they cover the working tree above)

| # | file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|---|
| F1 | crates/canon/src/eval/mod.rs:87 | NEEDS-CHANGE | introduced | The module doc says "each section is absent" and that the decision is byte for byte the claim evaluation. For any protocol that declares an obligation, both are now false. | The published `reference/evaluation.md` is generated from this doc. |
| F2 | crates/canon-docs/src/pages.rs:1240 | NEEDS-CHANGE | introduced | The generator has the sentence "obligations … are not [evaluated]" written into its code. The status page still says Planned, and two concept pages say planned or not yet. | Public site readers. Canon's AGENTS.md says to "update the status page when a story lands". |
| F3 | crates/canon-docs/src/pages.rs:493 | CONFIRMED | pre-existing | For output documents, `(false, _) => "always"` ignores `field.optional`. So `documents.md:67` says `obligations` is always present, but it is absent when no obligation is declared. At base 4514e81 the same row existed and the key was never written. | Readers of `reference/documents.md`. |

**5. Attacked and not broken**
- **Ordering:** obligations follow the `BTreeMap` order, which is the same code-point order `json.rs` uses for `claims` (green case).
- **Exclusions:** the section reads claim values computed after the exclusion stages (mod.rs:179-180). The stages exclude nothing yet, so this cannot be driven.
- **Cycles:** `claims::values` refuses `claim-cycle` before `obligations::section` runs, and obligations cannot reference each other.
- **Evidence passed as `&[]`:** this matches the validator refusing evidence in a discharge predicate (`validate/mod.rs:228`). In an IR a caller builds, it stays `open` (green case).
- **FALSE and UNKNOWN collapsing:** design § 14 and `protocol-core.md` only show `status: open`. The claims section keeps the two values apart, as § 8 requires, and the story chose a two-state status.
- **Absent, not empty:** `adversary_skel_cli.rs` checks the base fixture's whole decision, which has no key. The controls in my F3 case confirm it.
- **Doc comments in `obligations.rs`:** they match the behaviour.

**6. Paths written outside the worktree**
- ~/.cache/ga-wave-2026-10-04-w7/canon-obligations/scratch/adversary1-suite.log
- ~/.cache/b10x-target/canon-w7-obligations (the assigned build dir)

I did not take a worktree session lease.

**7. Findings block**
```findings
- file: crates/canon/src/eval/mod.rs
  line: 87
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The evaluator module doc, published as reference/evaluation.md:97, still says every section is absent and the decision is the claim evaluation byte for byte, which is false for any protocol declaring an obligation."
- file: crates/canon-docs/src/pages.rs
  line: 1240
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The generated worked example, the status page (where-this-stands.md:22) and two concept pages (evaluation.md:40, protocols.md:55) still say obligations are not evaluated or are planned."
- file: crates/canon-docs/src/pages.rs
  line: 493
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "The documents reference marks every canon-decision/1 field present 'always', ignoring Optional, so the obligations row is wrong for a protocol that declares no obligation."
```

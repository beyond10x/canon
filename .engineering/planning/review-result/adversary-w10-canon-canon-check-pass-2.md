---
format: aep.planning-md/3
id: review-result:adversary-w10-canon-canon-check-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w10 adversary, canon story:canon-check, pass 2
relations:
- reviews: story:canon-check
revision: 1
---
```
unit: canon/canon-check, worktree ~/.local/state/worktree/trees/b10x/canon/canon-w10-canon-check at HEAD 26c5046 plus one untracked test file
verdict: NEEDS-CHANGE
cases: executed 446→455, red 3
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 9 paths, listed in part 6
needs-coordinator: yes. The story's acceptance sentence and decision 6 disagree on whether an outcome with an alternative path that needs no authority is a bypass (finding 2).
```

**1. Diff stat**

`git --no-pager diff --stat` is empty: no tracked file changed. `git status --short` shows one untracked file, `?? crates/canon/tests/adversary2_check.rs`, which is a test file. No implementation file was touched, nothing under `.engineering/` was written, and nothing was committed.

**2. Cases added** (all in `crates/canon/tests/adversary2_check.rs`)

| Case | Now | What it kills |
|---|---|---|
| `the_published_meaning_of_independent_of_matches_what_check_erases` | **red** | doc drift (finding 1) |
| `an_outcome_reachable_through_a_free_alternative_to_governed_evidence_is_reported` | **red** | acceptance gap (finding 2) |
| `a_bypass_is_not_hidden_by_erasing_every_governed_dimension_at_once` | **red** | erasing everything at once hides a bypass (finding 3) |
| `with_every_artifact_named_there_is_no_unbound_dimension` | green | M2 |
| `the_unbound_dimension_is_about_an_artifact_no_match_names_even_when_the_first_is_named` | green | M2 |
| `the_bound_refusal_names_bound_dimensions_then_the_unbound_one` | green | M2, M7 |
| `declaration_order_does_not_change_the_report` | green | M3, M13 |
| `a_claim_built_on_a_claim_about_another_artifact_erases_both_dimensions` | green | M3 |
| `a_bypass_on_evidence_about_one_artifact_names_only_that_record` | green | none of the mutants I ran |

Red output from running the file alone, before the suite (`scratch/adversary-p2/red.log`):
```
crates/canon/src/model/properties.rs says a property's claim stands for the evidence kinds it reads; check erases only the records the claim reads (kind and subject)
an outcome reachable without any authority-requiring action was not reported:
checked: protocol `p` revision 1: 12 states, 0 properties, 0 findings
  left: "checked: protocol `p` revision 1: 12 states, 0 properties, 0 findings\n"
 right: "authority-bypass: outcome `closed` is legitimate without authority in state {evidence `g` about `a`, evidence `g` about `b`}\nchecked: protocol `p` revision 1: 12 states, 0 properties, 1 finding\n"
test result: FAILED. 6 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

Mutants were run in a scratch copy with its own target dir; the full workspace was run with `--no-fail-fast`:
- **M2** (`space.rs:99`, `find(|a| !named.contains(a))` changed to `find(|_a| true)`): every pre-existing test stayed green. Only my file failed: 449 passed, 6 failed.
- **M7** (`space.rs:100`, the unbound dimension put first): every pre-existing test stayed green. Only my file failed: 451 passed, 4 failed.
- **M1, M3, M13**: the existing suite already caught each of them.

**3. Gate** (in the unit build dir, after the cases existed)

| Command | Exit | Summary line |
|---|---|---|
| `cargo fmt --all --check` | 0 | (no output). The first run exited 1 on my own file's formatting; I ran rustfmt on that file and reran. |
| `cargo clippy … -D warnings` | 0 | `Finished \`dev\` profile [unoptimized] target(s) in 0.08s` |
| `cargo test --workspace --locked` (`--no-fail-fast`) | 101 | 77 summary lines, 452 passed, 3 failed. The only failures are the three red cases above. `adversary2_check`: `test result: FAILED. 6 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s` |
| ess validate + synthesize | 0 | `canon v1 — 3 file(s), valid` / `0 scenario(s) (0 authored), 0 refusal(s), written to ~/.cache/b10x-target/canon-w10-canon-check/suite.json` |
| `canon-docs generate --check` | 0 | `canon-docs: 15 generated files are current` |
| `canon-cli conform run` | 0 | `conform: 8 passed, 0 failed, 0 unreadable` |

`cargo test -- --list` shows all 9 new test names in this tree.

**4. Findings** (they cover HEAD 26c5046 plus my test file; `crates/canon/src/check/` and `model/properties.rs` do not exist at base 7d39813, so every finding is introduced)

| # | file:line | verdict / origin | What was measured | What reaches it |
|---|---|---|---|---|
| 1 | crates/canon/src/model/properties.rs:76 | NEEDS-CHANGE / introduced | Three places say a property's claim stands for "the evidence kinds it reads": this rustdoc, `ess/domains/check.yaml:68`, and the generated `documents.md:159`. But `check` erases only the dimensions the claim reads, by kind and subject. Under the documented meaning, the property in the case would fail; `check` says it holds. | Any `canon-properties/1` file whose claim reads a match bound to a subject (subject-bound matches ship in this wave). Fix: change the rustdoc and the ESS comment to say "the evidence records it reads", then regenerate the docs. |
| 2 | crates/canon/src/check/mod.rs:468 | CONFIRMED / introduced | `released: any [approval (needs authority), self_check (free)]` gets no finding, although the outcome is reachable through `run_check` alone. The acceptance says "a variant whose outcome is reachable without any authority-requiring action reports the bypass". Decision 6 and body revision 12 exclude this case. | Any outcome with a path that needs no authority beside one that does. The coordinator must decide which wording wins. One option: in a state that needs no authority, test against the state with every dimension the requirement reads erased. That still keeps `is: unknown` from being reported as a bypass. |
| 3 | crates/canon/src/check/mod.rs:473 | INFEASIBLE / introduced | The code erases every governed dimension at once. In `{g about a, g about b}`, `closed` holds only because of the record about `a` (which `self_approve` can produce): removing just that record blocks it. Removing both records reaches `{}`, where `closed` holds because nothing is present, so the bypass is not reported. | No known protocol has this shape; I built it. Fix: count a state as a bypass when erasing any single governed dimension that is present makes the outcome not legitimate. This keeps decision 6's `withdrawn` case unreported. |
| 4 | crates/canon/src/check/space.rs:99 | CONFIRMED / introduced | Nothing in the existing suite checks that the unbound dimension is about an artifact no match names. Mutant M2 survives the whole workspace. | My three cases now kill it. |
| 5 | crates/canon/src/check/space.rs:100 | CONFIRMED / introduced | Mutant M7 (unbound dimension first) survives the whole workspace. The module docs (mod.rs:14–18) do not say where the unbound dimension sits relative to the bound ones. It is also written as `` evidence kind `k` 8 `` with no artifact, beside `` `k` about `a` ``, so it reads as if it were the whole kind. | Anyone reading a bound refusal or a witness state. The bound-refusal case now pins the order. |

**5. Attacked and could not break**
- **Choosing the unbound dimension:** the evaluation treats all artifacts the same (same revision; invalidation ignores evidence; no clock). The choice cannot change whether an outcome is reachable, in either direction.
- **When every declared artifact is named:** having no unbound dimension is correct, because evidence about an undeclared artifact is refused.
- **Erasure for bypasses, per dimension vs per kind:** the two give the same result, because an outcome only reads its own dimensions.
- **Bound-refusal count with subjects:** correct: 73728 states, exact text.
- **Determinism:** the IR uses BTreeMap and the code uses BTreeSet. Reordered artifacts, kinds, claims and `all` members give the same bytes.
- **The `reader` field and an obligation that reaches an unproduced kind only through a claim:** both match revision 12 (the existing test covers this).
- **Docs:**
  - `cli.md`, the status row and the `product.json` row for `canon check` are true to what it prints.
  - The exit table is unchanged from pass 1.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w10/canon-canon-check/scratch/adversary-p2/red.log`
- `…/scratch/adversary-p2/mutants.log`
- `…/scratch/adversary-p2/gate/{fmt,clippy,test,ess,docs,conform}.log`
- Created and then deleted: `…/adversary-p2/mutant/`, `…/adversary-p2/target-mutant/` (682M), `space.rs.orig`, and a throwaway runner `mutants.sh`.
- The worktree session lease `canon-check-adversary-p2` was acquired and released.

**7. Findings block**

```findings
- file: crates/canon/src/model/properties.rs
  line: 76
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The rustdoc, ess/domains/check.yaml:68 and the generated documents.md all say a property's claim stands for the evidence kinds it reads, but check erases only the subject-bound dimensions the claim reads, so a documented failing property is reported as holding."
- file: crates/canon/src/check/mod.rs
  line: 468
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "An outcome reachable through an alternative path that needs no authority, beside governed evidence, gets no authority-bypass finding, against the acceptance's 'reachable without any authority-requiring action reports the bypass'."
- file: crates/canon/src/check/mod.rs
  line: 473
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "Erasing every governed dimension at once hides a bypass when the outcome also holds with no evidence, though removing the one free-produced governed record alone blocks it."
- file: crates/canon/src/check/space.rs
  line: 99
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "Letting the unbound dimension be about an already-named artifact survives the whole existing workspace suite; adversary2_check.rs now kills it."
- file: crates/canon/src/check/space.rs
  line: 100
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Putting the unbound dimension before the bound ones survives the whole existing suite, and the module docs do not state its position or name its artifact; the bound-refusal case now pins the order."
```

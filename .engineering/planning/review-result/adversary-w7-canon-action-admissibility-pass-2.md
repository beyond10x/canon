---
format: aep.planning-md/3
id: review-result:adversary-w7-canon-action-admissibility-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w7 adversary, canon story:action-admissibility, pass 2
relations:
- reviews: story:action-admissibility
revision: 1
---
unit: canon/action-admissibility, working tree on 5196c59 plus the uncommitted phase-2 and pass-1 changes, in `~/.local/state/worktree/trees/b10x/canon/canon-w7-action-admissibility`
verdict: CONFIRMED (1 warning, 1 note; nothing blocks the unit)
cases: executed 264→274, red 1
origin: introduced 2 / pre-existing 0 / undecided 0
write-outside-worktree: 1 path (plus the assigned build dir)
needs-coordinator: none

**1. Diff stat**

`git --no-pager diff --stat` lists the same 9 tracked files that were modified when I started: the implementor's phase 2 plus the pass-1 edits. That includes the non-test paths `eval/actions.rs`, `eval/authority.rs`, `conform/mod.rs` and `website/…`. I edited none of them. My additions are two untracked test files:
- `crates/canon/tests/adversary2_admissibility_reasons.rs` (7 cases)
- `crates/canon/tests/adversary2_admissibility_authority.rs` (3 cases)

**2. Cases added** (`--list` shows all 10 in this tree)

| case | asserts | now |
|---|---|---|
| `a_precondition_with_no_evidence_match_is_not_reported_as_evidence_decided` | `{any: []}` and `{not: {all: []}}` must not get the evidence-only reason `{precondition: …}` | **red** |
| `a_claim_tested_twice_in_one_precondition_is_named_once` | `all [a, a is false]`: `a` named once, for each value of `a` | green |
| `a_claim_tested_with_several_is_values` | `any` over all three `is` values is admissible; `all [a is unknown, a]` names `a`; `not (a is unknown)` | green |
| `polarity_is_carried_through_nested_not_any_all` | `not any [not a, all [b, not c]]` names exactly a,b,c, or b,c when `a` is unknown | green |
| `a_precondition_nested_to_the_ir_depth_bound_names_its_claim` | 4095 `not`s and 2047 `all`s: right reason, no stack overflow, under 20 s | green |
| `an_unmet_precondition_blocks_with_only_its_own_reasons` | with denied, undecided or granted capabilities, the action stays `blocked` with only the precondition's reasons | green |
| `a_protocol_without_actions_renders_no_actions_key` | `actions` is `None` and absent from the rendered output, even when `--authority` is given | green |
| `every_spelling_of_one_grant_is_read` | comments, CRLF, JSON, `---`/`...` markers | green |
| `a_document_with_no_list_is_refused_as_malformed` | empty, whitespace, comment-only, BOM-only and `---` documents → `malformed-input` | green |
| `a_very_large_list_is_read_and_checked_entry_by_entry` | 50,001 entries read; a duplicate at the end is named; an earlier invalid identifier is refused first | green |

Red output from running the reasons file alone, before the suite (the assertion moved from line 242 to line 258 after `rustfmt`; it is still red there):
```
thread 'a_precondition_with_no_evidence_match_is_not_reported_as_evidence_decided' panicked at crates/canon/tests/adversary2_admissibility_reasons.rs:242:9:
never: no evidence match decides it, yet: {"never":{"reasons":[{"precondition":"false"}],"status":"blocked"},"never_either":{"reasons":[{"precondition":"false"}],"status":"blocked"}}
test result: FAILED. 6 passed; 1 failed
```
In my first draft of the authority file, a case that put a BOM in front of a block list was red at the library level ("did not find expected '-' indicator"). I dropped that case because it is not a defect: the CLI strips the BOM before the library sees the text (`canon-cli/src/main.rs:48-53` `read_text`, which `evaluate.rs:37` uses).

**3. Suite run** (after the cases existed): `cargo test --workspace --locked --no-fail-fast` gives EXIT=101, 273 passed and 1 failed. The only failure is the red case above. 274 executed = 264 reported green + my 10. `rustfmt --check` and `cargo clippy -p b10x-canon` on both new files are clean.

**4. Findings**

| # | file:line | verdict | origin | finding | what reaches it |
|---|---|---|---|---|---|
| F1 | `crates/canon/src/eval/actions.rs:116` (doc :12-13, :105-107) | CONFIRMED (warning) | introduced | The fallback fires whenever no claim is named, not only when evidence decides the precondition. So `{any: []}` and `not {all: []}` report `{"precondition":"false"}` and read as evidence-decided. Suggested fix: give such a precondition its own reason (outcomes uses `{"requirement":"unsatisfiable"}`), or make the doc say "no claim test decides it". | Both preconditions pass `model::parse` and `ir::compile`, so `canon compile` accepts them. Who writes an always-false precondition: nothing found. |
| F2 | `crates/canon/src/eval/actions.rs:172` | CONFIRMED (note) | introduced | The reason shape differs from outcomes. The `{claim, value}` shape matches. Three things differ: (a) actions drop unmet evidence matches whenever a claim is named, where outcomes add `{evidence: kind}`; (b) an unsatisfiable precondition gives `{precondition: "false"}` here and `{requirement: "unsatisfiable"}` there; (c) a false `all` names only its false members here (`all [a false, b unknown]` → `[a]`), while outcomes descends into every member that is not as wanted (`canon-w7-outcomes/crates/canon/src/eval/outcomes.rs:120-124`) → `[a, b]`. | The actions side of (c) is measured by the unit's own test at `actions.rs:252-258`. (a) and the outcomes side are read from code, not run. |

**5. Attacked and could not break**
- Deep nesting up to the IR depth bound: correct reasons in about 0.8 s.
- Duplicate claim tests, and one claim tested with several `is` values: each claim named once, with its own value.
- An unmet precondition always gives `blocked` with only precondition reasons, as the doc says ("Authority is not consulted"). The scenario only grants the capability here, so no earlier case would catch checking authority before the precondition; my case now does.
- Authority documents: comments, CRLF, JSON, document markers, empty input, 50k entries, refusal order. The BOM is handled by the CLI.
- `actions` is absent for a protocol with no actions, both from the section and the rendered decision.
- Doc comments in `authority.rs`: all match behaviour. In `actions.rs`, everything except F1 matches.
- I took no worktree session lease.

**6. Written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w7/canon-action-admissibility/scratch/adversary2-suite.log`
- Build output in the assigned `~/.cache/b10x-target/canon-w7-action-admissibility`

**7. Findings block**
```findings
- file: crates/canon/src/eval/actions.rs
  line: 116
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "The {precondition: value} fallback fires whenever no claim is named, so {any: []} and {not: {all: []}}, which contain no evidence match, are reported in the form the module docs reserve for evidence-decided preconditions."
- file: crates/canon/src/eval/actions.rs
  line: 172
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Action reasons diverge from the outcomes unit: evidence matches are dropped when a claim is named instead of given as {evidence: kind}, an unsatisfiable precondition is {precondition: false} not {requirement: unsatisfiable}, and a false all names only its false members where outcomes names every unmet member."
```

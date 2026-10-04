---
format: aep.planning-md/3
id: review-result:adversary-w2-canon-canon-ir-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w2 adversary, canon story:canon-ir, pass 1
relations:
- reviews: story:canon-ir
revision: 1
---
unit: canon/canon-ir, working tree at ~/.local/state/worktree/trees/b10x/canon/canon-w2-canon-ir (base dac4265 plus the uncommitted change)
verdict: CONFIRMED (1 red case; it fails at the base too, so it was there before this unit)
cases: executed 47→55, red 1
origin: introduced 5 / pre-existing 1 / undecided 0
wrote-outside-worktree: 3 directories plus loose scratch files (part 6), and one stray file that I wrote by mistake and deleted
needs-coordinator: route the red case (it lives in `model/`, which the implementor doesn't own) and decide the duplicate-member question

The § 30 promises mostly hold. The one red case is a file with a byte-order mark (BOM), which `canon` refuses as a parse error. Six mutants survive the unit's own suite; my new cases catch each of them.

**1. Files I touched** — `git diff --stat` shows only the implementor's two modified files (`main.rs` 56, `ir/mod.rs` 373, the same as at handover). My additions are untracked and both are test files:
- `crates/canon/tests/adversary_ir_canonical.rs`
- `crates/canon-cli/tests/adversary_compile_bom.rs`

I changed no implementation file.

**2. Cases added (each run alone before the suite)**

| Test | What it checks | Now | Fails against mutant |
|---|---|---|---|
| `each_truth_value_compiles_to_its_own_name` | `is: false/true/unknown` each come out under their own name | green | truth-swap, truth-false-is-true |
| `a_declared_obligation_compiles_with_its_description` | obligations appear with their descriptions | green | drop-obligations, obligation-no-description |
| `action_precondition_members_compile_in_one_order` | a reordered precondition gives the same bytes | green | precondition-unnormalized |
| `predicate_member_order_is_pinned_where_members_differ_only_in_result_or_truth` | exact order: result `null` before a string, then `false < true < unknown` | green | value-variant-order, truth-false-is-true |
| `declaration_keys_sort_by_code_point_including_astral_identifiers` | B, b, é, U+FF61, U+1F600 come out in code-point order whatever the source order | green | — |
| `line_endings_and_yaml_style_do_not_change_the_ir` | CRLF and flow style with an anchor/alias give the base bytes | green | — |
| `the_largest_revision_is_written_exactly` | `u64::MAX` is written exactly | green | — |
| `a_byte_order_mark_does_not_change_the_ir` (CLI) | the base fixture with a UTF-8 BOM gives the same output as without | **red** | — |

Red output of the BOM case, from its first run alone:
```
panicked at crates/canon-cli/tests/adversary_compile_bom.rs:32:5:
assertion `left == right` failed: exit Some(1)
  left: "error[parse]: missing field `protocol` at line 1 column 2\n"
 right: ""
```
At first the BOM check sat inside the style test, failed there, and I split it into its own CLI case. A binary built from base dac4265 gives the same result for `canon validate` on the BOM file: `error[parse]: missing field protocol`, exit 1.

**Mutant results** (`scratch/mutants.log`; mutated copy of the tree, never the tree itself):

| Mutant | Unit's suite | With my cases |
|---|---|---|
| truth-swap (`ir/mod.rs:308`) | GREEN | RED |
| truth-false-is-true | GREEN | RED |
| drop-obligations (`:113`) | GREEN | RED |
| obligation-no-description | GREEN | RED |
| precondition-unnormalized (`:134`) | GREEN | RED |
| value-variant-order (`json.rs:15`) | GREEN | RED |

The unit's own suite already catches the other 11 mutants: requires-unsorted, may-produce-unsorted, not-unnormalized, sort-reversed, nested-unnormalized, no-validate, default-precondition-any, escape-space, claim-unnormalized, outcome-unnormalized, result-dropped.

**3. Full suite (run after the cases existed)**
- Command: `cargo test --workspace --locked --no-fail-fast` → EXIT=101.
- Every binary passed except `adversary_compile_bom` (0 passed, 1 failed). 55 tests executed: 47 existing plus my 8.
- `cargo fmt --all --check` → 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` → 0.

**4. Findings**

| file:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|
| `crates/canon/src/ir/mod.rs:308` | CONFIRMED / introduced | Truth names are never checked: swapping `false` and `unknown`, or writing `false` as `true`, leaves the suite green | every claim test written with `is:` |
| `crates/canon/src/ir/mod.rs:113` | CONFIRMED / introduced | No test compiles a non-empty obligations section, so dropping it (or its descriptions) leaves the suite green | any protocol with obligations |
| `crates/canon/src/ir/mod.rs:134` | CONFIRMED / introduced | No precondition other than `{all: []}` is ever compiled, so skipping its normalization leaves the suite green | any action with a precondition |
| `crates/canon/src/ir/json.rs:15` | CONFIRMED / introduced | The canonical member order comes from the derived variant order of a private enum. Reordering the variants changes the bytes (and so the hash) of an unchanged protocol, and the suite stays green | evidence matches of the same kind with and without `result` |
| `crates/canon/src/model/parse.rs:25` | CONFIRMED / pre-existing | A file starting with a UTF-8 BOM is refused as a parse error by both `compile` and `validate` | editors that save with a BOM. The fix could be in `parse` or in the CLI's `read_protocol` (`main.rs:54`) |
| `crates/canon/src/ir/mod.rs:20` | CONFIRMED / introduced (semantic question) | The module doc says "equivalent documents give the same bytes", but `all/any` with a repeated member, and repeated `requires`/`may_produce`, compile differently from the version without the repeat (probe-dup-a vs probe-dup-b, diff exit 1). In three-valued logic `all[a,a]` means the same as `all[a]`. The story doesn't decide this, and the unit test at `mod.rs:346` pins that duplicates are kept. Either remove duplicates or narrow the doc claim | any repeated member |

**5. Attacked and could not break**
- **JSON validity:** a probe with `"`, `\`, NUL, DEL, U+2028, BOM, emoji, NEL and ESC in ids and descriptions passes `jq -e`. Its key order matches `jq -S`; the only difference is that `jq` escapes DEL, which this code writes raw as its doc says.
- **Path and working-directory leakage:** none in stdout.
- **Determinism:** no HashMap is used; integers print exactly.
- **Omitted defaults:** leaving out `is: true`, empty sections and `precondition` gives the same bytes as writing them.
- **Protocol id and revision (§ 37):** both are present.
- **Invalid documents:** refused with the validator's problems.

**6. Paths written outside the worktree**
- `~/.cache/b10x-target/canon-w2-canon-ir/adversary-mutant/` (195M, build dir for the mutants and the base build)
- `~/.cache/b10x-target/canon-w2-canon-ir/tmp/adversary-bom-protocol.yaml` (written by the BOM test)
- `~/.cache/ga-wave-2026-10-04-w2/canon-canon-ir/scratch/` containing: `mutant-tree/`, `base-tree/`, `mutants.sh`, `mutants.log`, 17 `mutant-*.out`, `adv1-own-case.log`, `adv1-canonical-alone.log`, `adv1-bom-alone.log`, `adv1-suite.log`, `probe-strings.{yaml,json}`, `probe-dup-a.{yaml,json}`, `probe-dup-b.{yaml,json}`
- Mistake: I created `~/.local/state/worktree/trees/b10x/canon-w2-canon-ir-placeholder` (a one-byte file) and deleted it right away.

I took and released a worktree session lease named `adversary-canon-ir-pass1`.

**7. Findings block**
```findings
- file: crates/canon/src/ir/mod.rs
  line: 308
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "no unit test distinguishes the rendered truth names, so swapping false/unknown or collapsing false to true stays green"
- file: crates/canon/src/ir/mod.rs
  line: 113
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "no unit test compiles a non-empty obligations section, so dropping it or its descriptions stays green"
- file: crates/canon/src/ir/mod.rs
  line: 134
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "precondition normalization is never exercised, so compiling it unnormalized stays green"
- file: crates/canon/src/ir/json.rs
  line: 15
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "canonical member order rests on the private Value variant order, and reordering it changes canon-ir/1 bytes with the suite green"
- file: crates/canon/src/model/parse.rs
  line: 25
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: "a protocol file starting with a UTF-8 byte-order mark is refused as a parse error by compile and validate"
- file: crates/canon/src/ir/mod.rs
  line: 20
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "duplicate all/any/requires/may_produce members compile to different bytes than the deduplicated equivalent, contradicting the module doc's equivalence claim; the story is silent"
```

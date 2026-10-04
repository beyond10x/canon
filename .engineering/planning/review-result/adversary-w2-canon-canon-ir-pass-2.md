---
format: aep.planning-md/3
id: review-result:adversary-w2-canon-canon-ir-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w2 adversary, canon story:canon-ir, pass 2
relations:
- reviews: story:canon-ir
revision: 1
---
unit: canon/canon-ir, working tree on base dac4265 (uncommitted), worktree ~/.local/state/worktree/trees/b10x/canon/canon-w2-canon-ir
verdict: CONFIRMED (4 mutants the suite misses; no defect in the implementation)
cases: executed 61→76, red 0
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 21 paths (19 scratch entries, 1 mutant build dir, 1 test tmp area), listed in part 6
needs-coordinator: none

**1. Diff stat**
```
 crates/canon-cli/src/main.rs |  58 +++--
 crates/canon/src/ir/mod.rs   | 539 ++++++-
?? crates/canon-cli/tests/adversary2_compile.rs     (mine)
?? crates/canon/tests/adversary2_ir_order.rs        (mine)
?? crates/canon/tests/adversary2_ir_text.rs         (mine)
```
The two tracked files are the implementor's. The only files I wrote are the three new test files. I did not edit any implementation file.

**2. Cases added.** All are green on the real tree. Each file was run alone before the suite (logs `adv2-{order,text,cli}-alone.log`).

| File | What it asserts | Now |
|---|---|---|
| `crates/canon/tests/adversary2_ir_order.rs` (8) | Compiles 4000 generated predicates and checks each against an order written only from the doc comment (sorted, and only `==` members removed). Scrambling or repeating members gives the same bytes. 4000 pairs: different members are never merged, and either input order gives one result. 1500 triples: all 6 input orders give the same bytes. Pinned cases: two different `any`s; lists whose first members differ in the "greater" direction; deduplication against `not` and against matches differing only in result (none, `""`, `x`); the 4 non-normalizations the doc lists stay different; an id containing `"` or `\` is escaped the same as a key and as a value | green |
| `crates/canon/tests/adversary2_ir_text.rs` (2) | Literal, folded, plain and double-quoted multi-line text gives the same IR with LF, CRLF or bare-CR line endings. The base fixture (read at run time) gives the same IR with no final newline, with extra blank lines, and with CR endings | green |
| `crates/canon-cli/tests/adversary2_compile.rs` (5) | A parse error, an empty file, a directory, non-UTF-8 bytes and a wrong `format` each leave stdout empty and exit non-zero (1 or 2 as stated) | green |

Mutant runs, done on a copy in `scratch/adv2-mutant-tree`. "Existing suite" means the workspace without my three files.

| Mutant | Existing suite | My cases | Red output (abridged) |
|---|---|---|---|
| remove the `(Any, Any)` arm, `ir/mod.rs:230` | GREEN | RED | `left: All([Any([b])])` vs `right: All([Any([a]), Any([b])])` (the second `any` is dropped) |
| `if order == Ordering::Less`, `ir/mod.rs:221` | GREEN | RED | `left: Any([All([a])])` vs `right: Any([All([a]), All([b])])` |
| keys written unescaped, `ir/json.rs:71` | GREEN | RED | `claims key q\"t not escaped` |
| `text` sorts shorter strings first, `ir/mod.rs:216` | GREEN | RED | case 17: `😀` placed before `q"t` |
| remove the `(Not, Not)` arm | RED | RED | already caught |

**3. Suite run** (after the cases existed). Command: `cargo test --workspace --locked` with `CARGO_TARGET_DIR=~/.cache/b10x-target/canon-w2-canon-ir`.
- `test EXIT=0`: 76 passed, 0 failed.
- Without my 3 files (15 cases) the suite runs 61.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: EXIT=0.
- `cargo fmt --all --check`: EXIT=0. I ran `rustfmt` on my own 3 files only.

**4. Findings**

| file:line | Verdict | Origin | What was measured / what reaches it |
|---|---|---|---|
| `crates/canon/src/ir/mod.rs:230` | CONFIRMED | introduced | No existing case compares two different `any`s. If the arm were lost, members would be merged silently. Reached by every `all`/`any` holding two `any`s. My cases now cover it. |
| `crates/canon/src/ir/mod.rs:221` | CONFIRMED | introduced | No existing case compares lists whose first pair differs in the "greater" direction. My cases now cover it. |
| `crates/canon/src/ir/json.rs:71` | CONFIRMED | introduced | Key escaping is never exercised. The validator accepts ids containing `"` and `\`. My cases now cover it. |
| `crates/canon/src/ir/mod.rs:216` | CONFIRMED | introduced | Code-point order on ids of different lengths is never compared. My cases now cover it. |
| `crates/canon/tests/adversary_ir_canonical.rs:119` | CONFIRMED | introduced (pass 1) | Uses `include_str!`, which breaks the "read source paths at run time" rule: a test binary reused from a shared build dir would check a stale fixture. `adversary2_ir_text.rs` reads it at run time. |

**5. What I attacked and could not break**
- `canonical_order` is a total order and agrees with equality (8000+ generated cases). Deduplication removes only equal members, and never merges `not x` with `x` or matches that differ only in result.
- Ids are escaped and sorted the same way as keys and as values. Code-point order is used throughout: the `BTreeMap<String>` keys, the `Ord` on `requires` and `may_produce`, and `chars().cmp` in predicates.
- The IR is hash-stable across CRLF, CR, no final newline, extra blank lines and multi-line scalar styles.
- `canon compile` on invalid input or an unreadable path exits non-zero with nothing on stdout.
- The doc's list of what is not normalized matches behaviour.
- The protocol id and revision are carried, pinned exactly by `the_ir_names_its_format_and_the_protocol_id_and_revision`. Design § 37's "which Canon semantics version" is covered only by `format: canon-ir/1`.
- Design § 29 proposes `--out` and a directory `--path`. Neither exists: a directory exits 2. The story asks only for printing, so I did not raise it as a finding.
- A UTF-16 file is refused with `error[unreadable]` and exit 2. The BOM comment's YAML 1.2 § 5.2 citation also covers UTF-16, but the refusal is honest and nothing reaches it, so I did not raise it as a finding.

**6. Paths written outside the worktree**
- 19 entries in `~/.cache/ga-wave-2026-10-04-w2/canon-canon-ir/scratch/`:
  - `adv2-mutants.sh` and `adv2-mutants.log`
  - `adv2-mutant-tree/`
  - 8 mutant outputs: `adv2-mutant-{any-any-arm-dropped,members-return-only-on-less,key-unescaped,text-shorter-first}-{existing,mine}.out`
  - 2 more mutant outputs: `adv2-mutant-not-not-arm-dropped-{existing,mine}.out`
  - logs: `adv2-order-alone.log`, `adv2-text-alone.log`, `adv2-cli-alone.log`, `adv2-fmt.log`, `adv2-clippy.log`, `adv2-suite.log`
- `~/.cache/b10x-target/canon-w2-canon-ir/adversary2-mutant` (287M).
- The test files under `CARGO_TARGET_TMPDIR` (`~/.cache/b10x-target/canon-w2-canon-ir/tmp/adversary2-compile-*`).
- My session lease is released.

**7. Findings block**
```findings
- file: crates/canon/src/ir/mod.rs
  line: 230
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "dropping the (Any, Any) arm merges different any members and the unit's suite stays green; adversary2_ir_order.rs now kills it"
- file: crates/canon/src/ir/mod.rs
  line: 221
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "returning early only on Less makes lists whose first members differ upward compare equal and get deduplicated, unseen by the unit's suite; adversary2_ir_order.rs now kills it"
- file: crates/canon/src/ir/json.rs
  line: 71
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "writing object keys unescaped survives the unit's suite although the validator admits ids containing a quote or backslash; adversary2_ir_order.rs now kills it"
- file: crates/canon/src/ir/mod.rs
  line: 216
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "a length-first text order survives the unit's suite because no case compares predicate ids of different lengths; adversary2_ir_order.rs now kills it"
- file: crates/canon/tests/adversary_ir_canonical.rs
  line: 119
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the pass-1 case embeds the base fixture with include_str!, so a binary reused from a shared build dir checks a stale fixture"
```

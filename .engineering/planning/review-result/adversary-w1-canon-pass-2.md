---
format: aep.planning-md/3
id: review-result:adversary-w1-canon-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w1 adversary, canon unit, pass 2
relations:
- reviews: story:protocol-source-model
revision: 1
---
unit: canon — uncommitted working tree on base `7840f412`, ``
verdict: NEEDS-CHANGE
cases: executed 24→30, red 5
origin: introduced 7 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 (1 deleted)
needs-coordinator: none

**1. What I touched.** `git diff --stat 7840f412` is the same as in pass 1: `Cargo.lock`, `crates/canon/Cargo.toml` and `crates/canon/src/lib.rs`, all changed by the implementor. I added two untracked test files and changed no implementation file:
- `crates/canon/tests/adversary2_source_model.rs`
- `crates/canon-cli/tests/adversary2_validate.rs`

**2. Cases added.** Each was run alone before the suite.

| case | asserts | now |
|---|---|---|
| `an_explicit_null_identifier_is_not_read_as_the_text_of_the_null` | `id: ~` / `id: null` and `capability: ~` / `capability: null` are refused | red |
| `a_declaration_written_with_no_value_is_refused` | `a:` / `a: ~` written under each declaration section is refused | red |
| `unresolved_references_come_in_the_order_their_declarations_are_written` | problems follow source order, as `validate/mod.rs:6` says | red |
| `rendered_identifiers_carry_no_invisible_or_reordering_characters` | no U+200B, U+202E or similar characters appear raw in a rendered problem | red |
| `the_valid_line_carries_no_reordering_characters` | the CLI `valid:` line has no raw U+202E | red |
| `validate_exits_2_on_a_path_it_cannot_read` | a missing `--path` exits 2 with `error[unreadable]:` | green; catches a mutant |

Red output, verbatim:
```
left: ["protocol id = `~`", "action capability = `~`", "protocol id = `null`", "action capability = `null`"]  right: []
left: ["actions: /   a:", "obligations: /   o:"]  right: []
left: ["missing_three", "missing_two", "missing_one"]  right: ["missing_one", "missing_two", "missing_three"]
rendered raw: "action `a` references evidence kind `note\u{200b}`, which is not declared" displays as: action `a` references evidence kind `note​`, which is not declared
exit Some(0), stdout "valid: protocol `p\u{202e}lanif.noitagitsevni` revision 1\n"
```
`a: ~`, `x: null` and `k: ~` are refused at parse. Only the empty form `a:` gets through.

Mutant proof: in a scratch copy I changed `main.rs:50` (`return ExitCode::from(UNREADABLE)`) to `ExitCode::SUCCESS`. The 6 existing CLI cases still passed. The new case failed with `left: Some(0) right: Some(2)`.

**3. Suite** (run after all the cases existed; `CARGO_TARGET_DIR=…/canon-w1`)
- `cargo fmt -p b10x-canon -p canon-cli --check`: exit 0
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0
- `cargo test --workspace --locked --no-fail-fast`: exit 101. 30 cases ran: 25 passed, 5 failed (my 5 red cases). The 24 earlier cases (source_model 17, adversary_source_model 1, validate 5, adversary_validate 1) all pass. The "before" count of 24 is this same run with my two files left out.

**4. Findings** (all on the tree above)

| file:line | verdict | what reaches it |
|---|---|---|
| `crates/canon/src/model/ids.rs:14` | NEEDS-CHANGE, introduced | An author writes `id: ~` or `capability: null`. `canon validate` then exits 0 for protocol `` `~` ``. No fixture does this. |
| `crates/canon/src/model/mod.rs:193` | CONFIRMED, introduced | `actions:\n  deploy:` is accepted as an action with no precondition, capability or effect. `present.rs:1` says a key written with no value is an error. |
| `crates/canon/src/validate/mod.rs:6` | CONFIRMED, introduced | Any document whose sections are not in claims → actions → outcomes order. The output is still deterministic, so rewording the doc is enough. |
| `crates/canon/src/model/mod.rs:210` | CONFIRMED, introduced | The grammar admits Unicode format characters (category Cf), and `one_line` prints them raw. A message says `note` is undeclared when `note` is declared. |
| `crates/canon-cli/src/main.rs:62` | CONFIRMED, introduced | The `valid:` line does not use `one_line`. A bidi override reverses the protocol id it displays. |
| `crates/canon-cli/src/main.rs:50` | CONFIRMED, introduced | A mistyped `--path`. No existing case covers this branch; the new case does. |
| `crates/canon/tests/source_model.rs:544` | CONFIRMED, introduced | The guard looks only for `serde(default`. A new bare `Option<T>` field still turns null into None and would not be caught. No failing case is possible without changing an implementation file. |

Fixes I did not apply:
- Identifiers: refuse null for required identifiers too, for example a `present::required`-style deserializer on the id newtypes.
- Declarations: decide whether an empty declaration is allowed; then either document it or send declaration values through `present`.
- Format characters: reject category Cf in `is_identifier`. That also fixes the CLI line.

**5. Attacked and held**
- The pass-1 `is:` / `is: ~` / `is: null` case is now green.
- `a: ~`, `x: null` and `k: ~` declarations are refused at parse.
- Control characters and non-space whitespace (including U+2028) are escaped, as the existing tests show.
- The reworded cycle comment matches the depth-first search: every group of claims that cycle produces at least one report. I checked this by reasoning only; I wrote no case.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w1/canon/scratch/adv2-mutant/` (136M including its own `target/`): deleted.
- `~/.cache/b10x-target/canon-w1/tmp/adversary2-bidi-id.yaml`: written by my test each time it runs; left in place.

The `canon-w1` build dir now also holds my two test binaries. Free space on `/` is 11G.

```findings
[
  {"file": "crates/canon/src/model/ids.rs", "line": 14, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "An explicit null in a required identifier (`id: ~`, `capability: null`) deserializes as the literal text `~` or `null`, which passes the new grammar, so `canon validate` accepts a protocol named `~` while `id:` and `effect: ~` are refused."},
  {"file": "crates/canon/src/model/mod.rs", "line": 193, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "A declaration written with an empty value (`actions:\\n  a:`) is accepted as an all-default entry while `a: ~` is refused, contrary to present.rs's rule that a key written with no value is an error."},
  {"file": "crates/canon/src/validate/mod.rs", "line": 6, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The doc says unresolved references are reported in the order the referring declarations appear, but they are reported claims, then actions, then outcomes, whatever the source order."},
  {"file": "crates/canon/src/model/mod.rs", "line": 210, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Unicode format characters such as U+200B and U+202E pass is_identifier and are rendered raw by one_line, so a problem can read `evidence kind note ... is not declared` for a document that declares `note`."},
  {"file": "crates/canon-cli/src/main.rs", "line": 62, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The `valid:` line prints the protocol id without one_line, so a bidi override in a valid id reverses what the terminal displays."},
  {"file": "crates/canon-cli/src/main.rs", "line": 50, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "No existing case drives the unreadable branch, so a mutant exiting 0 there kept the suite green; adversary2_validate.rs now catches it."},
  {"file": "crates/canon/tests/source_model.rs", "line": 544, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The explicit-null guard only matches `serde(default`, so a new bare `Option<T>` field, which serde also defaults on null, passes it."}
]
```

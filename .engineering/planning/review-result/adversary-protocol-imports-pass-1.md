---
format: aep.planning-md/3
id: review-result:adversary-protocol-imports-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:protocol-imports (wave 2026-11-20-w2): red, 1 finding, introduced'
relations:
- reviews: story:protocol-imports
revision: 1
---
Adversary pass 1 on story:protocol-imports, wave 2026-11-20-w2, `aep:adversary` on Opus. The
report below is as the agent returned it.

unit: story:protocol-imports, branch impl/protocol-imports, findings cover ef3d8b1 (my cases committed as 1bb572e)
verdict: red
cases: executed 917→921, red 4
origin: introduced 1, pre-existing 0, undecided 0
wrote-outside-worktree: 12 paths under $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w2/protocol-imports/adversary-p1/ (listed in part 6)
needs-coordinator: no

**1. Diff stat** (ef3d8b1..1bb572e). Only a test file changed:
```
 crates/canon/tests/adversary_protocol_imports.rs | 115 +++++++++++++++++++++++
 1 file changed, 115 insertions(+)
```
I made no source mutation. I did not create `adversary_protocol_imports_cli.rs` or any fixture.

**2. Cases added** in `crates/canon/tests/adversary_protocol_imports.rs`. All 4 are red now. Each case builds two parts, `a` and `b`, that declare one identifier written two ways. It first asserts that the two ways compile to the same `canon-ir/1`. It then asserts that a root importing both parts composes to the IR of the document that declares the identifier once.
- `claims_whose_all_members_are_written_in_another_order_are_one_declaration`
- `an_action_without_a_precondition_and_one_with_the_empty_all_are_one_declaration`
- `actions_whose_may_produce_is_written_in_another_order_are_one_declaration`
- `case_inputs_whose_values_are_written_in_another_order_are_one_declaration`

Red output, from running that test target alone (`cargo test -p b10x-canon --locked --test adversary_protocol_imports`, exit 101). One failure shown; the other three have the same shape:
```
panicked at crates/canon/tests/adversary_protocol_imports.rs:69:25:
two declarations that compile to the same canon-ir/1 are refused:
conflicting-imports: protocol `root` imports input `sev` from `a` and from `b`, declared differently
```
The other three are refused the same way, naming `action act` (twice) and `claim c`.

**3. Suite run**, after the cases existed: `export PATH=…/ess/toolchains/0.77.0:$PATH; cargo test --workspace --locked --no-fail-fast`, exit 101. Result: 917 passed, 4 failed (exactly my 4), 3 ignored. `executed <before>` is the implementor's 917.
- My first suite run left out the ess 0.77.0 PATH: 92 ESS tests failed on the toolchain version alone. I discarded that run (`suite.log`).
- Clippy with `-D warnings` on the new test target is clean, and rustfmt on the file is clean.

**4. Findings**

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/ir/imports.rs:386 | acceptance | warning | CONFIRMED | introduced | `canon compile`, `validate` or `check` with two `--import` parts that declare one identifier differently in source but identically in IR; also a `canon conform run` scenario with `imports` |

- **The defect:** `Merger::section` compares parsed source declarations with `*first != value`. `canon-ir/1` treats documents that differ only in its normalised ways as the same bytes (`ir/mod.rs:5-53`). Story § Outcome 4 says "two identical declarations are one declaration". The composition refuses as `conflicting-imports` what compiles to identical IR.
- **Other forms the CLI probe showed refused the same way:** `requires` in another order, and a repeated `may_produce` entry.
- **Fix, named, not applied:** compare the two declarations' IR form, for example the canonical JSON of the per-section IR entry, instead of the model value.
- **Not on the base:** `ir/imports.rs` does not exist at `15cfece`.

**5. Attacked and not broken**
- **Hand-written SHA-256:** checked against `sha256sum` for IR lengths covering every residue mod 64 (201 lengths), non-ASCII text and a 3 MB input. No mismatch.
- **Pin:** it equals `canon compile <part> --import … | sha256sum`, trailing newline included. The pin of an import that has its own imports is the digest of its composed IR. Uppercase, short and unprefixed digests are refused. A negative or huge revision does not parse.
- **Cycles:** lengths 1, 2 (not through the root) and 3 are each printed and caught before the pin check.
- **Diamond:** a shared part is inlined once. An import's import that is not passed gives `missing-import` naming the importer. The same file passed twice gives `duplicate-import-document`.
- **Refinement:** refining all 9 sections at once replaces every declaration, `case_open` included. A duplicate `refines` entry is accepted. Two imports that conflict, with the root redeclaring the identifier and listing it under both imports' `refines`, is accepted. With only one import listing it, `unrefined-redeclaration` names the other import.
- **Cross-section reuse:** the same identifier in different sections is legal in one document, so composing section by section matches the flattened document.
- **CLI:** a missing file exits 2, a directory exits 2, a non-`protocol/1` file exits 1 naming its path. `generate` exits 1 and writes nothing.
- **Scenarios:** an import symlinked outside the working directory is reported unreadable. `imports: []` composes with no imports. `read_ir` already requires byte identity.
- **Untested path (note, no case added):** I found no test covering the documented exception that a root redeclaration suppresses `conflicting-imports` (`imports.rs:386`, `!own.contains(id)`). I did not run the mutation, because the adversary procedure forbids touching files under attack. The behaviour itself is correct by CLI probe.

**6. Paths written outside the worktree**, all under `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w2/protocol-imports/adversary-p1/`:
- `msg-1.txt`, `case1.log`, `suite.log`, `suite2.log`
- `probe/sha.sh`, `probe/norm.sh`, `probe/refine.sh`, `probe/symlink.sh`, `probe/resolve.sh`, `probe/cross.yaml`
- `probe/sha/`, `probe/norm/`, `probe/refine/`, `probe/symlink/`, `probe/resolve/` (generated probe documents)

```findings
[{"file": "crates/canon/src/ir/imports.rs", "line": 386, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "compose compares parsed source declarations with != instead of their canon-ir/1 form, so two imports whose declaration compiles to identical IR (all members, may_produce or input values reordered, absent precondition versus all of nothing) are refused as conflicting-imports, against Outcome 4 that identical declarations are one"}]
```

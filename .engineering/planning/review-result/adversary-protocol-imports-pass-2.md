---
format: aep.planning-md/3
id: review-result:adversary-protocol-imports-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 on story:protocol-imports (wave 2026-11-20-w2): green, no finding'
relations:
- reviews: story:protocol-imports
revision: 1
---
Adversary pass 2 on story:protocol-imports, wave 2026-11-20-w2, `aep:adversary` on Opus, on the
correction `1bb572e..17f52ff`. The report below is as the agent returned it.

unit: story:protocol-imports, pass 2 on the correction 1bb572e..17f52ff (c6ba21b, 17f52ff), findings cover 17f52ff
verdict: green
cases: executed 922→922, red 0
origin: introduced 0, pre-existing 0, undecided 0
wrote-outside-worktree: 5 paths under $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w2/protocol-imports/adversary-p2/
needs-coordinator: no

I couldn't break the fix. Imports now conflict only when their compiled `canon-ir/1` entries differ, which closes pass 1's finding. I added no case and made no commit; the tree is unchanged at 17f52ff.

**Findings:** none.

**Attacked and not broken** (all through the current `canon` binary):
- **Different IR is still refused:** one declaration changed in each of the 9 sections gives `conflicting-imports` 9 of 9 times. So do these 5 single changes:
  - an action's `case_open`
  - a capability scope value
  - an action's `arguments` schema
  - an obligation's `applicable_when` absent versus `{all: []}` (the IR keeps these apart, as its docs say)
  - three imports where only the middle one differs
- **Same entry, different meaning underneath:** claim `c` is identical in both imports, but the evidence kind `d` it reads differs in `max_age`. `d` itself is refused, naming both imports, so neither side wins silently. Every one of the 9 entry paths is the key the IR really writes, so no lookup comes back empty on both sides and passes as equal.
- **Order:** two imports with the same IR but different source (`may_produce` reordered and repeated, an explicit empty precondition). The pin of an importer built on them is the same bytes in 3 orders: both `--import` orders, and with the file names swapped so the path sort flips.
- **An import that doesn't compile on its own** (the root declares what it is missing): refused as `invalid-import` before any comparison. No fallback, no panic.
- **The `refines` exception:** the root redeclares and only one import lists it under `refines`, so `unrefined-redeclaration` names the other import. The new test `a_refined_redeclaration_settles_two_imports_that_conflict` would fail if the exception were dropped, because it asserts empty stderr and exit 0.
- **Cost:** 30 imports sharing 200 evidence kinds and 200 claims compose and compile in 0.34 s with the debug build, and the 200 claims come out once.
- **Pass 1 cases:** all 4 are now green.

**Suite** (`export PATH=$HOME/.cache/ess/toolchains/0.77.0:$PATH; cargo test --workspace --locked --no-fail-fast`, exit 0): 922 passed, 0 failed, 3 ignored.

**Written outside the worktree:** `suite.log`, `probe/sections.sh`, `probe/more.sh` and `probe/dangling.sh`, plus the probe documents in `probe/sections/`, `probe/more/` and `probe/dangling/`.

```findings
[]
```

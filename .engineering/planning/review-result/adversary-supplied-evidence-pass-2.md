---
format: aep.planning-md/3
id: review-result:adversary-supplied-evidence-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 on story:supplied-evidence (wave 2026-11-20-w3): green, 1 note, introduced'
relations:
- reviews: story:supplied-evidence
revision: 1
---
Adversary pass 2 on story:supplied-evidence, wave 2026-11-20-w3, `aep:adversary` on Opus, on the
correction `2c212ec..4bd9c45`. The report below is as the agent returned it.

unit: story:supplied-evidence pass 2, branch impl/supplied-evidence at 4bd9c45 (+ adversary commit 201ced1)
verdict: green
cases: executed 980→986, red 0
origin: introduced 1, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w3/supplied-evidence/adversary-p2/ (alone.log, suite.log, msg-1.txt)
needs-coordinator: yes. Applying `check-help.patch` alone turns `task docs-check` red (finding 1).

The correction holds. My pass 1 case is rewritten to the decided rule and is green, and the full suite has 0 failed.

`git diff --stat 4bd9c45 HEAD`: `crates/canon/tests/adversary_supplied_evidence.rs | 180 +++++++++++++++++++---`. Only my test file changed.

**Cases in `crates/canon/tests/adversary_supplied_evidence.rs`** (all green on 4bd9c45; first run on their own: `test result: ok. 9 passed; 0 failed`)
- `a_supplied_kind_an_authority_action_produces_is_a_bypass` replaces the pass 1 red case:
  - with `supplied_by`, the bypass is reported, witness `{evidence `s`}`;
  - without it, 0 findings.
- New: the same kind also produced by a capability-free action gives one bypass, same witness, 6 states, with or without `supplied_by`.
- New: a supplied kind read only through a claim two claims down is still reported.
- New: the witness when a second governed kind sits beside the supplied one:
  - it is `{evidence `k`, evidence `s`}` both over the slice (24 states in the whole product) and over the whole product (12 states);
  - it is the state of least weight, ties broken by rendering.
- New: a second governed kind that only authority produces:
  - an outcome that needs both kinds (`all`) has 0 findings;
  - one that takes either (`any`) is a bypass on `{evidence `s`}`.
- New `canon diff` cases:
  - supplier A to B is `BREAKING`, A to A is no change, added is `EXPANSION`, removed is `BREAKING`;
  - a supplier change beside a maximum-age change on the same kind gives two lines, the maximum age first;
  - two kinds come in identifier order whatever order they are written in, and the output is the same on a repeat run.
- Kept from pass 1: the undeclared-subject probe and the import self-supply probe.

**Findings**

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w3/supplied-evidence/check-help.patch | contract-drift | note | CONFIRMED | introduced | `website/docs/reference/cli.md` is generated from this help text. The `ci` task runs `docs-check` (`canon-docs generate --check`), so applying the patch without `slot task docs-generate` fails it. |

**Attacked and not broken**
- The decided rule: 5 cases, listed above.
  - The unit's own test `a_supplied_kind_an_authority_action_produces_stays_governed` covers the capability-free producer only on the protocol *without* `supplied_by`. My case covers it with `supplied_by` too.
- The `canon diff` change: 2 cases, 6 transitions.
- `CANON-SUPPLY-001`: its 3 MUST sentences are each pinned by a step, checked by reading:
  - "carry that name" by step `compile`, which pins the exact IR bytes;
  - "evaluated to the decision without those entries" by `copied-record-names-an-undeclared-upstream` (same decision as `record-supplied`), backed by `copied-record-invalidated-on-a-declared-upstream`;
  - "refused as undeclared-artifact" by `unsupplied-kind-names-an-undeclared-upstream`. That step uses a different record (an `observation`), not literally "the same record", but it catches both mutants I tried: the exception applied to every kind, and the exception applied whenever the protocol has any supplied kind.
- `check-help.patch`: its 2 changed sentences are true of the code. It applies cleanly (`git apply --check`).
- Everything green in pass 1 stays green, as the summaries below show.

**Suite**: `export PATH=$HOME/.cache/ess/toolchains/0.77.0:$PATH; cargo test --workspace --no-fail-fast`, exit 0.
- Totals over 138 summaries: 986 passed, 0 failed, 3 ignored. No `test result: FAILED` line.
- My file: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`
- Conform: `conform: 21 passed, 0 failed, 0 unreadable` / `requirements: 22 implemented, 22 covered, 0 planned`

```findings
[{"file": "$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w3/supplied-evidence/check-help.patch", "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The generated website/docs/reference/cli.md mirrors this help text, so applying the patch without running slot task docs-generate turns task docs-check red."}]
```

---
format: aep.planning-md/3
id: story:semantic-diff
kind: story
status: proposed
title: Classify protocol changes with a semantic diff
refs:
- provider: taskboard
  reference: C-011
relations:
- decomposes: epic:canon-kernel
- depends_on: story:canon-ir
- depends_on: story:evidence-freshness
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:evaluator-skeleton
- depends_on: story:invalidation-rules
scope:
- confidence: cited
  path: crates/canon-cli/src/diff.rs
- confidence: cited
  path: crates/canon/src/diff/
- confidence: cited
  path: fixtures/investigation/semantic-diff/
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:51Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

`canon diff --from --to` (clap derive) compares two protocol revisions on their compiled
`canon-ir/1` and classifies every change as `TIGHTENING`, `RELAXATION`, `BREAKING`, `EXPANSION` or
`NO SEMANTIC CHANGE` (design § 31), so nobody has to infer what a protocol change does to legitimate
work from a raw source diff. Output is deterministic and lists each change with its category and
the identifiers it touches.

## Order (re-plan 2026-10-04)

- Removed: depends_on story:conformance-suite. It existed only because both stories changed
  `crates/canon-cli`; story:evaluator-skeleton splits the CLI so this story owns
  `crates/canon-cli/src/diff.rs` (where the skeleton declares `canon diff --from --to`) and
  story:conformance-suite owns `crates/canon-cli/src/conform.rs`. The two run in one wave.
- Kept: depends_on story:canon-ir (it compares compiled `canon-ir/1`), story:evidence-freshness
  (one classified change is a shortened maximum age).
- Added: depends_on story:evaluator-skeleton (the `diff` subcommand file), and
  story:invalidation-rules, the last of the stories that change `canon-ir/1`'s types
  (story:evidence-freshness, story:decision-outcomes, story:invalidation-rules). A classifier
  merged beside one of them would be written against an IR that changes under it; the edge is on
  that type, not on a shared file.

## Scope

- Out: changes to constructs the kernel does not yet have (independence, protocol imports, upstream
  invalidation) and live-case impact, which needs case data Canon does not hold (design § 38).
- Fixtures: the revision pairs live in `fixtures/investigation/semantic-diff/`, a directory rather
  than one variant file, because each pair is two protocol documents. The base is not edited.
- Surfaces: `crates/canon/src/diff/`, `crates/canon-cli/src/diff.rs`,
  `fixtures/investigation/semantic-diff/`.

## ESS first

- Specification change: none in `ess/` (`canon diff` belongs to the command surface
  story:ess-command-surface declares, which depends on this story). The specification this story
  starts from is the eight revision pairs of § Acceptance: the first commit adds
  `fixtures/investigation/semantic-diff/` and the named test with its expected classifications.
- Red on that commit: `diff_classifies_investigation_revisions` fails, because the skeleton's
  `canon diff` refuses as not built.

## Acceptance

The named test `diff_classifies_investigation_revisions` (in `crates/canon-cli/tests/`) passes with
eight expectations, one per revision pair listed here, each pair a directory under
`fixtures/investigation/semantic-diff/`: an added obligation is `TIGHTENING`; a shortened evidence
maximum age is `TIGHTENING`; a removed capability requirement is `RELAXATION`; an added action is
`EXPANSION`; a changed outcome requirement is `BREAKING`; a description-only edit is
`NO SEMANTIC CHANGE`; a conjunct added under a claim's `all` is `TIGHTENING`; an alternative added
under a claim's `any` is `RELAXATION` (design § 31, "new alternate evidence path added"). The last
two were added on 2026-10-07 for story:protocol-floor, whose refinement check reads them. Stories
that land later add pairs of their own to the same directory and test (story:case-inputs).

## Source

TASKBOARD C-011 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 29, § 31, § 38.

## Order after issue 5 (2026-10-07)

First of the stories https://github.com/beyond10x/canon/issues/5 asks to be ordered (2026-10-07):
story:semantic-diff → story:case-inputs → story:protocol-imports → story:protocol-floor;
story:ess-command-surface after story:semantic-diff and before story:protocol-imports;
story:case-composition after story:protocol-imports. Every story this one depends on is implemented
and no blocker stops it, so it is the next wave. story:case-inputs extends this story's classifier
with its own constructs; story:protocol-floor reads this story's classification to decide whether
a protocol stays above its floor.

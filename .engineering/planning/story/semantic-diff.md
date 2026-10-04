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
- depends_on: story:conformance-suite
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon-cli/
- confidence: cited
  path: crates/canon/src/diff/
- confidence: cited
  path: fixtures/investigation/semantic-diff/
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:51Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

`canon diff --from --to` (clap derive) compares two protocol revisions on their compiled
`canon-ir/1` and classifies every change as `TIGHTENING`, `RELAXATION`, `BREAKING`, `EXPANSION` or
`NO SEMANTIC CHANGE` (design § 31), so nobody has to infer what a protocol change does to legitimate
work from a raw source diff. Output is deterministic and lists each change with its category and
the identifiers it touches.

## Order (operator decision 2026-10-04)

This story and story:conformance-suite both change `crates/canon-cli`; this story depends_on
story:conformance-suite so they do not run at once. It also depends_on story:evidence-freshness,
because one classified change is a shortened maximum age.

## Scope

- Out: changes to constructs the kernel does not yet have (independence, protocol imports, upstream
  invalidation) and live-case impact, which needs case data Canon does not hold (design § 38).
- Fixtures: the revision pairs live in `fixtures/investigation/semantic-diff/`, a directory rather
  than one variant file, because each pair is two protocol documents. The base is not edited.
- Surfaces: `crates/canon/src/diff/`, `crates/canon-cli/`, `fixtures/investigation/semantic-diff/`.

## Acceptance

The named test `diff_classifies_investigation_revisions` (in `crates/canon-cli/tests/`) passes with
six expectations, one per revision pair in `fixtures/investigation/semantic-diff/`: an added
obligation is `TIGHTENING`; a shortened evidence maximum age is `TIGHTENING`; a removed capability
requirement is `RELAXATION`; an added action is `EXPANSION`; a changed outcome requirement is
`BREAKING`; a description-only edit is `NO SEMANTIC CHANGE`.

## Source

TASKBOARD C-011 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 29, § 31, § 38.

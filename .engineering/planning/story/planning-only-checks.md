---
format: aep.planning-md/3
id: story:planning-only-checks
kind: story
status: active
title: Planning-only pull requests pass the required check without build or test
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: .github/workflows/check.yml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Taskfile.yml
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T21:46:06Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-21T21:46:07Z", actor: "human:timo", revision: 6}
---
## Outcome

A pull request whose changed files are all planning or prose files passes its required checks
without building or testing. The required job `canon / task check` (`.github/workflows/check.yml`)
still runs and reports: it lists the changed files, finds only such paths, and exits 0 before it
installs a toolchain. A pull request that changes any other file runs `task check` as today. A
change under the built documentation site (`website/`) runs the documentation check and skips
the build and test of the code.

The rule is the workspace file `~/beyond10x/AGENTS.md` § Planning-only pull requests skip the
tests (2026-11-21): "the required job runs, finds only such paths, and exits 0. One path list per
repository, in the workflow, with a test case for both sides of it." And: "The ruleset ... is
Timo's. Do not change it."

## State on 2026-11-21

- `check.yml` has one job, `canon / task check`, on `pull_request` and on pushes to `main`. It
  has no path condition: every pull request installs rustc, go-task and ESS and runs `task check`
  (timeout 30 minutes).
- Each wave of this plan pushes commits that change only `docs/plan/` and `.engineering/`
  (a recorded review, a status row); each ran the full job on pull request 28.
- `website/docs/reference/` and `website/static/schemas/` are generated from the code and
  compared by `task docs-check`; `website/status.yaml` is read by the generator. A change there
  is not prose.

## What the unit does

1. One path list, in one place the workflow and the test both read (a small script under
   `scripts/` or a step of the workflow; the unit chooses and says why): planning or prose is
   `.engineering/**`, `docs/**`, and `*.md` outside `website/`. Everything else is code. The unit
   reads the tree and names any path it had to add, with the reason; `AGENTS.md`, `README.md` and
   `CLAUDE.md` are prose.
2. The classifier takes the list of changed files and answers one of `planning-only`,
   `docs-site-only`, `code`. A list that cannot be read, or is empty, answers `code`.
3. `check.yml`: the same job name. First step after checkout: the changed files of the pull
   request against its base (`git diff --name-only` with enough history fetched), classified.
   `planning-only`: every later step is skipped and the job ends green. `docs-site-only`: the
   toolchain is installed and `task docs-check` runs, not `task check`. `code`, and every push to
   `main`: as today.
4. A test, run in the gate: both sides of the list (a planning-only list, a list with one code
   file among planning files, a `website/` list, an empty list, a rename from a planning path to
   a code path), and that the path list in the workflow is the one the test reads.
5. `AGENTS.md`: one paragraph that names the list's file and the three answers.

Not allowed: a change to a ruleset, to branch protection, or to the name of the required job; a
`paths:` filter on the workflow trigger (a job that does not run reports no status, and the
required check would wait forever).

## Scope

Written by the wave coordinator on 2026-11-21 from the tree at `a93962cd`.

- **Files:** `.github/workflows/check.yml` — cited
- **Files:** the classifier and its test (new; place chosen by the unit) — inferred
- **Files:** `Taskfile.yml` (a task that runs the classifier's test, called by `check`) — inferred
- **Files:** `AGENTS.md` — cited
- **Out:** `shared-gates.yml` (the shared source gates run on every pull request and are not this
  repository's to change); `pages.yml`; rulesets.
- **Would collide with:** story:fast-test-step and story:local-gate-equals-ci
  (`.github/workflows/check.yml`, `Taskfile.yml`, `AGENTS.md`).

## Acceptance

- The classifier's test passes in the gate and covers the five lists above.
- On the unit's own pull request or branch, shown with the run's link: a commit that changes only
  a file under `docs/plan/` ends the job `canon / task check` green without the step `task check`
  having run; a commit that changes a `.rs` file runs it.
- The job's name and the workflow's triggers are unchanged (`git diff` shows it).
- `AGENTS.md` names the path list's file.

## Source

`~/beyond10x/AGENTS.md` § Planning-only pull requests skip the tests. The `host-operator`
session sent the order to this session on 2026-11-21: "file the path-filter story".

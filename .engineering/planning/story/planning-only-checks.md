---
format: aep.planning-md/3
id: story:planning-only-checks
kind: story
status: implemented
title: Planning-only pull requests pass the required check without build or test
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: .github/workflows/check.yml
- confidence: inferred
  path: .github/workflows/pages.yml
- confidence: cited
  path: AGENTS.md
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: crates/canon/tests/planning_only_classifier.rs
- confidence: inferred
  path: scripts/classify-changes.sh
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T21:46:06Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-21T21:46:07Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-11-22T11:12:33Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1}}, correlation: "wave-2026-11-20-w9"}
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

Derived 2026-11-21 by `story-scoper`, from the tree at `efe0859e`. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **In:** a changed-file classifier (`planning-only` / `docs-site-only` / `code`), a pre-toolchain step in the `canon / task check` job, its test, and one `AGENTS.md` paragraph — cited (story, What the unit does 1-5)
- **Out:** `.github/workflows/shared-gates.yml` (a reusable `beyond10x/gates` workflow on `pull_request_target`, `.github/workflows/shared-gates.yml:4,16`), `.github/workflows/b10x-docs-site.yml` (`workflow_run` on main only, `:4-8`), rulesets — cited
- **Primary surface:** `.github/workflows/check.yml` (one job `check`, name `canon / task check`, triggers `pull_request` and push to `main`, no path condition, `:3-6,20-21`) — cited
- **Files:** `.github/workflows/check.yml` — cited. Checkout is `actions/checkout` with `persist-credentials: false` and no `fetch-depth` (`:27-29`), so depth 1. The base commit is not fetched; `fetch-depth: 0` at checkout works. A later fetch has no credentials. Use `git diff --no-renames --name-only`: with rename detection, a code file renamed into `docs/` lists only the planning path (inferred).
- **Files:** `scripts/` does not exist; the tree has no tracked `.sh` or `.py`, and `crates/canon-cli/tests/simulate_child_outcome.rs:395-403` fails the gate if any `*.py` is tracked, so the classifier is shell or a workflow step. Proposed `scripts/classify-changes.sh` — inferred
- **Files:** the test. `Taskfile.yml` runs only `cargo`, `ess` and `aep` (`Taskfile.yml:7-48`), no non-Rust test. `cargo test --workspace` (`:13`) runs any Rust test in `task check` with no Taskfile edit. Precedent: tests that read `.github/workflows/check.yml` (`crates/canon/tests/ess_gate.rs:432`) and `Taskfile.yml` (`crates/canon-cli/tests/conform_requirements.rs:164`). Proposed `crates/canon/tests/planning_only_classifier.rs` — inferred
- **Files:** `Taskfile.yml` — inferred, needed only if the test is not a Rust test. Constraint: the `check` recipe must keep `conform run --requirements conformance/requirements.yaml` (`conform_requirements.rs:176-182`, cited).
- **Files:** `AGENTS.md` — cited. The story calls it prose, but `crates/canon/tests/adversary2_ess_gate.rs:67-90` reads it. When § ESS contains "command surface" (`AGENTS.md:37` does), the test asserts the compiled ESS declares commands.
- **Files:** `.github/workflows/pages.yml` — inferred, a decision for the coordinator. Job `Build documentation` (`:22`) runs on every `pull_request` with no path condition and runs `cargo test -p canon-docs`, `cargo build`, `generate --check` and `npm ci && npm run build` (`:37-43`). It is not `canon / task check`, but a planning-only PR still builds code unless this workflow changes too. Release plans name "`task check`, the documentation build, and security and privacy checks" as green on PR 14 (`.engineering/planning/release-plan/canon-0-3-0.md:37-38`).
- **Classification:**
  - Planning/prose: `docs/` (14 `.md`, read by no crate or script: grep for `docs/`, `docs/plan`, `include_str!` is empty), `README.md`, `CHANGELOG.md`, `CLAUDE.md`, `LICENSE` (no reader) — cited by grep of `crates/` and `Taskfile.yml`.
  - Built site: `website/` (37 files), of which `website/docs/reference/`, `website/static/schemas/`, `website/data/` (owned whole) are generated and compared (`Taskfile.yml:32-36`; `crates/canon-docs/src/main.rs:115-148`). The story omits `website/data/` (3 files, including `status.json`).
  - Code: `crates/`, `ess/` (gated, `ess_gate.rs`), `conformance/` (run by `Taskfile.yml:14`), `fixtures/` (232 files, no `.md`; read by tests and `canon-docs`, `crates/canon-docs/src/landing.rs:21`), `Cargo.*`, `Taskfile.yml`, `.github/`, `.gitignore` — cited.
  - Test-read `.md`: `AGENTS.md` (above). The hand-written `website/docs/concepts/{evidence-and-revisions,evaluation,protocols}.md` and `website/docs/status/where-this-stands.md` are read by `crates/canon/tests/adversary_binding_docs.rs:58-71` and `adversary_obligations_docs.rs:90-103`, and all `website/docs/**.md` by `crates/canon-docs/src/main.rs:189`. A `docs-site-only` run of `task docs-check` skips those tests (`Taskfile.yml:13`) — cited.
- **`.engineering/` is read:** `crates/canon-docs/src/status.rs:17,86` reads `.engineering/planning/story/<name>.md` front matter (`status`, `refs`) for each of the 47 stories named in `website/status.yaml`. It writes the result to `website/data/status.json`, and `task docs-check` fails on drift (`status.rs:35-45`; test `status.rs:211-219`). A story flipped to `implemented` without regenerating `status.json` is a planning-only change that `docs-check` rejects. Suggested: treat `.engineering/planning/story/**` as `docs-site-only`; the rest of `.engineering/` has no reader — cited, suggestion inferred. No `b10x-gates scan-text` or `.engineering` scan in the tree; the `ess/` scan is limited to `ess/`, and `ess/domains/cli.yaml` mentions `.engineering` only in comments.
- **Constraint on `check.yml`:** `ess_gate.rs:432-444` asserts `ESS_VERSION:` appears exactly once, equal to the pin. The new step must not add another `ESS_VERSION:` line — cited
- **Required checks:** the tree does not list them. `AGENTS.md:145` says "required source checks"; no ruleset note exists. `canon / task check` is the only job name the story states; it was not verified against GitHub.
- **Workflows:** `check.yml` (`check`, `canon / task check`: builds and tests, no path condition); `pages.yml` ("Documentation validation", `build` / `Build documentation`, on `push` to main, `pull_request`, `workflow_dispatch`: builds and tests, no path condition); `shared-gates.yml` ("Shared source gates", job `common`, reusable, `pull_request_target`, push to main, tags: no build); `b10x-docs-site.yml` ("Documentation site", job `publish`, `workflow_run`, `workflow_dispatch`, bot and main only; `if:` condition) — cited
- **Not touched:** `crates/canon-docs`, `website/status.yaml`, the `Taskfile.yml` `check` recipe order, the workflow triggers — cited
- **Confidence:** medium — the check.yml, AGENTS.md and test surfaces are read in the tree, but whether `pages.yml` and `common` are required checks is not in the tree.
- **Would collide with:** `.github/workflows/check.yml`, `Taskfile.yml`, `AGENTS.md` (`story:fast-test-step`, `story:local-gate-equals-ci`); `.github/workflows/pages.yml` (`story:local-gate-equals-ci`); the new `scripts/` directory (`story:local-gate-equals-ci` plans `scripts/` for `ci-watch`, line 64).
- **Not established by the scoper:** required check names on GitHub; the job name `common` produces under `shared-gates.yml`; whether the repository is public (decides whether a late `git fetch` can work); that PR 28 ran the full job.

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

## Added at stage 1 of wave 10, 2026-11-21: what the scope changes

The scope (§ Scope, read from the tree at `efe0859e`) found readers this story did not know. The
coordinator decided the following; each binds the unit in addition to § Acceptance.

- **`AGENTS.md` is `code`.** `crates/canon/tests/adversary2_ess_gate.rs:67-90` reads it. A pull
  request that changes it runs the full job.
- **`.engineering/planning/story/**` is `docs-site-only`.** `crates/canon-docs/src/status.rs:17,86`
  reads each story's front matter and writes `website/data/status.json`; `task docs-check` fails
  on drift. A story moved without a regenerated `status.json` must go red in its own pull request,
  not in the next one that touches code. The rest of `.engineering/` and `docs/` have no reader
  and are `planning-only`. The unit states the grep that shows it for every path it classes
  `planning-only`: no test, no gate step and no generator reads a file of that class.
- **`website/data/` belongs to the built site**, with `website/docs/reference/` and
  `website/static/schemas/`.
- **A `docs-site-only` run** runs `task docs-check` and the tests that read hand-written pages of
  the site: today `crates/canon/tests/adversary_binding_docs.rs`,
  `crates/canon/tests/adversary_obligations_docs.rs` and the tests of `canon-docs`. The unit
  lists them by reading the tree and puts them in one task the job calls. It does not run the
  rest of the test step.
- **`.github/workflows/pages.yml` uses the same path list.** Its job `Build documentation` builds
  and tests on every pull request today. On `planning-only` it exits 0 before the toolchain; on
  `docs-site-only` and `code` it runs as now. The job's name and triggers do not change.
- **The classifier is a shell script or Rust, never Python**
  (`crates/canon-cli/tests/simulate_child_outcome.rs:395-403` fails the gate on a tracked `*.py`).
  Its test is a Rust test, so `task check` runs it with no Taskfile change. One path list, read
  by both workflows.
- **The changed files are read with full history:** the checkout fetches the base
  (`fetch-depth: 0`), and the list comes from `git diff --no-renames --name-only`, so a code file
  renamed into `docs/` still lists its old path. On a push to `main` and on any failure to
  compute the list the answer is `code`.
- **`ESS_VERSION:` stays exactly once in `check.yml`** (`crates/canon/tests/ess_gate.rs:432-444`).
- The names of the required checks are not in the tree. The unit keeps every job `name:` as it
  is and changes no trigger; it does not read or change a ruleset.

Test cases added to § Acceptance's five lists: a pull request of only `AGENTS.md` is `code`; one
of only `.engineering/planning/story/x.md` is `docs-site-only`; one of only
`.engineering/planning/epic/x.md` and `docs/plan/y.md` is `planning-only`; a rename of
`crates/a.rs` to `docs/a.md` is `code`.

## Scope as built

Written at the close of wave 2026-11-20-w9 from the wave page, the review of the integration branch (`review-result:review-wave-9-gate`) and the merge of `impl/planning-only-checks` (`a382eedc`: 6 files, +306 −3). It corrects the body where the two differ.

- Changed: `.github/workflows/check.yml`, `.github/workflows/pages.yml`, `AGENTS.md`, `Taskfile.yml` (task `docs-site-check`), `crates/canon/tests/planning_only_classifier.rs` and the new `scripts/classify-changes.sh`; the same six paths as the typed scope, none outside it.
- Path list as built: `AGENTS.md` at any depth is code; story files and `website/` are docs-site; `.engineering/` outside the stories, `docs/` and root `*.md` are planning; any other path with a `/` is code. Decided by the unit: a `*.md` below the root is code; `pages.yml` skips only on `planning-only`; the ess install step runs only on `code`; a push to `main` is `code`.
- After the merge: correction unit 1 (`a27aeab95`) makes `task docs-site-check` run `cargo nextest run -p canon-docs` under the 60 s kill, and `pages.yml` installs nextest (gate G3). The review noted that a classifier answer outside the three words runs no step (G7, INFEASIBLE today; story:review-notes-wave-9).
- Acceptance: the classifier test passed in the gate below (11 passed). `git diff a382eedc^1 a382eedc -- .github/workflows/check.yml` changes no `name:` and no trigger line. `AGENTS.md` names `scripts/classify-changes.sh`. **Not shown:** a CI run link where a commit that changes only `docs/plan/` ends `canon / task check` green without `task check`, and one where a `.rs` change runs it. That needs a planning-only pull request after the merge.
- This close-out commit changes `.engineering/planning/story/`, which the classifier calls docs-site: its CI run checks `website/data/status.json`, which the close-out does not regenerate.

**Gate of the close-out**, on `wave/2026-11-21` at `2f3d14b14`, 2026-11-22 10:43 to 11:01 UTC, host load 67 on 20 CPUs, rustc 1.100.0, each step run once by the coordinator's gate script (not `task check` as one command): build exit 0 (125 s); `task ess-gate` 0; `cargo fmt --check` 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` 0; `task test` 0 (1275 tests; the step's own line: `ess-gate 2.2 s + workspace 50.4 s + -p b10x-canon 2.7 s = 55.4 s, bound 120 s`; slowest 16.9 s); `canon conform run` 0 (`30 passed, 0 failed`; `31 implemented, 31 covered`); `task docs-check` 0 (18 generated files current); `task test-slow` exit 201 (63 tests: 53 passed, 10 killed at 60 s; its CI step may fail, D123, story:slow-lane-under-bound). Recorded as `test_result` evidence. The four review results of the wave are `review-result:review-wave-9-eval`, `-simulate`, `-model` and `-gate`.

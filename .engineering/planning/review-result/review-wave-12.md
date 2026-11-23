---
format: aep.planning-md/3
id: review-result:review-wave-12
kind: review-result
status: active
title: 'Review of wave 12 at 98ae8c3b4: needs change, 3 red cases, 7 findings'
relations:
- reviews: story:conform-compiles-the-fixture-once
- reviews: story:check-evaluates-statuses-only
- reviews: story:order-timeline-built-once
- reviews: story:authority-lookup-by-capability
- reviews: story:freshness-needs-trusted-timestamps
- reviews: story:approvals-bound-to-evidence-content
- reviews: story:generate-rollback-in-private-staging
revision: 1
---
Review of the integration branch `wave/2026-11-22-w12` at `98ae8c3b4` (base `70db25076`, Canon 0.12.0), one `aep:adversary` on Opus over the seven stories of the wave, 2026-11-23. The reviewer's cases are on local branch `review/w12-wave` at `d18ae5b07`.

unit: wave 2026-11-22-w12, integration branch at `98ae8c3b4`
verdict: NEEDS-CHANGE
cases: executed 1278→1281, red 3
origin: introduced 7, pre-existing 0, undecided 0
needs-coordinator: F2's severity (the race was reproduced only with gdb holding the run)

## Findings and what became of each

| id | severity | unit | finding | disposition |
|---|---|---|---|---|
| F1 | blocker | 5 | `--strict` checks only records the revision binding left; the order forms read every accepted record, so a future-dated record bound to an older revision decides `first`, `before`, `after` or `since` | Fixed: `b52e2b9a2`, merged `c182256f8`. Strict refuses over every accepted record; the old pin is replaced by its opposite |
| F2 | warning | 7 | the staging directory is opened and written by path: a swap for a symlink makes `canon generate` chmod a directory outside `--out` to 0700 and write through it | Fixed: `c30178161`, merged `c0db97b26`. The directory is held as an `O_PATH` handle and reached through `/proc/self/fd`; the gdb reproduction leaves the victim at 755 |
| F3 | warning | 5 | the generated CLI reference writes `--strict <SKEW>` where the help writes `--strict [<SKEW>]` | Fixed: `1f15dbdb2`. Table rows use clap's own rendering; only the `--strict` row changed |
| F4 | note | 6 | the docs say replaced contents change no byte of the decision; a claim reading the replaced result changes it | Fixed: `c7acbd6a9`. Wording names the fields that decide a ground, `producer` included |
| F5 | note | 7 | under umask 0477 or 0777 the run exits 2 where 0.12.0 succeeded; a read-only `--out` names the directory, not the first file | Fixed: `c30178161`; both behave as 0.12.0 |
| F6 | note | 7 | a staging directory left by a killed run is a hidden dot name and `out-not-empty` does not name it | Fixed: `c30178161`; the message names the entry |
| F7 | note | 7 | no fallback on a filesystem without hard links | Fixed through a test seam: `c30178161`. Real vfat, exFAT and FUSE are untested (need root) |

Tried and not broken (from the report): 697 runs of the wave binary against `70db25076` over the 173 `protocol/1` fixtures (compile, check with and without `--properties`, simulate, generate files and modes, conform run) gave 0 differences; 120 hand-written `canon evaluate` cases gave 0 differences; `--strict` that refuses nothing gives the bytes of the default; a compile failure in conform keeps its text and step; nothing that calls `evaluate_unexplained` reads an explanation; the four slow tests of unit 2 exist by name and `cargo nextest list --profile slow` lists them.

The four `adversary2_ess_model` tests were not run by the reviewer: each starts a cargo probe build and takes 10 to 60 s (story:probe-built-once-for-ess-model-tests).

```findings
[
{"file":"crates/canon/src/eval/mod.rs","line":594,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F1: strict mode checks only records revision binding left, while the order forms read every accepted record, so a future-dated record of a max_age kind bound to an older revision decides an order form under --strict"},
{"file":"crates/canon-cli/src/generate.rs","line":214,"category":"concurrency","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"F2: the staging directory is opened and later written by path, so a writer of --out that swaps it for a symlink makes canon generate chmod a directory outside --out to 0700 and write through it"},
{"file":"crates/canon-docs/src/pages.rs","line":211,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F3: the generated CLI reference writes --strict <SKEW> as a required value while the help writes --strict [<SKEW>]"},
{"file":"crates/canon/src/eval/grounds.rs","line":20,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"F4: the docs say replaced contents under a cited id change no byte of the decision, but a claim reading the replaced result changes it"},
{"file":"crates/canon-cli/src/generate.rs","line":304,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"F5: under a umask masking owner read the run now exits 2 where 0.12.0 succeeded, and a read-only --out is now reported as the directory instead of the first file"},
{"file":"crates/canon-cli/src/generate.rs","line":354,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"F6: a staging directory left by a killed run is a dot-name hidden from ls, and the out-not-empty refusal does not name it"},
{"file":"crates/canon-cli/src/generate.rs","line":335,"category":"judgement","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"F7: placing by hard link has no fallback, so a --out on a filesystem without hard links now fails where 0.12.0 wrote directly; not reproducible on this host without root"}
]
```

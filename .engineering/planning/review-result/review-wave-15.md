---
format: aep.planning-md/3
id: review-result:review-wave-15
kind: review-result
status: active
title: 'Review of wave 15 at ffbead979: approve, 0 red cases, 3 findings'
relations:
- reviews: story:evidence-digest-binds-approvals
revision: 1
---
Review of the integration branch `wave/2026-11-24-w15` at `ffbead979` (base `247df6e5d`, Canon 0.15.0), one `aep:adversary` on Opus over `story:evidence-digest-binds-approvals`, 2026-11-24. The reviewer's cases are commit `589cf4bc6` (branch `review/w15-wave`).

unit: wave 2026-11-24-w15, integration branch at `ffbead979`
verdict: CONFIRMED (approve; two warnings-or-notes to fix, no red case)
cases: executed 175→177, red 0
origin: introduced 3, pre-existing 0, undecided 0
needs-coordinator: release notes; a status entry; ESS 0.98.0 on PATH fails 3 `ess_model_matches` tests outside `task ess-gate` (environment)

## Findings and what became of each

| id | severity | finding | disposition |
|---|---|---|---|
| F1 | warning | `Standing::lapse` (`eval/grounds.rs:284`) scans every given record by id and rehashes it for each ground with a digest: 32000 digest grounds took 4.4 to 16.2 s against 1.0 to 1.5 s for bare ids | Correction unit (see the wave page) |
| F2 | note | `canon digest --help` says a file that is not a record is refused, yet a record with an invalid id or instant prints a digest | Correction unit: help and ESS comment say what is checked |
| F3 | note | `website/status.yaml` has no entry for the story | Added in the release commit |

Tried and not broken (from the report): the canonical form covers every field of `EvidenceRecord` (no collision found; YAML numbers, duplicate keys refused; tags, anchors and a BOM read the same); only `decide` reads grounds; a ground with a digest and no record gives `present: false`; every malformed digest form is `malformed-input` naming the entry; `canon digest` equals an independent `sha2` computation over every repository record (test `every_repository_record_has_the_documented_digest`) and RFC 8785 escapes and UTF-16 key order (test `escapes_and_utf16_key_order_follow_rfc_8785_and_hold_the_grant`); a 0.15.0 binary refuses the map form; 1305 runs against `247df6e5d` differ only where the unit named them.

```findings
[
{"file":"crates/canon/src/eval/grounds.rs","line":284,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"F1: every ground with a digest scans all given records by id and rehashes the record, so evaluation cost grows with entries times records"},
{"file":"crates/canon-cli/src/lib.rs","line":410,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"F2: canon digest --help says a file that is not a canon-evidence/1 record is refused, yet a record with an invalid id or instant prints a digest and exits 0"},
{"file":"website/status.yaml","line":1,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"F3: the status page has no capability for story:evidence-digest-binds-approvals or CANON-GROUND-002"}
]
```

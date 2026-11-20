---
format: aep.planning-md/3
id: story:invalidation-evaluation-cost
kind: story
status: active
title: Evaluation under invalidation rules is at most quadratic in the claim count
relations:
- decomposes: epic:canon-kernel
- depends_on: story:invalidation-rules
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon/src/eval/claims.rs
- confidence: cited
  path: crates/canon/src/eval/invalidation.rs
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: cited
  path: crates/canon/src/explain/mod.rs
- confidence: cited
  path: crates/canon/tests/adversary2_review_hardening.rs
- confidence: inferred
  path: crates/canon/tests/invalidation_evaluation_cost.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
- {from: "proposed", to: "active", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
---
## Outcome

Evaluating a protocol whose claims each carry an invalidation rule takes time no worse than
quadratic in the number of claims, within the depth bound (4096 chained claims,
`crates/canon/src/eval/depth.rs:8`). No output changes: for every input the decision, the excluded
evidence and the explanation stay the same byte for byte.

## Found by

Adversary pass 2 of story:review-hardening-w7 (wave 2026-10-04-w14): a chain of claims with one
invalidation rule each took 0.37 s at 50 claims and 84 s at 400 (the same on a0cc691), about cubic.
The recorded red message is in
`review-result:adversary-w14-canon-review-hardening-w7-pass-2`: "a chain eight times as long took
84.157740088s, against 372.684007ms: more than quadratic growth".

Three causes were suspected and not attributed. The scoper read each on 2026-11-20 at `8f1f452`;
all three are still in the code:

1. Memo keys clone the context. `crates/canon/src/eval/claims.rs:135` clones the whole
   `EvidenceId` set into each `(ClaimId, Context)` key of `values` (`claims.rs:121`), and
   `claims.rs:190-191` builds a fresh union for each claim test.
2. The fixpoint rescans every claim per round. `invalidated_claims`
   (`crates/canon/src/eval/invalidation.rs:122-143`) finds one more claim per round in a chain and
   scans all claims each round, once per rule (`invalidation.rs:108`). The scoper's reading, not a
   measurement: this alone gives cubic growth.
3. `Invalidated::lists` (`invalidation.rs:45-50`) calls `reached_matches` (`invalidation.rs:55-73`)
   on every call. It is called per kept record per claim at `crates/canon/src/eval/mod.rs:601` and
   per record at `crates/canon/src/explain/mod.rs:236`.

## ESS first

Exempt under AGENTS.md § ESS ("Only a change with no behaviour change is exempt, and its story
says so"). This story changes no behaviour; only evaluation cost changes. `ess/` is not changed and
no conformance scenario is added.

It is test first all the same. Commit 1 adds only
`crates/canon/tests/invalidation_evaluation_cost.rs`, holding the two tests of § Acceptance and one
pinned decision for a 5-claim invalidated chain. The timing test fails on that commit and the
failing run is recorded; the pinned decision passes there and must keep passing. Later commits make
the timing test pass without changing any of the three.

The original test body was never committed: `git log --all -S linear_in_the_chain` finds only the
forwarding comment at `crates/canon/tests/adversary2_review_hardening.rs:126` (commit `a0af24a`)
and the planning commit `66c8d4b`. The scratch directory the draft named
(`ga-wave-2026-10-04-w14/canon-review-hardening-w7/scratch/`) no longer exists on the workstation
and only ever held logs. The test is rebuilt from the shape the review recorded.

## Scope

Scoped 2026-11-20 by `story-scoper` against `8f1f452`, written by the wave coordinator. Every line
is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **In:** what evaluation costs when invalidation rules are present: the `invalidated_claims`
  fixpoint, the `(claim, context)` memo, and `Invalidated::lists` with its two callers; the two
  tests and the pinned decision. — cited
- **Out:** any change to an evaluation output; any change under `ess/`; a new conformance
  scenario. — cited
- **Out:** the cost of `canon check`. `crates/canon/src/check/space.rs:449` calls
  `invalidated_claims` and gets the speed-up without being edited, as long as the signature stays.
  — inferred
- **Out:** the `//!` module documentation of `crates/canon/src/eval/mod.rs`. `canon-docs` renders
  it into `website/docs/reference/evaluation.md` (`crates/canon-docs/src/pages.rs:60`). Notes on
  cost go in item documentation (`///`), so no generated page changes. — cited
- **Primary surface:** `crates/canon/src/eval/invalidation.rs` — cited
- **Files:** `crates/canon/src/eval/claims.rs` — cited
- **Files:** `crates/canon/src/eval/mod.rs` (`excluded_for`, `mod.rs:537`; the `lists` call at
  `mod.rs:601`) — cited
- **Files:** `crates/canon/src/explain/mod.rs` (the `lists` call at `explain/mod.rs:236`, through
  `read_by` at `:503`) — cited
- **Files:** `crates/canon/tests/invalidation_evaluation_cost.rs` (new) — inferred, proposed by
  the scoper
- **Files:** `crates/canon/tests/adversary2_review_hardening.rs:126` (the forwarding comment,
  removed when the test lands) — cited
- **Also likely:** `crates/canon/src/check/space.rs`, only if `invalidated_claims` changes
  signature — inferred, not in the typed scope
- **Coordinator's at integration:** nothing. A cost change adds no capability to
  `website/status.yaml` and regenerates no page.
- **Confidence:** high. The story and the review name the sites, and each was read on 2026-11-20.
- **Would collide with:** any unit that edits claim evaluation, the invalidation stage or
  `excluded_for` under `crates/canon/src/eval/`, or `crates/canon/src/explain/mod.rs`.
  story:capability-scope may edit `explain/mod.rs:371-382` (inferred there), so the two run in
  different waves.
- **Safety fact:** every evaluation output stays the same byte for byte. Holding that needs
  `crates/canon/tests/adversary_invalidation.rs`, `adversary2_invalidation.rs` and the conformance
  suite to stay green, plus the pinned decision. Unproven.
- **Not established by the scoper:** the share of each cause in the 84 s (nothing was run);
  whether quadratic is reachable while contexts stay distinct per starting claim, which is the
  implementor's design choice (interning contexts, or cutting them to the records a claim's
  reached matches can read).

## Acceptance

In `crates/canon/tests/invalidation_evaluation_cost.rs`, with no `#[ignore]`:

1. `claims_kept_apart_by_invalidation_rules_are_evaluated_in_time_linear_in_the_chain` passes. The
   name is the review's and is kept so it can be traced; the bound it asserts is quadratic. Over a
   chain where claim `c_i` tests `c_{i+1}`, the last claim reads one evidence kind, rule `r_i`
   invalidates `c_i`, and one record moved per rule: the 400-claim chain takes at most 64 times as
   long as the 50-claim chain. Each size is timed as the minimum of 3 runs. The long chain runs on
   a worker thread with a deadline of 64 times the short time plus 2 s, so a red run ends in about
   25 s and not in 84 s.
2. `a_chain_at_the_depth_bound_evaluates_within_budget` passes: a 4000-claim chain of the same
   shape evaluates in under 10 s in a release build. In a debug build the test runs the same chain
   and asserts 120 s; that debug budget was set by the coordinator on 2026-11-20 without a
   measurement, and the implementor reports the measured time.
3. The pinned decision for the 5-claim invalidated chain is byte-identical before and after the
   change.

The release budget is checked with
`cargo test --release -p b10x-canon --test invalidation_evaluation_cost`, run through `slot`.

## Source

`review-result:adversary-w14-canon-review-hardening-w7-pass-2`; scoping of 2026-11-20
(`approval-record:accept-drafts-2026-11-20`).

---
format: aep.planning-md/3
id: story:slow-lane-under-bound
kind: story
status: implemented
title: Every test of the slow lane ends within 60 s, so its CI step can fail the job again
tags:
- from-finding
- test-budget
refs:
- provider: decision
  reference: D123
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: .config/nextest.toml
- confidence: cited
  path: .github/workflows/check.yml
- confidence: inferred
  path: crates/canon/src/check/mod.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T17:21:51Z", actor: "human:timo", revision: 5, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
- {from: "proposed", to: "active", at: "2026-11-22T17:21:51Z", actor: "human:timo", revision: 6, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
- {from: "active", to: "implemented", at: "2026-11-22T21:20:35Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
---
## Outcome

Every test of the `slow` lane (`task test-slow`) ends within 60 s on a loaded host, so the lane's step in CI can fail the job again. Today the lane is allowed to fail (`continue-on-error: true` in `.github/workflows/check.yml`, Timo, 2026-11-22 10:35 CET via `host-operator`, decision D123: "A: slow lane may fail, story"), because tests of it end at the 60 s kill.

1. **No test of the lane is killed.** Each of the tests below ends in under 60 s on the host at load 40, or is split so that no part of it runs longer.
2. **The lane's CI step fails the job again.** `continue-on-error: true` is removed from the `task test-slow` step, and the `AGENTS.md` sentence that names this story is removed in the same change.
3. **No assertion is weakened.** Every case a test checks today is checked in the gate lane or the slow lane, with the same expected text.

## Why

- First whole run of the lane on `wave/2026-11-21` at `f7dcbf11a`, 2026-11-22, host load 80 on 20 CPUs: "62 tests run: 37 passed, 2 failed, 23 timed out" in 1200 s (`w10/gate/test-slow.out`, scratch of the wave coordinator). The unit that built the lane measured 56 passed and 6 timed out at load 40 to 45 (story:fast-test-step, wave page). The number of kills follows host load: these tests need 28 to 44 s of CPU alone (the bisect of the wave: `every_value_of_since…` 9.8 to 14.2 s CPU, `generated_protocols_check_alike_sliced_and_whole_0` 30.7 to 32.6 s CPU).
- The generator study of the wave found where the time goes for the slowest one: `generate::generate` was 99.4 % of `every_generated_witness_passes_and_is_minimal_over_every_investigation_protocol` (state evaluation 1.6 to 2 ms per state in a debug build, no single function over 1.5 %); the change `c499c378b` cut its evaluations from 144,961 to 45,499 and its time from 249 to 544 s to 89 to 201 s, which is still over 60 s.
- Workspace rule `~/beyond10x/AGENTS.md` § "Tests stay fast": no test over 60 s; an expensive product is made once (a committed generated file with a digest of its sources, or one shared fixture), full regeneration runs in the `slow` lane and compares bytes.

## The tests

Killed at 60 s in that run (binary, test):

- `b10x-canon` lib: `check::tests::a_rule_dimension_counts_toward_the_bound_and_is_named`, `a_slice_holds_what_its_last_records_are_placed_against`, `generated_protocols_check_alike_sliced_and_whole_0` to `_3`
- `b10x-canon::main`: `adversary2_invalidation::the_bound_refusal_names_many_upstream_artifacts`, `adversary_check_slices::probe_after`, `probe_before`, `probe_chain`, `probe_order`, `probe_unchanged_reports_as_the_whole_product`, `adversary_check_space::a_space_of_exactly_the_bound_is_checked_and_one_past_it_refused`, `adversary_evidence_order_since::check_reaches_every_combination_of_one_kind_and_two_kind_since`, `…of_since_with_after_and_matches`, `adversary_scenario_generation::every_generated_witness_passes_and_is_minimal_over_every_investigation_protocol`, `case_composition::check_reaches_every_combination_of_current_and_any_forms`, `invalidation_evaluation_cost::a_chain_at_the_depth_bound_evaluates_within_budget`, `missing_keys::every_left_out_key_of_a_protocol_or_a_scenario_is_named`, `review_properties::generated_evaluations_keep_every_documented_invariant`
- `canon-cli::main`: `adversary2_review_hardening::check_refuses_a_long_chain_of_claims_without_aborting`, `adversary_check_slices_cli::check_help_does_not_refuse_every_space_above_the_bound`, `canon_check::a_sliced_summary_gives_the_whole_product`

Failed (not killed): `invalidation_evaluation_cost::claims_kept_apart_by_invalidation_rules_are_evaluated_in_time_linear_in_the_chain` (a wall-time ratio, 2.6 s: flaky under load; bound it by CPU time as the `loom` session did for its own ratio tests) and one more in the same log.

## Approach to start from

- The generator tests (`adversary_scenario_generation`, `case_composition::…`): commit each fixture's scenarios with a digest of their sources; the gate lane runs the committed scenarios and the removals (about 3 s); the `slow` lane regenerates and compares bytes (generator study, option "committed scenarios").
- The checks over spaces near the bound (`probe_*`, `a_space_of_exactly_the_bound…`, `a_rule_dimension…`): the cost is state evaluation over 65,536 states at 1.6 to 2 ms each; evaluate for statuses only (the study found a full explanation is built for every state), or reuse the whole product's evaluations for the per-slice checks (3 times fewer evaluations).
- `generated_protocols_check_alike_sliced_and_whole_*`: check about 40 protocols per seed in the gate lane, all 150 in `slow`.
- A timing test is bounded by process CPU time, not wall time.

## Scope

- **In:** Outcome 1 to 3 for the tests listed. — cited
- **Out:** a change of what any test asserts; raising a timeout; a change to the state-space bound. — cited
- **Surfaces:** `crates/canon/src/check/` (tests and the evaluation path), `crates/canon/src/generate/`, `crates/canon/tests/`, `crates/canon-cli/tests/`, `.config/nextest.toml`, `.github/workflows/check.yml`, `AGENTS.md`. — inferred
- **Confidence:** high on the list; medium on the approach (generator study: constant-factor waste, not measured on a quiet host).

## Acceptance

`task test-slow` on the host with load at or under 40 ends with no test killed and no test failed; the `task test-slow` step of `check.yml` has no `continue-on-error`; a CI run of the pull request that removes it is green (link recorded as `test_result` evidence).

## Source

- **Request:** Timo's pick D123 of 2026-11-22 10:35 CET, relayed by `host-operator`: "A: slow lane may fail, story".
- **Read:** `w10/gate/test-slow.out`, `w10/gate/rerun-default.out`, the generator study report and the bisect report of the wave coordinator (scratch).
- **Not run:** the lane on a quiet host.

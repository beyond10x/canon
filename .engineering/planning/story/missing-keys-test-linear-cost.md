---
format: aep.planning-md/3
id: story:missing-keys-test-linear-cost
kind: story
status: draft
title: The missing-keys test reads each fixture once and costs linear time
relations:
- decomposes: epic:canon-kernel
revision: 1
---
## Problem

`crates/canon/tests/missing_keys.rs` `every_left_out_key_of_a_protocol_or_a_scenario_is_named` removes each key of every `protocol/1` fixture in turn and serializes and parses the whole document again for each key, so its cost grows with the square of each fixture's size. On 2026-11-23 it took 8.5 s on an idle build node and was killed at 60 s twice on the shared host at load 90 to 107, also when run alone (wave 14 slow lane, gate at `352569c62`). 211 protocol fixtures, 185 distinct texts, 11391 lines; sum of squared line counts 804681, of which wave 14's new fixtures add 41693.

## Acceptance

- The test reads each distinct fixture text once (no duplicate texts) and its work grows linearly with fixture size, or it is bounded by a stated number of removals per type position.
- Its time alone on the shared host under the slow lane is under 5 s at the load of a gate run, measured and named in the change.
- The property it checks is unchanged: every key the fixtures hold is left out at least once in some document, and every refusal names the key and does not speak of an explicit null.

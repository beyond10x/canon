---
format: aep.planning-md/3
id: story:generate-umask-test-leaves-unremovable-scratch
kind: story
status: draft
title: The generate staging umask test leaves no unreadable scratch under target/tmp
relations:
- decomposes: epic:canon-kernel
revision: 1
---
## Outcome

The test of `canon generate` under a restrictive umask leaves nothing under `target/tmp` that its owner cannot read or write, so `worktree finish` and `discard-cache` remove the tree without a manual `chmod`.

## Why

- Wave 13, 2026-11-23: `worktree finish --archive` failed on two trees because the generate staging umask test left mode-0 files under `target/tmp/generate-staging-umask/...`. Each time the fix was `chmod -R u+rwX <tree>/target/tmp` and a retry.
- Found by the coordinator of wave 12 and 13, not reproduced by a separate run.

## Scope

- **In:** the test that sets the umask for `canon generate` (`crates/canon-cli/tests/generate_staging.rs`, inferred): restore permissions in a drop guard, or run it in a directory the harness removes with `chmod` first.
- **Out:** any change to `canon generate` itself.
- **Surfaces:** `crates/canon-cli/tests/generate_staging.rs` (inferred).

## Acceptance

After `cargo nextest run -p canon-cli -E 'test(/generate_staging/)'`, `find target/tmp ! -perm -u+rw` prints nothing.

## Source

- **Request:** the coordinator of waves 12 and 13, observation of 2026-11-23.

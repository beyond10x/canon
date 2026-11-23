---
format: aep.planning-md/3
id: story:hyphenated-flags-in-ess-command-test
kind: story
status: draft
title: adversary_ess_commands accepts a flag name with a hyphen
relations:
- decomposes: epic:canon-kernel
revision: 1
---
## Outcome

`adversary_ess_commands` accepts a flag whose name holds a hyphen, so a new option such as `--strict-skew` does not need a workaround in the test.

## Why

- `crates/canon-cli/tests/adversary_ess_commands.rs:236-272` compares flag names verbatim against the spec; unit 5 of wave 12 (`--strict`) found the comparison blocks hyphenated names.
- Found while building `canon evaluate --strict [<SKEW>]`; the unit worked around it. The workaround stays until the comparison is fixed.

## Scope

- **In:** the flag-name comparison in `adversary_ess_commands.rs`.
- **Out:** the CLI options themselves.
- **Surfaces:** `crates/canon-cli/tests/adversary_ess_commands.rs`.

## Acceptance

A test with a spec flag `--two-words` and a CLI option of that name passes, and renaming the CLI option makes it fail.

## Source

- **Request:** wave 2026-11-22-w12, unit 5 report.

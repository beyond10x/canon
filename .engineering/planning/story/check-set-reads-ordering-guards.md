---
format: aep.planning-md/3
id: story:check-set-reads-ordering-guards
kind: story
status: draft
title: canon check-set checks runs_only_when guards across protocol files
relations:
- decomposes: epic:canon-kernel
revision: 1
---
## Problem

Wave 14 unit 1 added the action key `runs_only_when` and `canon check` proves it per protocol. `canon check-set` and the finding `unproduced-evidence` do not read guards (unit 1 report, "left out"), so a guard that names evidence only another protocol of the set produces is not checked across files.

## Acceptance

- `canon check-set` reports a guard that reads an evidence kind no protocol of the set produces, with a finding code of its own, and a fixture under `fixtures/check-across-protocol-files/` for each side.
- Existing `check-set` output on every fixture without a guard is byte-identical.

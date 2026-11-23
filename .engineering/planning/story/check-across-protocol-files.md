---
format: aep.planning-md/3
id: story:check-across-protocol-files
kind: story
status: draft
title: canon check resolves opened protocol ids, named child outcomes and the producers of supplied evidence
relations:
- decomposes: epic:canon-kernel
revision: 1
---
## Outcome

A check over a set of protocol files fails when a parent opens a protocol that does not exist, names a child outcome the child never declares, or reads a supplied kind that no action of its supplier produces.

## Why

- Gap 1: case_open.protocol and supplied_by are named and never resolved; canon check passes such a parent in silence. The consumer wrote a script over the IR to do it.
- Reported by the session `bf-runbook` (dev-operator), 2026-11-22, from `canon-gaps.md` in the dev-operator spec directory (22 composed protocols, `runbook` 0.5.0 linking Canon 0.11.0). Canon has not reproduced it; the file quotes the exact error texts.

## Scope

- **In:** the outcome above.
- **Out:** any change to the bytes of existing `protocol/1` inputs; a format change waits for Timo.
- **Surfaces:** to be scoped before a wave.

## Acceptance

A fixture set with each of the three defects fails the check with a named error; the repository's existing fixtures still pass byte for byte.

## Source

- **Request:** `bf-runbook`, gap list of 2026-11-22.

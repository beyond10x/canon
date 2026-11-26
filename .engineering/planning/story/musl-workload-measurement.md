---
format: aep.planning-md/3
id: story:musl-workload-measurement
kind: story
status: active
title: gnu, musl and musl with mimalloc are measured on one workload before a default is named
relations:
- decomposes: epic:linux-release-archives
- serves: vision:governed-autonomy
- depends_on: story:linux-release-archives
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-26T15:34:04Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-26T15:34:04Z", actor: "human:operator", revision: 3}
---
## Outcome

One Canon workload is measured with three release builds side by side (gnu 2.17, musl with the
system allocator, musl with mimalloc), and the result decides which archive the installation page
names first and whether the musl build links mimalloc.

## Why

musl's allocator is known to be slower than glibc's under allocation-heavy work; the owner asked
for a measurement before musl is called the default, and for mimalloc only if it is not slower.

## Scope

- **In:**
  - the workload: `canon conform run` over the conformance suite plus `canon check` of the largest
    protocol fixture, each run 10 times per build on one idle host, wall time and peak RSS, median
    and spread;
  - a script that runs the measurement without the `slot` prefix (it measures; the builds inside
    it take slots through the shim);
  - the table and the decision committed under `docs/design/linux-release.md`;
  - mimalloc behind a cargo feature used only by the musl build, enabled only when its median is
    not above the system allocator's;
  - the decision on the default goes to the operator with the table.
- **Out:** tuning Canon's own allocation.
- **Surfaces:** `scripts/measure-release.sh`, `docs/design/linux-release.md`,
  `crates/canon-cli/Cargo.toml` (feature), `scripts/package.sh`.

## Acceptance

The committed table holds 10 runs per build and workload, its medians, and the commit measured;
the musl archive links mimalloc only when the table shows it not slower; the installation page's
default matches the table's decision.

## Source

- The owner, 2026-11-25, option A of the Linux release question.

---
format: aep.planning-md/3
id: story:linux-release-archives
kind: story
status: implemented
title: package.yml builds checked gnu 2.17 and static musl archives of canon
relations:
- decomposes: epic:linux-release-archives
- serves: vision:governed-autonomy
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-26T15:34:04Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-26T15:34:04Z", actor: "human:operator", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-26T17:52:01Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A workflow `package.yml` (on a tag and on demand) and a local task `task package -- <target>`
build `canon` for `x86_64-unknown-linux-gnu` with `cargo zigbuild --target
x86_64-unknown-linux-gnu.2.17` and for `x86_64-unknown-linux-musl`, check both, and produce
`canon-<version>-<target>.tar.gz` plus `SHA256SUMS` as workflow artifacts.

## Why

Epic outcome; Canon ships no Linux asset today.

## Scope

- **In:**
  - the build of both targets in one script used by the workflow and the task (the local gate
    equals the forge); cargo-zigbuild and zig pinned by version and checksum;
  - `objdump -T` of the gnu binary: the highest `GLIBC_` version is 2.17 or lower, else fail
    naming the symbol;
  - the musl binary: `readelf -l` shows no `INTERP`, else fail;
  - a smoke run of each packaged binary: `canon --version` equals the workspace version and
    `canon conform run` over the suite passes;
  - archives and `SHA256SUMS` uploaded as workflow artifacts only; the workflow gets no write
    permission. Attaching them to the GitHub Release is a release step run through the bot
    (`b10x-gates gh run download`, `b10x-gates gh release upload`), written in `AGENTS.md`
    § Releases, which today says there is no binary packaging;
  - the installation page names both archives.
- **Out:** the default choice and mimalloc (story:musl-workload-measurement).
- **Surfaces:** `.github/workflows/package.yml`, `scripts/package.sh`, `Taskfile.yml`,
  `AGENTS.md`, `website/docs/` installation page.

## Acceptance

`task package -- x86_64-unknown-linux-gnu` and `-- x86_64-unknown-linux-musl` each leave an
archive and its checksum line; a gnu binary built without the 2.17 suffix fails the glibc check
naming a symbol above 2.17 (negative control); a dynamically linked binary fails the musl check
(negative control); the workflow run on the pull request produces both artifacts. Each pool job
of the build ends within 10 minutes.

## Source

- The owner, 2026-11-25, option A of the Linux release question.

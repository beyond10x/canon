---
format: aep.planning-md/3
id: epic:linux-release-archives
kind: epic
status: active
title: 'Linux release archives: static musl beside gnu with a glibc 2.17 floor'
summary: Each release carries x86_64 gnu (glibc <= 2.17) and static musl archives of canon; the default is chosen on a measured workload.
relations:
- serves: vision:governed-autonomy
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-26T15:34:04Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-26T15:34:04Z", actor: "human:operator", revision: 3}
---
## Outcome

Each Canon release carries two Linux x86_64 archives of the `canon` binary beside its source
archives: `x86_64-unknown-linux-gnu`, linked against glibc 2.17 at most, and
`x86_64-unknown-linux-musl`, fully static. Which one is the documented default is decided on a
measured workload, and mimalloc is used only where it is measured not slower.

## Why

Canon releases are source releases; a consumer that runs `canon` on Linux builds it first, and a
host with an older glibc cannot run a binary built on a current runner. The owner chose on
2026-11-25 to ship static musl archives beside glibc ones, with a glibc 2.17 floor for the gnu
build checked by `objdump -T`, and to measure before calling musl the default.

## Scope

- **In:** a packaging workflow and a local task that build both archives the same way; the
  glibc-floor and static-link checks; a smoke run of each archive; `SHA256SUMS`; attaching the
  archives to the GitHub Release through the bot; a side-by-side measurement of gnu, musl and musl
  with mimalloc on one workload; the release notes in `AGENTS.md` and the installation page.
- **Out:** other architectures and operating systems; crates.io; package managers; signing.

## Acceptance

A release candidate build produces `canon-<version>-x86_64-unknown-linux-gnu.tar.gz` and
`canon-<version>-x86_64-unknown-linux-musl.tar.gz` with `SHA256SUMS`; `objdump -T` of the gnu
binary names no `GLIBC_` version above 2.17; the musl binary has no program interpreter; each
archive's `canon` passes the conformance suite; the measurement table is committed with the
default it supports.

## Source

- The owner, 2026-11-25: option A of the Linux release question (musl archives beside gnu,
  glibc 2.17 floor, measure first, mimalloc only if not slower).

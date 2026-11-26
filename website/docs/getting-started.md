---
title: Getting started
description: Build the canon command line from source, then validate, compile and evaluate a protocol.
---

# Getting started

Install source release `0.16.0`. You need `rustup`: the repository's
`rust-toolchain.toml` pins the compiler (1.100.0) and rustup installs it on first use;
the code uses the 2024 edition. The `canon-cli` crate installs a binary named `canon`.

```bash
git clone --branch 0.16.0 https://github.com/beyond10x/canon.git
cd canon
cargo install --locked --path crates/canon-cli
canon --help
```

## Linux binaries

:::planned[Linux x86_64 archives]

From the next release, each release also carries two archives of the `canon` binary for Linux
x86_64 beside its source, and a `SHA256SUMS` file naming their checksums:

- `canon-<version>-x86_64-unknown-linux-musl.tar.gz`, the default: statically linked, with no C
  library needed on the host, and the mimalloc allocator;
- `canon-<version>-x86_64-unknown-linux-gnu.tar.gz`, linked against glibc 2.17 or older, for a
  host where you prefer glibc. Its peak memory was about half the musl archive's.

On the measured workloads the musl archive was not slower than the gnu one; the measurement and
the rule that chose the default are in the repository's `docs/design/linux-release.md`.

Each holds the binary, `LICENSE` and `README.md` in a directory `canon-<version>-<target>/`.
Release `0.16.0` carries neither; build it from source as above.

:::

## Validate a protocol

The repository ships a small protocol, an investigation. Run this from the repository root:

```bash
canon validate --path fixtures/investigation/protocol.yaml
```

A valid document exits with status 0 and prints one line naming the protocol and its revision. An
invalid one exits with status 1 and prints one line per problem on standard error, each tagged with
its [problem code](./reference/validation.md#problem-codes).

## Compile it

```bash
canon compile --path fixtures/investigation/protocol.yaml > investigation.ir.json
```

This writes the protocol's [`canon-ir/1`](./reference/canon-ir.md) as JSON: keys sorted, defaults
written out, one byte sequence for one meaning.

## Evaluate a case

An evaluation needs the compiled protocol, a `canon-case/1` snapshot of the case and a directory of
`canon-evidence/1` records ([evaluation documents](./reference/documents.md)):

```bash
canon evaluate --ir investigation.ir.json --case case.yaml --evidence evidence/
```

It prints a `canon-decision/1` document giving every claim the value `true`, `false` or `unknown`,
each declared obligation `open` or `discharged`, each declared action `admissible`,
`approval-required` or `blocked`, and each declared outcome `legitimate` or `blocked`. Two options
add inputs: `--authority` names a `canon-authority/1` document of granted and denied capabilities,
and `--at` gives the evaluation instant against which evidence expires. The
[worked example](./reference/investigation-example.md) shows the inputs and the output for
several evidence situations, and the [evaluation reference](./reference/evaluation.md) gives the
rules.

## Run the conformance scenarios

```bash
canon conform run
```

From the repository root this runs every [`canon-conformance/1` scenario](./reference/conformance.md)
in `conformance/scenarios`. Scenarios can compile a fixture and evaluate a case, with the
authority decisions and evaluation instant a step names.

```bash
canon conform run --requirements conformance/requirements.yaml
```

adds the checks against the [requirements catalogue](./reference/conformance.md#the-requirements-catalogue):
every requirement Canon implements is covered by a passing scenario, and every evaluation gives the
same decision when run again and with its evidence, authority and decisions reversed.

## Next

- Every command and option: [the CLI reference](./reference/cli.md).
- Every key of a protocol: [the `protocol/1` reference](./reference/protocol.md).

Contributors run `task check`, which also needs the [Task runner](https://taskfile.dev/) and the ESS
release pinned in `ess/ess-inputs.yaml`.

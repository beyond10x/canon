---
title: Getting started
description: Build the canon command line from source, then validate, compile and evaluate a protocol.
---

# Getting started

There is no release yet, so build `canon` from source. You need a current stable Rust toolchain;
the code uses the 2024 edition. The `canon-cli` crate installs a binary named `canon`.

```bash
git clone https://github.com/beyond10x/canon.git
cd canon
cargo install --locked --path crates/canon-cli
canon --help
```

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

It prints a `canon-decision/1` document giving every claim the value `true`, `false` or `unknown`.
The [worked example](./reference/investigation-example.md) shows the inputs and the output for
several evidence situations. Only claims are evaluated today.

## Run the conformance scenarios

```bash
canon conform run
```

From the repository root this runs every [`canon-conformance/1` scenario](./reference/conformance.md)
in `conformance/scenarios`. Scenarios can compile a fixture and evaluate a case; the `authority`
and `at` inputs a scenario may name are not read yet.

## Next

- Every command and option: [the CLI reference](./reference/cli.md).
- Every key of a protocol: [the `protocol/1` reference](./reference/protocol.md).

Contributors run `task check`, which also needs the [Task runner](https://taskfile.dev/) and the ESS
release pinned in `ess/ess-inputs.yaml`.

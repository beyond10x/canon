# Canon

A formal language and deterministic calculus for evidence-governed protocols.

Canon answers, for one case under one protocol: what is known, what is unknown, which obligations
remain open, which actions are admissible, which need authority, and whether an outcome has been
earned. Claims are three-valued (`TRUE`, `FALSE`, `UNKNOWN`); evidence is bound to the revision it
was observed on, so evidence for a superseded revision leaves a claim `UNKNOWN` rather than `TRUE`.

The `b10x-canon-expr` library adds typed catalog assertions: exact comparisons, recipes,
deduplicated acquisition plans and pure evaluation over retained observations. Read the
[assertion guide](https://beyond10x.github.io/canon/docs/concepts/assertions) or its
[source](website/docs/concepts/assertions.md).

[Documentation](https://beyond10x.github.io/canon/) covers the protocol model, CLI and evidence
semantics. [Engineering Protocols](https://beyond10x.github.io/engineering-protocols/)
([GitHub](https://github.com/beyond10x/engineering-protocols)) supplies engineering vocabulary and
acquisition; Canon remains independent of that domain.

## Status

Release **0.17.0**. `b10x-canon` implements protocol parsing, validation,
compilation, evaluation, semantic diff and a walk of the states a case can reach, with protocols composed from pinned imports, case inputs, predicates over evidence order, cases that open child cases and read their outcomes, evidence requirements that demand an independent producer, scoped capabilities with authority decisions bound to a case revision and to the grounds they rested on, one argument schema per action with a validator a runner calls, actions declared read-only or write, trigger rules that say which rows of a snapshot start a protocol, evaluated to a plan and checked before they are published, and a normative requirements catalogue its conformance suite is checked against; `canon-cli` exposes those operations. `b10x-canon-expr` is the
generic assertion library. See the [current status](https://beyond10x.github.io/canon/docs/status/where-this-stands)
and [release notes](CHANGELOG.md).

## Install

On Linux x86_64, download one archive from the
[GitHub Release 0.17.0](https://github.com/beyond10x/canon/releases/tag/0.17.0), check it against
`SHA256SUMS` and put `canon` on your `PATH`:

- `canon-0.17.0-x86_64-unknown-linux-musl.tar.gz`, the default: static, no C library needed, with
  the mimalloc allocator;
- `canon-0.17.0-x86_64-unknown-linux-gnu.tar.gz`: linked against glibc 2.17 or older.

```console
sha256sum --check --ignore-missing SHA256SUMS
tar -xzf canon-0.17.0-x86_64-unknown-linux-musl.tar.gz
canon-0.17.0-x86_64-unknown-linux-musl/canon --version
```

Elsewhere, or to build from source:

```console
cargo install --locked --git https://github.com/beyond10x/canon --tag 0.17.0 canon-cli
```

The library crates `b10x-canon` and `b10x-canon-expr` can likewise be pinned to Git tag `0.17.0`.
Nothing is published to crates.io.

## Build

```console
task check
```

## Licence

Apache-2.0.

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

Source release **0.1.0**. `b10x-canon` implements protocol parsing, validation,
compilation and evaluation; `canon-cli` exposes those operations. `b10x-canon-expr` is the
generic assertion library. See the [current status](https://beyond10x.github.io/canon/docs/status/where-this-stands)
and [release notes](CHANGELOG.md).

## Install

```console
cargo install --locked --git https://github.com/beyond10x/canon --tag 0.1.0 canon-cli
```

The library crates `b10x-canon` and `b10x-canon-expr` can likewise be pinned to Git tag `0.1.0`.
This release provides source archives; it does not publish binaries or crates to crates.io.

## Build

```console
task check
```

## Licence

Apache-2.0.

# Canon

A formal language and deterministic calculus for evidence-governed protocols.

Canon answers, for one case under one protocol: what is known, what is unknown, which obligations
remain open, which actions are admissible, which need authority, and whether an outcome has been
earned. Claims are three-valued (`TRUE`, `FALSE`, `UNKNOWN`); evidence is bound to the revision it
was observed on, so evidence for a superseded revision leaves a claim `UNKNOWN` rather than `TRUE`.

Canon knows nothing about engineering, agents or model providers. Domains are written on top of it;
the first is [ELS](https://github.com/beyond10x/els), the engineering domain.

## Status

Bootstrap. The crate `b10x-canon` holds the first scaffold types; the kernel is being built against
the plan in [`docs/design/canon-protocol-calculus-design.md`](docs/design/canon-protocol-calculus-design.md)
and the contract sketch in [`docs/contracts/protocol-core.md`](docs/contracts/protocol-core.md).

Planned crates: model, parse, validate, compile, IR, eval, explain, diff, conformance, CLI.

## Build

```console
task check
```

## Licence

Apache-2.0.

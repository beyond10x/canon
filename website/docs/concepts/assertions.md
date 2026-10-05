---
title: Assertions with collected evidence
sidebar_position: 5
description: How typed catalog expressions become deterministic evaluations over explicit observations.
lede: A catalog expression states an exact condition, and a retained evidence set makes its evaluation replayable.
source: crates/canon-expr/src; crates/canon-expr/tests/conformance.rs; crates/canon-expr/tests/adversary.rs
---

# Assertions with collected evidence

The source tree includes `b10x-canon-expr`, a Rust library implementing `canon-expr/1`. It is a
new source capability, with no release claimed here. Its language, catalog and evidence contracts
are generic. [Engineering Protocols](https://beyond10x.github.io/engineering-protocols/)
([GitHub](https://github.com/beyond10x/engineering-protocols)) supplies engineering vocabulary,
provider registration and the gate command line.

An assertion has a Boolean result. A catalog might declare a `quality` observation returning a
closed record, allowing `quality.score > 0.8 and quality.regressions == 0`. The catalog owns
those names and their types. Catalog additions do not change the grammar.

## From source to a decision

The lexer uses Logos to recognize tokens with byte spans. Identifiers use ASCII; quoted strings
use JSON escaping and can contain Unicode. A handwritten Pratt parser builds a bounded syntax
tree. Its precedence runs from `or`, through `and`, `not`, comparisons, unary minus, and finally
calls, fields and indexes. Thus `not a == b` means `not (a == b)`. Comparisons cannot chain without
explicit parentheses. `(fact).field` explicitly traverses a record; `fact.field` first resolves
as a catalog name.

The checker resolves catalog signatures before any provider runs. It checks named and positional
arguments, defaults, record fields and Boolean roots, then expands recipes. Recipes have only
their declared parameters in scope. Cycles, excessive expansion and unknown variables are errors.

Planning binds variables and creates a deduplicated list of observations. Observation arguments
must be calculable without another observation. All requests are collected even when an assertion
contains `false and observe()` or `true or observe()`. Boolean logic controls the result, not
acquisition scheduling.

The evaluator receives a plan, observations, source and context identities, and an explicit time.
It reads no filesystem, clock or network. A matching observation supplies a typed value;
missing, stale or unavailable observations produce `UNKNOWN`. Malformed evidence is an error,
including in a Boolean branch whose result would otherwise be masked. Only `TRUE` satisfies an
assertion; an empty assertion set is refused.

## Exact values and replay

Integers and decimal coefficients fit signed i128. Decimal values carry a scale rather than a
floating point approximation, so a threshold such as `0.8` has exact semantics. Overflow is an
error. Record fields are closed, collections have one element type, and values have no implicit
truthiness.

An optional value with no member is a known absence. It differs from unavailable evidence.
The `Present` pure builtin can test that absence; optional values cannot be traversed as if they
were records. Empty collections and optional absence need a type supplied by a signature or
explicit checking context. Serialized plans retain that context, including types that an absent
value alone cannot reveal.

Plan identities bind the language, sources, catalog, context schema and bound values. Request
identities also bind provider digests, arguments and source/context identities. The embedding
runner supplies identities that cover relevant dirty and untracked inputs as well as configuration.
Decoding a stored plan recompiles it and verifies the retained identity and acquisition list.
Replaying evaluation needs only the retained inputs.

## Extending and maintaining the language

A catalog entry can invoke a typed observation provider, expand a recipe using existing entries,
or select a fixed pure builtin. Provider processes and their supervision belong to the embedding
runner. A new domain namespace uses the same parser and evaluator. New syntax, operators or value
semantics require a language-version compatibility decision; a changed function signature or
provider implementation changes the bound catalog or provider identity.

The implementation keeps lexing/parsing, checking/planning, exact numbers, wire types and pure
evaluation in separate modules. Conformance tests exercise numeric boundaries, complete
three-valued truth tables, recipes, canonical formatting, malformed tokens, replay and resource
limits. Independent adversarial cases remain in the suite. The [generated contract reference](../reference/assertions.md)
lists the current wire model and limits directly from the source.

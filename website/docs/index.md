---
title: What Canon is
sidebar_label: What Canon is
slug: /
description: Canon is a formal language and deterministic calculus for evidence-governed protocols.
---

# Canon

**What is known. What is owed. What has been earned.**

Canon is a formal language and a deterministic calculus for evidence-governed protocols. For one
case under one protocol it is meant to answer: what is known, what is unknown, which obligations
remain open, which actions are admissible, which need authority, and whether an outcome has been
earned.

:::note[Status: bootstrap]

Canon is early. Today it parses, validates and compiles protocols, evaluates every claim of a
case in three-valued truth (`canon evaluate`), applying evidence only to the revision it is bound
to, and runs conformance scenarios. Obligations, action admissibility, outcomes and freshness are
not built yet. Every page says which parts are shipped
and which are planned; [where this stands](./status/where-this-stands.md) has the full list.

:::

## Legitimacy, not sequencing

A workflow answers "what executes next?". Canon answers "what is currently legitimate?". A protocol
declares the claims that can be made, the evidence that can establish them, the actions that are
possible and the outcomes under which a case may conclude. Canon evaluates that declaration; it
does not do the work.

- **Declared as data.** A protocol is a YAML document in the `protocol/1` format. Canon parses it,
  checks every reference and compiles it into `canon-ir/1`, a normalized form whose bytes are
  stable enough to hash.
- **Pure and deterministic.** The library reads no clock, environment, network or filesystem.
  Callers pass documents in. The same normalized input is meant to give the same normalized output,
  byte for byte.
- **Domain-neutral.** Canon contains no engineering vocabulary and knows nothing about agents or
  model providers. Domains are written on top of it; the first is
  [ELS](https://github.com/beyond10x/els), the engineering domain.

## What Canon does not do

Canon does not execute anything. It is not a workflow engine, planner, scheduler, issue tracker,
database, agent harness, authorization server or CI runner. Systems that do those things can
consume what Canon decides.

## Where to go next

- [Getting started](./getting-started.md): build `canon`, validate and compile a protocol.
- [The `protocol/1` language](./concepts/protocols.md): how a protocol is put together.
- [Three-valued truth](./concepts/three-valued-truth.md) and
  [evidence bound to revisions](./concepts/evidence-and-revisions.md): the two ideas Canon is built
  around.
- [A worked example](./reference/investigation-example.md): one protocol, validated and compiled.

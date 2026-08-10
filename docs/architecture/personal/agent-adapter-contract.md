# Universal Agent Adapter Contract

- Status: informative Personal architecture design
- Change class: product-semantic documentation
- Related: [ADR-0043](../../adr/0043-personal-universal-agent-adapter.md),
  [ADR-0044](../../adr/0044-personal-multi-agent-mainline.md), P8-T02/P8-T03
- Normative machines: unchanged; future `agent-adapter-manifest` requires Lane-CTR

## 1. Purpose

Define how any third-party Agent integrates with CognitiveOS Personal without
becoming an authority writer. Pi remains the only Linux 1.0 qualified adapter;
this contract is the reusable shape for later independent qualification.

## 2. Protocol boundary

- **Inbound/outbound adaptation protocol:** AKP envelopes only.
- **Discovery semantics:** may align with A2A Agent Card *fields* (name,
  capabilities, versions) as local metadata; **no** default public network
  listener, TLS PKI, or service mesh.
- **Tool ecosystems (MCP etc.):** optional post-1.0 capability trains; they do
  not bypass Intent/Effect or Tool validators.

## 3. Required adapter capabilities

1. **Declare:** package identity, protocol version, supported operations,
   candidate-only guarantee, and channel requirements.
2. **Register / install / activate / rollback / uninstall** under daemon-owned
   Runtime lifecycle facts (same lifecycle vocabulary as managed Pi).
3. **Translate** Agent I/O ↔ AKP candidates/observations; never mint Task,
   Effect, Verification, budget, or capability authority.
4. **Isolate** Task vs management credentials, caches, cursors, and retry state.
5. **Fail closed** when the Agent emits authority-shaped payloads, secret
   material, or out-of-lease mutations.

## 4. Lifecycle (logical)

```text
acquire → verify → install → register → activate
                 ↘ rollback
stop → uninstall / quarantine
```

Each transition appends durable Runtime facts. Evidence from one Agent never
qualifies another (B09-style independence).

## 5. Negative properties (design)

- Agent self-report of completion ≠ Task acceptance.
- Sidecar crash ≠ Effect reconciliation success.
- Capability expansion from Agent proposal without daemon attenuation is
  rejected.
- Cross-tenant or cross-scope leakage fail closed (RFC-0001 compatible).

## 6. Non-claims

This file is design only. Implementation and machine contracts land in P8-T02
and Lane-CTR. No Gate/release/Profile claim.

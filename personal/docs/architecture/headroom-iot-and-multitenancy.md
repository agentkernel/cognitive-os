# Headroom: IoT / Embodied and Enterprise Multi-Tenancy

- Status: architecture headroom only (not a formal plan task)
- Related: [ADR-0045](../../../docs/adr/0045-personal-os-positioning.md),
  RFC-0001, CognitiveOS embodied/safety domain notes in the whitepaper

## 1. Purpose

Reserve design vocabulary so future IoT/embodied and enterprise multi-tenant
work can attach without inventing a parallel OS. **No Phase 8/9 task** is opened
by this chapter.

## 2. IoT / embodied bridge (reserved)

- Host OS / device firmware remain outside Personal's claim surface.
- Embodied safety arbitration (CognitiveOS dual-kernel vocabulary) may later
  consume Personal Task/Effect evidence as slow-path cognition, never the reverse.
- Edge agents would still be AKP candidates under local or gateway daemons;
  resource budgets and fencing remain mandatory.

## 3. Enterprise multi-tenant bridge (reserved)

- RFC-0001 tenant/scope/purpose/retention already shape governed objects.
- A future control plane would add policy distribution and attestation, not a
  second authority writer inside a Personal node.
- Personal Linux 1.0/2.0 design remains owner-local single-principal by default.

## 4. Non-claims

This chapter creates no implementation commitment, Gate, release, Profile, IoT
certification, or multi-tenant SaaS claim.

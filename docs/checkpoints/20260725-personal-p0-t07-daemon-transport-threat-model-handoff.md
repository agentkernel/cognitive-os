# 20260725 Personal P0-T07 Daemon Transport / Threat Model Handoff

## 1. Task Snapshot

- Task: `P0-T07` — daemon transport、认证和威胁模型
- Date: 2026-07-25
- Branch: `lane/personal-p0-t07-daemon-transport-threat-model`
- Base commit: `6848782` (`main` @ PR #90 / P0-T05)
- Lane: Personal / Lane-DOC design freeze (does not take Lane-RUN ownership of
  `cognitive-runtime` or implement business routes)
- Status: **done** (design freeze + formal ledger update; no daemon
  implementation claimed)

## 2. Completed in this atomic batch

- Added ADR-0019:
  `docs/adr/0019-personal-daemon-transport-auth-threat-model.md`
  - Default listen: HTTP/1.1 over Unix domain socket under XDG runtime
  - Optional loopback TCP (`127.0.0.1` / `[::1]`) for tests only
  - Disabled-by-default listener until explicit daemon start
  - Channel-scoped bearer bootstrap; cookies forbidden
  - Resource bounds table for P1-T04
  - Threat model covers CSRF, DNS rebinding, token theft, channel confusion,
    replay (plus remote exposure / secret-log non-claims)
- Aligned root `plan.md` P0-T07 task card to ADR-0019; moved planned license
  ADR reference on P0-T03 from ADR-019 to ADR-0020 to avoid number collision
- Updated formal Personal ledger: P0-T07 `done`; Phase 0 done count 5 / 2
  remaining (`P0-T03`, `P0-T06`)
- Updated `docs/plan/PROGRESS.md` Personal row and recent handoff list
- Indexed ADR-0017/0018/0019 in `docs/README.md`

## 3. Not completed / out of scope

- No changes to `apps/kernel-server` routes, bounds enforcement, or live auth
  (owned by P1-T04 after P1-T01 path layout).
- No UDS/TCP listener implementation, session token minting, or security tests.
- P0-T03 (owner license/platform/distribution GO/NO-GO) still not-started.
- P0-T06 (Pi PoC) remains blocked on P0-T03.
- G0 / B01-B12 / Profile claims are **not** made by this batch.

## 4. Tests and evidence

| Check | Status | Result |
|---|---|---|
| Design acceptance (ADR threat rows present) | pass (document review) | CSRF, DNS rebinding, token theft, channel confusion, replay covered in ADR-0019 §4 |
| Business route / daemon auth implementation | not-run / not-in-scope | Explicitly deferred to P1-T04 |
| pnpm run check:consistency | pass (local + CI) | Docs-only; no registry change. |
| git diff --check | pass (local) | — |
| CI workspace verify | pass | run 30154100260 Ubuntu/Windows success. |
| Personal Gates / B01-B12 / Profile | not-run | No claim |

## 5. Design and safety boundaries

- Clients remain non-authority; daemon remains sole writer once implemented.
- Transport tokens are not Secret Service material (ADR-0018) and must not
  enter SQLite authority tables, logs, evidence, env, or argv.
- ADR-0003 remains the envelope/HTTP+SSE baseline; ADR-0019 is the Personal
  binding for listen path, auth bootstrap, bounds, and threats.
- Same-user local malware residual risk is accepted and documented for v1.

## 6. Next entry

1. PR #91 merged to main at ff341ef; CI run 30154100260 green.
2. Remaining Phase 0:
   - **P0-T03** requires owner license / platform / distribution decision —
     do not invent GO/NO-GO.
   - **P0-T06** depends on P0-T03.
3. Dependency-satisfied after P0-T07: **P1-T01** (migrations + XDG; depends
   only on P0-T04) may start in parallel with waiting on P0-T03, but P1-T04
   still needs both P0-T07 and P1-T01.
4. Suggested prompt: `Continue Personal plan. Read AGENTS.md, PROGRESS.md,
   20260725-personal-p0-t07-daemon-transport-threat-model-handoff.md,
   PARALLEL-LANES.md, PERSONAL-DEVELOPMENT-PLAN.md. If P0-T03 is selected,
   stop and ask owner for license/platform/distribution. Prefer next
   dependency-satisfied task (e.g. P1-T01) without claiming G0/Profile.`

## 7. Snapshot

- PROGRESS updated: yes (P0-T07 done; no Profile claim).
- Formal Personal ledger updated: yes (`done`).
- Commits: d443a0e (branch); merged squash ff341ef on main via PR #91.
- PR: [#91](https://github.com/agentkernel/cognitive-os/pull/91) merged.
- CI: [30154100260](https://github.com/agentkernel/cognitive-os/actions/runs/30154100260) Ubuntu + Windows success.

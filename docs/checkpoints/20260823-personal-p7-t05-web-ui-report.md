# P7-T05 Non-blocking Web UI — running report

- Task: `P7-T05`
- Lease: `lease/personal/P7-T05/web-ui`
- Branch: `personal/P7-T05-web-ui`
- Change class: `product-semantic + structural` (ADR/inventory/docs) plus
  `implementation-only` (Node inventory validator)
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, Agent-benefit, or
  “Web UI implemented” promotion

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. Local `node --test tools/test/p7_t05_web_ui_inventory.test.mjs` **pass**
   (8/8) on `DEV-WIN-GNU-01`:
   - invented generic lifecycle refused
   - invented daemon route refused
   - Task-channel secret-bearing key write refused
   - localStorage session refused
   - browser SQLite/SecretStore/filesystem/shell/provider-direct refused
   - Task cancel and Agent pause recorded `unavailable`/`not-run`
   - detach observation must not invoke cancel/stop
   - canonical inventory matches daemon source and ADR-0053
2. Local Rust build/test **not-run** (`RUST-LINK-DEV-WIN-GNU-01`).
3. Daemon Origin/Referer + `GET /ui` front-door tests written in
   `apps/kernel-server/src/personal/server.rs` (foreign/null/https/wrong-port
   Origin; missing bundle `not_available`; path traversal). Execution
   **not-run** locally; routed to CI-UBUNTU-01 / CI-WINDOWS-MSVC-01 /
   `DEV-LINUX-NATIVE-01`.
4. Browser/SPA journeys **not-run** / **not_available**: no approved
   `cognitiveos-clients` checkout.
5. Linux/native live UI **not-run**: blocked on the same checkout.
6. Secret-redaction SPA negatives **not-run**: no SPA to instrument.
   Inventory proves the key route is management-only and forbids DOM/storage
   persistence in the client policy; that is not live redaction evidence.

## D01

Accepted [ADR-0053](../adr/0053-personal-web-ui-stack.md):

- React + TypeScript + Vite in `cognitiveos-clients/pc/web/`
- daemon same-origin `/ui/` serving; Vite preview is not the product origin
- memory-only sessions; cookies forbidden
- Origin/Referer loopback allowlist **enforced** (`LOCAL_ORIGIN_HEADER_REJECTED`)
- `GET /ui` without a bundle is `503` `not_available` (`LOCAL_UI_BUNDLE_UNAVAILABLE`) with CSP
- MIT/Apache/BSD runtime deps only

Frozen inventory:
[web-ui-route-inventory.json](../architecture/personal/web-ui-route-inventory.json).

Honest missing typed HTTP (UI must render unavailable/not-run):

- Task cancel
- Agent pause / resume / stop / restart / quarantine

## Blocker

- `blocked_paths`: `cognitiveos-clients/pc/web/`
- `blocked_task_ids`: `P7-T05` (D02–D07)
- owner: repository owner
- reason: no approved local checkout of
  [cognitiveos-clients](https://github.com/agentkernel/cognitiveos-clients)
  (GitHub remote exists and is public; sibling and documented paths were
  absent). This session must not create a parallel repository, must not
  recreate `clients/**` in `agent-kernel`, and must not implement the SPA in
  `apps/cognitiveos-console`.
- unique recovery action: clone or register an approved
  `cognitiveos-clients` checkout, then continue D02 on `pc/web/` against the
  exact pushed revision of this task branch.

## Unique next action

Owner provides the approved client checkout. Keep this Draft PR; do not
merge as `done`; do not auto-claim P6 / P7-T06 / P7-T07.

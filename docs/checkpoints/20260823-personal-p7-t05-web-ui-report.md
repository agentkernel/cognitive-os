# P7-T05 Non-blocking Web UI — running report

- Task: `P7-T05`
- Lease: `lease/personal/P7-T05/web-ui`
- Kernel branch: `personal/P7-T05-web-ui`
- Clients branch: `personal/P7-T05-web-ui` @ `987eca0`
- Clients Draft PR: https://github.com/agentkernel/cognitiveos-clients/pull/1
- Approved checkout: `D:\cognitiveos-clients` (`pc/web/`)
- Change class: `product-semantic + structural` (docs) plus
  `implementation-only` (SPA in the official clients repo; daemon front door
  already delivered in D01)
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

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
   Origin; missing bundle `not_available`; path traversal).
4. `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2`, `hal9000`) at exact
   `05a3afa13cfba405d7fb5030810b0e8d4fc47b5c`, worktree
   `/home/wuz/agent-kernel-worktrees/p7-t05-05a3afa1` (removed after the run),
   rustc 1.97.1, `CARGO_TARGET_DIR` shared:
   - `browser_origin_allowlist_rejects_foreign_and_null` **pass**
   - `web_ui_paths_reject_traversal_and_percent_encoding` **pass**
   - `foreign_origin_is_rejected_on_the_front_door` **pass**
   - `matching_loopback_origin_is_accepted_on_health` **pass**
   - `missing_ui_bundle_is_not_available_not_a_fake_spa` **pass**
   - `web_ui_serves_index_with_csp_and_rejects_traversal` **pass**
   - `cargo clippy -p kernel-server --all-targets --locked -- -D warnings` **pass**
   - `cargo fmt --all -- --check` **pass**
   Not B01. Worktree cleaned.
5. Required CI `32614594696` at `05a3afa1` **pass** (Ubuntu, Windows, required-ci).
6. Owner approved clone of official `agentkernel/cognitiveos-clients` to
   `D:\cognitiveos-clients`. Checkout **registered**.
7. Local SPA unit/negatives on `DEV-WIN-GNU-01` in
   `D:\cognitiveos-clients\pc\web\`: `pnpm test` **17/17 pass**
   (secret redaction, forbidden browser targets, unavailable ops, binding
   gates, completion non-claim, escaped markup, memory-only session, channel
   isolation, watch gap/detach, header-injection reject, identity separation,
   DOM redaction). `pnpm build` **pass** (Vite `base: '/ui/'`).
8. Linux/native live UI (browser against daemon-served SPA) **pending** D07:
   requires pushed exact kernel + clients revisions.

## D01

Accepted [ADR-0053](../adr/0053-personal-web-ui-stack.md). Frozen inventory:
[web-ui-route-inventory.json](../architecture/personal/web-ui-route-inventory.json).

Honest missing typed HTTP (UI renders unavailable/not-run):

- Task cancel
- Agent pause / resume / stop / restart / quarantine

## D02–D06

Implemented in `D:\cognitiveos-clients\pc\web\`:

- HashRouter shell, memory-only `POST /local/session` (management + Task)
- Home status/readiness/doctor; Agent inventory/detail with distinct identities
- Provider create + SecretStore key rotate + model probe
- Fixed Agent binding with client-side CAS / no-fallback / no per-request override
- Task/Effect/Evidence/Observation; watch stale/disconnect; detach client-only
- Skip link, focus-visible, semantic tables, CSP, responsive layout, latency display

## Unique next action

Push clients `personal/P7-T05-web-ui` and kernel docs. Run Linux/native
daemon-served SPA on exact immutable revisions. Then complete D07 acceptance,
ready/merge, lease close, branch delete, local `main` fast-forward.

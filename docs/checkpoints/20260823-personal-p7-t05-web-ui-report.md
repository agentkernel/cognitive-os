# P7-T05 Non-blocking Web UI — running report and closure

- Task: `P7-T05`
- Status: `done` (pending PR merge / branch delete / main fast-forward in this session)
- Lease: `lease/personal/P7-T05/web-ui` (closed in this change set)
- Kernel branch: `personal/P7-T05-web-ui`
- Kernel Draft PR: https://github.com/agentkernel/cognitive-os/pull/261
- Clients branch: `personal/P7-T05-web-ui` @ `9ed33fd`
- Clients Draft PR: https://github.com/agentkernel/cognitiveos-clients/pull/1
- Approved checkout: `D:\cognitiveos-clients` (`pc/web/`)
- Product daemon revision: `05a3afa13cfba405d7fb5030810b0e8d4fc47b5c`
- Docs/validation kernel revision: `f891f7395bcfb0c0ad8ca5768ddd34fd88b90756`
- Change class: `product-semantic + structural` (docs) plus `implementation-only`
  (SPA in the official clients repo; daemon front door in D01)
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. Local `node --test tools/test/p7_t05_web_ui_inventory.test.mjs` **pass** (8/8)
   on `DEV-WIN-GNU-01`.
2. Local Rust build/test **not-run** (`RUST-LINK-DEV-WIN-GNU-01`).
3. `DEV-LINUX-NATIVE-01` at `05a3afa1`: kernel-server Origin/`GET /ui` **6/6**,
   Clippy `-D warnings`, fmt. Required CI `32614594696` **pass**.
4. Owner-approved official clone registered at `D:\cognitiveos-clients`.
5. Local SPA `pnpm test` **17/17** and `pnpm build` **pass** (`DEV-WIN-GNU-01`).
6. Required CI `32616659449` at kernel docs-head `f891f739` **pass** (Ubuntu,
   Windows, required-ci).
7. `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2`, `hal9000`) at exact kernel
   `f891f739` + clients `9ed33fd` (git bundles; GitHub fetch HTTP/2 failed).
   rustc 1.97.1. Worktree `/home/wuz/agent-kernel-worktrees/p7-t05-f891f739`
   removed after the run. Runtime `/tmp/p7-t05-runtime-f891f739` removed.
   Port 48191 free after cleanup.
   - SPA `pnpm@10.33.2 test` **17/17** **pass**
   - SPA `pnpm build` **pass**
   - kernel-server Origin/`GET /ui` unit **6/6** **pass**
   - `GET /ui/` **200**, title present, no `sk-` / `ss://` in index
   - foreign Origin **403**
   - `GET /ui/agents` **404** (no BrowserRouter fallback; HashRouter used)
   - Chrome headless Home **pass**; `#/session` **pass**; no secret material in DOM
   - `POST /local/session` management + Task **pass**; `/personal/status` **200**
   - management bearer on `/task/effects` **403** (channel mismatch)
   - Live Provider key rotate / SecretStore write **not-run** (no Provider key
     entered; keys only via approved SecretStore / non-logging paths)
   - Click-through Agent/Provider/Task forms after session **not-run** (headless
     dump-dom + API session; not a full UI driver)
   Not B01. No secret payloads in evidence.

## Acceptance mapping

| Formal acceptance | Evidence |
|---|---|
| Agent inventory / distinct identities | SPA Agents + detail pages; identity unit test |
| Typed lifecycle only when HTTP exists | pause/resume/stop/restart/quarantine/cancel render `not-run` |
| Provider account create / rotate / probe | Forms in `pc/web`; live SecretStore **not-run** |
| Secret redaction | Unit + DOM tests; Linux index/chrome dump have no key material |
| Fixed binding, CAS, no fallback | Client-side `acceptBindingMutation` negatives |
| Task/Effect/Evidence/watch/detach | Task page + watch unit tests; completion stays unknown |
| Keyboard / CSP / pinned build | Skip link, focus-visible, CSP meta, exact deps, Vite `base: '/ui/'` |
| Same-origin daemon serving | Linux `GET /ui/` 200 against daemon-served `dist/` |
| Docs / inventory / ADR | This change set + ADR-0053 + inventory `client_checkout.registered` |

## Unique next action

Merge kernel PR #261 and clients PR #1, delete safe task branches, fast-forward
local `main`, confirm clean worktree. Do not auto-claim P6 / P7-T06 / P7-T07.

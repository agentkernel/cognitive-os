# P11-T13 OPC `/ui/` IA Dual Track — closure

- Task: `P11-T13` / slice `P11-T13/D01` (Dual Track empty IA close door)
- Change class: `implementation-only` (daemon-served `/ui/` L1 Today/Projects/Knowledge + Settings + assistant rail; no `core/specs`; no kernel-server lease; Vite is not the product origin)
- Branch: `personal/P11-T13-opc-ia`
- Content HEAD: `e86ddd7b1e3ca97022444b02abc2ecad446f1942`
- Docs-head: `0608e44eff6a5c3e65f8a7c629098746f965efb5`
- Required-CI / PR head: `e4f001794faac190c33e4769a604be45b0f48dad`
- Merge revision: `main@46eebeca4b6c545bb14bc04e72cf506a6d36f702`
- Pull request: [#291](https://github.com/agentkernel/cognitive-os/pull/291) (merged 2026-08-31)
- Lease: `lease/personal/P11-T13/opc-ia` (closed into PARALLEL-LANES §3.1 by this ledger)
- Required CI on content `e86ddd7b`: **SUCCESS** — run [33345249452](https://github.com/agentkernel/cognitive-os/actions/runs/33345249452)
- Required CI on docs-head `0608e44e`: **SUCCESS** — run [33346409425](https://github.com/agentkernel/cognitive-os/actions/runs/33346409425)
- Required CI on PR head `e4f00179`: **SUCCESS** — run [33347348125](https://github.com/agentkernel/cognitive-os/actions/runs/33347348125): `resolve validation route` **SUCCESS** [99353906862](https://github.com/agentkernel/cognitive-os/actions/runs/33347348125/job/99353906862), `verify (ubuntu-latest)` **SUCCESS** [99353917907](https://github.com/agentkernel/cognitive-os/actions/runs/33347348125/job/99353917907) ~3m40s, `verify (windows-latest)` **SUCCESS** [99353917842](https://github.com/agentkernel/cognitive-os/actions/runs/33347348125/job/99353917842) ~12m17s, `required-ci` **SUCCESS** [99355676396](https://github.com/agentkernel/cognitive-os/actions/runs/33347348125/job/99355676396)
- Claim ceiling: `hypothesis`
- Evaluation routing: **OFF**

## Acceptance mapping

D01 covers the T13 Dual Track close door: L1 is Today/Projects/Knowledge + side-foot Settings + assistant rail; no-authority states are empty/unavailable/denied/disconnected/stub (not fake chrome); zero fake Confirm/Approve/Create/Activate on those paths. Complete `/ui/` (Today packet canvas, Windows native, NVDA/200%/host-theme) is **not** claimed. NVDA / 200% layout / host-theme remain hung **not-run**. `DEV-WINDOWS-NATIVE-OPC-01` is unqualified ⇒ native OPC E2E **not-run**.

| Acceptance item | Evidence |
|---|---|
| L1 is Today/Projects/Knowledge; Settings in side-foot; Team/Inbox not L1 | Vitest N6; dump-dom L1 on `DEV-LINUX-NATIVE-01` |
| Empty Project list is empty, not fake OPC chrome; 0 fake action buttons | Vitest N1 |
| 403/503/stub/disconnect are not empty | Vitest N2–N5 |
| Assistant rail has no Approve control | Vitest N7 (control check, not deny-copy word scan) |
| No Project id ⇒ no pending-previews / vault.index / memory list | Vitest N8 |
| HITL announce-only; no Confirm; Today deep-links `#/projects?preview=` | Vitest N9, N15, N16 |
| Vault/Memory 403 does not invent files; no ingest | Vitest N10 |
| Settings StandingApprovalPolicy list-only; no member budget chrome | Vitest N11 |
| Vite is not claimed as product origin | Vitest N12 |
| POST confirm / vault.apply-authority off client whitelist | Vitest N13 |
| Settings Advanced (Linux 1.0) collapsed by default | Vitest N14 |
| `#/hitl`, `#/inbox`, `#/team` are missing routes | Vitest N17 |
| Local Vitest + `tsc`/Vite | **pass** 43 files / 340 tests; CSS 23.25–23. kB range recorded in the running report |
| Host daemon `GET /ui/` | **pass** `127.0.0.1:48691` HTML 200 (`index-eaQPggwn.js`); existing `:48181` untouched |
| Host dump-dom L1 chrome | **pass** both `127.0.0.1:48182` and `:48691` (Today/Projects/Knowledge/Settings present; Team=0; Inbox=0; Vite preview 0) |
| Host CDP click-through | **partial** `48182` L1 / empty honesty / no fake main actions / projects / knowledge **pass**; Settings click `TimeoutError` (hung, not a product fail). `48691` session POST 401 vs the runtime bootstrap file (stale identity after two daemons shared a runtime-root) |
| NVDA / 200% layout / host-theme contrast | **not-run** (card hung) |
| Windows native OPC E2E | **not-run** (`DEV-WINDOWS-NATIVE-OPC-01` unqualified) |
| `DEV-WIN-GNU-01` cargo | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |
| Workspace `required-ci` on `e86ddd7b` | **SUCCESS** run [33345249452](https://github.com/agentkernel/cognitive-os/actions/runs/33345249452) |
| Workspace `required-ci` on `0608e44e` | **SUCCESS** run [33346409425](https://github.com/agentkernel/cognitive-os/actions/runs/33346409425) |
| Workspace `required-ci` on PR head `e4f00179` | **SUCCESS** run [33347348125](https://github.com/agentkernel/cognitive-os/actions/runs/33347348125) |

## Validation

| Unit | Result | Env | Revision |
|---|---|---|---|
| `pnpm test` `clients/pc/web` N1–N17 | **pass** 340/340 | `DEV-WIN-GNU-01` | `e86ddd7b1e3ca97022444b02abc2ecad446f1942` |
| `pnpm run build` (`tsc --noEmit` + Vite) | **pass** | `DEV-WIN-GNU-01` | `e86ddd7b1e3ca97022444b02abc2ecad446f1942` |
| Host daemon `GET /ui/` | **pass** listen + 200 HTML | `DEV-LINUX-NATIVE-01` `127.0.0.1:48691` | `e4f001794faac190c33e4769a604be45b0f48dad` |
| Host dump-dom L1 | **pass** | `DEV-LINUX-NATIVE-01` `:48182` and `:48691` | `e4f001794faac190c33e4769a604be45b0f48dad` |
| Host CDP click-through | **partial** (Settings timeout) | `DEV-LINUX-NATIVE-01` `:48182` | `e4f001794faac190c33e4769a604be45b0f48dad` |
| NVDA / 200% / host-theme | **not-run** | hung | `e4f00179` |
| Windows native OPC E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` | `e4f00179` |
| Rust link on `DEV-WIN-GNU-01` | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | `e4f00179` |
| `verify (ubuntu-latest)` on `e4f00179` | **SUCCESS** [99353917907](https://github.com/agentkernel/cognitive-os/actions/runs/33347348125/job/99353917907) | `CI-UBUNTU-01` | `e4f001794faac190c33e4769a604be45b0f48dad` |
| `verify (windows-latest)` on `e4f00179` | **SUCCESS** [99353917842](https://github.com/agentkernel/cognitive-os/actions/runs/33347348125/job/99353917842) | `CI-WINDOWS-MSVC-01` | `e4f001794faac190c33e4769a604be45b0f48dad` |
| `required-ci` on PR head `e4f00179` | **SUCCESS** [99355676396](https://github.com/agentkernel/cognitive-os/actions/runs/33347348125/job/99355676396) | GitHub Actions run [33347348125](https://github.com/agentkernel/cognitive-os/actions/runs/33347348125) | `e4f001794faac190c33e4769a604be45b0f48dad` |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit, or complete `/ui/` acceptance (A7: local/CI/host dump-dom is hypothesis only). Not a Today packet canvas. Not Team/Inbox L1. Chat Approve is not a Control Plane control. Linux 1.0 six-family pages remain real Advanced/secondary routes. Vite is not the product origin. `48691` session 401 is a disposable-runtime identity collision, not a product fail. Existing host `:48181` was not touched. B01 unused. Do not auto-claim `P11-T02`. Do not unpark `P11-T14`/`T15`. Evaluation routing OFF.

## Deterministic closure

1. Dual Track L1 + N1–N17 Vitest **pass** 340/340 on `DEV-WIN-GNU-01`;
2. required CI [33345249452](https://github.com/agentkernel/cognitive-os/actions/runs/33345249452) **SUCCESS** on `e86ddd7b`, [33346409425](https://github.com/agentkernel/cognitive-os/actions/runs/33346409425) **SUCCESS** on `0608e44e`, and [33347348125](https://github.com/agentkernel/cognitive-os/actions/runs/33347348125) **SUCCESS** on `e4f00179`;
3. PR [#291](https://github.com/agentkernel/cognitive-os/pull/291) merged as `main@46eebeca` on 2026-08-31;
4. lease `lease/personal/P11-T13/opc-ia` moved to §3.1;
5. remote `personal/P11-T13-opc-ia` deleted when GitHub already did after merge; local task branch deleted when safe; local `main` fast-forwarded to the merge plus this status/closure commit.

Unique next: do **not** auto-claim `P11-T02`. Do not unpark `P11-T14`/`T15`. Remaining ready implementation is owner-gated (`P11-T02` Windows host; parked T14/T15). Evaluation routing OFF.

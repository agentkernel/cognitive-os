# P12-T07 Knowledge ingest — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P12-T07` / slice `P12-T07/D01`
- Branch: `personal/P12-T07-knowledge-ingest`
- Lease: `lease/personal/P12-T07/knowledge-ingest`
- Change class: `implementation-only` (daemon-served `/ui/` Knowledge ingest; no new authority writer; no `core/specs`)
- Unique next: Dual Track TS then Draft PR + required CI

Product origin is daemon-served `/ui/`. Vite/canvas is not the product. Ingest is POST `/management/project/v1/vault.import` (owner-paste) then `vault.index.rebuild`. Why this fragment reads GET `vault.index` inject_order + excerpts. Files are not Project authority (`vault.apply-authority` stays off the client whitelist). Import failure keeps the original fields. Secret-shaped paste is not POSTed. Obsidian is not bundled. Host filesystem E2E is `not-run`. Not T08 connections. Not T15. NVDA/200%/host-theme remain hung. Native UI E2E = `DEV-WINDOWS-NATIVE-OPC-01` / `not-run`. `DEV-WIN-GNU-01` cargo is `not-run`.

## Units

| Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|
| Merge PR [#299](https://github.com/agentkernel/cognitive-os/pull/299) (P12-T06) | **pass** | GitHub | `main@a5265b22` | Required CI [33396112669](https://github.com/agentkernel/cognitive-os/actions/runs/33396112669) **SUCCESS** at `89f85f16` (ubuntu 3m35s, windows 15m2s, required-ci 4s). Remote task branch deleted. |
| Claim `lease/personal/P12-T07/knowledge-ingest` | **pass** | `DEV-WIN-GNU-01` | worktree `D:/agent-kernel-wt-P12-T07` on `origin/main@a5265b22` | DOC-REFRAME retained. Evaluation routing OFF. |
| Dual Track TS Knowledge ingest (`knowledgeIngest` + `vault` + `opcIa` + normalize) | **pass** | `DEV-WIN-GNU-01` | working tree | personal-web-ui **391/391**. Import/rebuild; failure keeps original; secret-shaped paste not POSTed; Why this fragment from daemon; apply-authority not a client route. Native UI E2E **not-run**. NVDA/200%/host-theme **not-run**. GNU cargo **not-run**. |
| NVDA / 200% / host-theme | **not-run** | Requires-environment | — | hung; not a P12 close gate |
| Native UI E2E / host FS E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` unqualified | — | not a product fail |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | — | route to `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` |

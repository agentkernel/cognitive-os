# P12-T09 right-rail canvas write — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P12-T09` / slice `P12-T09/D01`
- Branch: `personal/P12-T09-rail-write`
- Lease: `lease/personal/P12-T09/rail-write`
- Change class: `implementation-only` (daemon-served `/ui/` right-rail write; no new authority writer; no `core/specs`)
- Unique next: P12 Remaining = 0; do not auto-claim `P11-T15`

Product origin is daemon-served `/ui/`. Vite/canvas is not the product. The rail walks edit → review → POST `/management/project/v1/assistant.turn` (candidate propose) then POST `/management/project/v1/draft.apply` (owner management write). Chat has no Approve. HITL Confirm stays on the Projects canvas (T06). Empty home / wizard / creating-only Today hide the rail. The rail does not write SecretStore, archive, or authority-confirm. Preview announce is not Approve. Preview bypass is refused: Write to canvas exists only after review. Dual Track: 0 fake Create/Activate/Approve. Pi Linux qualification does not transfer. Not T15. NVDA/200%/host-theme remain hung. Native UI E2E = `DEV-WINDOWS-NATIVE-OPC-01` / `not-run`. `DEV-WIN-GNU-01` cargo is `not-run`.

## Units

| Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|
| Merge PR [#301](https://github.com/agentkernel/cognitive-os/pull/301) (P12-T08) | **pass** | GitHub | `main@4afc28b9` | Required CI [33418686755](https://github.com/agentkernel/cognitive-os/actions/runs/33418686755) **SUCCESS** at `21036106`. Remote task branch retained historically. |
| Claim `lease/personal/P12-T09/rail-write` | **pass** | `DEV-WIN-GNU-01` | worktree `D:/agent-kernel-wt-P12-T09` from `origin/main@4afc28b9` | Dirty `d:\agent-kernel` and `D:/agent-kernel-wt-P12-T07` not overwritten. DOC-REFRAME retained. Evaluation routing OFF. |
| Dual Track TS right-rail write (`assistant` + `RailCanvasWrite` + `railWrite` + `opcIa` + normalize) | **pass** | `DEV-WIN-GNU-01` | `1017aa19` | personal-web-ui **417/417** (55 files). Edit → review → `assistant.turn` then `draft.apply`; Enter opens review without posting; Discard posts nothing; secret-shaped paste not POSTed; turn 403 does not apply; apply 409 is not success; empty home hides the rail. Native UI E2E **not-run**. NVDA/200%/host-theme **not-run**. GNU cargo **not-run**. |
| `check:consistency` | **pass** | `DEV-WIN-GNU-01` | `1017aa19` | OK (275 requirements, 55 error codes, 74 schemas, 89 vectors, leases). |
| Draft PR [#302](https://github.com/agentkernel/cognitive-os/pull/302) | **pass** | GitHub | `1017aa19` | Unique next = required CI. |
| Required CI [33427885119](https://github.com/agentkernel/cognitive-os/actions/runs/33427885119) | **pass** | GitHub | `e0343853` | resolve SUCCESS; ubuntu 3m56s SUCCESS; windows 12m53s SUCCESS; required-ci SUCCESS. Unique next = close T09 (P12 Remaining=0). |
| NVDA / 200% / host-theme | **not-run** | Requires-environment | — | hung; not a P12 close gate |
| Native UI E2E / Pi Linux | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` / Pi pin | — | Pi qualification does not transfer; not a product fail |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | — | route to `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` |

# P14-T08 Knowledge v9 IA — closure

- Task: `P14-T08` / slices `P14-T08/D01` + `P14-T08/D02`
- Change class: `implementation-only` (Knowledge `/ui/` IA on existing Vault `vault.import` + P13-T07 labeled Memory; **no new numbered migration**; no Obsidian)
- Lease: `lease/personal/P14-T08/knowledge-ia` (close this delivery → PARALLEL-LANES §3.1)
- Branch: `personal/P14-T08-knowledge-ia` (worktree `D:\agent-kernel-p14-t08`; Draft PR [#328](https://github.com/agentkernel/cognitive-os/pull/328))
- Validated implementation HEAD: `899b65f6` (required CI [33975024580](https://github.com/agentkernel/cognitive-os/actions/runs/33975024580) **SUCCESS**: resolve 4s, ubuntu 4m32s, windows 15m7s, required-ci 3s)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI / guest `/ui/` is not Gate / release / Profile / Windows qualification)
- Evaluation routing: **OFF**
- Product origin: daemon-served `/ui/` on `B01-Desktop-Linux-002` (Vite is not the product origin)
- Host FS / native file-picker E2E: `not-run`

## 1. Acceptance mapping (formal plan P14-T08 + EVAL-016 J5)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| Knowledge = v9 files / Why this fragment / import IA, not HTTP-paste-only | `KnowledgePage` `data-knowledge-ia=v9`; tabs Files / Import / Why this fragment / Memory; import uses `input[name=vault-files]` + existing `POST vault.import` | Dual Track no-project lock; live `/ui/` four tabs + file input | Dual Track `knowledgeIa`; live J5 |
| Backed by existing Vault authority (P13-T07 / P12-T07) | No `vault.apply-authority`; files remain not a Charter | Dual Track file-as-authority refused | `knowledgeIngest` N1 |
| Secret ingest refused | `containsSecretMaterial`; no POST; fields stay | Dual Track secret-shaped paste/file | `knowledgeIngest` N2 |
| 0 fake Admit; chat auto-admission Requires-backend | Memory tab copy + no Admit buttons; inspect/correct/promote/forget retained | Dual Track + live Memory tab | `knowledgeMemory`; live J5 |
| Obsidian not bundled | Honesty note names a companion Markdown app, not Obsidian | Dual Track copy; live page scan `hasObsidian=false` | Dual Track N5; live J5 |
| `JOURNEY-BROWSER-SYNC-01` | Real click/type/navigate on daemon `/ui/` + regression J0/J10/J18/J19 + closed Phase 14 pack (`P14-T01` docs only) | Empty import refuse; dead hashes | running report D02 rows |

Drift refused: HTTP paste as the Knowledge IA; file-as-authority; secret ingest; bundled Obsidian; Vite as product origin; fake Admit; weakening P13-T07 Memory tombstone/promote.

## 2. Validation summary

| Environment | Result |
|---|---|
| `DEV-WIN-GNU-01` Dual Track | `knowledgeIa` + `knowledgeIngest` + `knowledgeMemory` + `opcIa` **41/41** |
| Local `pnpm build` / `tsc` | **pass** after FileList mock fix |
| `pnpm run check:consistency` | **pass** at claim HEAD |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | run [33975024580](https://github.com/agentkernel/cognitive-os/actions/runs/33975024580) **SUCCESS** at `899b65f6` (resolve 4s, ubuntu 4m32s, windows 15m7s, required-ci 3s) |
| `B01-Desktop-Linux-002` `/ui/` | J5 **pass**; J0/J10/J18/J19 **pass**; product URL `http://127.0.0.1:48681/ui/` through owner SSH forward. Daemon binary left on EVAL-016 pin `711a5a7c` (this task is TS-only). |
| Host OS file dialog | **not-run** |

## 3. Non-claims

Not T02 create-wizard, not T03 Write Project, not T04 members, not T05 Attempt/Runs/Outputs, not T06 Today, not T07 Settings L1 / palette / PrimaryNav. `opcIa.test.tsx` Knowledge tab helpers only. `DOC-PERSONAL-2.0-OPC-REFRAME` product docs unread-write. No Gate / release / Profile / B01 / EVAL-016 revival.

## 4. Unique next

Consumed after ready/merge of PR #328. Unique next remains claim **`P14-T02`** (Worker 1). This worker does not claim T02/T03/T04 or T07/T06. Claim `P14-T05` only if `P14-T03` is done and T05 is unclaimed — T03 is still `not-started` at this close.

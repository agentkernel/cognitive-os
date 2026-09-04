# P13-T07 Knowledge + Memory authority — closure

- Task: `P13-T07` / slice `P13-T07/D01` (single Delivery Slice)
- Change class: `implementation-only` (labeled Vault read + Memory auto-admit / promote on existing Memory tables; Knowledge `/ui/` caller; **no new numbered migration**; T06 owns `personal_db.rs` v39)
- Lease: `lease/personal/P13-T07/knowledge-memory` (close this delivery → PARALLEL-LANES §3.1)
- Branch: `personal/P13-T07-knowledge-memory` (worktree `D:\agent-kernel-wt-P13-T07`; PR [#319](https://github.com/agentkernel/cognitive-os/pull/319))
- Validated implementation HEAD: `6927efe6` (parents `070fd243` + `origin/main@2217722d` / T10 merged PR #318)
- Required CI on validated HEAD: [33756037394](https://github.com/agentkernel/cognitive-os/actions/runs/33756037394) **SUCCESS** (resolve 2s, ubuntu 4m26s, windows 13m30s, required-ci 2s)
- Fold HEAD `bbe661b5` (parents `c2820523` + `origin/main@22718d74` / T08 merged PR #317; conflicts only in plan docs + regenerated `ref.http-api`): required CI [33837343012](https://github.com/agentkernel/cognitive-os/actions/runs/33837343012) **SUCCESS**
- **Merged** 2026-09-04: PR [#319](https://github.com/agentkernel/cognitive-os/pull/319) at `main@015afcb8`; local + remote task branch deleted; worktree removed; lease closed → PARALLEL-LANES §3.1
- Claim ceiling: `hypothesis` (A7: local MSVC / Dual Track / ordinary CI is not Gate / release / Profile / Windows qualification)
- Evaluation routing: **OFF**
- Host FS/privacy E2E: `not-run` until `P13-T13`

## 1. Acceptance mapping (formal plan P13-T07 card + `P13-T07/D01`)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| Knowledge fragment shows provenance / rights / freshness / exclusion / untrusted-observation; files ≠ authority | store labeled Vault index; HTTP `GET vault.labeled` / `vault.documents`; web `KnowledgePage` `opc-knowledge-labels` + `is_authority` stays false | store `p13_t07_labeled_fragments_expose_provenance_rights_freshness_exclusion`; `p13_t07_vault_file_still_cannot_become_project_authority`; Dual Track “no apply authority” | store 8/8; HTTP labeled 200 + `"is_authority":false`; Dual Track 22/22 |
| Reindex / import-failure keep originals visible | documents list keeps `not-indexed` until rebuild; import-failure surfaces original fields | store `p13_t07_import_without_rebuild_keeps_not_indexed_document_visible`; HTTP documents 200 contains `not-indexed`; Dual Track documents region | store + HTTP + Dual Track |
| Memory inspect / correct / promote / forget on **management HTTP**; tombstone does not resurrect | management `memory/remember` / `correct` / `forget` / `promotes` / `promote.request` / `promote.confirm`; web Memory panel posts those paths | store `p13_t07_tombstoned_memory_cannot_be_promoted`; Dual Track inspect→`memory/correct`; HTTP task aliases 403 | store + HTTP + Dual Track |
| Chat auto-admission into inspectable Memory; UI honest-empty / Requires-backend; **0 fake Admit buttons** | store admission from archive; HTTP `POST memory/auto-admit.chat` (management only); UI `opc-knowledge-auto-admit` = `CHAT_AUTO_ADMIT_REQUIRES_BACKEND` (T06 turns exist on `main` but this surface does not list them as admit candidates) | store `p13_t07_agent_cannot_self_admit_chat_into_memory`; `p13_t07_secret_shaped_chat_is_not_admitted`; Dual Track 0 Admit buttons / no auto-admit POST | Dual Track; HTTP task alias 403; management auto-admit 404 then 201 |
| Cross-Project promote needs Owner digest-bound preview confirm | `promote.request` → `pending` + digest; `promote.confirm` digest-bound; request does not copy | store `p13_t07_cross_project_promote_requires_owner_confirm`; HTTP `p13_t07_promote_preview_then_confirm_on_management_http`; Dual Track “does not yet have a copy” | store + HTTP + Dual Track |
| Cross-project labeled read / Agent self-admission / secret-shaped Memory fail closed | store Forbidden; HTTP 403 / refuse | store `p13_t07_cross_project_labeled_read_is_forbidden`; HTTP overreach 403; N3/N4 | store + HTTP |
| Last-write-wins without a conflict record is rejected | existing Vault conflict record + Knowledge ingest error surface | Dual Track `knowledgeIngest.test.tsx` last-write-wins; vault conflicts projection | Dual Track (P12-T07 retained) |
| Bundled Obsidian is not a product surface | no Obsidian app / vault-as-authority path added | non-claim (absence) | no product route |

Formal-plan 关闭门, sentence by sentence: (1) fragment labels visible — **true**; (2) reindex / import-failure keep originals — **true**; (3) Memory four actions on management HTTP — **true**; (4) tombstone does not resurrect — **true**; (5) cross-Project promote needs Owner preview — **true**. Chat auto-admission is an authority path on management HTTP; the Knowledge auto-admit **list** stays honestly empty / Requires-backend (0 Admit buttons) because this surface does not invent T06 turn candidates.

Drift negatives from the card, all refused / never produced: file-as-authority (Invalid); cross-project labeled read (403); tombstone promote (refused); Agent self-admission (Forbidden / 403 aliases); secret/PII Memory (`sk-` refused); last-write-wins without conflict (ingest error); bundled Obsidian (not shipped).

## 2. Validation summary

| Environment | Result |
|---|---|
| Local MSVC override (`rustc` host `x86_64-pc-windows-msvc`; `CARGO_PROFILE_DEV_DEBUG=0`) | store `p13_t07_knowledge_memory` **8/8**; kernel-server `p13_t07` **2/2**; development evidence only |
| `DEV-WIN-GNU-01` | Dual Track vitest `knowledgeMemory` + `vault` + `knowledgeIngest` **22/22**; `check:consistency` / handbook / generate-handbook `--check` **pass** after fold `6927efe6` |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | run [33756037394](https://github.com/agentkernel/cognitive-os/actions/runs/33756037394) **SUCCESS** at `6927efe6` |
| Prior implementation HEAD `070fd243` | required CI [33752950398](https://github.com/agentkernel/cognitive-os/actions/runs/33752950398) **SUCCESS** (then DIRTY after T10 merge) |
| `DEV-LINUX-NATIVE-01` | **not-run** this close (slice required validation is Dual Track TS + Ubuntu/Windows CI, not live Linux E2E) |
| `DEV-WINDOWS-NATIVE-OPC-01` | **not-run** (host FS/privacy E2E waits `P13-T13`) |
| `B01-Desktop-Linux-002` | **not-run** (no guest `/ui/` deploy) |

## 3. Non-claims

Not T06 (source, already merged), not T08 Settings, not T09 lifecycle, not T10 grants, not T11 reflection, not T12/D02 rendered review, not T13 Windows FS/privacy. No new numbered migration. No fake Admit button. No bundled Obsidian. No Gate / release / Profile / B01 / Windows qualification. `PERS-PR-051` stays `not-run` until T08/T09 also close.

## 4. Unique next

Consumed: merged at `main@015afcb8`. Serial continuation (owner instruction 2026-09-04): close `P13-T09` (Draft PR [#321](https://github.com/agentkernel/cognitive-os/pull/321)) next, then `P13-T11` (Draft PR [#320](https://github.com/agentkernel/cognitive-os/pull/320)), then `P13-T12/D02`; `P13-T13` waits for the owner host and an owner disk decision before any local test.

# 20260720 Lane-CTR Handoff（M1 契约批）

## 1. 本次会话完成

按 `docs/prompts/lane-ctr.md` 五任务全数交付（分支 `lane/ctr`，rebase 到 `4a1c6c3` 后哈希如下）：

- **任务 1 — F-003 收尾**（`c6027bb`；F-003、REQ-GOBJ-HEADER-001/REF-001/MIG-001）：
  - 核验 `c190d7b` 两份向量调整**合法**：schema-meta-001 跟随 registry owner_spec 同批改指；effect-state-closure-008 对齐 `effect.transitions.json` v0.2 机器真相（OUTCOME_UNKNOWN 直接出口仅 RECONCILED），deny 期望与错误码未动、无负例删除。
  - 新增 2 份双轨拒绝负例向量 GOBJ-LEGACY-METADATA-001 / GOBJ-LEGACY-STRONGREF-001（registry tests 已映射，matrix 再生成；状态 not-run，不虚报）。
  - 合同层可执行复验（非 runner）：`crates/cognitive-contracts/tests/schema_contract.rs`（jsonschema 0.47 dev-dep）+ `packages/contracts-ts/src/schema-contract.test.ts`（ajv 2020）双侧证明 56 schema 可编译、两负例被拒、迁移正例通过、legacy `$defs` 保持 deprecated + 零引用。
  - legacy `$defs` 决策 = **保留**（deprecated、零引用、仅作 §6 legacy adapter 的版本钉扎映射源；REQ-GOBJ-REF-003/MIG-001 的被引用物）。
- **任务 2 — D-001/D-006 `$id` 统一**（`0ee2741`）：56 份 schema 全量 `$id` == 文件名（43 份改写）；`check-consistency.mjs` 移除剥离 `$id` 兼容层并新增 `$id` 策略红灯；策略写入 `conformance/README.md` 与 `.cursor/rules/12`；D-001/D-006 闭合。
- **任务 3 — codegen（ADR-0006/IMP-08）**（`32c8b84`）：自研生成器 `crates/cognitive-contracts/src/bin/contracts-codegen.rs` 单源双语言；IMP-08 A.1 14 对象 ↔ 17 schema + `$ref` 闭包（+actor-chain、conversation-binding）= 19 模块 × Rust/TS 双侧入库；文件头带源路径 + canonical schema digest（schema-bundle/0.1 域）+ 生成器版本；shape-level 语义（条件约束留给 schema 校验）、legacy `$defs` 排除；ci.yml 挂 regenerate-and-diff 门；`tests/generated_types.rs` / `generated-types.test.ts` 接入 canonical 编码层（typed 往返 digest 不变、未知成员/legacy 形态拒绝、EffectState 对迁移表穷尽、头部 digest 与当前 schema 一致）。
- **任务 4 — 注册式 bundle digest（§13）**（`c790172`；D-011）：`cognitive_contracts::bundle` / contracts-ts `bundle.ts` 双语言实现 canonical logical manifest（{id, version, media_type, content_digest} 按 id 排序、拒绝重复/空集）；runner 的 provisional_digests → registered_digests（requirement_set_digest 覆盖 3 registry + 5 迁移表，schema_bundle_digest 覆盖 56 schema）；每资产 SemVer 缺口登记 D-011 并以套件版本 `0.1.0-draft.1` 最小修正（conformance-evidence.md §6 重写）；emit_golden 双侧追加 live schema-bundle digest 进跨语言门。
- **任务 5 — golden §14 全覆盖**（`bd3c884`）：新模块 `projection`（Rust/TS 双语言：digest projection、self-digest 验证、canonical 时间戳形式、digest 字符串形式、unknown-critical-extension 门）；新 golden 文件 `tests/golden/digest-and-projection-fixtures.json`（TS 生成、Rust 逐字节复验）覆盖 §14 余项全部：digest projection 正/负例（wrong self-field inclusion/exclusion、wrong domain、inserted defaults）、set manifest 正/负例、offset/local/leap-second/trailing-zero 时间戳、uppercase/truncated digest、altered schema digest、unknown critical extension；canonical-json-fixtures 增补 infinity 负例（additive，0.1.0→0.2.0）。
- **文档联动批**（本 handoff 同批）：findings-ledger（F-003/D-001/D-006/D-011/IMP-08）、PROGRESS（向量 76 实测、车道表触碰通告）、conformance-evidence §6、ADR-0006 Delivery record、conformance/README `$id` 策略、tests/golden/README。

## 2. 未完成 / 进行中

- F-003 唯一剩余 gate：Lane-CFR runner 真实执行 2 份负例向量（合同层复验不计为向量执行）。
- D-004（层 7/8 slug）留给 Lane-CFR runner 交付时处置。
- matrix 的 impl/impl_tests 字段未回填（等 runner 证据口径后随 M1 出口评审补）。
- PR 建立与合并状态见会话最终报告（本 handoff 写于推送前）。

## 3. 测试与证据状态

- Rust：32 测试通过（contracts 15 单元 + 5 schema-contract + 4 generated-types + 6 projection-fixtures + 2 golden，conformance 2）；clippy -D warnings 绿；fmt 绿；codegen 再生成 diff 为空。
- TS：31 测试通过（contracts-ts 27 / tools 2 / sdk-ts 1 / agent-shell 1）。
- check-consistency OK（273 REQ / 55 码 / 56 schema / **76 向量**）；gen-matrix --check 新鲜；static_check.py ALL CHECKS PASSED；conformance runner 枚举 76 全 not-run；validate-manifest OK；跨语言 golden **byte-identical**（含 live schema-bundle manifest digest）。
- 注入演练（检查器被修改，docs-sync-contract §5 义务）：注入孤儿 REQ 引用 + 断链 + `$id` 违例 → `check-consistency: 4 violation(s)` 逐条指出（含新红灯 `schema $id must equal its file name`）→ 还原复绿。输出全文随 PR 描述。
- 向量状态：76 全 not-run（新增 2 份负例同样 not-run，无虚报）。

## 4. 未决风险与漂移

- **D-007 撞号已处置**：Lane-CON 同日占用 D-007~D-010，本车道的"每资产 SemVer 缺口"由 D-007 改号 **D-011**（台账条目已注明改号原因；rebase 冲突仅 findings-ledger 一处，已按后合并者责任解决）。
- **本地 Windows 工具链现象**（不影响 CI）：本机 rustup 为 x86_64-pc-windows-gnu 宿主，缺 GNU binutils 的 `as/dlltool`，raw-dylib 依赖（jsonschema→getrandom 等）需 shim：把 llvm-tools 组件的 `llvm-ar.exe` 复制为 `%USERPROFILE%\.cargo\dlltool-shim\dlltool.exe`（同目录放 libgcc_s_seh-1.dll、libwinpthread-1.dll）并前置 PATH。CI 用 MSVC 宿主不受影响；Lane-CFR 本机开发若引入同类依赖需同一 shim。
- Windows PowerShell `>` 重定向写 BOM+CRLF：golden 文件再生成必须用 `tests/golden/README.md` 里的 node 写入程序（已文档化）。
- 规范表面零新增：无新 REQ/错误码/对象族/Profile；向量增补与 schema `$id` 改写均为契约变更并同批联动。

## 5. 下一步入口

- 建议：启动 **Lane-CFR**（`docs/prompts/lane-cfr.md`）——开工先 rebase 本批对 `tools/src/check-consistency.mjs`、`tools/static_check.py`、`.github/workflows/ci.yml`、`crates/cognitive-conformance/src/lib.rs` 的触碰点（清单见 PROGRESS 车道表）。
- runner 执行能力落地后：真实执行 76 向量（重点 2 份 F-003 负例）→ F-003 关闭 → M1 出口评审（`docs/prompts/milestone-m1.md`）。
- 工作分支：`lane/cfr`（从合并后 main 新建）。
- 第一个动作：`git fetch origin; git worktree add ../agent-kernel-cfr -b lane/cfr origin/main`，读 PROGRESS 车道表触碰通告。

## 6. 快照

- PROGRESS 已更新：是（M1 in-progress、向量 76、D-001/D-006/D-011 闭合、触碰通告、handoff 列表）。
- 本次提交：`c6027bb`（任务 1）→ `0ee2741`（任务 2）→ `32c8b84`（任务 3）→ `c790172`（任务 4）→ `bd3c884`（任务 5）→ 本 handoff 批（PROGRESS + ledger + handoff，哈希见 git log）。基线 `4a1c6c3`（含 Lane-CON 四提交）。

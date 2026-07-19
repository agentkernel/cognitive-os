# 文档联动与防漂移契约（docs-sync-contract）

- Standard ID: `cognitiveos.standard.docs-sync-contract/0.1`
- Version: v0.1 Draft
- Status: repo governance standard（约束本仓库全部提交；不产生 CognitiveOS 规范要求）
- Date: 2026-07-20
- 执行机制：`.cursor/rules/02-workflow-docs-sync.mdc`（会话义务）+ `tools/src/check-consistency.mjs`（CI 红灯）

## 1. 变更三分类

| 类型 | 定义 | 例子 |
|---|---|---|
| **修正型** | typo、断链、漂移修复、计数更新——**不改语义** | 修 `$ref` 路径；D-005 版本枚举放宽；更新 PROGRESS 计数 |
| **语义型** | 改行为、状态机、错误码、schema 约束、验收口径 | 新增迁移 guard；收紧 schema 字段；改验收判据 |
| **结构型** | 重构、新增/删除对象族、Profile、子系统 | F-003 单轨迁移；新增 Profile（v0.1 前禁止） |

不确定归类时按更重的一档处理。**语义真相落在 registry/schema/companion/transition/vector；白皮书随后对齐**——冲突期以机器资产为准并登记漂移。

## 2. 联动义务（同一 PR / 提交批次内完成）

**修正型**：改动本体 + 提交说明注明"修正型" + 若属漂移修复，findings-ledger 漂移节登记/闭合。

**语义型**，除上述外必须同批联动更新以下受影响项（无影响者在 PR 描述写明"无"）：

1. 受影响的白皮书章节与版本说明（informative 对齐）；
2. 对应 companion 规范（`specs/*/README.md`、RFC-0001）；
3. `specs/registry/*.yaml`（REQ/错误码/状态域）；
4. `specs/schemas/*`（含再生成的语言绑定，ADR-0006）；
5. `conformance/vectors/*`（禁止删除负例或放宽 expected 迎合实现）；
6. 实现与测试；
7. `docs/traceability/matrix.yaml`（跑 `gen-matrix`）与 findings-ledger；
8. 受影响的产品文档：`apps/cognitiveos-console/PRODUCT-DESIGN.md` **至少加漂移标注**（在其文首"漂移登记"节追加一行：日期、变更、受影响章节；不改写正文——该文件由 Console 车道维护）。

**结构型**，再加：

9. 新增 ADR（沿用 `docs/adr/` 格式）；
10. 迁移说明（旧对象/旧引用如何处置，读者为实现者与 runner）。

## 3. 影响面扫描方法（结果写入 PR 描述）

以下列键做全仓 grep（排除 `History/`、`target/`、`node_modules/`、`dist/`）：

- REQ-ID（如 `REQ-EFF-002`）；错误码（如 `EFFECT_IDEMPOTENCY_CONFLICT`）；
- schema 文件名与 `$id`（如 `effect.schema.json`）；
- 白皮书锚点/章节号（如 `§16.6`）与标准文件名；
- 再用 `docs/traceability/matrix.yaml` 反查该 REQ 的 impl/tests/evidence/docs 字段得到代码与文档落点。

PowerShell 示例：

```powershell
rg -n "REQ-EFF-002|EFFECT_IDEMPOTENCY_CONFLICT|effect.schema.json" --glob '!History/**' --glob '!target/**' --glob '!node_modules/**'
```

## 4. 白皮书/评审文档的特殊地位

`CognitiveOS-Architecture.md`（informative）语义滞后允许存在，但必须登记：漂移在 findings-ledger 漂移节记录，修订按批次合并。两份评审文档与 `RFC-0001` 历史结论**不回改**（historical 证据），现状变化只写台账。

## 5. CI 强制（红灯即失败）

`tools/src/check-consistency.mjs`（在 CI 的 consistency job 运行）：

1. 全部 JSON/YAML 可解析；schema 过 draft 2020-12 元校验且相对 `$ref` 全可解析；迁移表对表 schema 校验；
2. registry↔schema↔vector 双向无孤儿（REQ 无测试映射、测试 ID 无向量、向量 REQ/错误码不在 registry、schema 不可达均为红灯）；
3. 活文档相对链接不断链、不指向 `History/`；
4. 活文档中完整 REQ-ID 引用必须存在于 registry（孤儿引用红灯）；
5. `matrix.yaml` 覆盖全部 273 REQ 且引用路径真实存在；`gen-matrix --check` 无 drift；
6. findings-ledger 覆盖 F-001~F-030 与 IMP-01~18 全部条目。

破坏性验证义务：本契约生效时（M0）已做一次注入演练——临时分支故意制造孤儿 REQ 引用与断链，确认 CI 检查失败并指出位置后回滚（记录见 M0 milestone review §注入演练）。此后每次**修改检查器本身**的 PR 必须重跑一次注入演练并在 PR 描述附输出。

## 6. 完成前检查（作者自查清单）

- [ ] 变更分类已声明（修正型/语义型/结构型）
- [ ] §2 对应档位的联动清单逐项完成或写明"无影响"
- [ ] §3 扫描结果贴入 PR 描述
- [ ] `pnpm run check:consistency` 本地绿
- [ ] PROGRESS 已更新；触碰 F/IMP/漂移时 findings-ledger 已更新

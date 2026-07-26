# clients/prompts — 客户端提示词索引

> 类别：prompt index ｜ owner：Lane-CON ｜ 状态：索引（各提示词自带 gate 状态）

- **用途**：客户端相关接续提示词的唯一索引（Agent Hub 提示词正文在 [clients/agent-hub/prompts/](../agent-hub/prompts/README.md)）。提示词是会话工具，不是事实来源；执行前先核对对应 plan 与 gate。
- **持续开发执行**：[continuous-development-execution.md](continuous-development-execution.md)——按 [development-plan](../plan/development-plan.md) 自主推进 Phase 0–4 的委托执行提示词（Owner 2026-07-26 授权自动 commit/push/merge；gate 纪律与 PoC 豁免边界内嵌）。
- **Agent Hub 提示词**（已迁入）：[agent-hub/prompts/README.md](../agent-hub/prompts/README.md)——12 宏车道 + 6 Adapter，全部 `blocked`；唯一不违反 gate 的可推进项是接口一手核验（`AH-CTR-02` 类 informative 工作）。
- **留在 `docs/prompts/` 的 Console/车道提示词**（不迁移，此处只索引）：
  - [lane-con.md](../../docs/prompts/lane-con.md)：Lane-CON 车道占位提示词；
  - [console-agent-hub-direct-mode-product-design.md](../../docs/prompts/console-agent-hub-direct-mode-product-design.md)（已执行）；
  - [console-client-directory-index-and-maintenance.md](../../docs/prompts/console-client-directory-index-and-maintenance.md)（已执行）；
  - [console-client-project-foundation-and-doc-migration.md](../../docs/prompts/console-client-project-foundation-and-doc-migration.md)（本次迁移任务规范）；
  - [console-mobile-platform-product-design.md](../../docs/prompts/console-mobile-platform-product-design.md)（已执行）。
- **边界**：`blocked` 提示词在 gate 解阻前不得启动编码或 mock；提示词执行状态以各文件自述与 PROGRESS 为准。

# clients/prompts — 客户端提示词索引

> 类别：prompt index ｜ owner：Lane-CON ｜ 状态：索引（各提示词自带 gate 状态）

- **用途**：客户端相关接续提示词的唯一索引。提示词是会话工具，不是事实来源；执行前先核对对应 plan 与 gate。
- **Agent Hub 提示词**（B5 迁移前现址）：[docs/prompts/agent-hub/README.md](../../docs/prompts/agent-hub/README.md)——12 宏车道 + 6 Adapter，全部 `blocked`；唯一不违反 gate 的可推进项是接口一手核验（`AH-CTR-02` 类 informative 工作）。
- **留在 `docs/prompts/` 的 Console/车道提示词**（不迁移，此处只索引）：
  - [lane-con.md](../../docs/prompts/lane-con.md)：Lane-CON 车道占位提示词；
  - [console-agent-hub-direct-mode-product-design.md](../../docs/prompts/console-agent-hub-direct-mode-product-design.md)（已执行）；
  - [console-client-directory-index-and-maintenance.md](../../docs/prompts/console-client-directory-index-and-maintenance.md)（已执行）；
  - [console-client-project-foundation-and-doc-migration.md](../../docs/prompts/console-client-project-foundation-and-doc-migration.md)（本次迁移任务规范）；
  - [console-mobile-platform-product-design.md](../../docs/prompts/console-mobile-platform-product-design.md)（已执行）。
- **边界**：`blocked` 提示词在 gate 解阻前不得启动编码或 mock；提示词执行状态以各文件自述与 PROGRESS 为准。

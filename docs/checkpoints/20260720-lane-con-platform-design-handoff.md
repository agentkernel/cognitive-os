# 20260720 Lane-CON Platform Design Handoff

## 1. 本次会话完成

- 复核 `main`/HEAD 与 clean worktree 后，在批准的窄幅例外内完成纯 informative 文档工作；未修改 registry/schema/transition/vector/实现代码，未读取或引用 `History/`。
- 新增 `docs/platforms/` 五份入口与产品文档：
  - macOS v1：Universal 2、macOS 14+ 候选、PKG、machine daemon/per-user broker/helper、Touch ID R1、App Sandbox target、PoC/PRD/来源 ledger；
  - Linux v1：Ubuntu 24.04 x86_64 GNOME/Wayland `.deb`、A/B slot、systemd/D-Bus/polkit/Secret Service、PoC/PRD/来源 ledger；
  - parity matrix 与 `CONSOLE-MAC-V1-DEC-*` / `CONSOLE-LNX-V1-DEC-*` 决策记录。
- 治理例外已同步到 PARALLEL-LANES、DEVELOPMENT-PLAN 和 Lane-CON 提示词；实现 gate 改指可定位的平台 Open PoC/GA gates。
- findings-ledger 新增并关闭 D-007~D-010：
  - D-005 transition schema 状态在 Console 文档中的漂移；
  - F-003 已迁移但待 M1 runner/codegen 复验的状态漂移；
  - 失效 `PRODUCT-DESIGN §12.6 POC-01~12` gate；
  - F-003 后 PRODUCT-DESIGN 缺漂移登记节。
- 对 PRODUCT-DESIGN 与 Console requirements trace 做最小事实修正，保留旧 §17/§20.3 anchor、legacy ID 映射。
- 终审修复两项高置信问题：四态声明错误挂接 `REQ-SHELL-STATUS-001`（改为 product-only）；Linux 初始 A/B payload 与无 APT repo 情况下 `.deb` 来源验证链不闭合（补 threshold-signed release manifest、首 slot 激活流程）。
- 提交：
  - `26b84fe` — `docs(console): define desktop platform designs and exception (D-007..D-010)`
  - `000f6ab` — `docs(console): link desktop platform product entries`
  - 本 handoff/PROGRESS 批次提交哈希见本提交后的 `git log`。

## 2. 未完成 / 进行中

- Console implementation 未启动；依赖组 1/2/7、M5 出口和平台真实 PoC gate 均未满足。
- macOS `MAC-POC-01..12` 与 Linux `LNX-POC-01..12` 全部 `not-run`。
- 平台专属 machine contracts 均未登记：daemon/helper/broker IPC、claim、signed lease、threshold metadata、Touch ID display、A/B switch、notification handle、WebKit floor 等。
- Tauri 2/原生 shell 技术 ADR 未批准。
- macOS GA OS/Intel floor 必须在发布时重新核实；Linux 的 24 个月支持不得越过 Ubuntu/WebKitGTK floor。

## 3. 测试与证据状态

- `pnpm run check:consistency`：通过；输出 `273 requirements, 55 error codes, 56 schemas, 74 vectors, markdown links and traceability verified`。
- `git diff --check` / staged diff check：通过。
- 平台 ID 检查：48 个 PRD（macOS 24 + Linux 24）字段完整且唯一；22 个 DEC（11 + 11）唯一。
- 状态声明检查：全部平台 PRD 为 `not-implemented`，Evidence 只使用 `none/not-run`。
- ReadLints：所改文档无诊断。
- CI 远端：未执行。
- 向量：仍为 74 `not-run`；本会话没有 runner 行为执行。
- 产品/平台 evidence：`none`；不存在 Profile conformance 声明。

## 4. 未决风险与漂移

- 开放 P0/P1 与 D-001/D-004/D-006 状态未被本会话改变；权威仍是 findings-ledger。
- D-007~D-010 是文档事实漂移，已在同批修正；不表示 F-003 行为已验证。
- macOS GUI App Sandbox 与 machine daemon XPC 是否可闭合仍是 GA blocker，失败不得静默降级。
- Linux 无 APT repo 的 `.deb` bootstrap 信任、A/B 原子性、dpkg ownership、slot/data compatibility 仍需真实负例。
- WebKit/WKWebView security floor 和 kill switch 必须以目标平台实际 build/backport 证据验证。
- secure storage、lock/switch-user、per-user notification routing、emergency stop/unknown 都缺实际证据。

## 5. 下一步入口

- 建议提示词：`docs/prompts/lane-con.md`
- 工作分支：Lane-CON 仍未激活实现分支；informative 文档按例外随评审 PR。
- 第一个动作：读取 PROGRESS、`docs/platforms/README.md#console-实现-gate` 和目标平台 Open PoC 表，确认任务仅属文档例外；未满足 backend gate 时禁止实现或 mock PoC。

## 6. 快照

- PROGRESS 已更新：是。
- Console 状态：tracking-only（informative 文档例外），implementation 未启动。
- 本次提交列表：`26b84fe`、`000f6ab`、本 handoff/PROGRESS 批次。

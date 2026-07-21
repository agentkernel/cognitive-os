# 20260721 Lane-TSC Handoff（M5 真 HTTP/SSE 对接 kernel-server）

## 1. 本次会话完成

- **`packages/sdk-ts`**：`HttpSseTransport` 换绑 M5 kernel-server 面——management → `POST /management/<op>`；task shell → `POST /shell/<verb>`（`shell.control`→`/shell/cancel`）；watch → `GET /task/watch` SSE。通道根不相交；管理通道拒开 watch。
- **`kernelServerPath`** 纯函数 + 单测；可选 live 集成 `http_live.test.ts`（检测 `KERNEL_SERVER_BIN` 或 `target/debug/kernel-server`；本机实测 3 项全绿）。
- **`apps/agent-shell`**：`createLiveShellSession` 经 task 凭证 + HttpSseTransport 接线；单元钉扎 phase=idle。
- 客户端仍非 authority；向量零改写零执行。

## 2. 未完成 / 进行中

- Shell proposal/preview/submit 的完整 HTTP 语义仍依赖 RUN 扩展 `--once` 面（当前 detach/cancel/attach/watch 已对接）。
- Lane-CFR M5 向量行为执行与 F-011 闭合 = 下一批。

## 3. 测试与证据状态

- sdk-ts：**72** pass（含 3 live）；agent-shell：**13** pass。
- `pnpm -r build/test` 绿（本车道包）。

## 4. 未决风险与漂移

- 无新漂移。`--once` 单连接参考服务不足以覆盖长驻多路 watch；生产服务循环非本里程碑。

## 5. 下一步入口

- `docs/prompts/lane-cfr.md`：执行 F-011 / shell-cancel / detach / watch-resume / intent-supersede 等。
- 工作分支：`lane/tsc` → merge 后 `lane/cfr`。

## 6. 快照

- PROGRESS 已更新：是。
- 本次提交：sdk-ts transport + live tests；agent-shell live；docs。

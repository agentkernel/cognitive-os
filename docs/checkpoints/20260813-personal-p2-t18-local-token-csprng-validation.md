# P2-T18 本地令牌 CSPRNG 修正 — 增量验证报告

- Task: `P2-T18`
- Slice queue: `P2-T18/D01` → `P2-T18/D02`
- Branch: `personal/P2-T18-local-token-csprng`
- Base: `origin/main@326f97728ab6aaaacceaedd2156d953231b32e01`
- Lease: `lease/personal/P2-T18/local-token-csprng`
- Classification: `implementation-only` security corrective；既有 ADR-0019/0022
  安全语义不变。
- Evidence ceiling: ordinary implementation evidence only；不产生 Gate、release、
  Profile、B01 或 campaign 结论。
- Secret discipline: 本报告、测试输出与证据只记录控制流和计数，绝不记录 token bytes。

本文件是本任务唯一 running validation report。每个完成的验证单元在下一单元开始前追加；
已记录条目不改写，必要时以 superseding entry 更正。

## 增量条目

### V00 — 基线令牌来源静态检查

- Revision: `326f97728ab6aaaacceaedd2156d953231b32e01`
- Environment/instrument: isolated Windows worktree；针对
  `apps/kernel-server/src/personal/auth.rs` 的定向 source inspection
- Started/retained denominator: 1/1 source unit
- Outcome: `fail`（预期的 failure-first 基线）
- Result: 生产 `generate_opaque_token` 明确引用 `DefaultHasher`、进程 ID、`Instant`
  timing 与自制 `random_u64`，且没有 OS CSPRNG 调用。已加入
  `p2_t18_local_token_csprng::production_token_generation_uses_only_the_os_csprng`
  静态负例；断言文本不包含任何 token material。
- Disposition: 保留该失败测试并在下一实现 revision 使其通过；禁止 PID/time/hash fallback。

### V01 — failure-first Rust 测试执行路由

- Revision: 首个 failure-first commit 待生成
- Environment/instrument: `DEV-WIN-GNU-01`
- Started/retained denominator: 0/1
- Outcome: `not-run`
- Reason: 本地主机是已登记不支持 Rust linking 的 Windows GNU 环境；未违反
  `RUST-LINK-DEV-WIN-GNU-01` 重跑 `cargo test`。
- Disposition: 首个 commit 推送后由 exact-revision `DEV-LINUX-NATIVE-01` 观察预期失败；
  最终候选同时由 GitHub Ubuntu/Windows CI 验证。

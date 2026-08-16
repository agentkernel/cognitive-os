---
doc_id: ai.validation-commands
locale: zh-CN
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: package.json
  - path: tools/package.json
  - path: tools/src/generate-handbook.mjs
  - path: .github/workflows/ci.yml
  - path: docs/plan/PERSONAL-TEST-ENVIRONMENTS.md
    symbols: ["COMMAND-SHELL-PS51", "RUST-LINK-DEV-WIN-GNU-01"]
fingerprint: "sha256:694c34328de5ccdc2a54c8df18e469322c91b69de53289f355c58a9a4a4ec531"
non_claims:
  - 命令可用不等于证据；只有实际执行的检查才算数，且本地结果绝不升格 Gate/release/Profile 声明。
---

# 验证命令

环境路由是前置条件，由
[`PERSONAL-TEST-ENVIRONMENTS.md`](../../../docs/plan/PERSONAL-TEST-ENVIRONMENTS.md) 拥有。

## 全平台安全（含 Windows GNU 主机）

```powershell
pnpm install --frozen-lockfile
pnpm -r build
pnpm -r test
pnpm run check:consistency          # tools/src/check-consistency.mjs
node tools/src/gen-matrix.mjs --check
node tools/src/check-handbook.mjs   # 手册防漂移门
node tools/src/generate-handbook.mjs --check
node tools/src/docs-sync-gate.mjs --staged   # commit 前文档同步门（--push / --range）
pnpm run hooks:install              # 每克隆一次：注册 .githooks pre-commit/pre-push
cargo fmt --all -- --check          # 仅格式化；不触发链接
git diff --check
```

## 必须走受支持 CI（Ubuntu / Windows MSVC）或 exact-revision native Linux

```bash
cargo build --workspace --locked
cargo test --workspace --locked -- --test-threads=1
cargo test -p kernel-server tool_executor --locked -- --test-threads=1
cargo test -p kernel-server readiness --locked -- --test-threads=1
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run -p cognitive-conformance --bin conformance-runner
cargo run -p cognitive-contracts --bin contracts-codegen   # 之后 diff 生成目录
```

绝不在本地 Windows GNU 主机运行上述命令：linker 失败（exit 121）是已登记的环境边界，
不是需要复现的信号。远程/native 验证只消费已推送的不可变 revision——绝不复制工作树。
P2-T25 的聚焦 HTTP 覆盖在 `apps/kernel-server/tests/p2_t25_tool_lifecycle.rs`
（lifecycle、selection 与钉住 HTTPS origin 登记表）。

## CI 在每个 PR 上强制什么

[`ci.yml`](../../../.github/workflows/ci.yml) 的 `verify` 矩阵（Ubuntu + Windows
MSVC）：TypeScript 构建/测试、Rust 构建/测试/clippy/fmt、codegen 漂移 diff、
consistency 检查、traceability 新鲜度、带固定五态计数与证据诚实断言的符合性 runner、
错误实现自检、跨语言 golden digest 字节一致。

## 已知过期入口

`pnpm run verify:local`（V01 编排器）钉住了过期的符合性计数，在本基线不是可用的本地
门；请改用上面的单项命令。

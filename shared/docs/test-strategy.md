# 共用客户端测试策略

> 类别：informative test strategy ｜ owner：Lane-CON（各层执行归所属车道）｜ 日期：2026-07-20
>
> 本文只定义客户端侧测试分层与证据口径的**消费方视图**；不新增测试义务、不替代 [conformance/README.md](../../../conformance/README.md) 的测试层定义，也不构成客户端“测试已执行”声明。全局 M1–M4 已有合同/内核行为证据，但客户端平台层仍为 `not-run` / `none` / `planned`。

## 1. Test pyramid 分层

| 层 | 范围 | 归属/入口 | 当前状态 |
|---|---|---|---|
| 包级单元测试 | contracts-ts、sdk-ts、agent-shell 的合同/客户端实现测试 | Lane-CTR / Lane-TSC；[contracts-ts](../../../packages/contracts-ts/src/index.ts) | TS 客户端包内测试已执行；不计平台 PoC 或 conformance 向量执行 |
| 跨语言 golden | Rust/TS canonical 编码与 digest 夹具对比 | Lane-CTR；[tests/golden/README.md](../../../tests/golden/README.md) | 夹具已提供；CI golden job 对比（非客户端功能证据） |
| conformance 向量 | 84 份声明式向量、15 测试层 | Lane-CFR runner；[conformance/README.md](../../../conformance/README.md) | 全局 46 `pass` / 38 `not-run`；已执行项不构成客户端平台证据 |
| 平台 PoC | 各平台真实 API/真实 OS 行为验证 | Lane-CON 编排、gate 权威见 [readiness-gates](../../governance/readiness-gates.md) | `MAC/LNX-POC-*`、`IOS-POC-*`、Android `POC-*` 全部 `not-run` |
| 端到端（E2E） | 客户端↔kernel-server 全链路 | [tests/e2e](../../../tests/e2e/README.md)（占位，M4/M5 充实） | `planned`；未执行 |

## 2. 证据口径

- 结果五态与证据 digest 规则的 canonical：[docs/standards/conformance-evidence.md](../../../docs/standards/conformance-evidence.md)；本文不复制。
- schema-valid ≠ behavior-pass；包级单元测试通过不替代 conformance runner 执行；PoC 计划存在不等于 PoC pass。
- 证据文件不入库：以 digest 引用 `artifacts/evidence/` 产物；客户端证据指针集中在 [governance/evidence-index.md](../../governance/evidence-index.md)。
- 安全负例不可豁免（不得以 documented-degradation 抵扣）；拒绝路径断言"拒绝前无副作用"。

## 3. 与 gate 的关系

测试分层不改变实现 gate：任何客户端实现里程碑仍须满足 [Console 实现 gate](../../governance/readiness-gates.md#console-实现-gate)（Agent Hub 另加其 [GOVERNANCE §7](../../agent-hub/docs/GOVERNANCE.md#7-实现-gate不可跳过)）。在 gate 通过前，本文所列各层保持 `not-run`/`planned`，不得虚报。

# 共用客户端测试策略

> 类别：informative test strategy ｜ owner：Lane-CON（各层执行归所属车道）｜ 日期：2026-07-20
>
> 本文只定义客户端侧测试分层与证据口径的**消费方视图**；不新增测试义务、不替代 [conformance/README.md](../../../conformance/README.md) 的测试层定义，也不构成任何"测试已执行"声明。**当前全部客户端相关层状态：`not-run` / `none` / `planned`。**

## 1. Test pyramid 分层

| 层 | 范围 | 归属/入口 | 当前状态 |
|---|---|---|---|
| 包级单元测试 | 当前仅 `packages/contracts-ts` 有意义（合同编码/bundle/projection/golden 对比）；`sdk-ts`、`agent-shell` 为 M0 skeleton | Lane-CTR / Lane-TSC；[contracts-ts](../../../packages/contracts-ts/src/index.ts) | contracts-ts 包级测试存在（不作 REQ 级声明）；sdk/shell `planned` |
| 跨语言 golden | Rust/TS canonical 编码与 digest 夹具对比 | Lane-CTR；[tests/golden/README.md](../../../tests/golden/README.md) | 夹具已提供；CI golden job 对比（非客户端功能证据） |
| conformance 向量 | 76 份声明式向量、15 测试层 | Lane-CFR runner；[conformance/README.md](../../../conformance/README.md) | **76 份全部 `not-run`**（runner 为枚举骨架，执行能力待 Lane-CFR） |
| 平台 PoC | 各平台真实 API/真实 OS 行为验证 | Lane-CON 编排、gate 权威见 [readiness-gates](../../governance/readiness-gates.md) | `MAC/LNX-POC-*`、`IOS-POC-*`、Android `POC-*` 全部 `not-run` |
| 端到端（E2E） | 客户端↔kernel-server 全链路 | [tests/e2e](../../../tests/e2e/README.md)（占位，M4/M5 充实） | `planned`；未执行 |

## 2. 证据口径

- 结果五态与证据 digest 规则的 canonical：[docs/standards/conformance-evidence.md](../../../docs/standards/conformance-evidence.md)；本文不复制。
- schema-valid ≠ behavior-pass；包级单元测试通过不替代 conformance runner 执行；PoC 计划存在不等于 PoC pass。
- 证据文件不入库：以 digest 引用 `artifacts/evidence/` 产物；客户端证据指针集中在 [governance/evidence-index.md](../../governance/evidence-index.md)。
- 安全负例不可豁免（不得以 documented-degradation 抵扣）；拒绝路径断言"拒绝前无副作用"。

## 3. 与 gate 的关系

测试分层不改变实现 gate：任何客户端实现里程碑仍须满足 [Console 实现 gate](../../../docs/platforms/README.md#console-实现-gate)（Agent Hub 另加其 [GOVERNANCE §7](../../agent-hub/docs/GOVERNANCE.md#7-实现-gate不可跳过)）。在 gate 通过前，本文所列各层保持 `not-run`/`planned`，不得虚报。

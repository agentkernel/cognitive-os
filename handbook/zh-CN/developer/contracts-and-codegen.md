---
doc_id: dev.contracts-codegen
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-contracts/src/bin/contracts-codegen.rs
    symbols: ["CORE_SET"]
  - path: crates/cognitive-contracts/src/canonical.rs
    symbols: ["canonicalize", "digest_json"]
  - path: packages/contracts-ts/src/canonical.ts
  - path: crates/cognitive-domain/src/transitions.rs
    symbols: ["table", "find_edge"]
contracts:
  - specs/registry/requirements.yaml
  - specs/registry/errors.yaml
  - specs/registry/state-domains.yaml
tests:
  - crates/cognitive-contracts/tests/golden_fixtures.rs
  - tests/golden/README.md
fingerprint: "sha256:a1c451d34454a9eb22c9936b6b0ea92f8536a5df3dd8bd25d18fb0e6b724f8ee"
non_claims:
  - 生成绑定是形状级投影；JSON Schema 仍是唯一形状真相，codegen 绝不放松它们。
---

# 合同与代码生成

## Canonical 编码

两种语言实现完全相同的 canonical JSON profile（键排序、无冗余空白、禁止非有限数、
`-0` 规范化、仅整数范围、NFC 字符串）与域分离 SHA-256 digest
（`digest_json(input, domain)`）。跨语言字节一致由 golden fixture
（`tests/golden/*.json`）强制：Rust 与 TS 输出器在 CI 中必须逐字节复现。

## Codegen 流水线

`contracts-codegen` 读取 `specs/schemas/` 中钉住的 `CORE_SET`，产出确定性 Rust 模块
（`crates/cognitive-contracts/src/generated/`，53 个）与 TypeScript 模块
（`packages/contracts-ts/src/generated/`，含 index/registry 共 55 个）。产物入库；
CI 重生成并 diff，因此手改或 schema 漂移都会失败。错误注册表
（`specs/registry/errors.yaml`，55 码）在两侧生成 `RegisteredErrorCode` 枚举；未知
码在解析时 fail-closed。

## 内嵌转移表

`cognitive-domain` 编译期内嵌五张 `specs/transitions/*.transitions.json` 表；
`table(domain)` 暴露版本 + canonical digest，kernel 第 1 步表 pin 检查精确对照它
们。不走 Lane-CTR 合同程序直接改转移 JSON 会破坏 pin——设计使然。

## 合同变更纪律（Lane-CTR）

真正的合同变更同批联动：注册表条目、schema、两侧生成树、转移表（如适用）、符合性向
量与 docs-sync 义务——之后漂移门、consistency 检查器、traceability 矩阵、符合性
runner 才会一致通过。实现代码绝不为让测试通过而改 `specs/`（公理 A6）。

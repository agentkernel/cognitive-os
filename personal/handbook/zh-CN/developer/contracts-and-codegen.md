---
doc_id: dev.contracts-codegen
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: core/crates/cognitive-contracts/src/bin/contracts-codegen.rs
    symbols: ["CORE_SET"]
  - path: core/crates/cognitive-contracts/src/canonical.rs
    symbols: ["canonicalize", "digest_json"]
  - path: core/packages/contracts-ts/src/canonical.ts
  - path: core/crates/cognitive-domain/src/transitions.rs
    symbols: ["table", "find_edge"]
contracts:
  - core/specs/registry/requirements.yaml
  - core/specs/registry/errors.yaml
  - core/specs/registry/state-domains.yaml
tests:
  - core/crates/cognitive-contracts/tests/golden_fixtures.rs
  - core/tests/golden/README.md
fingerprint: "sha256:022dfbbee98991f7715eb4e092f6ebc52e28b6ed95b90dc122c2a1cb0fcc8a46"
non_claims:
  - 生成绑定是形状级投影；JSON Schema 仍是唯一形状真相，codegen 绝不放松它们。
---

# 合同与代码生成

## Canonical 编码

两种语言实现完全相同的 canonical JSON profile（键排序、无冗余空白、禁止非有限数、
`-0` 规范化、仅整数范围、NFC 字符串）与域分离 SHA-256 digest
（`digest_json(input, domain)`）。跨语言字节一致由 golden fixture
（`core/tests/golden/*.json`）强制：Rust 与 TS 输出器在 CI 中必须逐字节复现。

## Codegen 流水线

`contracts-codegen` 读取 `core/specs/schemas/` 中钉住的 `CORE_SET`，产出确定性 Rust 模块
（`core/crates/cognitive-contracts/src/generated/`，53 个）与 TypeScript 模块
（`core/packages/contracts-ts/src/generated/`，含 index/registry 共 55 个）。产物入库；
CI 重生成并 diff，因此手改或 schema 漂移都会失败。错误注册表
（`core/specs/registry/errors.yaml`，55 码）在两侧生成 `RegisteredErrorCode` 枚举；未知
码在解析时 fail-closed。

## 内嵌转移表

`cognitive-domain` 编译期内嵌五张 `core/specs/transitions/*.transitions.json` 表；
`table(domain)` 暴露版本 + canonical digest，kernel 第 1 步表 pin 检查精确对照它
们。不走 Lane-CTR 合同程序直接改转移 JSON 会破坏 pin——设计使然。

## 合同变更纪律（Lane-CTR）

真正的合同变更同批联动：注册表条目、schema、两侧生成树、转移表（如适用）、符合性向
量与 docs-sync 义务——之后漂移门、consistency 检查器、traceability 矩阵、符合性
runner 才会一致通过。实现代码绝不为让测试通过而改 `core/specs/`（公理 A6）。

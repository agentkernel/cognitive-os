# shared/docs/telemetry-evidence — 遥测与证据口径

> 类别：informative ｜ owner：Lane-CON ｜ 状态：`planned`；telemetry implementation `not-implemented`

- **用途**：客户端遥测、脱敏、留存与证据分类口径的说明域。
- **内容**：
  - 遥测/脱敏/留存政策：[telemetry-redaction-retention-policy.md](telemetry-redaction-retention-policy.md)；
  - 证据分类与五态结果口径 canonical：[docs/standards/conformance-evidence.md](../../../../docs/standards/conformance-evidence.md)（只引用不复制）。
- **边界**：当前不存在任何客户端遥测实现；证据文件不入库、以 digest 引用（`artifacts/evidence/`）；`not-run`/`planned` 不得呈现为通过。
- **gate**：遥测提供方/驻留/retention 是未冻结产品选择（PRODUCT-DESIGN §20.2），实现受 [Console 实现 gate](../../../governance/readiness-gates.md#console-实现-gate) 阻断。

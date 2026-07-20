# shared/docs/identity-session — 身份与 session 消费说明

> 类别：informative ｜ owner：Lane-CON ｜ 状态：`planned`

- **用途**：客户端侧身份、认证 session、设备与凭据边界的说明入口。
- **canonical 指针**（不复制正文）：
  - 判定顺序与 capability 口径：[docs/standards/authn-authz-capability.md](../../../../docs/standards/authn-authz-capability.md)；
  - Windows v1 身份/TOFU/账号产品行为：[trust-safety-ux §3](../../../pc/docs/security/trust-safety-ux.md#3-身份tofu-与账号)。
- **边界**：客户端不是 IdP、不是 authority；认证/授权判定在服务端；本目录不定义凭据存储格式或密钥规则。
- **gate**：[Console 实现 gate](../../../governance/readiness-gates.md#console-实现-gate)；SDK 侧归 [Lane-TSC](../../../../docs/plan/PARALLEL-LANES.md)。

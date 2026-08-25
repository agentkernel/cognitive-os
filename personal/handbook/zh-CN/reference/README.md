---
doc_id: ref.index
locale: zh-CN
kind: navigation
audience: [user, developer, ai]
generated: false
---

# 参考手册

机器派生参考由 `node tools/src/generate-handbook.mjs` 从实现源码与注册合同**生成**——
绝不手改（CI 强制字节一致）：

- [`cognitive` CLI](cli-cognitive.md) —— 产品 CLI usage，二进制原文
- [`admin-cli`](cli-admin.md) —— 管理回退入口 usage
- [HTTP API](http-api.md) —— 全部 daemon 路由及方法/通道
- [错误码](errors.md) —— 全部 55 个注册码
- [配置与状态文件](config-files.md)
- [环境变量](environment-variables.md)
- [状态迁移](state-transitions.md) —— 五台注册状态机
- [JSON Schema](schemas.md) —— 按 `$id` 列出的全部机器 schema
- [原生 Tool 目录](tool-catalog.md)

手工维护、指纹守护：

- [能力状态矩阵](capability-status.md) —— 全产品面的 implemented / partial /
  designed / unavailable
- [兼容性](compatibility.md) —— 平台、pin 与支持边界

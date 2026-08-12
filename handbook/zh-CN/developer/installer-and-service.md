---
doc_id: dev.installer-service
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-runtime/src/linux_bundle.rs
    symbols: ["verify_linux_bundle", "stage_verified_bundle", "activate_after_health_check"]
  - path: crates/cognitive-runtime/src/linux_bundle_installation.rs
    symbols: ["PreparedLinuxBundleInstallation", "install_linux_bundle"]
  - path: crates/cognitive-runtime/src/linux_bundle_service.rs
    symbols: ["install_linux_bundle_single_service", "render_personal_user_service_unit"]
  - path: deploy/linux/install.sh
  - path: crates/cognitive-runtime/src/bin/linux_bundle_campaign_builder.rs
tests:
  - crates/cognitive-runtime/tests/linux_bundle_single_service.rs
  - crates/cognitive-runtime/tests/linux_installer_bootstrap.rs
  - crates/cognitive-runtime/tests/linux_bundle_installation.rs
fingerprint: "sha256:2c6d89ca70c76f223dfd52bbbe917bf2d818934af47b10246592164eb06dc64c"
non_claims:
  - campaign 构建器使用实验签名密钥；此处不做生产签名仪式、GitHub Release 或 B01 声明。
---

# 安装器与服务

## 离线 bundle 校验器

`verify_linux_bundle` 校验对 canonical bundle 元数据的 Ed25519 签名 attestation：产品/平台
身份、与版本一致的根目录、逐条目精确 SHA-256 + 大小、路径安全（无绝对/`..`/符号链接
逃逸），并拒绝内嵌 Node/Pi 载荷（`node_modules`、`pi-runtime/` 等）。`stage_verified_bundle`
在解出到 `deployments/<version>/` 时重复校验并施加 0700/0755/0644 模式；
`activate_after_health_check` 先过健康门再原子切换 `active-version` 文本指针。

## 带锁安装事务

`PreparedLinuxBundleInstallation::prepare` → `install_linux_bundle`：跨进程 OS 文件
锁（`installer.lock`）序列化安装器；步骤为 verify → stage →（可选 systemd unit 渲
染/启用）→ 健康探测 → activate → 不可变回执。失败按逆序补偿（恢复前一 unit 与指
针、移除已 stage 版本）。回执与失败报告类型化；`--dry-run` 只做校验。

## 单服务生产形态

`install_linux_bundle_single_service` 渲染 `cognitiveos-personal.service`（用户
systemd）：`ExecStart=<版本化 kernel-server> --personal --bind 127.0.0.1:48181`、
`NoNewPrivileges=true`、`Restart=on-failure`，兼容 `cognitive-daemon@.service` 模
板。健康确认同时要求 `GET /personal/health` 存活**与** MainPID 位于预期部署根之下的
身份——同端口的冒名进程会使激活失败。

## 引导链

渲染出的 `install.sh`（模板在 `deploy/linux/`，由 campaign 构建器填入钉住 URL 与
digest）经 HTTPS 有界下载、单一钉住跳转主机、对安装器二进制做 SHA-256 校验后移交
Rust 安装器——无 `curl | sh`、无 sudo、无内嵌 secret。构建器
（`linux_bundle_builder`）组装 daemon+CLI+installer bundle 并用**实验**密钥签名；
release-manifest 校验（`release_manifest.rs`）作为独立的 P7-T01 门覆盖 manifest 身
份/digest/工具链 pin。

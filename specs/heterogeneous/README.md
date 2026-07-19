# CognitiveOS Heterogeneous Companion Specification

> 版本：v0.2 Draft

> 状态：Companion Specification；仅定义语义与符合性要求，不表示存在实现。

> 标识：`cognitiveos.heterogeneous/0.2`

> 范围：CPU/GPU/NPU/FPGA/CIM 等异构资源图、句柄、放置、校准、漂移、误差与回退


## 1. 规范约定

本文中的 **MUST**、**MUST NOT**、**SHOULD**、**SHOULD NOT**、**MAY** 按 RFC 2119 与 RFC 8174 解释。

只有大写英文规范词构成规范性要求；普通中文“必须/应当”仅作说明。

本规范叠加于 CognitiveOS Core；冲突时采用更严格的安全、授权、预算与证据边界。

实现状态必须声明为 `implemented`、`planned`、`experimental` 或 `unsupported`，不得把本草案当作实现证据。

## 2. 范围与非目标

本 Profile 规范跨异构计算/存储/互连资源的可审计放置语义。

它不保证某硬件更快、更省能或更准确；性能只在固定工作负载和证据条件下成立。

CIM 为 compute-in-memory，包括数字、模拟或混合实现；近似结果默认不是授权或安全证明。

## 3. ResourceGraph

ResourceGraph 节点描述 compute、memory、storage、sensor/actuator gateway 与 trust domain。

边描述带宽、延迟、能耗、可达性、数据移动、加密和故障相关性。

节点字段包括 kind、vendor/model、firmware/driver、supported operations、precision、capacity、residency、isolation、health、thermal、endurance 和 certification envelope。

图是版本化观察/策略输入，不因资源自报而自动成为 authority。

[REQ-HET-RG-001] 每次受治理 placement **MUST** 固定 ResourceGraph 版本、节点/路径健康度与策略 digest。

## 4. ResourceHandle 与 BufferHandle

ResourceHandle 是带 tenant、device identity、epoch、permissions、lease 和 integrity 的不可伪造引用。

BufferHandle 固定 shape、dtype、layout、size、residency、sensitivity、owner、lifetime 与 content/version digest。

句柄不能携带秘密，也不能仅凭地址授予访问。

跨 trust domain 映射或 DMA 前重新授权并验证边界。

[REQ-HET-HDL-001] handle **MUST** 作用域化、可撤销、带 epoch，并拒绝越界、悬挂和跨租户使用。

[REQ-HET-DATA-001] 数据移动 **MUST** 验证驻留、敏感度、目的、目标域和 egress 预算。

## 5. Placement

PlacementRequest 声明 operation descriptor、输入 handles、精度/误差要求、deadline、预算、residency、trust、可恢复性和 fallback。

PlacementDecision 固定 graph、policy、operation/kernel、compiler、driver、firmware、calibration、路径、资源预留和预期误差。

调度器可按延迟、吞吐、费用、能耗或热约束优化，但不得越过硬安全与数据边界。

运行时实际 placement 与 decision 不一致时失败或重新决策，不可静默迁移到更弱域。

[REQ-HET-PLC-001] placement **MUST** 可复现地固定所有影响语义、驻留和误差的版本。

[REQ-HET-PLC-002] fallback **MUST** 在执行前声明并单独满足授权、预算、驻留和验证条件。

## 6. CIM 校准、漂移与误差

CalibrationRecord 固定设备、阵列、时间、温度/电压范围、方法、参考数据、误差模型和有效期。

DriftObservation 记录相对 calibration 的偏移、测量条件、置信度和阈值。

ErrorEnvelope 声明 quantization、noise、saturation、retention、write variability、accumulation 与端到端任务误差界。

超出温度、电压、耐久、漂移或误差包络时停止使用该 placement，转入验证过的 fallback 或失败。

抽样校验频率由风险、漂移和认证证据决定，不在本规范硬编码常数。

[REQ-HET-CAL-001] CIM 执行 **MUST** 固定有效 CalibrationRecord 与 ErrorEnvelope。

[REQ-HET-DRIFT-001] 漂移超阈值或校准过期 **MUST** 隔离资源并触发重校准或 fallback。

[REQ-HET-ERR-001] 近似输出 **MUST** 携带实际/估计误差、包络版本和验证状态。

## 7. 结果验证与回退

关键结果可采用精确后端复算、抽样、冗余路径、范围证明或领域 verifier。

fallback 是新 placement decision；迁移保存输入 digest、未决 effect、成本和 lineage。

fallback 失败不得再次静默降低精度或扩大出域。

授权、capability 验证、audit 完整性、CAS 与最终具身安全仲裁默认使用确定性/经认证路径。

[REQ-PROFILE-HET-001] 实现 **MUST** 固定 placement、校准和误差版本，验证驻留及 fallback，且 **MUST NOT** 将超认证包络近似结果用于授权、审计完整性或最终安全仲裁。

[REQ-HET-FB-001] fallback 结果 **MUST** 重新验证并保留从原 decision 到新 decision 的谱系。

## 8. 生命周期与最小操作

资源生命周期为 `DISCOVERED -> ATTESTED -> AVAILABLE -> RESERVED -> ACTIVE -> DRAINING -> OFFLINE/QUARANTINED`；状态转换保留 causation 与健康证据。

`DescribeResources` 返回固定 ResourceGraph 视图；`ReservePlacement` 原子预留容量和预算；`SubmitPlacement` 绑定 decision 与 handles；`ObservePlacement` 返回进度、成本、漂移和错误；`ReleasePlacement` 回收未消费资源但不抹除成本。

设备进入 DRAINING 后不得接收新 placement；已有 governed effect 必须完成、迁移或进入显式 unknown-outcome 对账。

资源健康、固件、驱动、编译器、校准或 trust domain 改变时，现有 reservation 必须重新验证。

[REQ-HET-LIFE-001] 资源状态转换 **MUST** 使用 CAS/epoch，并防止 OFFLINE、QUARANTINED 或旧 epoch 资源接受新工作。

[REQ-HET-OPS-001] 异构实现 **MUST** 提供上述资源描述、预留、提交、观察与释放的等价传输无关语义。

[REQ-HET-OBS-001] 运行观测 **MUST** 报告实际资源、路径、校准、误差、热状态、消耗与 fallback 事件。

## 9. 失败、安全与符合性

错误包括 RESOURCE_UNAVAILABLE、HANDLE_STALE、PLACEMENT_CONFLICT、RESIDENCY_DENIED、CALIBRATION_EXPIRED、DRIFT_EXCEEDED、ERROR_BOUND_EXCEEDED、FALLBACK_FAILED。

unknown hardware outcome 按 Effect 协议 reconcile；设备 reset 不能自动证明 kernel 未执行。

供应链、固件、编译器和 operation descriptor digest 变化触发重新验收。

符合性场景：悬挂 handle 拒绝；跨租户 buffer 拒绝；CIM 漂移触发隔离；fallback 不违反驻留；超误差结果不进入授权；设备断电恢复不重复外部效果。

[REQ-HET-SEC-001] 实现 **MUST** 测试恶意 descriptor、DMA 越界、固件漂移、热超限、校准投毒和 fallback 降级。

[REQ-HET-CONF-001] 声明 **MUST** 列出支持资源、operation/precision、校准方法、误差证据、驻留域、fallback 与未认证用途。


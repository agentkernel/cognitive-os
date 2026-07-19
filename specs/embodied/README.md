# CognitiveOS Embodied Safety Companion Specification

> 版本：v0.2 Draft

> 状态：Companion Specification；仅定义语义与符合性要求，不表示存在实现。

> 标识：`cognitiveos.embodied/0.2`

> 范围：具身世界状态、时间/坐标/不确定性、三回路与独立安全域


## 1. 规范约定

本文中的 **MUST**、**MUST NOT**、**SHOULD**、**SHOULD NOT**、**MAY** 按 RFC 2119 与 RFC 8174 解释。

只有大写英文规范词构成规范性要求；普通中文“必须/应当”仅作说明。

本规范叠加于 CognitiveOS Core；冲突时采用更严格的安全、授权、预算与证据边界。

实现状态必须声明为 `implemented`、`planned`、`experimental` 或 `unsupported`，不得把本草案当作实现证据。

## 2. 范围与安全声明

本 Profile 治理连接物理传感器和执行器的 CognitiveOS 部署。

它不替代行业功能安全、机器人、医疗、车辆或机械标准，也不提供通用 WCET 常数。

认知输出始终是候选建议；最终执行权限由安全域与执行器仲裁。

## 3. 世界状态五类

`PhysicalState`：位置、速度、姿态、力、温度、能量等物理量。

`EnvironmentState`：地图、障碍、表面、天气、工作区与动态对象。

`HumanState`：人员存在、姿态、意图估计、授权与安全距离；推断必须带不确定性。

`DeviceState`：传感器、执行器、控制器、健康度、模式与故障。

`SafetyState`：安全包络、联锁、急停、hazard、降级和 safe state。

五类可分 authority；融合视图不能抹除来源、时间或冲突。

[REQ-EMB-STATE-001] 每个具身观察 **MUST** 标注世界状态类别、source、authority、时间、坐标系、不确定性和有效期。

## 4. 时间、坐标与不确定性

时间至少区分 measurement_time、ingest_time、decision_time 与 actuation_time。

时钟源、同步方式、最大偏差和过期阈值在 manifest 声明。

坐标引用包含 frame id、transform chain、transform version 与有效时间。

单位采用显式单位系统；禁止依靠未声明默认单位。

不确定性可表示 covariance、区间、置信等级、质量标志或领域适当模型，但必须可机器判断是否超包络。

未知、过期、外推超限、frame 缺失或 transform 冲突不可压缩成确定事实。

[REQ-EMB-TIME-001] 安全相关决策 **MUST** 验证 observation age、clock bound 和 actuation deadline。

[REQ-EMB-FRAME-001] 跨坐标数据组合 **MUST** 固定 transform 版本并验证时间适用性。

[REQ-EMB-UNC-001] 超出声明不确定性包络的观察 **MUST** 降级、停止或进入 safe state。

## 5. 三回路

安全回路：独立、最短路径、确定性监测 hazard，执行急停/限幅/safe state。

控制回路：在认证周期内执行轨迹跟踪、姿态或设备控制，服从安全包络。

认知回路：低频规划、语义理解、任务分解和候选 skill 选择。

三回路通过版本化 setpoint、constraint、observation 与 event 交换，不共享隐式权威。

认知回路不得直接绕过控制器写原始 actuator command，除非该路径本身纳入适用认证与安全仲裁。

[REQ-EMB-LOOP-001] 安全回路 **MUST** 能在认知服务和其网络不可用时独立进入 safe state。

[REQ-EMB-LOOP-002] 控制与认知命令 **MUST** 带 deadline、frame、unit、envelope、capability 和 fencing token。

## 6. 技能与执行协议

技能描述声明 precondition、postcondition、workspace、速度/力/热限制、终止条件、取消和 fallback。

Intent 固定 world snapshot、技能版本、参数、轨迹/目标 digest 与安全包络。

执行前由安全域检查 current state；执行中持续检查 envelope；执行后由物理证据验证。

receipt 不证明物理世界达到目标。

emergency_safety 可先执行后记录，仅用于减轻迫近风险且不得扩大权限。

[REQ-EMB-ACT-001] governed physical effect **MUST** 按 Intent→Authorize→Safety Admit→Execute→Observe→Verify→Commit 治理。

[REQ-EFF-007] emergency 路径 **MUST** fail safe，普通认知活动不得覆盖最终执行器仲裁。

## 7. 安全域

安全域包含独立 watchdog、急停输入、安全包络存储、最终命令仲裁、健康监测与受保护 audit。

安全策略、固件、校准和 envelope 变更经独立 authority、版本发布和回滚。

普通 Agent、模型、远端工具和业务网络无权禁用 watchdog 或扩大 envelope。

失联、过期命令、传感器冲突、过热、定位失效或 fencing 失败触发预声明降级/safe state。

[REQ-PROFILE-EMB-001] 急停、安全包络和最终执行器仲裁 **MUST** 独立于普通认知活动及其网络路径。

[REQ-EMB-SAFE-001] 安全域配置 **MUST** 受独立 authority、完整性校验、版本和审计保护。

## 8. 失败、安全与符合性

OUTCOME_UNKNOWN 物理效果不得盲重发；先观察设备与世界状态并 reconcile。

停止命令本身可能未知，必须依赖 watchdog/安全回路而非网络确认。

安全 audit 默认不保存无关人类敏感原始数据，只保存必要受控证据。

符合性场景：认知网络断开仍急停；过期/错 frame 命令拒绝；传感器冲突进入降级；旧 epoch actuator 命令拒绝；模型声明完成但物理 verifier 不通过；超 envelope 输出被仲裁限幅/拒绝。

[REQ-EMB-SEC-001] 实现 **MUST** 测试断网、时钟偏差、坐标错误、单位错误、传感器欺骗、watchdog 触发和 actuator stuck。

[REQ-EMB-CONF-001] 符合性声明 **MUST** 列出适用行业标准、hazard analysis、时限证据、认证边界、已知失效模式与测试引用。


## Shell 与用户旅程映射
Shell 的 stop/kill 不能替代 safety stop；用户只能请求有界 setpoint、技能取消或 emergency action。

[REQ-EMB-SHELL-001] 具身 Shell **MUST** 分别显示 cancellation、控制器状态、watchdog 与 physical safe-state 证据，且普通/管理通道均不能扩大 safety envelope。

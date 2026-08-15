//! 守护进程私有的 Memory/Skill 受治理消费端口。
//!
//! 该端口只记录精确 scope/pin/digest 装载事实，不授予客户端写权限，也不完成 Task。

use crate::ports::StorePortError;
use cognitive_domain::ObjectId;

/// 一次受治理消费要装入 Context 的 Memory 精确钉。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryConsumptionPin {
    pub memory_id: ObjectId,
    pub source_id: ObjectId,
    pub source_digest: String,
}

/// 一个仍满足当前 Task、scope、purpose 与 source 治理绑定的 Memory 元数据候选。
///
/// 该值不含正文，消费方必须先验证这些字段，再调用 `ContextStore` 读取 body。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EligibleMemoryConsumption {
    pub pin: MemoryConsumptionPin,
    pub tenant_id: String,
    pub owner_ref: String,
    pub resource_scope: String,
    pub target_scope: String,
    pub purpose: String,
    pub source_provenance_ref: String,
}

/// 一次受治理消费要装入 Context 的 Skill 精确钉。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillConsumptionPin {
    pub binding_id: ObjectId,
    pub revision_id: ObjectId,
    pub package_id: ObjectId,
    pub content_digest: String,
}

/// 一条只追加的 Memory/Skill 消费记录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemorySkillConsumptionRecord {
    pub consumption_id: ObjectId,
    pub task_ref: String,
    pub contract_epoch: i64,
    pub context_request_id: ObjectId,
    pub context_request_digest: String,
    pub session_ref: String,
    pub reuse_of: Option<ObjectId>,
    pub memory: Vec<MemoryConsumptionPin>,
    pub skill: Vec<SkillConsumptionPin>,
    pub canonical_json: String,
}

/// 守护进程私有的 Memory/Skill 消费持久化。
pub trait MemorySkillConsumptionStore {
    /// 列出当前 scope/purpose 下仍可装载的 Memory 精确钉。
    /// 遗忘、过期、源漂移项必须缺席。
    fn list_eligible_memory_pins(
        &self,
        governance_scope: &str,
        task_ref: &str,
        purpose: &str,
        observed_at_unix_seconds: i64,
    ) -> Result<Vec<EligibleMemoryConsumption>, StorePortError>;

    /// 列出对当前 Task/workspace 仍有效的 Skill 精确钉。
    /// 已撤销或范围不匹配的绑定必须缺席。
    fn list_eligible_skill_pins(
        &self,
        workspace_scope: &str,
        task_ref: &str,
    ) -> Result<Vec<SkillConsumptionPin>, StorePortError>;

    /// 追加一条不可变消费记录。同一 session 绑定冲突必须失败闭合。
    fn append_memory_skill_consumption(
        &self,
        record: &MemorySkillConsumptionRecord,
    ) -> Result<(), StorePortError>;

    /// 按身份加载一条消费记录。
    fn load_memory_skill_consumption(
        &self,
        consumption_id: &ObjectId,
    ) -> Result<Option<MemorySkillConsumptionRecord>, StorePortError>;

    /// 加载同一 Task/epoch/request 上最近一条消费记录，供跨会话复用。
    fn load_latest_memory_skill_consumption(
        &self,
        task_ref: &str,
        contract_epoch: i64,
        context_request_id: &ObjectId,
    ) -> Result<Option<MemorySkillConsumptionRecord>, StorePortError>;

    /// 读取一个精确 Skill revision 的内容钉与规范载荷，供 digest 复核。
    fn load_skill_revision_payload(
        &self,
        revision_id: &ObjectId,
    ) -> Result<Option<(String, String)>, StorePortError>;
}

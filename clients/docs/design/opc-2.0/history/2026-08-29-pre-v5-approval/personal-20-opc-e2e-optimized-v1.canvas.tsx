/**
 * PERSONAL 2.0 OPC INTERACTION PROTOTYPE — optimized v1
 *
 * Built-in mock data and local React state only. This Canvas does not connect
 * to a daemon, network, storage, filesystem, Provider, model, Skill, MCP
 * server, connector, or SecretStore. It cannot create Projects, send messages,
 * install capabilities, grant permissions, publish, reconcile Effects, admit
 * Memory, or issue receipts. Target-state samples are labelled explicitly.
 * HITL buttons are simulated: they change local prototype state only.
 *
 * v1 delta vs post-subtraction chrome: ① horizontal gated confirm cards with
 * canvas↔chat sync; Knowledge import + filters; Knowledge/Settings chat
 * default-collapsed (column restore, not overlay). Not V2 CEO-rail chrome.
 *
 * Archived 2026-08-29. Not current product chrome. Current chrome is
 * personal-20-opc-e2e-optimized-v5.
 *
 * Design artifact:
 * d:\agent-kernel\clients\docs\design\opc-2.0\history\2026-08-29-pre-v5-approval\personal-20-opc-e2e-optimized-v1.canvas.tsx
 * Cursor-openable archived copy (IDE detection path; not a second baseline):
 * C:\Users\wuron\.cursor\projects\d-agent-kernel\canvases\history\2026-08-29-pre-v5-approval\personal-20-opc-e2e-optimized-v1.canvas.tsx
 * Hosted and repository copies must stay byte-aligned.
 */

import {
  Callout,
  Select,
  TextArea,
  TextInput,
  useEffect,
  useHostTheme,
  useState,
  type CSSProperties,
} from "cursor/canvas";

type Scene =
  | "empty-home"
  | "create-init"
  | "create-members"
  | "create-process"
  | "create-test"
  | "create-joint"
  | "today-incomplete"
  | "today"
  | "projects"
  | "project"
  | "add-member"
  | "hitl"
  | "knowledge"
  | "settings"
  | "state-lab";

type Lifecycle = "empty" | "creating" | "live";
type Period = "today" | "week" | "month";
type Tone = "neutral" | "good" | "warn" | "bad" | "info";
type StateKey =
  | "loading"
  | "empty"
  | "working"
  | "error"
  | "success"
  | "partial"
  | "blocked"
  | "unknown"
  | "offline";
type SurfaceKey =
  | "today"
  | "create"
  | "projects"
  | "project"
  | "hitl"
  | "knowledge"
  | "settings";
type ConfirmId =
  | "process"
  | "outputs"
  | "cycle"
  | "format"
  | "skill"
  | "tools"
  | "mcp"
  | "knowledge"
  | "env"
  | "files"
  | "auto"
  | "triggers"
  | "cost"
  | "rights"
  | "preview"
  | "method";
type WizardId = "brief" | ConfirmId;
type PreviewAge = "fresh" | "stale";
type HitlFate = "idle" | "approved" | "narrowed" | "rejected" | "stopped";
type ConnectionStatus = "none" | "connected" | "failed";
type KnowledgeTab = "files" | "import" | "why" | "memory";
type KnowledgeScope = "all" | "shared" | "weekly";
type KnowledgeKind = "all" | "markdown" | "pdf" | "image" | "link";
type ImportPhase =
  | "idle"
  | "importing"
  | "duplicate"
  | "parse-fail"
  | "secret-detected"
  | "indexed";
type ImportPolicy = "copy" | "reference";
type ImportSourceKind = "files" | "link" | "image" | "video";
type ImportDest = "shared" | "weekly";
type WizardReceipt = { id: number; text: string };
type KnowledgeFile = {
  id: string;
  title: string;
  project: Exclude<KnowledgeScope, "all">;
  projectLabel: string;
  kind: Exclude<KnowledgeKind, "all">;
  statusLabel: string;
  tone: Tone;
};

type MemberDraft = {
  id: string;
  name: string;
  duty: string;
  handoff: string;
  model: string;
  joined: boolean;
};

type ProcessStage = {
  id: string;
  label: string;
  owner: string;
  status: string;
  tone: Tone;
  mark: "none" | "auth" | "verify";
  complete: number;
  fail: number;
  avg: string;
  success: string;
};

const SCENES: ReadonlyArray<{ id: Scene; label: string }> = [
  { id: "empty-home", label: "Empty Home · 创建 Project only" },
  { id: "create-init", label: "Create ① project init" },
  { id: "create-members", label: "Create ② member init" },
  { id: "create-process", label: "Create ③ process init" },
  { id: "create-test", label: "Create ④ per-stage test" },
  { id: "create-joint", label: "Create ⑤ joint debug" },
  { id: "today-incomplete", label: "Today · continue create only" },
  { id: "today", label: "Today · after ⑤" },
  { id: "projects", label: "Projects · list and copy" },
  { id: "project", label: "Live Project · process axis" },
  { id: "add-member", label: "Add member" },
  { id: "hitl", label: "HITL canvas preview" },
  { id: "knowledge", label: "Knowledge" },
  { id: "settings", label: "Settings · models and skips" },
  { id: "state-lab", label: "State Lab · rendered coverage" },
];

const SCENE_TITLES: Record<Scene, string> = {
  "empty-home": "Today",
  "create-init": "创建 Project · ① 项目初始化",
  "create-members": "创建 Project · ② 成员初始化",
  "create-process": "创建 Project · ③ 流程初始化",
  "create-test": "创建 Project · ④ 分环节测试",
  "create-joint": "创建 Project · ⑤ 联合调试",
  "today-incomplete": "Today",
  today: "Today",
  projects: "Projects",
  project: "周报与客户跟进",
  "add-member": "加人",
  hitl: "需要你拍板",
  knowledge: "Knowledge",
  settings: "Settings",
  "state-lab": "State Lab",
};

const CONFIRM_ITEMS: ReadonlyArray<{ id: ConfirmId; label: string; detail: string }> = [
  { id: "process", label: "业务流程", detail: "收集事实 → 分析 → 起草周报 → 核对 → 交给 Owner" },
  { id: "outputs", label: "各环节产出", detail: "事实清单、建议稿、周报草稿、核对记录、可打开周报" },
  { id: "cycle", label: "周期", detail: "每周一 09:00，仅在本机在线时运行" },
  { id: "format", label: "保存形式", detail: "Markdown 周报 + 附件清单；不是发布到社交网络" },
  { id: "skill", label: "Skill", detail: "来源、版本、许可待审。安装不是授权。" },
  { id: "tools", label: "工具", detail: "检索与文档整理。无假 Install。" },
  { id: "mcp", label: "MCP", detail: "精确版本与权限需另批。无市场安装按钮。" },
  { id: "knowledge", label: "知识库", detail: "本项目资料；Obsidian 为底座，不必安装该应用" },
  { id: "env", label: "外部工作环境", detail: "本机在线。无云端 24/7 承诺。" },
  { id: "files", label: "文件权限", detail: "仅当前项目目录。扩权要再批。" },
  { id: "auto", label: "自动 / 批准", detail: "内部起草可自动；对外发送走画布预览" },
  { id: "triggers", label: "触发", detail: "手动、日程、已验收产出。同类不重叠。" },
  { id: "cost", label: "费用", detail: "估计或实际须标注来源；未知不写 0" },
  { id: "rights", label: "来源权利", detail: "外部文本不可信，不能当指令执行" },
  { id: "preview", label: "总预览", detail: "确认前项目未上线。离开会留草稿。" },
  { id: "method", label: "执行方式", detail: "每环节怎么做、周期、触发。不出现底层引擎名。" },
];

const WIZARD_STEPS: ReadonlyArray<{
  id: WizardId;
  label: string;
  detail: string;
  ownerAuthored: boolean;
}> = [
  {
    id: "brief",
    label: "业务描述",
    detail: "用业务语言说清楚要办成什么。离开会接续。连接失败要说出问题所在。",
    ownerAuthored: true,
  },
  ...CONFIRM_ITEMS.map((item) => ({ ...item, ownerAuthored: false })),
];

const KNOWLEDGE_FILES: readonly KnowledgeFile[] = [
  {
    id: "follow-md",
    title: "本周客户跟进.md",
    project: "weekly",
    projectLabel: "周报与客户跟进",
    kind: "markdown",
    statusLabel: "已索引 · 来源：Owner 导入",
    tone: "good",
  },
  {
    id: "scan-pdf",
    title: "扫描件.pdf",
    project: "weekly",
    projectLabel: "周报与客户跟进",
    kind: "pdf",
    statusLabel: "解析失败。原件保留。",
    tone: "bad",
  },
  {
    id: "brand-md",
    title: "品牌口径.md",
    project: "shared",
    projectLabel: "Owner 共享",
    kind: "markdown",
    statusLabel: "已索引 · 跨项目共享",
    tone: "good",
  },
  {
    id: "shot-png",
    title: "竞品截图.png",
    project: "weekly",
    projectLabel: "周报与客户跟进",
    kind: "image",
    statusLabel: "已索引 · 图片元数据",
    tone: "info",
  },
  {
    id: "public-link",
    title: "公开研究笔记",
    project: "shared",
    projectLabel: "Owner 共享",
    kind: "link",
    statusLabel: "引用 · 未复制原文",
    tone: "neutral",
  },
];

const PROCESS_STAGES: readonly ProcessStage[] = [
  {
    id: "collect",
    label: "收集本周事实",
    owner: "梅",
    status: "进行中 · 已 41 分钟",
    tone: "info",
    mark: "none",
    complete: 2,
    fail: 0,
    avg: "18 分",
    success: "2/2",
  },
  {
    id: "analyze",
    label: "分析与建议",
    owner: "林",
    status: "等待事实清单",
    tone: "neutral",
    mark: "none",
    complete: 1,
    fail: 0,
    avg: "24 分",
    success: "1/1",
  },
  {
    id: "draft",
    label: "起草周报",
    owner: "锐",
    status: "未开始",
    tone: "neutral",
    mark: "none",
    complete: 0,
    fail: 0,
    avg: "—",
    success: "—",
  },
  {
    id: "verify",
    label: "核对证据",
    owner: "林",
    status: "未开始",
    tone: "neutral",
    mark: "verify",
    complete: 0,
    fail: 0,
    avg: "—",
    success: "—",
  },
  {
    id: "deliver",
    label: "交给 Owner",
    owner: "林",
    status: "要你授权发送摘要",
    tone: "warn",
    mark: "auth",
    complete: 0,
    fail: 0,
    avg: "—",
    success: "—",
  },
];

const STATE_KEYS: readonly StateKey[] = [
  "loading",
  "empty",
  "working",
  "error",
  "success",
  "partial",
  "blocked",
  "unknown",
  "offline",
];

const STATE_LABELS: Record<StateKey, string> = {
  loading: "Loading",
  empty: "Empty",
  working: "Working",
  error: "Error",
  success: "Success",
  partial: "Partial",
  blocked: "Blocked",
  unknown: "Unknown",
  offline: "Offline",
};

const STATE_TONES: Record<StateKey, Tone> = {
  loading: "info",
  empty: "neutral",
  working: "info",
  error: "bad",
  success: "good",
  partial: "warn",
  blocked: "bad",
  unknown: "bad",
  offline: "warn",
};

const SURFACE_CONTEXT: Record<
  SurfaceKey,
  { label: string; object: string; source: string; firstAction: string; native: string }
> = {
  today: {
    label: "Today",
    object: "决策包与上线项目运行概览",
    source: "Project 运行投影",
    firstAction: "创建 Project，或处理这一件拍板",
    native: "空 Home 只留创建；未完成创建只留继续；上线后是决策包+概览，不是四泳道墙",
  },
  create: {
    label: "五段创建",
    object: "草稿、确认清单、班子、流程轴、可打开测试结果",
    source: "可恢复的创建草稿",
    firstAction: "用业务语言描述，或去 Settings 绑定助手",
    native: "①–⑤ 均为创建；⑤ 验收前没有日常 Today",
  },
  projects: {
    label: "Projects",
    object: "长期治理的工作空间",
    source: "Project 列表投影",
    firstAction: "创建 Project，或复制已上线项目为副本",
    native: "无默认/示范项目。副本未激活。",
  },
  project: {
    label: "已上线 Project",
    object: "业务流程轴与当前环节工作面",
    source: "当前阶段、成员、授权/核对应",
    firstAction: "只做这一环该做的事",
    native: "无可见 CEO 六步顶栏。聊天不能批。",
  },
  hitl: {
    label: "HITL 画布",
    object: "将做什么、完整预览、批准/改窄/拒绝",
    source: "daemon 签发预览的目标态样品",
    firstAction: "在画布上批准、改窄或拒绝",
    native: "过期预览不能批。执行中可停。聊天只有链接。",
  },
  knowledge: {
    label: "Knowledge",
    object: "项目资料、为什么用这段、可检查的 Memory",
    source: "本地资料与自动承认的对话记忆",
    firstAction: "导入资料，或检查/忘记一条 Memory",
    native: "无 Project 时锁定。③ 才为当前草稿打开。",
  },
  settings: {
    label: "Settings",
    object: "模型连接、本周不再问收回、通知恢复",
    source: "本地设置投影",
    firstAction: "下拉选择主流 Provider 并交接密钥",
    native: "无账单、无引擎商店、无 Inbox。密钥不回显。",
  },
};

const CREATE_SCENES: readonly Scene[] = [
  "create-init",
  "create-members",
  "create-process",
  "create-test",
  "create-joint",
];

const DEFAULT_BRIEF =
  "每周给自己一份可打开的经营周报，并跟进待回复客户。不要做成社交账号运营。";

function blankWizardValues(brief: string): Record<WizardId, string> {
  const values = { brief } as Record<WizardId, string>;
  for (const item of CONFIRM_ITEMS) values[item.id] = "";
  return values;
}

function blankWizardFlags(): Record<WizardId, boolean> {
  const flags = { brief: false } as Record<WizardId, boolean>;
  for (const item of CONFIRM_ITEMS) flags[item.id] = false;
  return flags;
}

function stateMessage(surface: SurfaceKey, state: StateKey) {
  const context = SURFACE_CONTEXT[surface];
  const messages: Record<StateKey, string> = {
    loading: `${context.label} 正在读取 ${context.object}。上次安全投影仍可见；离开不会丢掉草稿。`,
    empty: `${context.label} 还没有 ${context.object}。第一件有价值的事：${context.firstAction}。`,
    working: `${context.label} 进行中。进行中不是完成。真实控件已点名；没有假成功。`,
    error: `${context.label} 读 ${context.source} 失败。已输入内容和上次事实保留。`,
    success: `${context.label} 已刷新。变更、证据、新鲜度和下一步都看得见。`,
    partial: `${context.label} 有一部分可用，但有缺口。缺口不写成已就绪。`,
    blocked: `${context.label} 停在权限、输入或依赖上。已做的工作是安全的。`,
    unknown: `${context.label} 说不清结果。未知不是成功，也不是 0。禁止盲着重试。`,
    offline: `${context.label} 展示上次已知事实并标明已过时。主机离线不能当当前成功。`,
  };
  return messages[state];
}

function Tag({
  children,
  tone = "neutral",
}: {
  children: string;
  tone?: Tone;
}) {
  return (
    <span className="tag" data-tone={tone}>
      {children}
    </span>
  );
}

function Gap({
  children,
  environment = false,
}: {
  children: string;
  environment?: boolean;
}) {
  return (
    <Callout
      tone="warning"
      title={environment ? "Requires-backend + Requires-environment" : "Requires-backend"}
    >
      {children}
    </Callout>
  );
}

function noticeTone(
  tone: Tone,
): "info" | "success" | "warning" | "danger" | "neutral" {
  if (tone === "good") return "success";
  if (tone === "warn") return "warning";
  if (tone === "bad") return "danger";
  return tone;
}

function Notice({
  title,
  children,
  tone = "warn",
}: {
  title: string;
  children: string;
  tone?: Tone;
}) {
  return (
    <Callout title={title} tone={noticeTone(tone)}>
      {children}
    </Callout>
  );
}

function Heading({
  title,
  meta,
  action,
}: {
  title: string;
  meta?: string;
  action?: { label: string; onClick: () => void };
}) {
  return (
    <header className="section-heading">
      <div>
        <h3>{title}</h3>
        {meta ? <p>{meta}</p> : null}
      </div>
      {action ? (
        <button className="text-button" type="button" onClick={action.onClick}>
          {action.label}
        </button>
      ) : null}
    </header>
  );
}

function Segmented<T extends string>({
  label,
  value,
  items,
  onChange,
}: {
  label: string;
  value: T;
  items: ReadonlyArray<{ id: T; label: string }>;
  onChange: (value: T) => void;
}) {
  return (
    <div className="segmented" role="group" aria-label={label}>
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          aria-pressed={value === item.id}
          onClick={() => onChange(item.id)}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}

function StateBanner({
  surface,
  state,
}: {
  surface: SurfaceKey;
  state: StateKey;
}) {
  const context = SURFACE_CONTEXT[surface];
  return (
    <section className="state-panel" data-tone={STATE_TONES[state]} aria-live="polite">
      <header>
        <Tag tone={STATE_TONES[state]}>{STATE_LABELS[state]}</Tag>
        <strong>{context.label}</strong>
      </header>
      <p>{stateMessage(surface, state)}</p>
      <dl>
        <div>
          <dt>你还剩什么</dt>
          <dd>
            {state === "empty"
              ? "还没有任何已承认对象。不编造示范行。"
              : "本地原型保留的事实。不是当前 daemon 权威。"}
          </dd>
        </div>
        <div>
          <dt>你可以做什么</dt>
          <dd>
            {state === "blocked" || state === "unknown"
              ? "查看留下的工作。没有假 Confirm，也不能盲着重试。"
              : state === "offline"
                ? "读上次事实。没有 24/7 云补跑。"
                : context.firstAction}
          </dd>
        </div>
        <div>
          <dt>这一屏怎么露</dt>
          <dd>{context.native}</dd>
        </div>
      </dl>
    </section>
  );
}

function EmptyHomeScene({
  onCreate,
}: {
  onCreate: () => void;
}) {
  return (
    <div className="empty-home">
      <div>
        <Tag tone="info">还没有 Project</Tag>
        <h2>创建 Project</h2>
        <p>用业务语言办一件长期的事。右侧对话在创建页才会打开。Knowledge 此时锁定。</p>
        <button className="primary-button" type="button" onClick={onCreate}>
          创建 Project
        </button>
      </div>
    </div>
  );
}

function CreateInitScene({
  providerBound,
  wizardIndex,
  setWizardIndex,
  wizardValues,
  onEditValue,
  wizardConfirmed,
  wizardStale,
  confirmCurrent,
  onLeaveDraft,
  onMembers,
  goSettings,
}: {
  providerBound: boolean;
  wizardIndex: number;
  setWizardIndex: (index: number) => void;
  wizardValues: Record<WizardId, string>;
  onEditValue: (id: WizardId, value: string) => void;
  wizardConfirmed: Record<WizardId, boolean>;
  wizardStale: Record<WizardId, boolean>;
  confirmCurrent: () => void;
  onLeaveDraft: () => void;
  onMembers: () => void;
  goSettings: () => void;
}) {
  const step = WIZARD_STEPS[wizardIndex] ?? WIZARD_STEPS[0];
  const last = wizardIndex === WIZARD_STEPS.length - 1;
  const value = wizardValues[step.id];
  const confirmed = wizardConfirmed[step.id];
  const stale = Boolean(wizardStale[step.id]);
  const allConfirmed = WIZARD_STEPS.every((item) => wizardConfirmed[item.id]);
  const briefReady = wizardConfirmed.brief;
  const waitingForSuggestion = !step.ownerAuthored && !briefReady;
  const canConfirm =
    providerBound &&
    !confirmed &&
    value.trim().length > 0 &&
    !waitingForSuggestion;
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>① 逐项确认这件事</h2>
          <p>
            第一项由你写。其余项是助手建议，可改再确认。确认后才能到下一项。右侧对话与画布同步。
          </p>
        </div>
        <div className="header-actions">
          <button className="secondary-button" type="button" onClick={onLeaveDraft}>
            离开并保留草稿
          </button>
        </div>
      </section>
      {!providerBound ? (
        <Notice title="还没有绑定助手" tone="warn">
          右侧对话只会请你去 Settings 连接模型。不会静默绑定，也不会在聊天里要密钥。
        </Notice>
      ) : (
        <Notice title="目标态样品" tone="info">
          助手建议是样品，不是 daemon 权威。外部文本不可信。密钥永不进聊天。总预览前项目未上线。
        </Notice>
      )}
      <section className="work-surface wizard-surface">
        <Heading
          title={`${wizardIndex + 1} / ${WIZARD_STEPS.length} · ${step.label}`}
          meta={step.ownerAuthored ? "你来写。确认后才生成后续建议。" : "助手建议。可编辑后确认，或回到上一项。"}
        />
        <div className="wizard-dots" role="tablist" aria-label="初始化步骤">
          {WIZARD_STEPS.map((item, index) => {
            const reachable =
              index === 0 ||
              WIZARD_STEPS.slice(0, index).every((prior) => wizardConfirmed[prior.id]);
            return (
              <button
                key={item.id}
                type="button"
                role="tab"
                className="wizard-dot"
                aria-selected={index === wizardIndex}
                aria-label={`${item.label}${wizardConfirmed[item.id] ? " · 已确认" : " · 未确认"}`}
                disabled={!reachable}
                onClick={() => setWizardIndex(index)}
              />
            );
          })}
        </div>
        <div className="wizard-viewport">
          <div
            className="wizard-rail"
            style={{ ["--wizard-index" as string]: String(wizardIndex) } as CSSProperties}
          >
            {WIZARD_STEPS.map((item, index) => {
              const current = index === wizardIndex;
              const itemValue = wizardValues[item.id];
              const itemStale = Boolean(wizardStale[item.id]);
              const itemWaiting = !item.ownerAuthored && !briefReady;
              return (
                <article
                  key={item.id}
                  className="wizard-slide"
                  role="tabpanel"
                  aria-hidden={!current}
                  aria-label={item.label}
                  inert={!current}
                >
                  <div className="wizard-card">
                    <div className="wizard-card-meta">
                      <Tag tone={wizardConfirmed[item.id] ? "good" : itemStale ? "warn" : "neutral"}>
                        {wizardConfirmed[item.id]
                          ? "已确认"
                          : itemStale
                            ? "已过时"
                            : item.ownerAuthored
                              ? "由你填写"
                              : itemWaiting
                                ? "待生成建议"
                                : "助手建议"}
                      </Tag>
                      <small>{item.detail}</small>
                    </div>
                    <label className="field">
                      <span>{item.label}</span>
                      <TextArea
                        value={itemValue}
                        onChange={(next) => onEditValue(item.id, next)}
                        rows={8}
                        placeholder={
                          item.ownerAuthored
                            ? "例如：每周给自己一份可打开的经营周报，并跟进待回复客户…"
                            : itemWaiting
                              ? "确认业务描述后，这里会出现助手建议…"
                              : item.detail
                        }
                      />
                    </label>
                    {itemStale ? (
                      <Notice title="建议可能过时" tone="warn">
                        业务描述已改。请核对或改写本项后再确认。过时项不能当成已就绪。
                      </Notice>
                    ) : null}
                  </div>
                </article>
              );
            })}
          </div>
        </div>
        <div className="wizard-nav">
          <button
            className="secondary-button"
            type="button"
            disabled={wizardIndex === 0}
            onClick={() => setWizardIndex(Math.max(0, wizardIndex - 1))}
          >
            上一项
          </button>
          {!providerBound && step.id === "brief" ? (
            <button className="primary-button" type="button" onClick={goSettings}>
              去设置连接模型并绑定助手
            </button>
          ) : (
            <button
              className="primary-button"
              type="button"
              disabled={!canConfirm}
              onClick={confirmCurrent}
            >
              确认本项
            </button>
          )}
          {last ? (
            <button
              className="primary-button"
              type="button"
              disabled={!providerBound || !allConfirmed}
              onClick={onMembers}
            >
              总预览后进入 ② 成员初始化
            </button>
          ) : (
            <button
              className="secondary-button"
              type="button"
              disabled={!confirmed}
              onClick={() => setWizardIndex(wizardIndex + 1)}
            >
              下一项
            </button>
          )}
        </div>
        <p className="flow-end wizard-status" aria-live="polite">
          {!providerBound
            ? "没有「下一项」——先绑定助手。"
            : waitingForSuggestion
              ? "先确认业务描述，才会生成后续建议。"
              : confirmed
                ? last
                  ? allConfirmed
                    ? "清单已齐。项目仍未上线。"
                    : "本项已确认。还有未齐项。"
                  : "本项已确认。「下一项」可用。"
                : "确认本项后，「下一项」才可用。"}
        </p>
      </section>
      <Gap>创建、调研、总预览写入权威都需要 daemon。这里只改变本地原型状态。</Gap>
    </div>
  );
}

function CreateMembersScene({
  members,
  setModel,
  confirmRoster,
  onProcess,
}: {
  members: readonly MemberDraft[];
  setModel: (id: string, model: string) => void;
  confirmRoster: () => void;
  onProcess: () => void;
}) {
  const ready = members.every((member) => member.model !== "unselected");
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>② 确认这个班子</h2>
          <p>岗位名单 + 对话建议。每人必须显式选模型。Skill / MCP 放到 ③ 执行方式。</p>
        </div>
      </section>
      <section className="work-surface">
        <Heading title="岗位名单" meta="拒绝 = 不加入。没选模型 = 待定，不静默绑定。" />
        <div className="staff-table-wrap" tabIndex={0} aria-label="岗位名单">
          <table>
            <caption>目标态班子样品。不是示范产品项目。</caption>
            <thead>
              <tr>
                <th scope="col">岗位</th>
                <th scope="col">做什么</th>
                <th scope="col">交出什么</th>
                <th scope="col">模型</th>
              </tr>
            </thead>
            <tbody>
              {members.map((member) => (
                <tr key={member.id}>
                  <th scope="row">{member.name}</th>
                  <td>{member.duty}</td>
                  <td>{member.handoff}</td>
                  <td>
                    <Select
                      value={member.model}
                      onChange={(next) => setModel(member.id, next)}
                      options={[
                        { value: "unselected", label: "未选 · 待定" },
                        { value: "anthropic", label: "Anthropic · 平衡" },
                        { value: "openai", label: "OpenAI · 推理" },
                        { value: "google", label: "Google · 调研" },
                      ]}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="flow-actions">
          <button
            className="primary-button"
            type="button"
            disabled={!ready}
            onClick={() => {
              confirmRoster();
              onProcess();
            }}
          >
            确认这个班子
          </button>
          <span className="flow-end">{ready ? "可以进入流程初始化。" : "有人还没选模型。"}</span>
        </div>
      </section>
      <Gap>成员定义、模型绑定和权限写入需要 daemon。无静默扩权。</Gap>
    </div>
  );
}

function CreateProcessScene({
  stageId,
  setStageId,
  confirmedStages,
  confirmStage,
  onTest,
}: {
  stageId: string;
  setStageId: (id: string) => void;
  confirmedStages: readonly string[];
  confirmStage: (id: string) => void;
  onTest: () => void;
}) {
  const stage = PROCESS_STAGES.find((item) => item.id === stageId) ?? PROCESS_STAGES[0];
  const last = stageId === PROCESS_STAGES[PROCESS_STAGES.length - 1].id;
  const all = PROCESS_STAGES.every((item) => confirmedStages.includes(item.id));
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>③ 一条流程轴，一次只开一环</h2>
          <p>总目标：每周可打开的经营周报。总周期：周一。Knowledge 现在可为当前草稿打开。</p>
        </div>
      </section>
      <div className="process-axis" role="list">
        {PROCESS_STAGES.map((item) => (
          <button
            key={item.id}
            type="button"
            role="listitem"
            aria-current={item.id === stageId ? "step" : undefined}
            onClick={() => setStageId(item.id)}
          >
            <strong>{item.label}</strong>
            <small>{confirmedStages.includes(item.id) ? "已确认这一环" : "待确认"}</small>
          </button>
        ))}
      </div>
      <section className="work-surface">
        <Heading title={`这一环 · ${stage.label}`} meta={`负责：${stage.owner}。权限按业务后果写，不写引擎名。`} />
        <dl className="definition-list">
          <div>
            <dt>输入</dt>
            <dd>上一环产出或本项目知识库摘录。缺口留在轴上，不标已就绪。</dd>
          </div>
          <div>
            <dt>执行方式</dt>
            <dd>本环节怎么做、周期、触发。Skill / 工具 / MCP / 文件权限在此披露。</dd>
          </div>
          <div>
            <dt>权限后果</dt>
            <dd>只能读本项目资料。对外发送不在这一环自动发生。</dd>
          </div>
        </dl>
        <div className="flow-actions">
          <button
            className="primary-button"
            type="button"
            onClick={() => {
              confirmStage(stage.id);
              if (last && [...confirmedStages, stage.id].length >= PROCESS_STAGES.length) {
                onTest();
              } else {
                const index = PROCESS_STAGES.findIndex((item) => item.id === stage.id);
                const next = PROCESS_STAGES[index + 1];
                if (next) setStageId(next.id);
              }
            }}
          >
            {last ? "确认总目标与项目触发" : "确认这一环"}
          </button>
          <span className="flow-end">{all ? "轴已齐，进入分环节测试。" : "拒绝则留在这一环。"}</span>
        </div>
      </section>
      <Gap>流程、触发和权限修订需要 daemon 预览。离线能改流程，不能联网补执行方式。</Gap>
    </div>
  );
}

function CreateTestScene({
  stageId,
  setStageId,
  testState,
  setTestState,
  onJoint,
}: {
  stageId: string;
  setStageId: (id: string) => void;
  testState: "idle" | "running" | "pass" | "fail" | "unknown";
  setTestState: (value: "idle" | "running" | "pass" | "fail" | "unknown") => void;
  onJoint: () => void;
}) {
  const stage = PROCESS_STAGES.find((item) => item.id === stageId) ?? PROCESS_STAGES[0];
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>④ 测这一环，直到子产出可打开</h2>
          <p>未知不能通过。离线不能开测。不展示进程或引擎。</p>
        </div>
        <Segmented
          label="测试结果样品"
          value={testState}
          items={[
            { id: "idle", label: "未开始" },
            { id: "running", label: "进行中" },
            { id: "pass", label: "达标" },
            { id: "fail", label: "不通过" },
            { id: "unknown", label: "说不清" },
          ]}
          onChange={setTestState}
        />
      </section>
      <div className="process-axis" role="list">
        {PROCESS_STAGES.map((item) => (
          <button
            key={item.id}
            type="button"
            role="listitem"
            aria-current={item.id === stageId ? "step" : undefined}
            onClick={() => setStageId(item.id)}
          >
            <strong>{item.label}</strong>
            <small>{item.owner}</small>
          </button>
        ))}
      </div>
      <section className="work-surface">
        <Heading title={`正在测 · ${stage.label}`} meta="打开结果 + 是否达标" />
        {testState === "idle" ? (
          <p>还没开始测这一环。</p>
        ) : testState === "running" ? (
          <p>正在跑这一环。进行中不是完成。</p>
        ) : testState === "fail" ? (
          <Notice title="不通过" tone="bad">
            回到 ③ 改这一环。不跳下一环。
          </Notice>
        ) : testState === "unknown" ? (
          <Notice title="说不清" tone="bad">
            结果无法核对。不能点通过。
          </Notice>
        ) : (
          <Notice title="子产出可打开" tone="good">
            目标态样品：事实清单已打开，核对标记为通过。
          </Notice>
        )}
        <div className="flow-actions">
          <button
            className="primary-button"
            type="button"
            disabled={testState !== "pass"}
            onClick={onJoint}
          >
            通过，下一环
          </button>
          <button className="secondary-button" type="button" onClick={() => setTestState("running")}>
            开始测（原型）
          </button>
        </div>
      </section>
      <Gap environment>真实测试执行需要后端与合格环境。这里只切换样品状态。</Gap>
    </div>
  );
}

function CreateJointScene({
  jointState,
  setJointState,
  onAccept,
}: {
  jointState: "idle" | "running" | "pass" | "fail" | "unknown";
  setJointState: (value: "idle" | "running" | "pass" | "fail" | "unknown") => void;
  onAccept: () => void;
}) {
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>⑤ 联合调试 · 第一次成功</h2>
          <p>打开总成果 + 核对状态。无假发布。未知不能验收。离线不能联合调试。</p>
        </div>
        <Segmented
          label="联合结果样品"
          value={jointState}
          items={[
            { id: "idle", label: "未开始" },
            { id: "running", label: "进行中" },
            { id: "pass", label: "核对通过" },
            { id: "fail", label: "失败环节" },
            { id: "unknown", label: "说不清" },
          ]}
          onChange={setJointState}
        />
      </section>
      <section className="work-surface">
        <Heading title="全流程走到哪" meta="失败会指出环节并回 ④ / ③" />
        <ol className="run-steps">
          {PROCESS_STAGES.map((item, index) => (
            <li key={item.id} data-state={index < 4 ? "done" : jointState === "pass" ? "done" : "current"}>
              <strong>{item.label}</strong>
              <span>{index < 4 ? "子产出已打开" : "总成果待验收"}</span>
            </li>
          ))}
        </ol>
        {jointState === "fail" ? (
          <Notice title="失败在核对证据" tone="bad">
            回到 ④ 测该环。聊天不能当验收。
          </Notice>
        ) : jointState === "unknown" ? (
          <Notice title="核对不上" tone="bad">
            不能验收。
          </Notice>
        ) : jointState === "pass" ? (
          <Notice title="总成果可打开" tone="good">
            目标态样品：本周周报已打开，独立核对通过。这是 aha，不是发布。
          </Notice>
        ) : (
          <p>还没有可打开的总成果。</p>
        )}
        <div className="flow-actions">
          <button
            className="primary-button"
            type="button"
            disabled={jointState !== "pass"}
            onClick={onAccept}
          >
            验收，进入 Today
          </button>
          <span className="flow-end">没有 Publish 按钮。</span>
        </div>
      </section>
      <Gap>验收写入权威、独立核对和回执需要 daemon。聊天不能 验收。</Gap>
    </div>
  );
}

function TodayIncompleteScene({
  onContinue,
}: {
  onContinue: () => void;
}) {
  return (
    <div className="scene-stack">
      <section className="today-header">
        <div>
          <h2>创建还没走完</h2>
          <p>日常决策包要等 ⑤ 验收。现在 Today 只留这一件。</p>
        </div>
      </section>
      <section className="decision-packet">
        <header>
          <Tag tone="warn">未完成的创建</Tag>
          <span>不是日常拍板</span>
        </header>
        <h3>继续未完成的创建</h3>
        <p>五段向导还在进行。不要把卡片摆在中间当成已经成功。</p>
        <div className="packet-actions">
          <button className="primary-button" type="button" onClick={onContinue}>
            继续未完成的创建
          </button>
        </div>
      </section>
    </div>
  );
}

function TodayLiveScene({
  period,
  setPeriod,
  selectedRun,
  setSelectedRun,
  onDecision,
  onProject,
}: {
  period: Period;
  setPeriod: (value: Period) => void;
  selectedRun: string | null;
  setSelectedRun: (value: string | null) => void;
  onDecision: () => void;
  onProject: () => void;
}) {
  const selected = PROCESS_STAGES.find((item) => item.id === selectedRun);
  return (
    <div className="scene-stack">
      <section className="today-header">
        <div>
          <h2>看清并处理要你拍板的事</h2>
          <p>主按钮只在决策包上。四泳道不单独成块。点统计不能发布。聊天不能批准。</p>
        </div>
        <Segmented
          label="统计周期"
          value={period}
          items={[
            { id: "today", label: "今日" },
            { id: "week", label: "本周" },
            { id: "month", label: "本月" },
          ]}
          onChange={setPeriod}
        />
      </section>
      <section className="decision-packet">
        <header>
          <Tag tone="warn">要你拍板</Tag>
          <span>可逆 · 改窄仍要新预览</span>
        </header>
        <h3>是否把本周周报摘要发给已选客户</h3>
        <p>后果：对外沟通。选项 A 先走画布完整预览。内核真相：尚无合格连接器，不能真发。</p>
        <dl className="packet-facts">
          <div>
            <dt>可逆性</dt>
            <dd>发出前可拒。发出后需要回执，不能当聊天确认。</dd>
          </div>
          <div>
            <dt>备选</dt>
            <dd>只存周报、改窄收件人、或本周稍后再问。</dd>
          </div>
          <div>
            <dt>费用</dt>
            <dd>估计 ¥6.40 · 来源：模型用量样品。另一项费用未知，不写 0。</dd>
          </div>
          <div>
            <dt>为何先 A</dt>
            <dd>A 可打开、可核对。未知对外结果不能当成功。</dd>
          </div>
        </dl>
        <div className="packet-actions">
          <button className="primary-button" type="button" onClick={onDecision}>
            去处理这一件拍板
          </button>
          <button className="text-button" type="button">
            以后再说（仍留在 Today）
          </button>
        </div>
      </section>
      <section className="run-counts" aria-label="项目计数">
        <div>
          <span>创建的项目</span>
          <strong>1</strong>
          <small>含未上线草稿则另计</small>
        </div>
        <div>
          <span>已上线</span>
          <strong>1</strong>
          <small>无示范项目</small>
        </div>
        <div>
          <span>发生阻塞</span>
          <strong>1</strong>
          <small>点进项目处理</small>
        </div>
      </section>
      <section className="work-surface">
        <Heading title={`${period === "today" ? "今日" : period === "week" ? "本周" : "本月"}运行概览`} meta="当前状态、已完整执行次数、当前环节、已持续多久" />
        <button className="secondary-button" type="button" onClick={onProject}>
          打开 周报与客户跟进
        </button>
        <dl className="ledger-facts">
          <div>
            <dt>当前状态</dt>
            <dd>进行中 · 收集本周事实</dd>
          </div>
          <div>
            <dt>已完整执行</dt>
            <dd>{period === "today" ? "2" : period === "week" ? "9" : "说不清"}</dd>
          </div>
          <div>
            <dt>当前环节时长</dt>
            <dd>41 分钟 · 不是完成</dd>
          </div>
        </dl>
        <p>点某一上线项目看环节详情。费用未知的格子写「说不清」，不写 0。</p>
        <div className="process-axis" role="list">
          {PROCESS_STAGES.map((item) => (
            <button
              key={item.id}
              type="button"
              role="listitem"
              aria-current={selectedRun === item.id ? "step" : undefined}
              onClick={() => setSelectedRun(selectedRun === item.id ? null : item.id)}
            >
              <strong>{item.label}</strong>
              <small>{item.owner}</small>
            </button>
          ))}
        </div>
        {selected ? (
          <div className="stage-detail">
            <Heading title={`${selected.label} · ${selected.owner}`} meta="今日完整 / 失败 / 平均时长 / 成功率" />
            <dl className="ledger-facts">
              <div>
                <dt>完整</dt>
                <dd>{selected.complete}</dd>
              </div>
              <div>
                <dt>失败</dt>
                <dd>{selected.fail}</dd>
              </div>
              <div>
                <dt>平均时长</dt>
                <dd>{selected.avg}</dd>
              </div>
              <div>
                <dt>成功率</dt>
                <dd>{selected.success}</dd>
              </div>
            </dl>
          </div>
        ) : null}
      </section>
    </div>
  );
}

function ProjectsScene({
  lifecycle,
  copied,
  onCreate,
  onOpen,
  onCopy,
  onContinue,
}: {
  lifecycle: Lifecycle;
  copied: boolean;
  onCreate: () => void;
  onOpen: () => void;
  onCopy: () => void;
  onContinue: () => void;
}) {
  if (lifecycle === "empty") {
    return (
      <div className="scene-stack">
        <section className="today-header">
          <div>
            <h2>还没有 Project</h2>
            <p>空创建从这里进入。没有示范项目。</p>
          </div>
          <button className="primary-button" type="button" onClick={onCreate}>
            创建 Project
          </button>
        </section>
      </div>
    );
  }
  if (lifecycle === "creating") {
    return (
      <div className="scene-stack">
        <section className="today-header">
          <div>
            <h2>未完成的创建</h2>
            <p>Projects 现在只露出这一份草稿。</p>
          </div>
          <button className="primary-button" type="button" onClick={onContinue}>
            继续创建
          </button>
        </section>
      </div>
    );
  }
  return (
    <div className="scene-stack">
      <section className="today-header">
        <div>
          <h2>长期工作空间</h2>
          <p>已有可上线项目时，列表和项目页都提供复制。副本不带密钥、进行中任务、对外回执、本周不再问。</p>
        </div>
        <div className="header-actions">
          <button className="secondary-button" type="button" onClick={onCopy}>
            复制为副本
          </button>
          <button className="primary-button" type="button" onClick={onCreate}>
            创建 Project
          </button>
        </div>
      </section>
      {copied ? (
        <div className="copy-banner">
          <strong>周报与客户跟进（副本）</strong>
          <p>未激活草稿。改完走总预览再上线。④⑤ 可抽检或跳过。不从 ① 重来。</p>
        </div>
      ) : null}
      <section className="work-surface">
        <button className="secondary-button" type="button" onClick={onOpen}>
          打开 周报与客户跟进
        </button>
        <dl className="definition-list">
          <div>
            <dt>状态</dt>
            <dd>已上线 · 收集本周事实</dd>
          </div>
          <div>
            <dt>费用</dt>
            <dd>估计 ¥6.40 · 另有未知项不写 0</dd>
          </div>
        </dl>
      </section>
    </div>
  );
}

function ProjectLiveScene({
  stageId,
  setStageId,
  onHitl,
  onAdd,
  onCopy,
  onClose,
}: {
  stageId: string;
  setStageId: (id: string) => void;
  onHitl: () => void;
  onAdd: () => void;
  onCopy: () => void;
  onClose: () => void;
}) {
  const stage = PROCESS_STAGES.find((item) => item.id === stageId) ?? PROCESS_STAGES[0];
  const needsOwner = stage.mark !== "none";
  return (
    <div className="scene-stack">
      <section className="project-header">
        <div>
          <h2>周报与客户跟进</h2>
          <p>前台只按业务流程。没有 CEO 六步顶栏，没有 X 英雄圈，没有示范项目。</p>
        </div>
        <div className="header-actions">
          <button className="secondary-button" type="button" onClick={onCopy}>
            复制为副本
          </button>
          <button className="secondary-button" type="button" onClick={onAdd}>
            加人
          </button>
        </div>
      </section>
      <div className="process-axis" role="list">
        {PROCESS_STAGES.map((item) => (
          <button
            key={item.id}
            type="button"
            role="listitem"
            data-mark={item.mark === "none" ? undefined : item.mark}
            aria-current={item.id === stageId ? "step" : undefined}
            onClick={() => setStageId(item.id)}
          >
            <strong>{item.label}</strong>
            <small>{item.owner} · {item.status}</small>
          </button>
        ))}
      </div>
      <section className="work-surface">
        <Heading
          title={`这一环 · ${stage.label}`}
          meta={`状态：${stage.status}。负责：${stage.owner}。`}
        />
        <div className="packet-marks">
          {stage.mark === "auth" ? <Tag tone="warn">要你授权</Tag> : null}
          {stage.mark === "verify" ? <Tag tone="info">要你核对</Tag> : null}
          {stage.mark === "none" ? <Tag>无需你现在出手</Tag> : null}
        </div>
        {stage.id === "collect" ? (
          <p>梅在收集本周事实。已持续 41 分钟。进行中不是完成。</p>
        ) : stage.id === "draft" ? (
          <p>这一环还没开始。缺：上一环可打开的建议稿。</p>
        ) : stage.id === "deliver" ? (
          <p>摘要预览钉在这一环。主按钮在这里。聊天不能批。</p>
        ) : (
          <p>{stage.status}</p>
        )}
        {needsOwner ? (
          <div className="packet-actions">
            <button className="primary-button" type="button" onClick={onHitl}>
              {stage.mark === "auth" ? "去授权预览" : "去核对"}
            </button>
          </div>
        ) : null}
        <details className="trace-fold">
          <summary>普通过程痕迹 · 默认收起</summary>
          <p>工具调用与过程轨迹不是完成证据。独立核对才关闭工作。</p>
        </details>
        <button className="text-button" type="button" onClick={onClose}>
          打开成果并验收，回 Today
        </button>
      </section>
      <Gap>环节状态、授权和回执需要 daemon。没有假发布。</Gap>
    </div>
  );
}

function AddMemberScene({
  name,
  setName,
  duty,
  setDuty,
  model,
  setModel,
  joined,
  onJoin,
}: {
  name: string;
  setName: (value: string) => void;
  duty: string;
  setDuty: (value: string) => void;
  model: string;
  setModel: (value: string) => void;
  joined: boolean;
  onJoin: () => void;
}) {
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>给已上线项目补一个岗位</h2>
          <p>现有班子 + 对话建议 + 确认加入。模型必选。执行方式再披露一层。</p>
        </div>
      </section>
      <section className="work-surface">
        <Heading title="现有班子" meta="成员不跨项目共享。进程退出不会删掉人。" />
        <ul className="result-list">
          <li>
            <div>
              <strong>林 · Project Manager</strong>
              <span>计划、分派、核对</span>
            </div>
            <div>
              <Tag tone="info">进行中</Tag>
            </div>
          </li>
          <li>
            <div>
              <strong>梅 · 调研</strong>
              <span>事实清单</span>
            </div>
            <div>
              <Tag>排队</Tag>
            </div>
          </li>
          <li>
            <div>
              <strong>锐 · 撰稿</strong>
              <span>周报草稿</span>
            </div>
            <div>
              <Tag>等待交接</Tag>
            </div>
          </li>
        </ul>
      </section>
      <section className="work-surface">
        <Heading title="新岗位" meta="拒绝 = 不加入。没模型 = 待定，去 Settings。" />
        <label className="field">
          <span>岗位名</span>
          <TextInput value={name} onChange={setName} />
        </label>
        <label className="field">
          <span>做什么、交出什么</span>
          <TextArea value={duty} onChange={setDuty} rows={3} />
        </label>
        <label className="field">
          <span>模型（必选）</span>
          <Select
            value={model}
            onChange={setModel}
            options={[
              { value: "unselected", label: "未选 · 待定" },
              { value: "anthropic", label: "Anthropic · 平衡" },
              { value: "openai", label: "OpenAI · 推理" },
            ]}
          />
        </label>
        <div className="flow-actions">
          <button
            className="primary-button"
            type="button"
            disabled={model === "unselected" || name.trim().length === 0}
            onClick={onJoin}
          >
            确认加入
          </button>
          <span className="flow-end">加入后改流程/权限要再批，不静默扩权。</span>
        </div>
        {joined ? (
          <Notice title="已加入（原型）" tone="good">
            下一层才披露执行方式：Skill、MCP、文件权限。不是先装 MCP，也不是先看引擎。
          </Notice>
        ) : null}
      </section>
      <Gap>加人写入成员定义需要 daemon。离线能写职责，不能联网搜岗位方案。</Gap>
    </div>
  );
}

function HitlScene({
  previewAge,
  setPreviewAge,
  executing,
  setExecuting,
  fate,
  setFate,
  skipWeek,
  setSkipWeek,
  onBack,
}: {
  previewAge: PreviewAge;
  setPreviewAge: (value: PreviewAge) => void;
  executing: boolean;
  setExecuting: (value: boolean) => void;
  fate: HitlFate;
  setFate: (value: HitlFate) => void;
  skipWeek: boolean;
  setSkipWeek: (value: boolean) => void;
  onBack: () => void;
}) {
  const canApprove = previewAge === "fresh" && !executing;
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>画布预览 · 聊天不能批</h2>
          <p>将做什么 + 完整预览 + 批准 / 改窄 / 拒绝。执行中第四个行动是停。过期预览不能批。</p>
        </div>
        <Segmented
          label="预览新鲜度"
          value={previewAge}
          items={[
            { id: "fresh", label: "新鲜" },
            { id: "stale", label: "过期" },
          ]}
          onChange={setPreviewAge}
        />
      </section>
      <section className="decision-preview">
        <Heading title="将做什么" meta="对外沟通 · 可逆直到发出" />
        <p>把本周周报摘要发给已选的两名客户。不是全量发布，也不是社交发帖。</p>
        <Heading title="完整预览 / 差异" meta="目标态样品。不是已发出。" />
        <dl>
          <div>
            <dt>收件人</dt>
            <dd>客户 A、客户 B · 来自本项目资料</dd>
          </div>
          <div>
            <dt>正文</dt>
            <dd>三句摘要 + 可打开周报链接（本地）。无密钥、无隐藏指令。</dd>
          </div>
          <div>
            <dt>差异</dt>
            <dd>相对上次草稿：删掉了一条未授权引用。</dd>
          </div>
        </dl>
        <label>
          <input
            type="checkbox"
            checked={skipWeek}
            onChange={(event: { currentTarget: { checked: boolean } }) =>
              setSkipWeek(event.currentTarget.checked)
            }
          />
          {" "}本周此类不再问（到期失效，Settings 可收回）
        </label>
        <div className="hitl-actions">
          <button
            className="primary-button"
            type="button"
            disabled={!canApprove}
            onClick={() => {
              setExecuting(true);
              setFate("approved");
            }}
          >
            批准
          </button>
          <button
            className="secondary-button"
            type="button"
            onClick={() => setFate("narrowed")}
          >
            改窄
          </button>
          <button
            className="secondary-button"
            type="button"
            onClick={() => {
              setFate("rejected");
              onBack();
            }}
          >
            拒绝
          </button>
          {executing ? (
            <button
              className="secondary-button"
              type="button"
              onClick={() => {
                setExecuting(false);
                setFate("stopped");
              }}
            >
              停
            </button>
          ) : null}
        </div>
        {previewAge === "stale" ? (
          <Notice title="过期预览" tone="bad">
            不能批准。需要新预览。改窄后也要重新预览。
          </Notice>
        ) : null}
        {fate === "approved" ? (
          <Notice title="已请求发出（原型）" tone="info">
            回执将钉在环节页，可打开。未知对外结果禁止盲着重试。离线不能批准对外。
          </Notice>
        ) : null}
        {fate === "narrowed" ? (
          <Notice title="改窄需要新预览" tone="warn">
            旧预览作废。没有永久「以后别再问」。
          </Notice>
        ) : null}
        {fate === "stopped" ? (
          <Notice title="已停" tone="warn">
            进行中被停。不是成功回执。
          </Notice>
        ) : null}
      </section>
      <Gap environment>真实 Intent/Effect、围栏和回执需要 daemon。聊天只有链接，无批准。</Gap>
    </div>
  );
}

function KnowledgeScene({
  locked,
  tab,
  setTab,
  memoryForgotten,
  forgetMemory,
  draftOnly,
  filesEmpty = false,
  importPhase,
  setImportPhase,
}: {
  locked: boolean;
  tab: KnowledgeTab;
  setTab: (value: KnowledgeTab) => void;
  memoryForgotten: boolean;
  forgetMemory: () => void;
  draftOnly: boolean;
  filesEmpty?: boolean;
  importPhase: ImportPhase;
  setImportPhase: (value: ImportPhase) => void;
}) {
  const [scope, setScope] = useState<KnowledgeScope>("all");
  const [kind, setKind] = useState<KnowledgeKind>("all");
  const [query, setQuery] = useState("");
  const [dest, setDest] = useState<ImportDest>(draftOnly ? "weekly" : "weekly");
  const [policy, setPolicy] = useState<ImportPolicy>("copy");
  const [sourceKind, setSourceKind] = useState<ImportSourceKind>("files");
  const catalog = filesEmpty
    ? []
    : KNOWLEDGE_FILES.filter((file) => !draftOnly || file.project === "weekly");
  const visible = catalog.filter((file) => {
    if (scope !== "all" && file.project !== scope) return false;
    if (kind !== "all" && file.kind !== kind) return false;
    const needle = query.trim().toLowerCase();
    if (needle.length > 0 && !file.title.toLowerCase().includes(needle)) return false;
    return true;
  });
  const scopeItems: ReadonlyArray<{ id: KnowledgeScope; label: string }> = draftOnly
    ? [{ id: "weekly", label: "当前草稿" }]
    : [
        { id: "all", label: "全部" },
        { id: "shared", label: "Owner 共享" },
        { id: "weekly", label: "周报与客户跟进" },
      ];
  if (locked) {
    return (
      <div className="scene-stack">
        <section className="today-header">
          <div>
            <h2>Knowledge 已锁定</h2>
            <p>没有 Project 时不能进。创建到 ③ 需要输入时，只为当前草稿打开。</p>
          </div>
        </section>
      </div>
    );
  }
  return (
    <div className="scene-stack">
      <section className="today-header">
        <div>
          <h2>当前项目资料</h2>
          <p>Obsidian 是底座，不必安装该应用。解析失败保留原件可重试。离线只读上次索引。</p>
        </div>
        <Segmented
          label="Knowledge"
          value={tab}
          items={[
            { id: "files", label: "项目资料" },
            { id: "import", label: "导入" },
            { id: "why", label: "为什么用这段" },
            { id: "memory", label: "Memory" },
          ]}
          onChange={setTab}
        />
      </section>
      {tab === "files" ? (
        <section className="work-surface">
          <Heading
            title="资料"
            meta={draftOnly ? "只看当前创建草稿。无假云导入。" : "可按全部或具体项目、资料类型和关键词查看。"}
            action={{ label: "导入资料", onClick: () => setTab("import") }}
          />
          <div className="knowledge-filters">
            <Segmented
              label="项目范围"
              value={draftOnly ? "weekly" : scope}
              items={scopeItems}
              onChange={(next) => setScope(draftOnly ? "weekly" : next)}
            />
            <Segmented
              label="资料类型"
              value={kind}
              items={[
                { id: "all", label: "全部类型" },
                { id: "markdown", label: "Markdown" },
                { id: "pdf", label: "PDF" },
                { id: "image", label: "图片" },
                { id: "link", label: "链接" },
              ]}
              onChange={setKind}
            />
            <label className="field">
              <span>关键词</span>
              <TextInput
                value={query}
                onChange={setQuery}
                type="search"
                placeholder="按标题检索…"
              />
            </label>
          </div>
          {visible.length === 0 ? (
            <Notice title={catalog.length === 0 ? "还没资料" : "没有匹配的资料"} tone="info">
              {catalog.length === 0
                ? "空 = 还没资料。用导入把文件、链接或图片元数据放进当前范围。此原型不写磁盘。"
                : "当前范围、类型或关键词没有命中。这不是索引为零。"}
            </Notice>
          ) : (
            <ul className="result-list">
              {visible.map((file) => (
                <li key={file.id}>
                  <div>
                    <strong>{file.title}</strong>
                    <span>
                      {file.projectLabel} · {file.statusLabel}
                    </span>
                  </div>
                  <div>
                    <Tag tone={file.tone}>
                      {file.kind === "markdown"
                        ? "Markdown"
                        : file.kind === "pdf"
                          ? "PDF"
                          : file.kind === "image"
                            ? "图片"
                            : "链接"}
                    </Tag>
                  </div>
                </li>
              ))}
            </ul>
          )}
          {catalog.length === 0 ? (
            <div className="flow-actions">
              <button className="primary-button" type="button" onClick={() => setTab("import")}>
                导入资料
              </button>
              <span className="flow-end">导入是目标态。此原型不上传文件。</span>
            </div>
          ) : null}
          <p className="settings-note">此原型不上传文件、不写磁盘、不建索引。</p>
          <Gap>导入、去重、凭证检测和索引需要 daemon。没有 Install。</Gap>
        </section>
      ) : null}
      {tab === "import" ? (
        <section className="work-surface">
          <Heading title="导入资料" meta="选范围、复制或引用、来源种类。不写磁盘。密钥不得进入 Vault。" />
          <label className="field">
            <span>目标范围</span>
            <Select
              value={dest}
              onChange={(next) => setDest(next as ImportDest)}
              options={
                draftOnly
                  ? [{ value: "weekly", label: "当前草稿 · 周报与客户跟进" }]
                  : [
                      { value: "weekly", label: "周报与客户跟进" },
                      { value: "shared", label: "Owner 共享" },
                    ]
              }
            />
          </label>
          <label className="field">
            <span>复制或引用</span>
            <Select
              value={policy}
              onChange={(next) => setPolicy(next as ImportPolicy)}
              options={[
                { value: "copy", label: "复制到 Vault（须有权复用）" },
                { value: "reference", label: "引用原件（不复制正文）" },
              ]}
            />
          </label>
          <label className="field">
            <span>来源种类</span>
            <Select
              value={sourceKind}
              onChange={(next) => setSourceKind(next as ImportSourceKind)}
              options={[
                { value: "files", label: "文件或目录" },
                { value: "link", label: "链接" },
                { value: "image", label: "图片" },
                { value: "video", label: "视频元数据" },
              ]}
            />
          </label>
          <p className="settings-note">
            将核对来源权利、解析/OCR 预期和权限。仅 Owner 自有、许可、开源或公有领域可复制复用。离线不能云导入。
          </p>
          <div className="flow-actions">
            <button
              className="primary-button"
              type="button"
              disabled={importPhase === "importing"}
              onClick={() => setImportPhase("importing")}
            >
              开始导入（原型，不写磁盘）
            </button>
            <span className="flow-end">
              {importPhase === "idle"
                ? "尚未开始"
                : importPhase === "importing"
                  ? "进行中 · 进度只是原型文案"
                  : importPhase === "duplicate"
                    ? "重复 · 原件保留"
                    : importPhase === "parse-fail"
                      ? "解析失败 · 原件保留，可重试"
                      : importPhase === "secret-detected"
                        ? "检出密钥形态 · 改走 SecretStore，不进知识库"
                        : "已索引样品 · 不是磁盘写入"}
            </span>
          </div>
          {importPhase === "importing" || importPhase === "idle" ? null : (
            <Notice
              title={
                importPhase === "secret-detected"
                  ? "密钥不进 Vault"
                  : importPhase === "indexed"
                    ? "样品结果"
                    : "导入未完成"
              }
              tone={
                importPhase === "indexed"
                  ? "info"
                  : importPhase === "secret-detected"
                    ? "bad"
                    : "warn"
              }
            >
              {importPhase === "duplicate"
                ? "同一来源已在范围内。没有覆盖。没有假成功。"
                : importPhase === "parse-fail"
                  ? "解析失败。原件保留。可重试。未知不能标已索引。"
                  : importPhase === "secret-detected"
                    ? "凭证改走批准的 SecretStore 交接。不进知识库、聊天、Context 或 Memory。"
                    : "这是目标态样品行，不是文件系统或 daemon 回执。"}
            </Notice>
          )}
          {importPhase === "importing" ? (
            <div className="prototype-outcomes" role="group" aria-label="原型结果（不是真实导入）">
              <span>演示下一状态</span>
              <button className="inline-button" type="button" onClick={() => setImportPhase("duplicate")}>
                记为重复
              </button>
              <button className="inline-button" type="button" onClick={() => setImportPhase("parse-fail")}>
                解析失败
              </button>
              <button className="inline-button" type="button" onClick={() => setImportPhase("secret-detected")}>
                检出密钥
              </button>
              <button className="inline-button" type="button" onClick={() => setImportPhase("indexed")}>
                记为已索引样品
              </button>
            </div>
          ) : null}
          <Gap>导入、OCR、去重、索引和 SecretStore 接管需要 daemon。没有云导入按钮。</Gap>
        </section>
      ) : null}
      {tab === "why" ? (
        <section className="work-surface">
          <Heading title="为什么用了这段" meta="范围、来源、新鲜度、删减损失都要看见。" />
          <table>
            <caption>Context 摘录样品</caption>
            <thead>
              <tr>
                <th scope="col">片段</th>
                <th scope="col">为何选中</th>
                <th scope="col">新鲜度</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <th scope="row">客户 A 待回复</th>
                <td>当前任务合同点名本周跟进</td>
                <td>2 小时前</td>
              </tr>
              <tr>
                <th scope="row">上周验收周报</th>
                <td>固定决策：摘要不超过三句</td>
                <td>7 天前</td>
              </tr>
            </tbody>
          </table>
        </section>
      ) : null}
      {tab === "memory" ? (
        <section className="work-surface">
          <Heading title="对话自动进入可检查 Memory" meta="可改、可忘。跨项目提升要确认。Codex 是记忆架构，不是引擎商店。" />
          {memoryForgotten ? (
            <Notice title="已忘记（墓碑）" tone="info">
              索引不得把这条复活。这是原型状态，不是 daemon 墓碑。
            </Notice>
          ) : (
            <article className="memory-record">
              <header>
                <strong>Owner 偏好摘要不超过三句</strong>
                <span>来源：项目群对话 · 可检查</span>
              </header>
              <p>一次反馈不能偷偷改全局岗位。稳定偏好只会变成版本提案。</p>
              <button className="secondary-button" type="button" onClick={forgetMemory}>
                忘记这条
              </button>
            </article>
          )}
        </section>
      ) : null}
    </div>
  );
}

function SettingsScene({
  provider,
  setProvider,
  customUrl,
  setCustomUrl,
  compat,
  setCompat,
  customModel,
  setCustomModel,
  keyDraft,
  setKeyDraft,
  status,
  handoff,
  skipWeek,
  revokeSkip,
}: {
  provider: string;
  setProvider: (value: string) => void;
  customUrl: string;
  setCustomUrl: (value: string) => void;
  compat: string;
  setCompat: (value: string) => void;
  customModel: string;
  setCustomModel: (value: string) => void;
  keyDraft: string;
  setKeyDraft: (value: string) => void;
  status: ConnectionStatus;
  handoff: () => void;
  skipWeek: boolean;
  revokeSkip: () => void;
}) {
  const custom = provider === "custom";
  return (
    <div className="scene-stack">
      <section className="settings-header">
        <div>
          <h2>Settings</h2>
          <p>连接模型 · 收回本周不再问 · 通知恢复。无账单、无引擎商店、无 Inbox。</p>
        </div>
      </section>
      <section className="work-surface">
        <Heading title="Model Connections" meta="主流下拉 + 自定义 URL / 兼容 / 模型。Owner 输入密钥。" />
        <label className="field">
          <span>Provider 模板</span>
          <Select
            value={provider}
            onChange={setProvider}
            options={[
              { value: "anthropic", label: "Anthropic" },
              { value: "openai", label: "OpenAI" },
              { value: "google", label: "Google" },
              { value: "custom", label: "自定义" },
            ]}
          />
        </label>
        {custom ? (
          <div className="custom-fields">
            <label className="field">
              <span>自定义 URL</span>
              <TextInput value={customUrl} onChange={setCustomUrl} />
            </label>
            <label className="field">
              <span>兼容模式</span>
              <TextInput value={compat} onChange={setCompat} />
            </label>
            <label className="field">
              <span>模型名</span>
              <TextInput value={customModel} onChange={setCustomModel} />
            </label>
          </div>
        ) : null}
        <label className="field secret-field">
          <span>密钥（一次性交接）</span>
          <TextInput
            value={keyDraft}
            onChange={setKeyDraft}
            type="password"
            placeholder="输入后交接，界面不回显"
          />
          <small>A5：单向交给 SecretStore。DOM、聊天、git 都不保留明文。此原型在交接后清空输入。</small>
        </label>
        <div className="flow-actions">
          <button
            className="primary-button"
            type="button"
            disabled={keyDraft.trim().length === 0}
            onClick={handoff}
          >
            交接密钥（原型，不联网）
          </button>
          <span className="flow-end">
            {status === "connected"
              ? "已连接（原型）· 密钥已从输入框清除"
              : status === "failed"
                ? "失败 · 说出问题所在，不装成功"
                : "尚未连接"}
          </span>
        </div>
        <Gap>真实 SecretStore 接管需要 daemon。没有 Connect 假按钮去打 Provider。</Gap>
      </section>
      <section className="work-surface">
        <Heading title="本周不再问" meta="到期失效。这里可收回。" />
        {skipWeek ? (
          <div className="flow-actions">
            <p>本周「同类对外」跳过仍有效。</p>
            <button className="secondary-button" type="button" onClick={revokeSkip}>
              收回跳过
            </button>
          </div>
        ) : (
          <p>当前没有有效的时间盒跳过。</p>
        )}
      </section>
      <section className="work-surface">
        <Heading title="通知与恢复" meta="故障时才需要更深的运行信息。默认不展示底层引擎名。" />
        <p>主机离线则工作停止。没有云端 24/7。高级诊断是另层披露，不是日常导航。</p>
      </section>
    </div>
  );
}

function StateLabScene({
  surface,
  setSurface,
  state,
  setState,
  renderNative,
}: {
  surface: SurfaceKey;
  setSurface: (value: SurfaceKey) => void;
  state: StateKey;
  setState: (value: StateKey) => void;
  renderNative: (surface: SurfaceKey, state: StateKey) => ReturnType<typeof EmptyHomeScene>;
}) {
  return (
    <div className="scene-stack">
      <section className="state-lab-header">
        <div>
          <h2>State Lab 渲染覆盖。不是「Designed」矩阵。</h2>
          <p>下面是该表面在该状态下的真实版式。画布运行时、NVDA、对比度和 200% 布局仍是 not-run。</p>
        </div>
        <div className="state-lab-controls">
          <label>
            <span>Surface</span>
            <Select
              value={surface}
              onChange={(next) => setSurface(next as SurfaceKey)}
              options={(Object.keys(SURFACE_CONTEXT) as SurfaceKey[]).map((item) => ({
                value: item,
                label: SURFACE_CONTEXT[item].label,
              }))}
            />
          </label>
          <label>
            <span>State</span>
            <Select
              value={state}
              onChange={(next) => setState(next as StateKey)}
              options={STATE_KEYS.map((item) => ({
                value: item,
                label: STATE_LABELS[item],
              }))}
            />
          </label>
        </div>
      </section>
      <StateBanner surface={surface} state={state} />
      {renderNative(surface, state)}
    </div>
  );
}

function Conversation({
  scene,
  providerBound,
  drafts,
  setDrafts,
  status,
  setStatus,
  onOpenHitl,
  wizardStep,
  wizardValue,
  wizardConfirmed,
  wizardStale,
  receipts,
  onApplyToCard,
}: {
  scene: Scene;
  providerBound: boolean;
  drafts: string;
  setDrafts: (value: string) => void;
  status: string;
  setStatus: (value: string) => void;
  onOpenHitl: () => void;
  wizardStep: { id: WizardId; label: string };
  wizardValue: string;
  wizardConfirmed: boolean;
  wizardStale: boolean;
  receipts: readonly WizardReceipt[];
  onApplyToCard: () => void;
}) {
  const project =
    scene === "project" ||
    scene === "add-member" ||
    scene === "hitl" ||
    scene === "create-members" ||
    scene === "create-process" ||
    scene === "create-test" ||
    scene === "create-joint";
  const title = project ? "项目群" : "Personal Assistant";
  const creating = CREATE_SCENES.includes(scene);
  const addMention = (mention: string) => {
    const space = drafts.length > 0 && !drafts.endsWith(" ") ? " " : "";
    setDrafts(`${drafts}${space}${mention} `);
    setStatus(`${mention} 只进未发送草稿，不绕过任务权威。`);
  };
  return (
    <aside className="conversation" id="opc-conversation" aria-label={title}>
      <header>
        <div>
          <span>{project ? "Owner / 经理 / 成员" : "全局助手 · 最高 UX 特权，写入仍要预览"}</span>
          <h2>{title}</h2>
        </div>
      </header>
      {project ? (
        <div className="participants" role="group" aria-label="项目群成员">
          <span>Owner</span>
          <span>林 · 经理</span>
          <span>梅 · 调研</span>
          <span>锐 · 撰稿</span>
        </div>
      ) : null}
      <div className="messages" role="region" aria-label="原型对话样品">
        {scene === "create-init" ? (
          <article data-author="system" className="canvas-mirror">
            <span>画布当前项 · {wizardStep.label}</span>
            <p>{wizardValue.trim().length > 0 ? wizardValue : "（还没有内容）"}</p>
            <small>
              {wizardConfirmed ? "已确认" : wizardStale ? "已过时，需再确认" : "未确认"}
              。改画布会立刻反映在这里。聊天不能批准。
            </small>
          </article>
        ) : null}
        {scene === "create-init" && !providerBound ? (
          <>
            <article data-author="assistant">
              <span>助手 · 尚未绑定</span>
              <p>还没有模型。请去 Settings 连接 Provider 并绑定助手。我不会在聊天里收密钥。</p>
            </article>
            <article data-author="system">
              <span>无静默绑定</span>
              <p>连接失败会说出问题所在。没有 Connect 假按钮。</p>
            </article>
          </>
        ) : scene === "create-init" ? (
          <article data-author="assistant">
            <span>助手 · 候选</span>
            <p>
              请用业务语言描述项目情况和产出目标。确认第一项后，我会给出后续建议，你可在画布里改再确认。用下面「按这段改当前项」把草稿同步回输入框。没有发送，没有安装。
            </p>
          </article>
        ) : scene === "today" ? (
          <>
            <article data-author="assistant">
              <span>助手 · 可查询运行数据</span>
              <p>今日已完整执行 2 次。阻塞 1 个项目。费用有一项说不清，所以不是 0。</p>
              <small>我可以分析。我不能批准、验收或发布。</small>
            </article>
            <article data-author="system" className="approval-card">
              <span>有一件要你拍板 · 聊天无批准</span>
              <p>周报摘要对外发送需要画布预览。</p>
              <button className="inline-button" type="button" onClick={onOpenHitl}>
                打开画布预览
              </button>
            </article>
          </>
        ) : project && !creating ? (
          <>
            <article data-author="owner">
              <span>Owner</span>
              <p>@林 现在停在哪一步？</p>
            </article>
            <article data-author="manager">
              <span>林 · 默认发言</span>
              <p>停在收集本周事实。交给 Owner 那一环钉了授权。聊天不能批。</p>
            </article>
            <article data-author="system" className="approval-card">
              <span>HITL 只在画布</span>
              <p>成员只在被 @、交产出、交接、阻塞或要决策时主动说话。</p>
              <button className="inline-button" type="button" onClick={onOpenHitl}>
                打开画布预览
              </button>
            </article>
          </>
        ) : (
          <article data-author="assistant">
            <span>助手</span>
            <p>我可以解释、调研、起草并发起流程。写入必须经过预览 → 你确认 → 回执。</p>
          </article>
        )}
        {scene === "create-init"
          ? receipts.map((item) => (
              <article key={item.id} data-author="system">
                <span>画布回执</span>
                <p>{item.text}</p>
              </article>
            ))
          : null}
      </div>
      <div className="composer">
        {project ? (
          <div className="mention-buttons" role="group" aria-label="写入未发送草稿">
            <button type="button" onClick={() => addMention("@林")}>@林</button>
            <button type="button" onClick={() => addMention("@梅")}>@梅</button>
            <button type="button" onClick={() => addMention("@锐")}>@锐</button>
          </div>
        ) : null}
        <label>
          <span>消息 · {title}</span>
          <TextArea
            value={drafts}
            onChange={(next) => {
              setDrafts(next.slice(0, 1000));
              setStatus("未发送草稿。不能在聊天里批准、验收、发布或安装。");
            }}
            rows={4}
            placeholder={
              scene === "create-init"
                ? "用自然语言改当前项，再点「按这段改当前项」…"
                : project
                  ? "问经理或有界地改成员工作…"
                  : "问运行情况，或描述一件要办的事…"
            }
          />
        </label>
        <div className="composer-actions">
          {scene === "create-init" ? (
            <button
              className="primary-button"
              type="button"
              disabled={drafts.trim().length === 0}
              onClick={onApplyToCard}
            >
              按这段改当前项（原型）
            </button>
          ) : null}
          <button
            className="secondary-button"
            type="button"
            onClick={() => setStatus("本地解释了草稿。没有发送，没有生成任务权威。")}
          >
            预览未发送消息
          </button>
          <small aria-live="polite">{status}</small>
        </div>
        <Gap>发送、@ 路由和任务翻译需要 daemon。没有会写入权威的发送键。</Gap>
      </div>
    </aside>
  );
}

export default function Personal20OpcE2eOptimizedV1() {
  const theme = useHostTheme();
  const [scene, setScene] = useState<Scene>("empty-home");
  const [wizardIndex, setWizardIndex] = useState(0);
  const [wizardValues, setWizardValues] = useState<Record<WizardId, string>>(() =>
    blankWizardValues(DEFAULT_BRIEF),
  );
  const [wizardConfirmed, setWizardConfirmed] = useState<Record<WizardId, boolean>>(blankWizardFlags);
  const [wizardStale, setWizardStale] = useState<Record<WizardId, boolean>>(blankWizardFlags);
  const [receipts, setReceipts] = useState<WizardReceipt[]>([]);
  const [providerBound, setProviderBound] = useState(false);
  const [members, setMembers] = useState<MemberDraft[]>([
    { id: "lin", name: "林 · 经理", duty: "计划、分派、核对", handoff: "可打开周报与决策包", model: "unselected", joined: false },
    { id: "mei", name: "梅 · 调研", duty: "收集本周事实", handoff: "事实清单", model: "unselected", joined: false },
    { id: "rui", name: "锐 · 撰稿", duty: "起草周报", handoff: "周报草稿", model: "unselected", joined: false },
  ]);
  const [processStageId, setProcessStageId] = useState("collect");
  const [confirmedStages, setConfirmedStages] = useState<string[]>([]);
  const [testState, setTestState] = useState<"idle" | "running" | "pass" | "fail" | "unknown">("idle");
  const [jointState, setJointState] = useState<"idle" | "running" | "pass" | "fail" | "unknown">("idle");
  const [period, setPeriod] = useState<Period>("today");
  const [selectedRun, setSelectedRun] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [newName, setNewName] = useState("客户跟进人");
  const [newDuty, setNewDuty] = useState("跟进待回复客户，交出跟进记录。");
  const [newModel, setNewModel] = useState("unselected");
  const [joined, setJoined] = useState(false);
  const [previewAge, setPreviewAge] = useState<PreviewAge>("fresh");
  const [executing, setExecuting] = useState(false);
  const [fate, setFate] = useState<HitlFate>("idle");
  const [skipWeek, setSkipWeek] = useState(false);
  const [knowledgeTab, setKnowledgeTab] = useState<KnowledgeTab>("files");
  const [importPhase, setImportPhase] = useState<ImportPhase>("idle");
  const [chatOpen, setChatOpen] = useState(false);
  const [memoryForgotten, setMemoryForgotten] = useState(false);
  const [provider, setProvider] = useState("anthropic");
  const [customUrl, setCustomUrl] = useState("https://example.invalid/v1");
  const [compat, setCompat] = useState("openai-compatible");
  const [customModel, setCustomModel] = useState("");
  const [keyDraft, setKeyDraft] = useState("");
  const [connection, setConnection] = useState<ConnectionStatus>("none");
  const [labSurface, setLabSurface] = useState<SurfaceKey>("today");
  const [labState, setLabState] = useState<StateKey>("empty");
  const [drafts, setDrafts] = useState("");
  const [composerStatus, setComposerStatus] = useState("草稿只留在这个原型里。聊天不能批准。");
  const [lifecycle, setLifecycle] = useState<Lifecycle>("empty");
  const [createGate, setCreateGate] = useState(1);

  useEffect(() => {
    if (scene === "knowledge" || scene === "settings") {
      setChatOpen(false);
    }
  }, [scene]);

  const chatHidden =
    scene === "empty-home" ||
    ((scene === "knowledge" || scene === "settings") && !chatOpen);
  const currentWizard = WIZARD_STEPS[wizardIndex] ?? WIZARD_STEPS[0];
  const pushReceipt = (text: string) => {
    setReceipts((current) => {
      const id = (current[current.length - 1]?.id ?? 0) + 1;
      return [...current, { id, text }].slice(-8);
    });
  };
  const moveWizard = (index: number) => {
    if (index === wizardIndex || index < 0 || index >= WIZARD_STEPS.length) return;
    const label = WIZARD_STEPS[index].label;
    setWizardIndex(index);
    pushReceipt(index < wizardIndex ? `回到「${label}」。` : `进入「${label}」。`);
  };
  const onEditWizardValue = (id: WizardId, value: string) => {
    setWizardValues((current) => ({ ...current, [id]: value }));
    setWizardConfirmed((current) => (current[id] ? { ...current, [id]: false } : current));
    if (id === "brief") {
      setWizardStale((current) => {
        const next = { ...current, brief: false };
        for (const item of CONFIRM_ITEMS) next[item.id] = true;
        return next;
      });
    }
  };
  const confirmCurrentWizard = () => {
    const step = currentWizard;
    if (!providerBound || wizardValues[step.id].trim().length === 0) return;
    if (step.id === "brief") {
      setWizardValues((current) => {
        const next = { ...current };
        for (const item of CONFIRM_ITEMS) {
          if (next[item.id].trim().length === 0 || wizardStale[item.id]) {
            next[item.id] = item.detail;
          }
        }
        return next;
      });
      setWizardStale(blankWizardFlags());
    } else {
      setWizardStale((current) => ({ ...current, [step.id]: false }));
    }
    setWizardConfirmed((current) => ({ ...current, [step.id]: true }));
    pushReceipt(
      step.id === "preview"
        ? `已确认「${step.label}」。项目仍未上线。`
        : `已确认「${step.label}」。下一项可用。`,
    );
  };
  const applyDraftToCard = () => {
    const text = drafts.trim();
    if (text.length === 0) return;
    onEditWizardValue(currentWizard.id, text);
    setDrafts("");
    setComposerStatus("已把未发送草稿同步到当前画布项。没有发送，没有权威。");
    pushReceipt(`聊天草稿已写入「${currentWizard.label}」。请在画布上确认。`);
  };

  const projectsCurrent =
    scene === "projects" ||
    CREATE_SCENES.includes(scene) ||
    scene === "project" ||
    scene === "add-member" ||
    scene === "hitl";
  const knowledgeOk = lifecycle === "live" || (lifecycle === "creating" && createGate >= 3);
  const locationLabel = (() => {
    if (CREATE_SCENES.includes(scene)) return "Projects / 创建中";
    if (scene === "project" || scene === "add-member" || scene === "hitl") return "Projects / 周报与客户跟进";
    if (scene === "projects") return "Projects";
    if (scene === "settings") return "Settings";
    if (scene === "knowledge") return "Knowledge";
    if (scene === "state-lab") return "Prototype QA";
    return "Personal";
  })();

  const applyScenario = (next: Scene) => {
    setScene(next);
    if (next === "empty-home") {
      setLifecycle("empty");
      setCreateGate(1);
      return;
    }
    if (CREATE_SCENES.includes(next) || next === "today-incomplete") {
      setLifecycle("creating");
      if (next === "create-init") setCreateGate(Math.max(createGate, 1));
      if (next === "create-members") setCreateGate(Math.max(createGate, 2));
      if (next === "create-process") setCreateGate(Math.max(createGate, 3));
      if (next === "create-test") setCreateGate(Math.max(createGate, 4));
      if (next === "create-joint") setCreateGate(Math.max(createGate, 5));
      return;
    }
    if (next === "settings" || next === "state-lab") return;
    setLifecycle("live");
  };

  const onNavToday = () => {
    if (lifecycle === "empty") setScene("empty-home");
    else if (lifecycle === "creating") setScene("today-incomplete");
    else setScene("today");
  };

  const variables = {
    "--bg": theme.bg.editor,
    "--chrome": theme.bg.chrome,
    "--surface": theme.bg.elevated,
    "--fill": theme.fill.tertiary,
    "--fill-strong": theme.fill.secondary,
    "--line": theme.stroke.tertiary,
    "--line-strong": theme.stroke.secondary,
    "--focus": theme.stroke.focused,
    "--text": theme.text.primary,
    "--muted": theme.text.secondary,
    "--faint": theme.text.tertiary,
    "--accent": theme.accent.control,
    "--on-accent": theme.text.onAccent,
    "--good": theme.category.green,
    "--warn": theme.category.yellow,
    "--bad": theme.category.red,
    "--info": theme.category.blue,
    "--link": theme.text.link,
  } as CSSProperties;

  const renderMain = (active: Scene) => {
    if (active === "empty-home") {
      return <EmptyHomeScene onCreate={() => { setLifecycle("creating"); setCreateGate(1); setScene("create-init"); }} />;
    }
    if (active === "create-init") {
      return (
        <CreateInitScene
          providerBound={providerBound}
          wizardIndex={wizardIndex}
          setWizardIndex={moveWizard}
          wizardValues={wizardValues}
          onEditValue={onEditWizardValue}
          wizardConfirmed={wizardConfirmed}
          wizardStale={wizardStale}
          confirmCurrent={confirmCurrentWizard}
          onLeaveDraft={() => setScene("today-incomplete")}
          onMembers={() => { setCreateGate(2); setScene("create-members"); }}
          goSettings={() => setScene("settings")}
        />
      );
    }
    if (active === "create-members") {
      return (
        <CreateMembersScene
          members={members}
          setModel={(id, model) =>
            setMembers(members.map((member) => (member.id === id ? { ...member, model } : member)))
          }
          confirmRoster={() =>
            setMembers(members.map((member) => ({ ...member, joined: member.model !== "unselected" })))
          }
          onProcess={() => { setCreateGate(3); setScene("create-process"); }}
        />
      );
    }
    if (active === "create-process") {
      return (
        <CreateProcessScene
          stageId={processStageId}
          setStageId={setProcessStageId}
          confirmedStages={confirmedStages}
          confirmStage={(id) =>
            setConfirmedStages(confirmedStages.includes(id) ? confirmedStages : [...confirmedStages, id])
          }
          onTest={() => { setCreateGate(4); setScene("create-test"); }}
        />
      );
    }
    if (active === "create-test") {
      return (
        <CreateTestScene
          stageId={processStageId}
          setStageId={setProcessStageId}
          testState={testState}
          setTestState={setTestState}
          onJoint={() => { setCreateGate(5); setScene("create-joint"); }}
        />
      );
    }
    if (active === "create-joint") {
      return (
        <CreateJointScene
          jointState={jointState}
          setJointState={setJointState}
          onAccept={() => { setLifecycle("live"); setScene("today"); }}
        />
      );
    }
    if (active === "today-incomplete") {
      return <TodayIncompleteScene onContinue={() => setScene("create-init")} />;
    }
    if (active === "today") {
      return (
        <TodayLiveScene
          period={period}
          setPeriod={setPeriod}
          selectedRun={selectedRun}
          setSelectedRun={setSelectedRun}
          onDecision={() => setScene("hitl")}
          onProject={() => setScene("project")}
        />
      );
    }
    if (active === "projects") {
      return (
        <ProjectsScene
          lifecycle={lifecycle}
          copied={copied}
          onCreate={() => setScene("create-init")}
          onOpen={() => setScene("project")}
          onCopy={() => setCopied(true)}
          onContinue={() => setScene("create-init")}
        />
      );
    }
    if (active === "project") {
      return (
        <ProjectLiveScene
          stageId={processStageId}
          setStageId={setProcessStageId}
          onHitl={() => setScene("hitl")}
          onAdd={() => setScene("add-member")}
          onCopy={() => {
            setCopied(true);
            setScene("projects");
          }}
          onClose={() => setScene("today")}
        />
      );
    }
    if (active === "add-member") {
      return (
        <AddMemberScene
          name={newName}
          setName={setNewName}
          duty={newDuty}
          setDuty={setNewDuty}
          model={newModel}
          setModel={setNewModel}
          joined={joined}
          onJoin={() => setJoined(true)}
        />
      );
    }
    if (active === "hitl") {
      return (
        <HitlScene
          previewAge={previewAge}
          setPreviewAge={setPreviewAge}
          executing={executing}
          setExecuting={setExecuting}
          fate={fate}
          setFate={setFate}
          skipWeek={skipWeek}
          setSkipWeek={setSkipWeek}
          onBack={() => setScene("project")}
        />
      );
    }
    if (active === "knowledge") {
      return (
        <KnowledgeScene
          locked={!knowledgeOk}
          tab={knowledgeTab}
          setTab={setKnowledgeTab}
          memoryForgotten={memoryForgotten}
          forgetMemory={() => setMemoryForgotten(true)}
          draftOnly={lifecycle === "creating"}
          importPhase={importPhase}
          setImportPhase={setImportPhase}
        />
      );
    }
    if (active === "settings") {
      return (
        <SettingsScene
          provider={provider}
          setProvider={setProvider}
          customUrl={customUrl}
          setCustomUrl={setCustomUrl}
          compat={compat}
          setCompat={setCompat}
          customModel={customModel}
          setCustomModel={setCustomModel}
          keyDraft={keyDraft}
          setKeyDraft={setKeyDraft}
          status={connection}
          handoff={() => {
            setKeyDraft("");
            setConnection("connected");
            setProviderBound(true);
          }}
          skipWeek={skipWeek}
          revokeSkip={() => setSkipWeek(false)}
        />
      );
    }
    return (
      <StateLabScene
        surface={labSurface}
        setSurface={setLabSurface}
        state={labState}
        setState={setLabState}
        renderNative={(surface, state) => {
          if (surface === "today" && state === "empty") {
            return <EmptyHomeScene onCreate={() => { setLifecycle("creating"); setCreateGate(1); setScene("create-init"); }} />;
          }
          if (surface === "today" && (state === "working" || state === "success" || state === "partial")) {
            return (
              <TodayLiveScene
                period={period}
                setPeriod={setPeriod}
                selectedRun={selectedRun}
                setSelectedRun={setSelectedRun}
                onDecision={() => setScene("hitl")}
                onProject={() => setScene("project")}
              />
            );
          }
          if (surface === "today" && state === "blocked") {
            return <TodayIncompleteScene onContinue={() => setScene("create-init")} />;
          }
          if (surface === "knowledge" && state === "empty") {
            return (
              <KnowledgeScene
                locked={true}
                tab={knowledgeTab}
                setTab={setKnowledgeTab}
                memoryForgotten={memoryForgotten}
                forgetMemory={() => setMemoryForgotten(true)}
                draftOnly={false}
                importPhase={importPhase}
                setImportPhase={setImportPhase}
              />
            );
          }
          if (surface === "knowledge" && state === "working") {
            return (
              <KnowledgeScene
                locked={false}
                tab="import"
                setTab={setKnowledgeTab}
                memoryForgotten={memoryForgotten}
                forgetMemory={() => setMemoryForgotten(true)}
                draftOnly={false}
                importPhase={importPhase === "idle" ? "importing" : importPhase}
                setImportPhase={setImportPhase}
              />
            );
          }
          if (surface === "knowledge" && state === "partial") {
            return (
              <KnowledgeScene
                locked={false}
                tab="files"
                setTab={setKnowledgeTab}
                memoryForgotten={memoryForgotten}
                forgetMemory={() => setMemoryForgotten(true)}
                draftOnly={false}
                filesEmpty
                importPhase={importPhase}
                setImportPhase={setImportPhase}
              />
            );
          }
          if (surface === "knowledge" && (state === "success" || state === "loading")) {
            return (
              <KnowledgeScene
                locked={false}
                tab="files"
                setTab={setKnowledgeTab}
                memoryForgotten={memoryForgotten}
                forgetMemory={() => setMemoryForgotten(true)}
                draftOnly={false}
                importPhase={importPhase}
                setImportPhase={setImportPhase}
              />
            );
          }
          if (surface === "hitl") {
            return (
              <HitlScene
                previewAge={state === "offline" || state === "unknown" ? "stale" : "fresh"}
                setPreviewAge={setPreviewAge}
                executing={state === "working"}
                setExecuting={setExecuting}
                fate={state === "success" ? "approved" : "idle"}
                setFate={setFate}
                skipWeek={skipWeek}
                setSkipWeek={setSkipWeek}
                onBack={() => setScene("project")}
              />
            );
          }
          if (surface === "settings") {
            return (
              <SettingsScene
                provider={provider}
                setProvider={setProvider}
                customUrl={customUrl}
                setCustomUrl={setCustomUrl}
                compat={compat}
                setCompat={setCompat}
                customModel={customModel}
                setCustomModel={setCustomModel}
                keyDraft={keyDraft}
                setKeyDraft={setKeyDraft}
                status={state === "error" ? "failed" : state === "success" ? "connected" : "none"}
                handoff={() => {
                  setKeyDraft("");
                  setConnection("connected");
                  setProviderBound(true);
                }}
                skipWeek={skipWeek}
                revokeSkip={() => setSkipWeek(false)}
              />
            );
          }
          return (
            <section className="work-surface">
              <p>该状态已按 {SURFACE_CONTEXT[surface].label} 的真实版式渲染，而不是勾选矩阵。</p>
            </section>
          );
        }}
      />
    );
  };

  return (
    <div className="opc-e2e" style={variables}>
      <style>{`.opc-e2e {
          display: flex;
          flex-direction: column;
          flex-wrap: nowrap;
          width: 100%;
          max-width: 100%;
          min-width: 1100px;
          min-height: 100vh;
          overflow-x: auto;
          background: var(--bg);
          color: var(--text);
          color-scheme: light dark;
          font: 14px/1.5 system-ui, "Segoe UI Variable", "Segoe UI", sans-serif;
          font-optical-sizing: auto;
        }
        .opc-e2e *,
        .opc-e2e *::before,
        .opc-e2e *::after { box-sizing: border-box; }
        .opc-e2e button,
        .opc-e2e input,
        .opc-e2e select,
        .opc-e2e textarea {
          color: inherit;
          font: inherit;
          touch-action: manipulation;
          -webkit-tap-highlight-color: transparent;
        }
        .opc-e2e button { cursor: pointer; }
        .opc-e2e button:disabled { cursor: not-allowed; opacity: .56; }
        .opc-e2e button:active:not(:disabled) { transform: scale(.985); }
        .opc-e2e :focus-visible {
          outline: 3px solid var(--focus);
          outline-offset: 2px;
        }
        .opc-e2e ::selection { background: var(--accent); color: var(--on-accent); }
        .opc-e2e h1,
        .opc-e2e h2,
        .opc-e2e h3,
        .opc-e2e p { margin-block-start: 0; }
        .opc-e2e h1,
        .opc-e2e h2,
        .opc-e2e h3 {
          scroll-margin-top: 72px;
          text-wrap: balance;
        }
        .opc-e2e p { text-wrap: pretty; }
        .opc-e2e p,
        .opc-e2e dd,
        .opc-e2e td,
        .opc-e2e th,
        .opc-e2e span,
        .opc-e2e small { overflow-wrap: anywhere; }
        .opc-e2e h1 { margin: 0; font-size: 16px; line-height: 1.25; letter-spacing: -.012em; }
        .opc-e2e h2 { margin-block-end: 7px; font-size: 22px; line-height: 1.24; letter-spacing: -.022em; }
        .opc-e2e h3 { margin-block-end: 5px; font-size: 15px; line-height: 1.3; letter-spacing: -.008em; }
        .opc-e2e p { margin-block-end: 10px; max-width: 72ch; }
        .opc-e2e a { color: var(--link); text-underline-offset: .2em; }
        .opc-e2e caption {
          caption-side: top;
          text-align: start;
          padding-block-end: 8px;
          color: var(--muted);
          font-size: 12px;
        }
        .opc-e2e .skip-link {
          position: fixed;
          z-index: 100;
          inset-block-start: 8px;
          inset-inline-start: 8px;
          transform: translateY(-150%);
          border: 1px solid var(--line-strong);
          background: var(--surface);
          padding: 9px 12px;
        }
        .opc-e2e .skip-link:focus { transform: none; }
        .opc-e2e .prototype-bar {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 16px;
          min-height: 52px;
          border-block-end: 1px solid var(--line-strong);
          background: var(--chrome);
          padding: 7px 12px;
        }
        .opc-e2e .prototype-title { min-width: 0; }
        .opc-e2e .prototype-title span { display: block; color: var(--muted); font-size: 12px; }
        .opc-e2e .scenario-select {
          display: grid;
          grid-template-columns: auto minmax(240px, 360px);
          align-items: center;
          gap: 8px;
        }
        .opc-e2e .scenario-select span { color: var(--muted); font-size: 12px; font-weight: 650; }
        .opc-e2e .scenario-select select,
        .opc-e2e .field input,
        .opc-e2e .field select,
        .opc-e2e .field textarea,
        .opc-e2e .state-lab-controls select {
          min-height: 44px;
          min-width: 0;
          border: 1px solid var(--line-strong);
          border-radius: 6px;
          background: var(--surface);
          padding: 8px 10px;
        }
        .opc-e2e .prototype-bar,
        .opc-e2e .shell {
          flex: 0 0 auto;
          min-width: 1100px;
        }
        .opc-e2e .shell {
          display: grid;
          grid-template-columns: 176px minmax(576px, 1fr) 348px;
          grid-auto-flow: column;
          grid-auto-columns: min-content;
          min-width: 1100px;
          min-height: calc(100vh - 52px);
        }
        .opc-e2e .primary-nav {
          display: flex;
          flex-direction: column;
          min-width: 0;
          border-inline-end: 1px solid var(--line);
          background: var(--chrome);
          padding: 10px 8px;
        }
        .opc-e2e .brand {
          padding: 8px 10px 17px;
          font-size: 15px;
          font-weight: 760;
          letter-spacing: -.01em;
        }
        .opc-e2e .primary-nav button {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 8px;
          width: 100%;
          min-height: 44px;
          border: 1px solid transparent;
          border-radius: 6px;
          background: transparent;
          padding: 9px 10px;
          text-align: start;
        }
        .opc-e2e .primary-nav button:hover { background: var(--fill); }
        .opc-e2e .primary-nav button[aria-current="page"] {
          border-color: var(--line-strong);
          background: var(--fill-strong);
          font-weight: 720;
        }
        .opc-e2e .nav-space { flex: 1; min-height: 24px; }
        .opc-e2e .settings-nav {
          border-block-start: 1px solid var(--line);
          border-radius: 0;
          margin-block-start: 8px;
          padding-block-start: 13px;
        }
        .opc-e2e .main-column { min-width: 0; }
        .opc-e2e .context-header {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 16px;
          min-height: 56px;
          border-block-end: 1px solid var(--line);
          background: var(--surface);
          padding: 9px 16px;
        }
        .opc-e2e .context-header p { margin: 0 0 2px; color: var(--muted); font-size: 12px; }
        .opc-e2e .context-header .scene-label {
          margin: 0;
          color: var(--text);
          font-size: 13px;
          font-weight: 680;
        }
        .opc-e2e .cycle-status {
          margin: 4px 0 0;
          color: var(--muted);
          font-size: 12px;
          max-width: none;
        }
        .opc-e2e .context-tools { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; justify-content: flex-end; }
        .opc-e2e .provenance {
          display: inline-flex;
          align-items: center;
          width: max-content;
          min-height: 22px;
          border: 1px solid var(--line-strong);
          border-radius: 2px;
          padding: 1px 6px;
          color: var(--muted);
          font-size: 11px;
          font-weight: 720;
          letter-spacing: .04em;
          text-transform: uppercase;
        }
        .opc-e2e .provenance[data-kind="proposed"] { border-color: var(--warn); color: var(--text); }
        .opc-e2e .provenance[data-kind="governed"] { border-color: var(--info); color: var(--text); }
        .opc-e2e .provenance[data-kind="verified"] { border-color: var(--good); color: var(--text); }
        .opc-e2e .decision-packet {
          display: grid;
          gap: 12px;
          min-width: 0;
          border: 1px solid var(--line-strong);
          border-radius: 7px;
          background: var(--surface);
          padding: 14px;
        }
        .opc-e2e .decision-packet > header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 12px;
        }
        .opc-e2e .decision-packet > header > span { color: var(--muted); font-size: 12px; }
        .opc-e2e .packet-marks { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
        .opc-e2e .decision-packet h3 { margin: 0; font-size: 19px; }
        .opc-e2e .decision-packet > p { margin: 0; color: var(--muted); max-width: none; }
        .opc-e2e .packet-facts {
          display: grid;
          grid-template-columns: repeat(4, minmax(0, 1fr));
          gap: 0;
          margin: 0;
          border-block: 1px solid var(--line);
        }
        .opc-e2e .packet-facts > div {
          min-width: 0;
          border-inline-end: 1px solid var(--line);
          padding: 10px 12px 10px 0;
        }
        .opc-e2e .packet-facts > div:nth-child(4n) { border-inline-end: 0; }
        .opc-e2e .packet-facts dt { color: var(--muted); font-size: 12px; }
        .opc-e2e .packet-facts dd { margin: 4px 0 0; }
        .opc-e2e .packet-actions { display: flex; flex-wrap: wrap; gap: 8px; }
        .opc-e2e .why-layer { min-width: 0; }
        .opc-e2e .why-layer summary,
        .opc-e2e .trace-fold summary {
          cursor: pointer;
          min-height: 44px;
          display: flex;
          align-items: center;
          color: var(--muted);
          font-size: 12px;
          font-weight: 680;
        }
        .opc-e2e .why-layer p,
        .opc-e2e .trace-fold p { margin: 8px 0 0; color: var(--muted); font-size: 13px; }
        .opc-e2e .trace-fold {
          border: 1px solid var(--line);
          border-radius: 6px;
          background: var(--fill);
          padding: 8px 10px;
          margin-block-end: 13px;
        }
        .opc-e2e .exception-lanes {
          display: grid;
          grid-template-columns: repeat(4, minmax(0, 1fr));
          gap: 0;
          border-block: 1px solid var(--line-strong);
        }
        .opc-e2e .exception-lanes button {
          display: grid;
          justify-items: start;
          gap: 6px;
          min-width: 0;
          min-height: 108px;
          border: 0;
          border-inline-end: 1px solid var(--line);
          border-radius: 0;
          background: transparent;
          padding: 12px 14px 12px 0;
          text-align: start;
        }
        .opc-e2e .exception-lanes button:last-child { border-inline-end: 0; }
        .opc-e2e .exception-lanes button:hover { background: var(--fill); }
        .opc-e2e .exception-lanes span { font-size: 12px; font-weight: 720; }
        .opc-e2e .exception-lanes button[data-tone="warn"] > span:first-child { color: var(--warn); }
        .opc-e2e .exception-lanes button[data-tone="info"] > span:first-child { color: var(--info); }
        .opc-e2e .exception-lanes button[data-tone="bad"] > span:first-child { color: var(--bad); }
        .opc-e2e .exception-lanes strong { font-size: 14px; }
        .opc-e2e .exception-lanes small { color: var(--muted); }
        .opc-e2e .staff-table-wrap { width: 100%; overflow: auto; }
        .opc-e2e .staff-table th small { display: block; color: var(--muted); font-weight: 400; }
        .opc-e2e .authority-path {
          display: grid;
          grid-template-columns: repeat(6, minmax(0, 1fr));
          gap: 6px;
          list-style: none;
          margin: 0 0 14px;
          padding: 0;
        }
        .opc-e2e .authority-path li {
          display: grid;
          gap: 6px;
          min-width: 0;
          border: 1px solid var(--line);
          border-radius: 6px;
          padding: 8px;
        }
        .opc-e2e .authority-path strong,
        .opc-e2e .authority-path span { display: block; }
        .opc-e2e .authority-path span { color: var(--muted); font-size: 12px; }
        .opc-e2e .authority-path li[data-state="done"] { border-color: var(--good); }
        .opc-e2e .authority-path li[data-state="current"] { border-color: var(--warn); background: var(--fill); }
        .opc-e2e .why-fragment { margin-block-start: 14px; }
        .opc-e2e .main-content { min-width: 0; padding: 18px; }
        .opc-e2e .scene-stack { display: grid; gap: 14px; }
        .opc-e2e .tag {
          display: inline-flex;
          align-items: center;
          width: max-content;
          max-width: 100%;
          min-height: 26px;
          border: 1px solid var(--line-strong);
          border-radius: 999px;
          padding: 3px 8px;
          color: var(--text);
          font-size: 12px;
          font-weight: 690;
          line-height: 1.25;
        }
        .opc-e2e .tag[data-tone="good"] { border-color: var(--good); }
        .opc-e2e .tag[data-tone="warn"] { border-color: var(--warn); }
        .opc-e2e .tag[data-tone="bad"] { border-color: var(--bad); }
        .opc-e2e .tag[data-tone="info"] { border-color: var(--info); }
        .opc-e2e .primary-button,
        .opc-e2e .secondary-button,
        .opc-e2e .text-button,
        .opc-e2e .inline-button,
        .opc-e2e .segmented button,
        .opc-e2e .stage-tabs button,
        .opc-e2e .mention-buttons button,
        .opc-e2e .step-nav button {
          min-height: 44px;
          border: 1px solid var(--line-strong);
          border-radius: 6px;
          background: var(--surface);
          padding: 8px 12px;
        }
        .opc-e2e .primary-button {
          border-color: var(--accent);
          background: var(--accent);
          color: var(--on-accent);
          font-weight: 750;
        }
        .opc-e2e .primary-button:hover:not(:disabled) {
          background: var(--fill-strong);
          color: var(--text);
        }
        .opc-e2e .secondary-button:hover,
        .opc-e2e .text-button:hover,
        .opc-e2e .inline-button:hover,
        .opc-e2e .segmented button:hover,
        .opc-e2e .stage-tabs button:hover,
        .opc-e2e .mention-buttons button:hover,
        .opc-e2e .step-nav button:hover { background: var(--fill); }
        .opc-e2e .text-button,
        .opc-e2e .inline-button { background: transparent; }
        .opc-e2e .inline-button { min-height: 40px; padding: 6px 9px; }
        .opc-e2e .segmented {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
        }
        .opc-e2e .segmented button[aria-pressed="true"],
        .opc-e2e .provider-options button[aria-pressed="true"] {
          border-color: var(--accent);
          background: var(--fill-strong);
          font-weight: 720;
        }
        .opc-e2e .section-heading {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 12px;
          border-block-end: 1px solid var(--line);
          padding-block-end: 10px;
        }
        .opc-e2e .section-heading h3 { margin: 0; }
        .opc-e2e .section-heading p { margin: 3px 0 0; color: var(--muted); font-size: 12px; }
        .opc-e2e .work-surface,
        .opc-e2e .comparison-surface,
        .opc-e2e .outcome-ledger,
        .opc-e2e .decision-preview,
        .opc-e2e .coverage-matrix,
        .opc-e2e .state-panel {
          min-width: 0;
          border: 1px solid var(--line-strong);
          border-radius: 7px;
          background: var(--surface);
          padding: 14px;
        }
        .opc-e2e .open-section { min-width: 0; padding: 4px 2px; }
        .opc-e2e .today-header,
        .opc-e2e .setup-header,
        .opc-e2e .project-header,
        .opc-e2e .temporary-header,
        .opc-e2e .operations-header,
        .opc-e2e .settings-header,
        .opc-e2e .capability-header,
        .opc-e2e .state-lab-header {
          display: flex;
          align-items: flex-end;
          justify-content: space-between;
          gap: 24px;
          border-block-end: 1px solid var(--line-strong);
          padding: 5px 2px 15px;
        }
        .opc-e2e .today-header p,
        .opc-e2e .setup-header p,
        .opc-e2e .project-header p,
        .opc-e2e .temporary-header p,
        .opc-e2e .operations-header p,
        .opc-e2e .settings-header p,
        .opc-e2e .capability-header p,
        .opc-e2e .state-lab-header p { margin: 0; color: var(--muted); }
        .opc-e2e .header-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 7px; }
        .opc-e2e .staff-strip { min-width: 0; }
        .opc-e2e .operating-report {
          min-width: 0;
          border: 1px solid var(--line-strong);
          border-radius: 7px;
          background: var(--surface);
          padding: 14px;
        }
        .opc-e2e .report-grid {
          display: grid;
          grid-template-columns: repeat(4, minmax(0, 1fr));
          gap: 0;
          border-block-start: 1px solid var(--line);
        }
        .opc-e2e .report-grid section {
          min-width: 0;
          border-inline-end: 1px solid var(--line);
          padding: 10px 12px 10px 0;
        }
        .opc-e2e .report-grid section:nth-child(4n) { border-inline-end: 0; }
        .opc-e2e .report-grid span,
        .opc-e2e .report-grid small { display: block; color: var(--muted); font-size: 12px; }
        .opc-e2e .report-grid strong { display: block; margin: 4px 0 3px; }
        .opc-e2e .thread-cards {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 8px;
          list-style: none;
          margin: 12px 0;
          padding: 0;
        }
        .opc-e2e .thread-cards li {
          min-width: 0;
          border: 1px solid var(--line);
          border-radius: 6px;
          padding: 10px;
        }
        .opc-e2e .thread-cards span { color: var(--muted); font-size: 12px; font-variant-numeric: tabular-nums; }
        .opc-e2e .thread-cards p { margin: 6px 0 0; font-size: 13px; }
        .opc-e2e .messages article.approval-card {
          border: 1px solid var(--line-strong);
          border-radius: 6px;
          background: var(--fill);
          padding: 9px;
        }
        .opc-e2e .ledger-facts,
        .opc-e2e .definition-list,
        .opc-e2e .artifact-parts,
        .opc-e2e .decision-preview dl { margin: 0; }
        .opc-e2e .definition-list > div,
        .opc-e2e .artifact-parts > div,
        .opc-e2e .decision-preview dl > div {
          display: grid;
          grid-template-columns: minmax(126px, .36fr) minmax(0, 1fr);
          gap: 10px;
          border-block-end: 1px solid var(--line);
          padding: 8px 0;
        }
        .opc-e2e .definition-list > div:last-child,
        .opc-e2e .artifact-parts > div:last-child,
        .opc-e2e .decision-preview dl > div:last-child { border-block-end: 0; }
        .opc-e2e dt { color: var(--muted); }
        .opc-e2e dd { min-width: 0; margin: 0; }
        .opc-e2e dd strong,
        .opc-e2e dd small { display: block; }
        .opc-e2e dd small { margin-block-start: 2px; color: var(--muted); font-size: 12px; }
        .opc-e2e .definition-list.compact > div { padding: 7px 0; }
        .opc-e2e .result-list {
          list-style: none;
          margin: 4px 0 0;
          padding: 0;
        }
        .opc-e2e .result-list li {
          display: grid;
          grid-template-columns: minmax(0, 1fr) minmax(148px, auto);
          gap: 14px;
          border-block-end: 1px solid var(--line);
          padding: 12px 0;
        }
        .opc-e2e .result-list li:last-child { border-block-end: 0; }
        .opc-e2e .result-list strong,
        .opc-e2e .result-list span,
        .opc-e2e .result-list small { display: block; }
        .opc-e2e .result-list span,
        .opc-e2e .result-list small { margin-block-start: 3px; color: var(--muted); }
        .opc-e2e .result-list li > div:last-child { display: grid; align-content: start; justify-items: end; gap: 4px; text-align: end; }
        .opc-e2e .accepted-line {
          display: grid;
          grid-template-columns: minmax(220px, .8fr) minmax(0, 1.2fr);
          gap: 20px;
          padding-block-start: 13px;
        }
        .opc-e2e .accepted-line > div strong,
        .opc-e2e .accepted-line > div span { display: block; }
        .opc-e2e .accepted-line > div strong { font-size: 16px; }
        .opc-e2e .accepted-line > div span { margin-block-start: 4px; color: var(--muted); }
        .opc-e2e .ledger-facts { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; }
        .opc-e2e .ledger-facts div { border-inline-start: 1px solid var(--line); padding-inline-start: 10px; }
        .opc-e2e .ledger-facts dt,
        .opc-e2e .ledger-facts dd { display: block; }
        .opc-e2e .first-run {
          display: grid;
          grid-template-columns: minmax(0, 1.35fr) minmax(280px, .65fr);
          gap: 28px;
          min-height: 320px;
          align-items: center;
          border-block: 1px solid var(--line-strong);
          padding: 34px 4px;
        }
        .opc-e2e .first-run-copy h3,
        .opc-e2e .first-run-copy h2 { margin-block-start: 12px; font-size: 21px; }
        .opc-e2e .first-run-copy p,
        .opc-e2e .first-run-copy li { color: var(--muted); }
        .opc-e2e .first-run-copy ul { display: grid; gap: 8px; padding-inline-start: 20px; }
        .opc-e2e .first-run-action { display: grid; gap: 9px; border-inline-start: 1px solid var(--line); padding-inline-start: 22px; }
        .opc-e2e .first-run-action span { color: var(--muted); }
        .opc-e2e .state-panel header { display: flex; align-items: center; gap: 8px; margin-block-end: 8px; }
        .opc-e2e .state-panel p { max-width: none; }
        .opc-e2e .state-panel dl { margin: 0; }
        .opc-e2e .state-panel dl > div {
          display: grid;
          grid-template-columns: minmax(140px, .32fr) minmax(0, 1fr);
          gap: 10px;
          border-block-end: 1px solid var(--line);
          padding: 8px 0;
        }
        .opc-e2e .state-panel[data-tone="bad"] { border-color: var(--bad); }
        .opc-e2e .state-panel[data-tone="warn"] { border-color: var(--warn); }
        .opc-e2e .state-panel[data-tone="info"] { border-color: var(--info); }
        .opc-e2e .state-panel[data-tone="good"] { border-color: var(--good); }
        .opc-e2e .settings-actions { display: flex; flex-wrap: wrap; gap: 8px; padding-block-start: 12px; }
        .opc-e2e .settings-note {
          margin: 10px 0 0;
          color: var(--muted);
          font-size: 12px;
        }
        .opc-e2e .step-count { display: grid; min-width: 125px; text-align: end; }
        .opc-e2e .step-count span { color: var(--muted); font-size: 12px; }
        .opc-e2e .step-nav {
          display: grid;
          grid-template-columns: repeat(5, minmax(0, 1fr));
          gap: 6px;
        }
        .opc-e2e .step-nav button {
          display: flex;
          align-items: center;
          justify-content: flex-start;
          gap: 8px;
          background: transparent;
          text-align: start;
        }
        .opc-e2e .step-nav button span { color: var(--muted); font-variant-numeric: tabular-nums; }
        .opc-e2e .step-nav button[aria-current="step"] { border-color: var(--accent); background: var(--fill-strong); }
        .opc-e2e .field { display: grid; gap: 5px; margin-block-start: 12px; }
        .opc-e2e .field > span { font-weight: 680; }
        .opc-e2e .field > small { color: var(--muted); font-size: 12px; }
        .opc-e2e .field textarea { min-height: 150px; resize: vertical; }
        .opc-e2e .research-summary,
        .opc-e2e .preview-summary,
        .opc-e2e .running-summary,
        .opc-e2e .memory-record header {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 16px;
          padding-block: 12px;
        }
        .opc-e2e .research-summary strong,
        .opc-e2e .research-summary span,
        .opc-e2e .preview-summary strong,
        .opc-e2e .preview-summary span,
        .opc-e2e .running-summary strong,
        .opc-e2e .running-summary span,
        .opc-e2e .running-summary small,
        .opc-e2e .memory-record header strong,
        .opc-e2e .memory-record header span { display: block; }
        .opc-e2e .research-summary span,
        .opc-e2e .preview-summary span,
        .opc-e2e .running-summary span,
        .opc-e2e .running-summary small,
        .opc-e2e .memory-record header span { color: var(--muted); }
        .opc-e2e .revision-label { text-align: end; }
        .opc-e2e .simulation-path,
        .opc-e2e .run-steps,
        .opc-e2e .reconcile-path,
        .opc-e2e .context-ladder {
          display: grid;
          list-style: none;
          margin: 12px 0;
          padding: 0;
        }
        .opc-e2e .simulation-path { grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 7px; }
        .opc-e2e .simulation-path li,
        .opc-e2e .run-steps li,
        .opc-e2e .reconcile-path li {
          min-width: 0;
          border: 1px solid var(--line);
          border-radius: 6px;
          padding: 10px;
        }
        .opc-e2e .simulation-path strong,
        .opc-e2e .simulation-path span,
        .opc-e2e .run-steps strong,
        .opc-e2e .run-steps span,
        .opc-e2e .reconcile-path strong,
        .opc-e2e .reconcile-path span { display: block; }
        .opc-e2e .simulation-path span,
        .opc-e2e .run-steps span,
        .opc-e2e .reconcile-path span { margin-block-start: 4px; color: var(--muted); font-size: 12px; }
        .opc-e2e .simulation-path li[data-state="done"] { border-color: var(--good); }
        .opc-e2e .simulation-path li[data-state="partial"] { border-color: var(--warn); }
        .opc-e2e .simulation-path li[data-state="blocked"] { border-color: var(--bad); }
        .opc-e2e .gap-summary {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 12px;
          margin-block-start: 12px;
        }
        .opc-e2e .gap-summary > div { display: grid; gap: 6px; border-block-start: 1px solid var(--line); padding-block-start: 10px; }
        .opc-e2e .gap-summary span { color: var(--muted); }
        .opc-e2e .flow-actions { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
        .opc-e2e .flow-end { color: var(--muted); }
        .opc-e2e .stage-tabs {
          display: flex;
          flex-wrap: wrap;
          gap: 5px;
          border-block-end: 1px solid var(--line);
          padding-block-end: 9px;
        }
        .opc-e2e .stage-tabs button { border-color: transparent; background: transparent; }
        .opc-e2e .stage-tabs button[aria-current="page"] { border-color: var(--line-strong); background: var(--fill-strong); font-weight: 720; }
        .opc-e2e .package-layout {
          display: grid;
          grid-template-columns: minmax(0, 1.35fr) minmax(240px, .65fr);
          gap: 16px;
        }
        .opc-e2e .artifact-preview { min-width: 0; border-block: 1px solid var(--line-strong); padding-block: 14px; }
        .opc-e2e .artifact-preview header { display: flex; justify-content: space-between; gap: 12px; }
        .opc-e2e .artifact-preview header h3 { margin-block-start: 8px; font-size: 19px; }
        .opc-e2e .artifact-preview header > span { color: var(--muted); }
        .opc-e2e .thread-copy { margin-block: 10px 16px; max-width: 56ch; font-size: 17px; line-height: 1.55; }
        .opc-e2e .acceptance-checks { border: 1px solid var(--line-strong); border-radius: 7px; background: var(--surface); padding: 13px; }
        .opc-e2e .acceptance-checks ul { list-style: none; margin: 8px 0; padding: 0; }
        .opc-e2e .acceptance-checks li { display: grid; grid-template-columns: 56px minmax(0, 1fr); gap: 8px; border-block-end: 1px solid var(--line); padding: 8px 0; }
        .opc-e2e .acceptance-checks li > span { color: var(--info); font-size: 12px; font-weight: 800; }
        .opc-e2e .acceptance-checks strong,
        .opc-e2e .acceptance-checks small { display: block; }
        .opc-e2e .acceptance-checks small,
        .opc-e2e .acceptance-checks p { color: var(--muted); font-size: 12px; }
        .opc-e2e .receipt-head { display: flex; align-items: center; gap: 10px; padding-block: 13px; }
        .opc-e2e .readback-grid,
        .opc-e2e .reflection-grid {
          display: grid;
          grid-template-columns: repeat(4, minmax(0, 1fr));
          gap: 0;
          margin-block: 12px;
          border-block: 1px solid var(--line);
        }
        .opc-e2e .reflection-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
        .opc-e2e .readback-grid > div,
        .opc-e2e .reflection-grid > div { min-width: 0; border-inline-end: 1px solid var(--line); padding: 11px; }
        .opc-e2e .readback-grid > div:last-child,
        .opc-e2e .reflection-grid > div:last-child { border-inline-end: 0; }
        .opc-e2e .readback-grid span,
        .opc-e2e .readback-grid strong,
        .opc-e2e .readback-grid small { display: block; }
        .opc-e2e .readback-grid span,
        .opc-e2e .readback-grid small,
        .opc-e2e .reflection-grid p { color: var(--muted); }
        .opc-e2e .readback-grid strong { margin-block: 5px; }
        .opc-e2e .loop-ledger { border-block-start: 1px solid var(--line-strong); padding-block-start: 12px; }
        .opc-e2e .loop-ledger ol {
          display: grid;
          grid-template-columns: repeat(8, minmax(116px, 1fr));
          gap: 6px;
          overflow-x: auto;
          list-style: none;
          margin: 10px 0 0;
          padding: 0 0 6px;
        }
        .opc-e2e .loop-ledger li { border: 1px solid var(--line); border-radius: 6px; padding: 8px; }
        .opc-e2e .loop-ledger strong,
        .opc-e2e .loop-ledger span { display: block; }
        .opc-e2e .loop-ledger span { margin-block-start: 3px; color: var(--muted); font-size: 12px; }
        .opc-e2e .loop-ledger li[data-state="done"] { border-color: var(--good); }
        .opc-e2e .loop-ledger li[data-state="partial"],
        .opc-e2e .loop-ledger li[data-state="waiting"] { border-color: var(--warn); }
        .opc-e2e .loop-ledger li[data-state="blocked"] { border-color: var(--bad); }
        .opc-e2e .loop-ledger li[data-state="sample"] { border-style: dashed; }
        .opc-e2e .comparison-table-wrap { width: 100%; overflow: auto; margin-block-start: 10px; }
        .opc-e2e table { width: 100%; border-collapse: collapse; font-variant-numeric: tabular-nums; }
        .opc-e2e th,
        .opc-e2e td { min-width: 112px; border-block-end: 1px solid var(--line); padding: 9px 8px; text-align: start; vertical-align: top; }
        .opc-e2e thead th { color: var(--muted); font-size: 12px; font-weight: 700; }
        .opc-e2e tbody th { font-weight: 700; }
        .opc-e2e tbody tr[data-selected="true"] { background: var(--fill); }
        .opc-e2e .typed-canvas-grid {
          display: grid;
          grid-template-columns: minmax(0, 1.15fr) minmax(260px, .85fr);
          gap: 16px;
        }
        .opc-e2e .decision-panel,
        .opc-e2e .evidence-panel { min-width: 0; border-block-start: 1px solid var(--line-strong); padding-block-start: 12px; }
        .opc-e2e .decision-panel p { font-size: 16px; font-weight: 650; }
        .opc-e2e .decision-panel small { color: var(--muted); }
        .opc-e2e .evidence-panel ul { list-style: none; margin: 8px 0 0; padding: 0; }
        .opc-e2e .evidence-panel li { display: grid; grid-template-columns: minmax(120px, .35fr) minmax(0, 1fr); gap: 8px; border-block-end: 1px solid var(--line); padding: 8px 0; }
        .opc-e2e .evidence-panel span { color: var(--muted); }
        .opc-e2e .object-chain {
          display: grid;
          grid-template-columns: minmax(140px, 1fr) auto minmax(140px, 1fr) auto minmax(120px, .8fr) auto minmax(165px, 1.2fr);
          align-items: center;
          gap: 7px;
          border-block-end: 1px solid var(--line-strong);
          padding-block-end: 14px;
        }
        .opc-e2e .object-chain > div { min-width: 0; border: 1px solid var(--line); border-radius: 6px; padding: 9px; }
        .opc-e2e .object-chain strong,
        .opc-e2e .object-chain div span { display: block; }
        .opc-e2e .object-chain div span,
        .opc-e2e .object-chain > span { color: var(--muted); font-size: 12px; }
        .opc-e2e .people-layout {
          display: grid;
          grid-template-columns: 220px minmax(0, 1fr);
          gap: 14px;
        }
        .opc-e2e .member-list { border: 1px solid var(--line-strong); border-radius: 7px; background: var(--surface); padding: 7px; }
        .opc-e2e .member-list button {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 8px;
          width: 100%;
          min-height: 58px;
          border: 1px solid transparent;
          border-block-end-color: var(--line);
          background: transparent;
          padding: 8px;
          text-align: start;
        }
        .opc-e2e .member-list button:hover { background: var(--fill); }
        .opc-e2e .member-list button[aria-current="page"] { border-color: var(--line-strong); background: var(--fill-strong); }
        .opc-e2e .member-list strong,
        .opc-e2e .member-list small { display: block; }
        .opc-e2e .member-list small { color: var(--muted); }
        .opc-e2e .version-compare {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 14px;
          margin-block: 12px;
        }
        .opc-e2e .version-compare > div { min-width: 0; border-block: 1px solid var(--line); padding-block: 11px; }
        .opc-e2e .version-compare span,
        .opc-e2e .version-compare strong { display: block; }
        .opc-e2e .version-compare span,
        .opc-e2e .version-compare p { color: var(--muted); }
        .opc-e2e .version-compare strong { margin-block: 5px; }
        .opc-e2e .run-steps,
        .opc-e2e .reconcile-path { grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 7px; }
        .opc-e2e .reconcile-path { grid-template-columns: repeat(4, minmax(0, 1fr)); }
        .opc-e2e .run-steps li[data-state="done"] { border-color: var(--good); }
        .opc-e2e .run-steps li[data-state="current"],
        .opc-e2e .reconcile-path li[data-state="current"] { border-color: var(--warn); background: var(--fill); }
        .opc-e2e .reconcile-path li[data-state="done"] { border-color: var(--info); }
        .opc-e2e .memory-record { padding-block-start: 4px; }
        .opc-e2e .memory-record > p { color: var(--muted); }
        .opc-e2e .memory-record .segmented { margin-block: 12px; }
        .opc-e2e .context-budget { display: grid; gap: 9px; padding-block: 12px; }
        .opc-e2e .context-budget strong,
        .opc-e2e .context-budget span { display: block; }
        .opc-e2e .context-budget span { color: var(--muted); }
        .opc-e2e .context-ladder { gap: 0; }
        .opc-e2e .context-ladder li {
          display: grid;
          grid-template-columns: 34px minmax(0, 1fr);
          gap: 10px;
          border-block-end: 1px solid var(--line);
          padding: 9px 0;
        }
        .opc-e2e .context-ladder li > span { color: var(--muted); }
        .opc-e2e .context-ladder strong,
        .opc-e2e .context-ladder small { display: block; }
        .opc-e2e .context-ladder small { color: var(--muted); }
        .opc-e2e .context-ladder li[data-protected="true"] strong::after { content: " · protected"; color: var(--info); font-size: 12px; }
        .opc-e2e .connection-layout {
          display: grid;
          grid-template-columns: minmax(0, 1.05fr) minmax(300px, .95fr);
          gap: 14px;
        }
        .opc-e2e .provider-options {
          display: grid;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          gap: 7px;
          margin-block: 12px;
        }
        .opc-e2e .provider-options button {
          min-height: 72px;
          border: 1px solid var(--line-strong);
          border-radius: 6px;
          background: transparent;
          padding: 10px;
          text-align: start;
        }
        .opc-e2e .provider-options strong,
        .opc-e2e .provider-options span { display: block; }
        .opc-e2e .provider-options span { margin-block-start: 4px; color: var(--muted); font-size: 12px; }
        .opc-e2e .custom-fields { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 0 16px; }
        .opc-e2e .secret-route { display: grid; align-content: center; gap: 5px; min-height: 88px; margin-block-start: 12px; border-block: 1px solid var(--line); }
        .opc-e2e .secret-route span { color: var(--muted); }
        .opc-e2e .capability-layout {
          display: grid;
          grid-template-columns: minmax(0, 1.25fr) minmax(270px, .75fr);
          gap: 14px;
        }
        .opc-e2e .review-rows { margin-block-start: 9px; }
        .opc-e2e .review-rows > div {
          display: grid;
          grid-template-columns: minmax(125px, .28fr) minmax(0, 1fr) minmax(130px, .35fr);
          gap: 10px;
          border-block-end: 1px solid var(--line);
          padding: 8px 0;
        }
        .opc-e2e .review-rows span,
        .opc-e2e .review-rows small { color: var(--muted); }
        .opc-e2e .decision-preview > p,
        .opc-e2e .decision-preview output { color: var(--muted); }
        .opc-e2e .decision-preview .segmented { display: grid; margin-block: 12px; }
        .opc-e2e .decision-preview output { display: block; border-block: 1px solid var(--line); padding-block: 10px; }
        .opc-e2e .state-lab-controls { display: grid; grid-template-columns: repeat(2, minmax(160px, 1fr)); gap: 8px; }
        .opc-e2e .state-lab-controls label { display: grid; gap: 4px; }
        .opc-e2e .state-lab-controls label span { color: var(--muted); font-size: 12px; }
        .opc-e2e .additional-states {
          display: grid;
          grid-template-columns: repeat(4, minmax(0, 1fr));
          border-block: 1px solid var(--line);
        }
        .opc-e2e .additional-states > div { min-width: 0; border-inline-end: 1px solid var(--line); padding: 11px; }
        .opc-e2e .additional-states > div:last-child { border-inline-end: 0; }
        .opc-e2e .additional-states strong,
        .opc-e2e .additional-states span { display: block; }
        .opc-e2e .additional-states span { margin-block-start: 4px; color: var(--muted); }
        .opc-e2e .conversation {
          display: flex;
          flex-direction: column;
          min-width: 0;
          min-height: 0;
          border-inline-start: 1px solid var(--line);
          background: var(--surface);
        }
        .opc-e2e .conversation > header {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 10px;
          border-block-end: 1px solid var(--line);
          padding: 12px;
        }
        .opc-e2e .conversation > header span { color: var(--muted); font-size: 12px; }
        .opc-e2e .conversation > header h2 { margin: 2px 0 0; font-size: 16px; }
        .opc-e2e .participants {
          display: flex;
          flex-wrap: wrap;
          gap: 5px;
          border-block-end: 1px solid var(--line);
          padding: 8px 12px;
        }
        .opc-e2e .participants span { border: 1px solid var(--line); border-radius: 999px; padding: 3px 7px; color: var(--muted); font-size: 12px; }
        .opc-e2e .messages { flex: 1; min-height: 230px; overflow-y: auto; overscroll-behavior: contain; padding: 12px; }
        .opc-e2e .messages article { border-block-end: 1px solid var(--line); margin-block-end: 13px; padding-block-end: 12px; }
        .opc-e2e .messages article > span { color: var(--muted); font-size: 12px; font-weight: 680; }
        .opc-e2e .messages article p { margin: 4px 0; }
        .opc-e2e .messages article small { display: block; color: var(--muted); }
        .opc-e2e .messages article .inline-button { margin-block-start: 8px; }
        .opc-e2e .messages article[data-author="owner"] p { margin-inline-start: 14px; font-weight: 620; }
        .opc-e2e .messages article[data-author="system"] { border: 1px solid var(--line); border-radius: 6px; background: var(--fill); padding: 9px; }
        .opc-e2e .composer { display: grid; gap: 8px; border-block-start: 1px solid var(--line); padding: 10px; }
        .opc-e2e .composer > label { display: grid; gap: 5px; }
        .opc-e2e .composer > label > span { font-weight: 680; }
        .opc-e2e .composer textarea {
          width: 100%;
          min-height: 96px;
          resize: vertical;
          border: 1px solid var(--line-strong);
          border-radius: 6px;
          background: var(--surface);
          padding: 9px 10px;
        }
        .opc-e2e .mention-buttons { display: flex; flex-wrap: wrap; gap: 5px; }
        .opc-e2e .mention-buttons button { min-height: 40px; padding: 6px 9px; }
        .opc-e2e .composer-actions { display: grid; gap: 6px; }
        .opc-e2e .composer-actions small { color: var(--muted); }
        .opc-e2e .composer .gap { grid-template-columns: 1fr; gap: 3px; font-size: 12px; }
        @media (prefers-reduced-motion: reduce) {
          .opc-e2e *,
          .opc-e2e *::before,
          .opc-e2e *::after {
            animation-duration: .01ms !important;
            transition-duration: .01ms !important;
            scroll-behavior: auto !important;
          }
          .opc-e2e button:active:not(:disabled) { transform: none; }
        }
        .opc-e2e .shell.chat-hidden {
          grid-template-columns: 176px minmax(576px, 1fr);
        }
        .opc-e2e .empty-home {
          display: grid;
          place-items: center;
          min-height: 460px;
          text-align: center;
          padding: 48px 16px;
        }
        .opc-e2e .empty-home h2 { margin: 12px 0; font-size: 28px; letter-spacing: -.03em; }
        .opc-e2e .empty-home p { margin: 0 auto 18px; color: var(--muted); }
        .opc-e2e .process-axis {
          display: grid;
          grid-auto-flow: column;
          grid-auto-columns: minmax(132px, 1fr);
          gap: 6px;
          overflow-x: auto;
          list-style: none;
          margin: 0 0 14px;
          padding: 0 0 6px;
        }
        .opc-e2e .process-axis button {
          display: grid;
          justify-items: start;
          gap: 4px;
          min-height: 72px;
          border: 1px solid var(--line);
          border-radius: 6px;
          background: var(--surface);
          padding: 8px 10px;
          text-align: start;
        }
        .opc-e2e .process-axis button[aria-current="step"] {
          border-color: var(--accent);
          background: var(--fill-strong);
          font-weight: 720;
        }
        .opc-e2e .process-axis button[data-mark="auth"] { border-color: var(--warn); }
        .opc-e2e .process-axis button[data-mark="verify"] { border-color: var(--info); }
        .opc-e2e .process-axis small { color: var(--muted); font-size: 12px; }
        .opc-e2e .run-counts {
          display: grid;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          gap: 0;
          border: 1px solid var(--line-strong);
          border-radius: 7px;
          background: var(--surface);
        }
        .opc-e2e .run-counts button,
        .opc-e2e .run-counts > div {
          min-width: 0;
          border: 0;
          border-inline-end: 1px solid var(--line);
          border-radius: 0;
          background: transparent;
          padding: 12px 14px;
          text-align: start;
        }
        .opc-e2e .run-counts > :last-child { border-inline-end: 0; }
        .opc-e2e .run-counts span,
        .opc-e2e .run-counts small { display: block; color: var(--muted); font-size: 12px; }
        .opc-e2e .run-counts strong { display: block; margin: 4px 0 2px; font-size: 20px; }
        .opc-e2e .stage-detail {
          display: grid;
          gap: 8px;
          margin-block-start: 12px;
        }
        .opc-e2e .confirm-list {
          list-style: none;
          margin: 0;
          padding: 0;
        }
        .opc-e2e .confirm-list li {
          display: grid;
          grid-template-columns: minmax(0, 1fr) auto;
          gap: 10px;
          align-items: center;
          border-block-end: 1px solid var(--line);
          padding: 10px 0;
        }
        .opc-e2e .wizard-dots {
          display: flex;
          flex-wrap: wrap;
          gap: 4px;
          margin: 0 0 12px;
        }
        .opc-e2e .wizard-dot {
          display: grid;
          place-items: center;
          width: 28px;
          min-height: 44px;
          padding: 0;
          border: 0;
          background: transparent;
        }
        .opc-e2e .wizard-dot::before {
          content: "";
          width: 8px;
          height: 8px;
          border-radius: 99px;
          background: var(--line-strong);
        }
        .opc-e2e .wizard-dot[aria-selected="true"]::before {
          width: 20px;
          background: var(--accent);
        }
        .opc-e2e .wizard-viewport {
          overflow: hidden;
          width: 100%;
        }
        .opc-e2e .wizard-rail {
          display: flex;
          width: 100%;
          transform: translateX(calc(-1 * var(--wizard-index, 0) * 100%));
          transition: transform 280ms ease;
        }
        .opc-e2e .wizard-slide {
          flex: 0 0 100%;
          min-width: 0;
          padding-inline-end: 22px;
          box-sizing: border-box;
        }
        .opc-e2e .wizard-card {
          display: grid;
          gap: 10px;
          min-height: 280px;
          border: 1px solid var(--line-strong);
          border-radius: 7px;
          background: var(--fill);
          padding: 12px;
        }
        .opc-e2e .wizard-card-meta {
          display: grid;
          gap: 6px;
        }
        .opc-e2e .wizard-card-meta small { color: var(--muted); }
        .opc-e2e .wizard-nav {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
          margin-block-start: 14px;
        }
        .opc-e2e .wizard-status { margin: 10px 0 0; }
        .opc-e2e .knowledge-filters {
          display: grid;
          gap: 10px;
          margin: 12px 0;
        }
        .opc-e2e .prototype-outcomes {
          display: flex;
          flex-wrap: wrap;
          align-items: center;
          gap: 8px;
          margin-block-start: 10px;
        }
        .opc-e2e .prototype-outcomes > span { color: var(--muted); font-size: 12px; }
        .opc-e2e .messages article.canvas-mirror {
          border: 1px solid var(--line-strong);
          background: var(--fill-strong);
        }
        .opc-e2e .hitl-actions { display: flex; flex-wrap: wrap; gap: 8px; }
        .opc-e2e .primary-nav button[aria-disabled="true"] { opacity: .45; }
        .opc-e2e .secret-field input { font-family: ui-monospace, Consolas, monospace; }
        .opc-e2e .copy-banner {
          border: 1px dashed var(--line-strong);
          border-radius: 7px;
          background: var(--fill);
          padding: 12px 14px;
        }
        @media (prefers-contrast: more) {
          .opc-e2e .work-surface,
          .opc-e2e .comparison-surface,
          .opc-e2e .decision-packet,
          .opc-e2e .outcome-ledger,
          .opc-e2e .decision-preview,
          .opc-e2e .coverage-matrix,
          .opc-e2e .state-panel,
          .opc-e2e .run-counts,
          .opc-e2e .process-axis button,
          .opc-e2e .wizard-card { border-color: var(--text); }
        }`}</style>

      <a className="skip-link" href="#opc-main">Skip to main workbench</a>

      <header className="prototype-bar">
        <div className="prototype-title">
          <h1>Personal 2.0 · OPC 端到端原型 · v1</h1>
          <span>① 横向确认卡片 · Knowledge 导入 · Knowledge/Settings 默认收起对话</span>
        </div>
        <label className="scenario-select">
          <span>Prototype scenario</span>
          <Select
            value={scene}
            onChange={(next) => applyScenario(next as Scene)}
            options={SCENES.map((item) => ({
              value: item.id,
              label: item.label,
            }))}
          />
        </label>
      </header>

      <div className={chatHidden ? "shell chat-hidden" : "shell"}>
        <nav className="primary-nav" aria-label="Personal primary navigation">
          <div className="brand">Personal</div>
          <button
            type="button"
            aria-current={scene === "today" || scene === "empty-home" || scene === "today-incomplete" ? "page" : undefined}
            onClick={onNavToday}
          >
            Today
            {scene === "today" ? <Tag tone="warn">1</Tag> : null}
          </button>
          <button
            type="button"
            aria-current={projectsCurrent ? "page" : undefined}
            onClick={() => setScene("projects")}
          >
            Projects
          </button>
          <button
            type="button"
            aria-current={scene === "knowledge" ? "page" : undefined}
            aria-disabled={!knowledgeOk && scene !== "knowledge"}
            onClick={() => setScene("knowledge")}
          >
            Knowledge
            {!knowledgeOk ? <Tag>锁</Tag> : null}
          </button>
          <div className="nav-space" />
          <button
            className="settings-nav"
            type="button"
            aria-current={scene === "settings" ? "page" : undefined}
            onClick={() => setScene("settings")}
          >
            Settings
          </button>
        </nav>

        <main className="main-column" id="opc-main">
          <header className="context-header">
            <div>
              <p>{locationLabel}</p>
              <p className="scene-label">{SCENE_TITLES[scene]}</p>
            </div>
            <div className="context-tools">
              <Tag tone="neutral">Windows 本机 · 在线时工作</Tag>
              <Tag tone="info">{chatHidden ? "对话已隐藏" : projectGroup(scene)}</Tag>
              {scene === "knowledge" || scene === "settings" ? (
                <button
                  className="secondary-button"
                  type="button"
                  onClick={() => setChatOpen(!chatOpen)}
                >
                  {chatOpen ? "收起对话" : "打开对话"}
                </button>
              ) : null}
            </div>
          </header>
          <div className="main-content">{renderMain(scene)}</div>
        </main>

        {chatHidden ? null : (
          <Conversation
            scene={scene}
            providerBound={providerBound}
            drafts={drafts}
            setDrafts={setDrafts}
            status={composerStatus}
            setStatus={setComposerStatus}
            onOpenHitl={() => setScene("hitl")}
            wizardStep={{ id: currentWizard.id, label: currentWizard.label }}
            wizardValue={wizardValues[currentWizard.id]}
            wizardConfirmed={Boolean(wizardConfirmed[currentWizard.id])}
            wizardStale={Boolean(wizardStale[currentWizard.id])}
            receipts={receipts}
            onApplyToCard={applyDraftToCard}
          />
        )}
      </div>
    </div>
  );
}

function projectGroup(scene: Scene): string {
  if (
    scene === "project" ||
    scene === "add-member" ||
    scene === "hitl" ||
    scene === "create-members" ||
    scene === "create-process" ||
    scene === "create-test" ||
    scene === "create-joint"
  ) {
    return "项目群";
  }
  return "Personal Assistant";
}

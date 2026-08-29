/**
 * PERSONAL 2.0 OPC INTERACTION PROTOTYPE — optimized v9
 *
 * Built-in mock data and local React state only. This Canvas does not connect
 * to a daemon, network, storage, filesystem, Provider, model, Skill, MCP
 * server, connector, or SecretStore. It cannot create Projects, send messages,
 * install capabilities, grant permissions, publish, reconcile Effects, admit
 * Memory, or issue receipts. Target-state samples are labelled explicitly.
 * HITL buttons are simulated: they change local prototype state only.
 *
 * v9 delta (baseline = owner-approved v8; do not overwrite v8):
 * 战役 Must-fix / Major：三栏锁定（仅空 Home 藏聊天）；L2 详情/成员/运行/产出；
 * 创建环顺序门与验收门；未完成创建按段续跑；日常 Today 项目行概览；加人拒绝/
 * pending/执行方式入口；成员身份在详情头；HITL 改窄作废旧预览；State Lab 挂真表面。
 * 仍无聊天 Approve、无 Install、无 CEO 六步轨、无四泳道、无 Team/Inbox 一级、无 X。
 *
 * Design artifact:
 * d:\agent-kernel\clients\docs\design\opc-2.0\personal-20-opc-e2e-optimized-v9.canvas.tsx
 * Cursor-openable copy (IDE detection path; not a second product baseline):
 * C:\Users\wuron\.cursor\projects\d-agent-kernel\canvases\personal-20-opc-e2e-optimized-v9.canvas.tsx
 * Hosted and repository copies must stay byte-aligned.
 * Baseline (do not overwrite):
 * d:\agent-kernel\clients\docs\design\opc-2.0\personal-20-opc-e2e-optimized-v8.canvas.tsx
 */

import {
  Callout,
  Select,
  TextArea,
  TextInput,
  useEffect,
  useHostTheme,
  useRef,
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
  | "project-detail"
  | "project-members"
  | "project-runs"
  | "project-outputs"
  | "add-member"
  | "member-config"
  | "hitl"
  | "knowledge"
  | "settings"
  | "state-lab";
type ProjectWorkScene = "project-detail" | "project-members" | "project-runs" | "project-outputs";
type MemberConfigTab =
  | "duty"
  | "input"
  | "output"
  | "skills"
  | "tools"
  | "prompt"
  | "loop"
  | "perms";

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
  | "members"
  | "runs"
  | "outputs"
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
type TestOutcome = "idle" | "running" | "pass" | "fail" | "unknown";
type PreviewAge = "fresh" | "stale" | "unknown";
type HitlFate = "idle" | "approved" | "narrowed" | "rejected" | "stopped";
type ConnectionStatus = "none" | "connected" | "failed";
type KnowledgeTab = "files" | "import" | "why" | "memory";
type KnowledgeScope = "all" | "shared" | "weekly" | "site";
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
type ChatAuthor = "owner" | "assistant" | "system";
type FieldProposal = {
  field: string;
  label: string;
  value: string;
  status: "pending" | "applied" | "dismissed";
};
type ChatTurn = {
  id: number;
  author: ChatAuthor;
  label: string;
  text: string;
  proposal?: FieldProposal;
};
type PendingCommit = {
  field: string;
  label: string;
  previous: string;
  next: string;
};
type StageDraft = { input: string; method: string; rights: string };
type KnowledgeFile = {
  id: string;
  title: string;
  project: Exclude<KnowledgeScope, "all">;
  projectLabel: string;
  kind: Exclude<KnowledgeKind, "all">;
  statusLabel: string;
  tone: Tone;
};

type RuntimeSlotId = "prompt" | "tools" | "skills" | "loop" | "mcp" | "files";
type SlotFill = "empty" | "draft" | "ready" | "needs-grant" | "unknown";
type MemberInitStatus = "idle" | "generating" | "ready" | "confirmed" | "partial" | "blocked" | "unknown";
type RuntimeSlot = {
  id: RuntimeSlotId;
  businessLabel: string;
  runtimeLabel: string;
  value: string;
  status: SlotFill;
};

type MemberDraft = {
  id: string;
  name: string;
  duty: string;
  handoff: string;
  model: string;
  joined: boolean;
  initStatus: MemberInitStatus;
  runtime: RuntimeSlot[];
};

type ProcessStage = {
  id: string;
  label: string;
  owner: string;
  status: string;
  currentStep: string;
  tone: Tone;
  mark: "none" | "auth" | "verify";
  todayOk: string;
  todayFail: string;
  todayAvg: string;
  success: string;
};

type SampleProjectId = "weekly" | "site" | "weekly-copy" | "creating-draft";
type OutputFormat = "document" | "checklist" | "packet" | "article" | "link";
type OutputSampleKey = "empty" | "document" | "packet" | "unknown" | "working" | "partial";
type ProjectKind = "live" | "copy-draft" | "creating";
type SampleOutput = {
  id: string;
  title: string;
  job: string;
  format: OutputFormat;
  accepted: boolean;
  needsHitl: boolean;
};
type SampleProject = {
  id: SampleProjectId;
  name: string;
  kind: ProjectKind;
  industry: string;
  goal: string;
  cycle: string;
  statusLine: string;
  costLine: string;
  blurb: string;
  currentStageId: string;
  stages: readonly ProcessStage[];
  participants: readonly string[];
  outputs: readonly SampleOutput[];
  todayOk: string;
  todayFail: string;
  todayAvg: string;
};

const SCENES: ReadonlyArray<{ id: Scene; label: string }> = [
  { id: "empty-home", label: "空首页 · 只创建项目" },
  { id: "create-init", label: "创建 ① 项目初始化" },
  { id: "create-process", label: "创建 ② 流程初始化" },
  { id: "create-members", label: "创建 ③ 成员初始化" },
  { id: "create-test", label: "创建 ④ 分环节测试" },
  { id: "create-joint", label: "创建 ⑤ 联合调试" },
  { id: "today-incomplete", label: "今日 · 只继续创建" },
  { id: "today", label: "今日 · ⑤ 之后" },
  { id: "projects", label: "项目列表 · 管理与复制" },
  { id: "project-detail", label: "项目详情 · 名称目标周期" },
  { id: "project-members", label: "成员管理 · 先选人再看配置" },
  { id: "project-runs", label: "运行管理 · 当前步骤" },
  { id: "project-outputs", label: "产出管理 · 助手编排" },
  { id: "add-member", label: "加人" },
  { id: "member-config", label: "成员配置 · 执行方式" },
  { id: "hitl", label: "画布拍板预览" },
  { id: "knowledge", label: "知识" },
  { id: "settings", label: "设置 · 模型与跳过收回" },
  { id: "state-lab", label: "状态实验室 · 真版式覆盖" },
];

const SCENE_TITLES: Record<Scene, string> = {
  "empty-home": "今日",
  "create-init": "创建项目 · ① 项目初始化",
  "create-process": "创建项目 · ② 流程初始化",
  "create-members": "创建项目 · ③ 成员初始化",
  "create-test": "创建项目 · ④ 分环节测试",
  "create-joint": "创建项目 · ⑤ 联合调试",
  "today-incomplete": "今日",
  today: "今日",
  projects: "项目列表",
  "project-detail": "项目详情",
  "project-members": "成员管理",
  "project-runs": "运行管理",
  "project-outputs": "产出管理",
  "add-member": "加人",
  "member-config": "成员配置 · 执行方式",
  hitl: "需要你拍板",
  knowledge: "知识",
  settings: "设置",
  "state-lab": "状态实验室",
};

const PROJECT_SUBNAV: ReadonlyArray<{ id: Scene; label: string }> = [
  { id: "project-detail", label: "详情" },
  { id: "project-members", label: "成员" },
  { id: "project-runs", label: "运行" },
  { id: "project-outputs", label: "产出" },
];

const MEMBER_CONFIG_TABS: ReadonlyArray<{ id: MemberConfigTab; label: string }> = [
  { id: "duty", label: "职责" },
  { id: "input", label: "输入" },
  { id: "output", label: "输出" },
  { id: "skills", label: "技能" },
  { id: "tools", label: "工具" },
  { id: "prompt", label: "工作说明" },
  { id: "loop", label: "周期与触发" },
  { id: "perms", label: "连接与权限" },
];

const CONFIRM_ITEMS: ReadonlyArray<{ id: ConfirmId; label: string; detail: string }> = [
  { id: "process", label: "业务流程", detail: "收集事实 → 分析 → 起草周报 → 核对 → 交给 Owner" },
  { id: "outputs", label: "各环节产出", detail: "事实清单、建议稿、周报草稿、核对记录、可打开周报" },
  { id: "cycle", label: "周期", detail: "每周一 09:00，仅在本机在线时运行" },
  { id: "format", label: "保存形式", detail: "Markdown 周报 + 附件清单；不是发布到社交网络" },
  { id: "skill", label: "本项目要用的能力", detail: "来源、版本、许可待审。能力包（有时也称 Skill）安装不是授权。" },
  { id: "tools", label: "工具", detail: "检索与文档整理。无假 Install。" },
  { id: "mcp", label: "外部连接", detail: "精确版本与权限需另批。无市场安装按钮。底层有时也称 MCP。" },
  { id: "knowledge", label: "知识库", detail: "本项目资料；Obsidian 为底座，不必安装该应用" },
  { id: "env", label: "外部工作环境", detail: "本机在线。无云端 24/7 承诺。" },
  { id: "files", label: "文件权限", detail: "仅当前项目目录。扩权要再批。" },
  { id: "auto", label: "自动 / 批准", detail: "内部起草可自动；对外发送走画布预览" },
  { id: "triggers", label: "触发", detail: "手动、日程、已验收产出。同类不重叠。" },
  { id: "cost", label: "费用", detail: "估计或实际须标注来源；未知不写 0" },
  { id: "rights", label: "来源权利", detail: "外部文本不可信，不能当指令执行" },
  { id: "method", label: "执行方式", detail: "每环节怎么做、周期、触发。不出现底层引擎名。" },
  { id: "preview", label: "总预览", detail: "确认前项目未上线。离开会留草稿。" },
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
    id: "inspect-md",
    title: "本周现场周检.md",
    project: "site",
    projectLabel: "设备现场周检",
    kind: "markdown",
    statusLabel: "已索引 · 来源：Owner 导入",
    tone: "good",
  },
  {
    id: "inspect-photo",
    title: "3 号机房照片.jpg",
    project: "site",
    projectLabel: "设备现场周检",
    kind: "image",
    statusLabel: "已索引 · 图片元数据",
    tone: "info",
  },
];

const PROCESS_STAGES: readonly ProcessStage[] = [
  {
    id: "collect",
    label: "收集本周事实",
    owner: "梅",
    status: "进行中 · 已 41 分钟",
    currentStep: "正在摘录「本周客户跟进.md」里待回复客户",
    tone: "info",
    mark: "none",
    todayOk: "2（样品）",
    todayFail: "0（样品）",
    todayAvg: "18 分（样品）",
    success: "2/2（样品）",
  },
  {
    id: "analyze",
    label: "分析与建议",
    owner: "林",
    status: "等待事实清单",
    currentStep: "未开始 · 缺上一环可打开清单",
    tone: "neutral",
    mark: "none",
    todayOk: "1（样品）",
    todayFail: "0（样品）",
    todayAvg: "24 分（样品）",
    success: "1/1（样品）",
  },
  {
    id: "draft",
    label: "起草周报",
    owner: "锐",
    status: "未开始",
    currentStep: "未开始",
    tone: "neutral",
    mark: "none",
    todayOk: "—",
    todayFail: "—",
    todayAvg: "—",
    success: "—",
  },
  {
    id: "verify",
    label: "核对证据",
    owner: "林",
    status: "未开始",
    currentStep: "未开始",
    tone: "neutral",
    mark: "verify",
    todayOk: "—",
    todayFail: "—",
    todayAvg: "—",
    success: "—",
  },
  {
    id: "deliver",
    label: "交给 Owner",
    owner: "林",
    status: "要你授权发送摘要",
    currentStep: "停在画布预览 · 不是已发出",
    tone: "warn",
    mark: "auth",
    todayOk: "—",
    todayFail: "—",
    todayAvg: "—",
    success: "—",
  },
];

const SITE_STAGES: readonly ProcessStage[] = [
  {
    id: "collect",
    label: "收集现场记录",
    owner: "韩",
    status: "进行中 · 已 27 分钟",
    currentStep: "正在整理 3 号机房巡检照片说明",
    tone: "info",
    mark: "none",
    todayOk: "1（样品）",
    todayFail: "0（样品）",
    todayAvg: "22 分（样品）",
    success: "1/1（样品）",
  },
  {
    id: "compare",
    label: "对照标准项",
    owner: "方",
    status: "等待现场记录",
    currentStep: "未开始 · 缺可打开现场记录",
    tone: "neutral",
    mark: "none",
    todayOk: "—",
    todayFail: "—",
    todayAvg: "—",
    success: "—",
  },
  {
    id: "draft",
    label: "起草周检清单",
    owner: "齐",
    status: "未开始",
    currentStep: "未开始",
    tone: "neutral",
    mark: "none",
    todayOk: "—",
    todayFail: "—",
    todayAvg: "—",
    success: "—",
  },
  {
    id: "verify",
    label: "核对证据",
    owner: "方",
    status: "未开始",
    currentStep: "未开始",
    tone: "neutral",
    mark: "verify",
    todayOk: "—",
    todayFail: "—",
    todayAvg: "—",
    success: "—",
  },
  {
    id: "deliver",
    label: "交给 Owner",
    owner: "方",
    status: "要你核对整改包",
    currentStep: "停在交付包预览 · 不是已发出",
    tone: "warn",
    mark: "verify",
    todayOk: "—",
    todayFail: "—",
    todayAvg: "—",
    success: "—",
  },
];

const WEEKLY_OUTPUTS: readonly SampleOutput[] = [
  {
    id: "weekly-report",
    title: "本周经营周报",
    job: "给 Owner 一份可打开、可核对的经营周报",
    format: "document",
    accepted: true,
    needsHitl: false,
  },
  {
    id: "facts",
    title: "事实清单",
    job: "核对本周已收集事实是否齐、是否可引用",
    format: "checklist",
    accepted: true,
    needsHitl: false,
  },
  {
    id: "follow-note",
    title: "客户跟进正文",
    job: "跟进说明正文加配图样品，不是社交发帖",
    format: "article",
    accepted: true,
    needsHitl: false,
  },
  {
    id: "send-pack",
    title: "摘要发送包",
    job: "对外发送前的完整预览，要 Owner 在画布拍板",
    format: "packet",
    accepted: true,
    needsHitl: true,
  },
  {
    id: "share-link",
    title: "周报打开链接",
    job: "本机可打开的周报位置，不是已公开发布",
    format: "link",
    accepted: true,
    needsHitl: false,
  },
];

const SITE_OUTPUTS: readonly SampleOutput[] = [
  {
    id: "inspect-list",
    title: "本周周检清单",
    job: "对照标准项后的可核对现场周检清单",
    format: "checklist",
    accepted: true,
    needsHitl: false,
  },
  {
    id: "inspect-note",
    title: "现场记录说明",
    job: "正文加现场照片样品；视频只标未播放样品",
    format: "article",
    accepted: true,
    needsHitl: false,
  },
  {
    id: "inspect-pack",
    title: "整改交付包",
    job: "交给 Owner 的整改包。计划不是已发出。",
    format: "packet",
    accepted: true,
    needsHitl: true,
  },
];

const SAMPLE_PROJECTS: readonly SampleProject[] = [
  {
    id: "weekly",
    name: "周报与客户跟进",
    kind: "live",
    industry: "经营闭环",
    goal: "每周给自己一份可打开的经营周报，并跟进待回复客户。",
    cycle: "每周一 09:00，仅本机在线时运行",
    statusLine: "已上线 · 收集本周事实",
    costLine: "估计 ¥6.40（样品）· 另有未知项不写 0",
    blurb: "经营闭环样品。不是社交账号运营，没有 X 英雄圈，没有示范项目。",
    currentStageId: "collect",
    stages: PROCESS_STAGES,
    participants: ["林 · 经理", "梅 · 调研", "锐 · 撰稿"],
    outputs: WEEKLY_OUTPUTS,
    todayOk: "2（样品）",
    todayFail: "0（样品）",
    todayAvg: "18 分（样品）",
  },
  {
    id: "site",
    name: "设备现场周检",
    kind: "live",
    industry: "现场交付",
    goal: "完成本周机房现场周检，交出可核对清单与整改包。",
    cycle: "每周五 16:00，仅本机在线时运行",
    statusLine: "已上线 · 收集现场记录",
    costLine: "估计 ¥2.80（样品）· 另有未知项不写 0",
    blurb: "非内容行业样品：现场周检。不是内容日历，不是商店。",
    currentStageId: "collect",
    stages: SITE_STAGES,
    participants: ["方 · 经理", "韩 · 现场记录", "齐 · 核对"],
    outputs: SITE_OUTPUTS,
    todayOk: "1（样品）",
    todayFail: "0（样品）",
    todayAvg: "22 分（样品）",
  },
  {
    id: "weekly-copy",
    name: "周报与客户跟进（副本）",
    kind: "copy-draft",
    industry: "经营闭环",
    goal: "副本未激活，目标仍可改。",
    cycle: "未激活 · 不上日程",
    statusLine: "未激活草稿",
    costLine: "副本不带密钥、进行中任务、对外回执、本周不再问",
    blurb: "改完走总预览再上线。④⑤ 可抽检或跳过。不从 ① 重来。",
    currentStageId: "collect",
    stages: PROCESS_STAGES,
    participants: [],
    outputs: [],
    todayOk: "未激活，不是 0",
    todayFail: "未激活，不是 0",
    todayAvg: "未激活",
  },
  {
    id: "creating-draft",
    name: "创建中 · 周报与客户跟进",
    kind: "creating",
    industry: "经营闭环",
    goal: "创建未完成，目标未上线。",
    cycle: "草稿 · 未上日程",
    statusLine: "未完成的创建",
    costLine: "草稿费用未知，不写 0",
    blurb: "项目列表现在只露出这一份草稿。",
    currentStageId: "collect",
    stages: PROCESS_STAGES,
    participants: [],
    outputs: [],
    todayOk: "未上线，不是 0",
    todayFail: "未上线，不是 0",
    todayAvg: "未上线",
  },
];

const OUTPUT_FORMATS: readonly OutputFormat[] = ["document", "checklist", "packet", "article", "link"];

function projectById(id: SampleProjectId): SampleProject {
  return SAMPLE_PROJECTS.find((item) => item.id === id) ?? SAMPLE_PROJECTS[0];
}

function listedProjects(lifecycle: Lifecycle, copied: boolean): SampleProject[] {
  if (lifecycle === "empty") return [];
  if (lifecycle === "creating") return [projectById("creating-draft")];
  const live = [projectById("weekly"), projectById("site")];
  if (copied) live.push(projectById("weekly-copy"));
  return live;
}

function formatLabel(format: OutputFormat): string {
  if (format === "document") return "可打开文稿";
  if (format === "checklist") return "核对清单";
  if (format === "packet") return "交付包";
  if (format === "link") return "链接";
  return "正文与配图";
}

function nextOutputFormat(current: OutputFormat): OutputFormat {
  const index = OUTPUT_FORMATS.indexOf(current);
  return OUTPUT_FORMATS[(index + 1) % OUTPUT_FORMATS.length] ?? "document";
}

function formatFromAsk(text: string, current: OutputFormat): OutputFormat {
  if (/清单|核对/.test(text)) return "checklist";
  if (/交付包|预览|发送包|决策包/.test(text)) return "packet";
  if (/链接|打开位置/.test(text)) return "link";
  if (/配图|照片|视频|正文/.test(text)) return "article";
  if (/文稿|文档|周报|说明/.test(text)) return "document";
  return nextOutputFormat(current);
}

function isOutputFormat(value: string): value is OutputFormat {
  return (OUTPUT_FORMATS as readonly string[]).includes(value);
}

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
  loading: "读取中",
  empty: "空",
  working: "进行中",
  error: "出错",
  success: "成功",
  partial: "部分",
  blocked: "阻塞",
  unknown: "未知",
  offline: "离线",
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
    label: "今日",
    object: "决策包与上线项目运行概览",
    source: "项目运行投影",
    firstAction: "创建项目，或处理这一件拍板",
    native: "空首页只留创建；未完成创建只留继续；上线后是决策包+概览，不是四泳道墙",
  },
  create: {
    label: "五段创建",
    object: "草稿、确认清单、流程轴、班子、可打开测试结果",
    source: "可恢复的创建草稿",
    firstAction: "用业务语言描述，或去设置绑定助手",
    native: "①–⑤ 均为创建；⑤ 验收前没有日常今日",
  },
  projects: {
    label: "项目列表",
    object: "长期治理的工作空间",
    source: "项目列表投影",
    firstAction: "创建项目，或一键复制已上线项目为副本",
    native: "无默认/示范项目。副本未激活。子菜单是四个去向，不是项目名。",
  },
  members: {
    label: "成员管理",
    object: "成员清单、流程轴负责人和可编辑职责",
    source: "当前项目成员投影",
    firstAction: "先选人再看配置，或加人",
    native: "未选空态。身份在详情头。八标签：职责 / 输入 / 输出 / 技能 / 工具 / 工作说明 / 周期与触发 / 连接与权限。无安装。",
  },
  runs: {
    label: "运行管理",
    object: "业务流程轴、当前步骤和今日执行情况",
    source: "当前项目运行投影",
    firstAction: "看这一环正在做什么，或处理要你拍板的环节",
    native: "数字标样品。未知不写 0。进行中不是完成。未激活项目诚实空。",
  },
  outputs: {
    label: "产出管理",
    object: "助手按产出编排的可打开成果",
    source: "已验收或进行中的项目结果样品",
    firstAction: "打开一份成果，或请助手换一种展示",
    native: "无固定模板。一次一种编排。换展示要聊天确认。聊天不能批。",
  },
  hitl: {
    label: "画布拍板",
    object: "将做什么、完整预览、批准/改窄/拒绝",
    source: "daemon 签发预览的目标态样品",
    firstAction: "在画布上批准、改窄或拒绝",
    native: "过期预览不能批。执行中可停。聊天只有链接。",
  },
  knowledge: {
    label: "知识",
    object: "项目资料、为什么用这段、可检查的记忆",
    source: "本地资料与自动承认的对话记忆",
    firstAction: "导入资料，或检查/忘记一条记忆",
    native: "无项目时锁定。② 流程初始化后才为当前草稿打开。",
  },
  settings: {
    label: "设置",
    object: "模型连接、本周不再问收回、通知恢复",
    source: "本地设置投影",
    firstAction: "下拉选择主流 Provider 并交接密钥",
    native: "无账单、无引擎商店、无收件箱。密钥不回显。",
  },
};

const CREATE_SCENES: readonly Scene[] = [
  "create-init",
  "create-process",
  "create-members",
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

function isSetupChat(scene: Scene, memberConfigFromLive = false) {
  if (CREATE_SCENES.includes(scene) || scene === "add-member") return true;
  if (scene === "member-config") return !memberConfigFromLive;
  return false;
}

function isLiveProjectChat(scene: Scene, memberConfigFromLive = false) {
  if (scene === "member-config") return memberConfigFromLive;
  return (
    scene === "project-detail" ||
    scene === "project-members" ||
    scene === "project-runs" ||
    scene === "project-outputs" ||
    scene === "hitl"
  );
}

function sceneForCreateGate(gate: number): Scene {
  if (gate >= 5) return "create-joint";
  if (gate >= 4) return "create-test";
  if (gate >= 3) return "create-members";
  if (gate >= 2) return "create-process";
  return "create-init";
}

function suggestRevision(label: string, value: string) {
  const trimmed = value.trim();
  if (trimmed.length === 0) return `${label}需要可打开、可核对的说法，不能留空。`;
  if (trimmed.includes("可打开") && trimmed.includes("可核对")) return trimmed;
  return `${trimmed} 交付须可打开、可核对。`;
}

function defaultStageDrafts(): Record<string, StageDraft> {
  const drafts: Record<string, StageDraft> = {};
  for (const stage of [...PROCESS_STAGES, ...SITE_STAGES]) {
    if (drafts[stage.id]) continue;
    drafts[stage.id] = {
      input: "上一环产出或本项目知识库摘录。缺口留在轴上，不标已就绪。",
      method: "本环节怎么做、周期、触发。Skill / 工具 / MCP / 文件权限在此披露。",
      rights: "只能读本项目资料。对外发送不在这一环自动发生。",
    };
  }
  drafts.compare = {
    input: "可打开的现场记录。缺口留在对照表上，不标已就绪。",
    method: "对照标准项。Skill / 工具 / MCP / 文件权限在此披露。",
    rights: "只能读本项目现场资料。对外发送不在这一环自动发生。",
  };
  return drafts;
}

const TEST_NOTE_DEFAULT = "目标态样品：这一环的子产出可打开，核对标记为通过。";
const JOINT_NOTE_DEFAULT = "周报 Markdown 可打开。核对通过。不是对外发布。";
const SETUP_ASSISTANT_INTRO =
  "右侧对话是创建和改项目 / 成员 / 测试 / 联调的主入口。你在中间画布改完后回车，确认就会以你的名义出现在这里。我可以建议优化；我改画布也要你在对话里点头。聊天不能批准、验收、发布或安装。";

function isStageDraftKey(value: string): value is keyof StageDraft {
  return value === "input" || value === "method" || value === "rights";
}

const STAGE_OUTPUTS: Record<string, string> = {
  collect: "可打开的事实清单",
  analyze: "可核对的建议稿",
  compare: "对照结果与缺口清单",
  draft: "周报草稿",
  verify: "核对记录",
  deliver: "可打开周报与决策包",
};

function stageOutputLabel(project: SampleProject, stageId: string): string {
  if (project.id === "site") {
    if (stageId === "collect") return "可打开现场记录";
    if (stageId === "compare") return "对照结果与缺口清单";
    if (stageId === "draft") return "周检清单草稿";
    if (stageId === "verify") return "核对记录";
    if (stageId === "deliver") return "整改交付包";
  }
  return STAGE_OUTPUTS[stageId] ?? "待定产出";
}

function previousStage(
  stages: readonly ProcessStage[],
  stageId: string,
): ProcessStage | null {
  const index = stages.findIndex((item) => item.id === stageId);
  if (index <= 0) return null;
  return stages[index - 1] ?? null;
}

const OWNER_PROFILES: Record<string, { id: string; name: string }> = {
  梅: { id: "mei", name: "梅 · 调研" },
  林: { id: "lin", name: "林 · 经理" },
  锐: { id: "rui", name: "锐 · 撰稿" },
  韩: { id: "han", name: "韩 · 现场记录" },
  方: { id: "fang", name: "方 · 经理" },
  齐: { id: "qi", name: "齐 · 核对" },
};

function ownedStagesFor(
  project: SampleProject,
  member: MemberDraft | null,
): ProcessStage[] {
  if (!member) return [];
  return project.stages.filter((stage) => OWNER_PROFILES[stage.owner]?.id === member.id);
}

const RUNTIME_SLOT_DEFS: ReadonlyArray<{
  id: RuntimeSlotId;
  businessLabel: string;
  runtimeLabel: string;
}> = [
  { id: "prompt", businessLabel: "工作说明", runtimeLabel: "提示词" },
  { id: "tools", businessLabel: "工具", runtimeLabel: "工具" },
  { id: "skills", businessLabel: "技能", runtimeLabel: "能力包" },
  { id: "loop", businessLabel: "周期与触发", runtimeLabel: "能力说法一层后才出现 loop 一词" },
  { id: "mcp", businessLabel: "外部连接", runtimeLabel: "能力说法一层后才出现 MCP。精确授权，无安装" },
  { id: "files", businessLabel: "文档范围", runtimeLabel: "文档权限" },
];

function emptyRuntime(): RuntimeSlot[] {
  return RUNTIME_SLOT_DEFS.map((item) => ({
    id: item.id,
    businessLabel: item.businessLabel,
    runtimeLabel: item.runtimeLabel,
    value: "",
    status: "empty",
  }));
}

function sampleRuntime(memberId: string): RuntimeSlot[] {
  const byMember: Record<string, Record<RuntimeSlotId, string>> = {
    mei: {
      prompt: "只收集本周可核对事实。不发明数字。缺口留在清单上。",
      tools: "本项目资料检索与摘录。无假安装。",
      skills: "来源、版本、许可待审。安装不是授权。",
      loop: "每周一，上一环交出后开始。同类不重叠。",
      mcp: "本岗位不接外部 MCP。不静默联网。",
      files: "仅当前项目目录只读。扩权要再批。",
    },
    lin: {
      prompt: "计划、分派、核对。对外发送必须走画布预览。",
      tools: "任务分派与核对清单。无假安装。",
      skills: "经理模板待审版本。安装不是授权。",
      loop: "每环交出后观察；阻塞或要决策时找 Owner。",
      mcp: "建议只读检索本项目资料。确认此人时一并授权。无市场按钮。",
      files: "本项目资料读写。对外发出不在自动发生。",
    },
    rui: {
      prompt: "按事实清单起草周报。不补未给出的数字。",
      tools: "文档整理与草稿比对。无假安装。",
      skills: "撰稿模板待审版本。安装不是授权。",
      loop: "建议稿可打开后开始起草。不与核对重叠。",
      mcp: "本岗位不接外部 MCP。",
      files: "仅周报草稿目录。扩权要再批。",
    },
    han: {
      prompt: "只记录现场可见事实。不发明读数。缺口留在记录上。",
      tools: "本项目现场记录摘录。无假安装。",
      skills: "现场记录模板待审。安装不是授权。",
      loop: "每周五现场结束后开始整理。同类不重叠。",
      mcp: "本岗位不接外部 MCP。不静默联网。",
      files: "仅当前项目现场目录只读。扩权要再批。",
    },
    fang: {
      prompt: "对照标准项、分派核对、把整改包交给 Owner。对外发送走画布。",
      tools: "标准项对照与核对清单。无假安装。",
      skills: "经理模板待审版本。安装不是授权。",
      loop: "每环交出后观察；阻塞或要决策时找 Owner。",
      mcp: "建议只读检索本项目资料。确认此人时一并授权。无市场按钮。",
      files: "本项目资料读写。对外发出不在自动发生。",
    },
    qi: {
      prompt: "按标准项起草周检清单。不补未给出的读数。",
      tools: "清单整理与证据比对。无假安装。",
      skills: "核对模板待审版本。安装不是授权。",
      loop: "对照完成后开始起草。不与现场记录重叠。",
      mcp: "本岗位不接外部 MCP。",
      files: "仅周检清单目录。扩权要再批。",
    },
  };
  const sample = byMember[memberId] ?? byMember.mei;
  return RUNTIME_SLOT_DEFS.map((item) => ({
    id: item.id,
    businessLabel: item.businessLabel,
    runtimeLabel: item.runtimeLabel,
    value: sample[item.id],
    status: item.id === "mcp" && memberId === "lin" ? "needs-grant" : "ready",
  }));
}

function slotTone(status: SlotFill): Tone {
  if (status === "ready") return "good";
  if (status === "needs-grant") return "warn";
  if (status === "unknown") return "bad";
  if (status === "draft") return "info";
  return "neutral";
}

function slotStatusLabel(status: SlotFill): string {
  if (status === "ready") return "已就位";
  if (status === "needs-grant") return "待授权";
  if (status === "unknown") return "说不清";
  if (status === "draft") return "草稿";
  return "未生成";
}

function initStatusLabel(status: MemberInitStatus): string {
  if (status === "confirmed") return "已就位";
  if (status === "ready") return "待你确认";
  if (status === "generating") return "生成中";
  if (status === "partial") return "有缺口";
  if (status === "blocked") return "缺模型";
  if (status === "unknown") return "说不清";
  return "未初始化";
}

function initStatusTone(status: MemberInitStatus): Tone {
  if (status === "confirmed") return "good";
  if (status === "ready") return "info";
  if (status === "generating") return "info";
  if (status === "partial" || status === "blocked") return "warn";
  if (status === "unknown") return "bad";
  return "neutral";
}

function runtimeComplete(runtime: readonly RuntimeSlot[]): boolean {
  return (
    runtime.length === RUNTIME_SLOT_DEFS.length &&
    runtime.every((slot) => slot.status === "ready" || slot.status === "needs-grant")
  );
}

function memberSeated(member: MemberDraft | undefined): boolean {
  return Boolean(
    member &&
      member.model !== "unselected" &&
      member.initStatus === "confirmed" &&
      runtimeComplete(member.runtime),
  );
}

function seatedSampleMember(
  id: string,
  name: string,
  duty: string,
  handoff: string,
): MemberDraft {
  return {
    id,
    name,
    duty,
    handoff,
    model: "anthropic",
    joined: true,
    initStatus: "confirmed",
    runtime: sampleRuntime(id),
  };
}

const WEEKLY_SAMPLE_MEMBERS: readonly MemberDraft[] = [
  seatedSampleMember("lin", "林 · 经理", "计划、分派、核对。对外发送必须走画布预览。", "可打开周报与决策包"),
  seatedSampleMember("mei", "梅 · 调研", "收集本周可核对事实，交出事实清单。", "事实清单"),
  seatedSampleMember("rui", "锐 · 撰稿", "按事实清单起草周报，不补未给出的数字。", "周报草稿"),
];

const SITE_SAMPLE_MEMBERS: readonly MemberDraft[] = [
  seatedSampleMember("fang", "方 · 经理", "对照标准项并交出整改包。对外发送走画布。", "整改交付包"),
  seatedSampleMember("han", "韩 · 现场记录", "记录现场可见事实，交出可打开现场记录。", "现场记录"),
  seatedSampleMember("qi", "齐 · 核对", "按标准项起草周检清单。缺口不写成已就绪。", "周检清单"),
];

function membersForProject(
  project: SampleProject,
  wizardMembers: readonly MemberDraft[],
  weeklyLive: readonly MemberDraft[],
  siteLive: readonly MemberDraft[],
): MemberDraft[] {
  if (project.kind !== "live") return [];
  if (project.id === "weekly" && wizardMembers.length > 0) return [...wizardMembers];
  if (project.id === "weekly") return [...weeklyLive];
  if (project.id === "site") return [...siteLive];
  return [];
}

function memberForStage(stage: ProcessStage, members: readonly MemberDraft[]): MemberDraft | undefined {
  const profile = OWNER_PROFILES[stage.owner];
  return members.find((item) => item.id === (profile?.id ?? ""));
}

function isRuntimeSlotId(value: string): value is RuntimeSlotId {
  return RUNTIME_SLOT_DEFS.some((item) => item.id === value);
}

function nextInitAfterRuntimeEdit(status: MemberInitStatus): MemberInitStatus {
  if (status === "confirmed" || status === "ready") return "partial";
  return status;
}

function currentInitHeadline(
  member: MemberDraft,
  generatingIndex: number | null,
): { kicker: string; title: string; hint: string } {
  if (member.model === "unselected") {
    return { kicker: "先选模型", title: "还不能生成", hint: "未选模型不会静默绑定。" };
  }
  if (member.initStatus === "generating" && generatingIndex !== null) {
    const def = RUNTIME_SLOT_DEFS[generatingIndex] ?? RUNTIME_SLOT_DEFS[0];
    return { kicker: "正在生成…", title: def.businessLabel, hint: "完整条文在配置页。" };
  }
  if (member.initStatus === "confirmed") {
    return { kicker: "此人已就位", title: "已确认执行方式", hint: "完整条文在配置页。" };
  }
  const incomplete = member.runtime.find((slot) => slot.status !== "ready" && slot.status !== "needs-grant");
  if (incomplete) {
    return {
      kicker: member.initStatus === "partial" ? "有缺口" : "下一步",
      title: incomplete.businessLabel,
      hint: incomplete.runtimeLabel,
    };
  }
  const grant = member.runtime.find((slot) => slot.status === "needs-grant");
  if (grant) {
    return { kicker: "待授权", title: grant.businessLabel, hint: "确认此人时一并授权预览。无安装。" };
  }
  return { kicker: "待你确认", title: "确认此人已就位", hint: "完整条文在配置页。聊天不能批准。" };
}

function grantRuntimeOnConfirm(runtime: readonly RuntimeSlot[]): RuntimeSlot[] {
  return runtime.map((slot) =>
    slot.status === "needs-grant" ? { ...slot, status: "ready" } : slot,
  );
}

function proposeMembersFromProcess(drafts: Record<string, StageDraft>): MemberDraft[] {
  const grouped = new Map<string, ProcessStage[]>();
  for (const stage of PROCESS_STAGES) {
    const list = grouped.get(stage.owner) ?? [];
    list.push(stage);
    grouped.set(stage.owner, list);
  }
  return [...grouped.entries()].map(([owner, stages]) => {
    const profile = OWNER_PROFILES[owner] ?? { id: owner, name: owner };
    const duty = stages
      .map((stage) => {
        const draft = drafts[stage.id];
        const method = draft?.method.trim() || "按这一环执行。";
        const input = draft?.input.trim() || "上一环产出或本项目资料。";
        return `负责「${stage.label}」。输入：${input} 做法：${method}`;
      })
      .join(" ");
    const handoff = stages.map((stage) => STAGE_OUTPUTS[stage.id] ?? `${stage.label}产出`).join("；");
    return {
      id: profile.id,
      name: profile.name,
      duty,
      handoff,
      model: "unselected",
      joined: false,
      initStatus: "idle",
      runtime: emptyRuntime(),
    };
  });
}

function rosterAssistantText(members: readonly MemberDraft[]): string {
  const lines = members.map((member) => `${member.name}：${member.duty} → 交出 ${member.handoff}`);
  return `已根据流程轴每个环节的输入与产出配置岗位。\n${lines.join("\n")}\n请选定模型，再逐人生成执行方式。未选模型不静默绑定。聊天不能批准。`;
}

function stateMessage(surface: SurfaceKey, state: StateKey) {
  const context = SURFACE_CONTEXT[surface];
  const messages: Record<StateKey, string> = {
    loading: `${context.label} 正在读取 ${context.object}… 上次安全投影仍可见；离开不会丢掉草稿。`,
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
      title={environment ? "缺后端 + 缺合格环境" : "缺后端"}
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
  children: NonNullable<Parameters<typeof Callout>[0]["children"]>;
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

function ProjectSwitcher({
  projects,
  currentId,
  onChange,
}: {
  projects: readonly SampleProject[];
  currentId: SampleProjectId;
  onChange: (id: SampleProjectId) => void;
}) {
  const options = projects.filter((item) => item.kind === "live" || item.kind === "copy-draft");
  if (options.length === 0) return null;
  return (
    <label className="project-switcher">
      <span>当前项目</span>
      <Select
        value={currentId}
        onChange={(next) => onChange(next as SampleProjectId)}
        options={options.map((item) => ({
          value: item.id,
          label: item.name,
        }))}
      />
    </label>
  );
}

function EditConfirmDialog({
  pending,
  onConfirm,
  onCancel,
}: {
  pending: PendingCommit;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  useEffect(() => {
    const onKey = (event: { key: string }) => {
      if (event.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);
  return (
    <div className="edit-dialog-scrim">
      <div className="edit-dialog" role="dialog" aria-modal="true" aria-labelledby="edit-dialog-title" aria-describedby="edit-dialog-copy">
        <h3 id="edit-dialog-title">确认把这项改动告诉助手？</h3>
        <p id="edit-dialog-copy">确认后会以你的名义出现在右侧对话。助手可以建议优化；它改画布也要你在对话里再点头。取消则还原画布。</p>
        <dl className="definition-list">
          <div>
            <dt>字段</dt>
            <dd>{pending.label}</dd>
          </div>
          <div>
            <dt>改为</dt>
            <dd>{pending.next}</dd>
          </div>
        </dl>
        <div className="flow-actions">
          <button className="primary-button" type="button" onClick={onConfirm}>
            确认并告知助手
          </button>
          <button className="secondary-button" type="button" onClick={onCancel}>
            取消并还原
          </button>
        </div>
      </div>
    </div>
  );
}

function CreateRosterDialog({
  replacing,
  onConfirm,
  onCancel,
}: {
  replacing: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  useEffect(() => {
    const onKey = (event: { key: string }) => {
      if (event.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);
  return (
    <div className="edit-dialog-scrim">
      <div className="edit-dialog" role="dialog" aria-modal="true" aria-labelledby="roster-dialog-title" aria-describedby="roster-dialog-copy">
        <h3 id="roster-dialog-title">{replacing ? "按当前流程重做岗位？" : "根据业务流程创建成员？"}</h3>
        <p id="roster-dialog-copy">
          确认后会以你的名义在右侧发送「根据业务流程创建成员」。助手按每环的输入与产出配置岗位，并写到画布。每人仍要你选定模型，不会静默绑定。
        </p>
        <div className="flow-actions">
          <button className="primary-button" type="button" onClick={onConfirm}>
            确认并告知助手
          </button>
          <button className="secondary-button" type="button" onClick={onCancel}>
            取消
          </button>
        </div>
      </div>
    </div>
  );
}

function GenerateRuntimeDialog({
  memberName,
  onConfirm,
  onCancel,
}: {
  memberName: string;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  useEffect(() => {
    const onKey = (event: { key: string }) => {
      if (event.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);
  return (
    <div className="edit-dialog-scrim">
      <div className="edit-dialog" role="dialog" aria-modal="true" aria-labelledby="runtime-dialog-title" aria-describedby="runtime-dialog-copy">
        <h3 id="runtime-dialog-title">为「{memberName}」生成执行方式？</h3>
        <p id="runtime-dialog-copy">
          确认后会以你的名义请助手生成工作说明、工具、能力包、周期与触发、外部连接和文档范围。写入仍要你确认此人已就位。没有安装按钮，聊天不能批准。
        </p>
        <div className="flow-actions">
          <button className="primary-button" type="button" onClick={onConfirm}>
            确认并告知助手
          </button>
          <button className="secondary-button" type="button" onClick={onCancel}>
            取消
          </button>
        </div>
      </div>
    </div>
  );
}

function SyncedField({
  field,
  label,
  value,
  onChange,
  onCommit,
  onFocusField,
  multiline = false,
  rows = 3,
  placeholder,
  disabled = false,
}: {
  field: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  onCommit: (field: string, label: string, next: string) => void;
  onFocusField: (field: string, label: string, value: string) => void;
  multiline?: boolean;
  rows?: number;
  placeholder?: string;
  disabled?: boolean;
}) {
  const snapshot = useRef(value);
  const onKeyDown = (event: { key: string; shiftKey: boolean; preventDefault: () => void }) => {
    if (event.key !== "Enter") return;
    if (multiline && event.shiftKey) return;
    event.preventDefault();
    onCommit(field, label, value);
  };
  const shared = {
    name: field,
    autoComplete: "off" as const,
    value,
    disabled,
    placeholder,
    onChange: (event: { currentTarget: { value: string } }) => onChange(event.currentTarget.value),
    onFocus: () => {
      snapshot.current = value;
      onFocusField(field, label, value);
    },
    onKeyDown,
  };
  return multiline ? (
    <textarea
      {...shared}
      rows={rows}
      aria-label={`${label}。回车告知助手，Shift+回车换行。`}
    />
  ) : (
    <input
      {...shared}
      type="text"
      aria-label={`${label}。回车告知助手。`}
    />
  );
}

function Segmented<T extends string>({
  label,
  value,
  items,
  onChange,
  tabs = false,
}: {
  label: string;
  value: T;
  items: ReadonlyArray<{ id: T; label: string }>;
  onChange: (value: T) => void;
  tabs?: boolean;
}) {
  const index = items.findIndex((item) => item.id === value);
  return (
    <div
      className="segmented"
      role={tabs ? "tablist" : "group"}
      aria-label={label}
      onKeyDown={
        tabs
          ? (event: { key: string; preventDefault: () => void }) => {
              if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") return;
              event.preventDefault();
              const step = event.key === "ArrowRight" ? 1 : -1;
              const next = items[(index + step + items.length) % items.length];
              if (next) onChange(next.id);
            }
          : undefined
      }
    >
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          role={tabs ? "tab" : undefined}
          id={tabs ? `tab-${item.id}` : undefined}
          aria-selected={tabs ? value === item.id : undefined}
          aria-pressed={tabs ? undefined : value === item.id}
          tabIndex={tabs ? (value === item.id ? 0 : -1) : undefined}
          onClick={() => onChange(item.id)}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}

function MemberModelSelect({
  member,
  setModel,
}: {
  member: MemberDraft;
  setModel: (id: string, model: string) => void;
}) {
  return (
    <label className="field">
      <span>
        模型
        <small> · 未选不会静默绑定。换模型会让执行方式回到待确认。</small>
      </span>
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
    </label>
  );
}

function MemberConfigPanel({
  member,
  project,
  stageDrafts,
  tab,
  setTab,
  setModel,
  setMemberText,
  setSlotValue,
  onCommitField,
  onFocusField,
}: {
  member: MemberDraft;
  project: SampleProject | null;
  stageDrafts: Record<string, StageDraft>;
  tab: MemberConfigTab;
  setTab: (value: MemberConfigTab) => void;
  setModel: (id: string, model: string) => void;
  setMemberText: (id: string, key: "duty" | "handoff", value: string) => void;
  setSlotValue: (id: string, slotId: RuntimeSlotId, value: string) => void;
  onCommitField: (field: string, label: string, next: string) => void;
  onFocusField: (field: string, label: string, value: string) => void;
}) {
  const ownedStages = project ? ownedStagesFor(project, member) : [];
  const singleSlotId =
    tab === "prompt"
      ? "prompt"
      : tab === "tools"
        ? "tools"
        : tab === "skills"
          ? "skills"
          : tab === "loop"
            ? "loop"
            : null;
  const singleSlot = singleSlotId
    ? member.runtime.find((slot) => slot.id === singleSlotId)
    : undefined;
  const mcpSlot = member.runtime.find((slot) => slot.id === "mcp");
  const filesSlot = member.runtime.find((slot) => slot.id === "files");
  return (
    <section className="member-detail" aria-label={`${member.name} · 配置标签页`}>
      <header className="member-detail-head">
        <div>
          <h3>{member.name}</h3>
          <p className="member-detail-meta">
            {member.joined ? "已加入本项目。" : "未加入。拒加入不会写成已加入。"}
            {ownedStages.length > 0
              ? ` 负责 ${ownedStages.map((stage) => stage.label).join("、")}。`
              : " 不在流程轴上。加人后还没指派环节。"}
          </p>
          <MemberModelSelect member={member} setModel={setModel} />
        </div>
        <div className="packet-marks">
          <Tag tone={initStatusTone(member.initStatus)}>{initStatusLabel(member.initStatus)}</Tag>
          {member.model === "unselected" ? <Tag tone="warn">模型待选</Tag> : null}
        </div>
      </header>
      <Segmented
        label={`${member.name} 配置标签页`}
        value={tab}
        items={MEMBER_CONFIG_TABS}
        onChange={setTab}
        tabs
      />
      <div
        className="member-tab-panel"
        role="tabpanel"
        aria-labelledby={`tab-${tab}`}
      >
        {tab === "duty" ? (
          <label className="field">
            <span>
              职责
              <small> · 做什么。改完回车 → 确认框 → 对话确认才写回画布。</small>
            </span>
            <SyncedField
              field={`member:${member.id}:duty`}
              label={`${member.name} · 职责`}
              value={member.duty}
              onChange={(next) => setMemberText(member.id, "duty", next)}
              onCommit={onCommitField}
              onFocusField={onFocusField}
              multiline
              rows={4}
            />
          </label>
        ) : null}
        {tab === "input" ? (
          ownedStages.length === 0 ? (
            <Notice title="还没有指派环节" tone="info">
              输入来自流程轴上一环的交出物，或本项目资料。加人后还没指派环节，不编造输入。
            </Notice>
          ) : (
            <div className="io-stack">
              <p className="sample-caption">
                输入是流程合同，不是这个人私有的另一份字段。改输入走流程轴修订，需要 daemon。这里只检查他吃什么。
              </p>
              {ownedStages.map((stage) => {
                const prev = project ? previousStage(project.stages, stage.id) : null;
                const draft = stageDrafts[stage.id];
                return (
                  <article key={stage.id} className="io-block">
                    <h4>{stage.label}</h4>
                    <dl className="definition-list">
                      <div>
                        <dt>来自</dt>
                        <dd>
                          {prev
                            ? `${prev.label} · ${project ? stageOutputLabel(project, prev.id) : "上一环产出"}`
                            : "本项目资料 / Owner 导入。没有上一环。"}
                        </dd>
                      </div>
                      <div>
                        <dt>本环输入</dt>
                        <dd>{draft?.input.trim() || "上一环产出或本项目资料。缺口留在轴上。"}</dd>
                      </div>
                    </dl>
                  </article>
                );
              })}
            </div>
          )
        ) : null}
        {tab === "output" ? (
          <>
            <label className="field">
              <span>
                交出什么
                <small> · 这个人的输出合同。改完回车 → 确认框 → 对话确认。</small>
              </span>
              <SyncedField
                field={`member:${member.id}:handoff`}
                label={`${member.name} · 交出什么`}
                value={member.handoff}
                onChange={(next) => setMemberText(member.id, "handoff", next)}
                onCommit={onCommitField}
                onFocusField={onFocusField}
                multiline
                rows={3}
              />
            </label>
            {ownedStages.length > 0 && project ? (
              <div className="io-stack">
                <p className="sample-caption">各环产出合同。执行进度在运行管理，不在这里改成已完成。</p>
                {ownedStages.map((stage) => (
                  <article key={stage.id} className="io-block">
                    <h4>{stage.label}</h4>
                    <p>{stageOutputLabel(project, stage.id)}</p>
                  </article>
                ))}
              </div>
            ) : null}
          </>
        ) : null}
        {singleSlot ? (
          <label className="field">
            <span>
              {singleSlot.businessLabel}
              <small>
                {" "}
                · {slotStatusLabel(singleSlot.status)}
                。改完回车 → 确认框 → 对话确认。无安装按钮。
              </small>
              <details className="why-layer">
                <summary>能力说法</summary>
                <p>{singleSlot.runtimeLabel}</p>
              </details>
            </span>
            <SyncedField
              field={`runtime:${member.id}:${singleSlot.id}`}
              label={`${member.name} · ${singleSlot.businessLabel}`}
              value={singleSlot.value}
              onChange={(next) => setSlotValue(member.id, singleSlot.id, next)}
              onCommit={onCommitField}
              onFocusField={onFocusField}
              multiline
              rows={4}
            />
          </label>
        ) : null}
        {tab === "perms" && mcpSlot && filesSlot ? (
          <>
            <label className="field">
              <span>
                {mcpSlot.businessLabel}
                <small>
                  {" "}
                  · {slotStatusLabel(mcpSlot.status)}
                  。精确授权另批。无市场安装。
                </small>
                <details className="why-layer">
                  <summary>能力说法</summary>
                  <p>{mcpSlot.runtimeLabel}</p>
                </details>
              </span>
              <SyncedField
                field={`runtime:${member.id}:mcp`}
                label={`${member.name} · ${mcpSlot.businessLabel}`}
                value={mcpSlot.value}
                onChange={(next) => setSlotValue(member.id, "mcp", next)}
                onCommit={onCommitField}
                onFocusField={onFocusField}
                multiline
                rows={3}
              />
            </label>
            <label className="field">
              <span>
                {filesSlot.businessLabel}
                <small>
                  {" "}
                  · {slotStatusLabel(filesSlot.status)}
                  。扩权要再批。
                </small>
                <details className="why-layer">
                  <summary>能力说法</summary>
                  <p>{filesSlot.runtimeLabel}</p>
                </details>
              </span>
              <SyncedField
                field={`runtime:${member.id}:files`}
                label={`${member.name} · ${filesSlot.businessLabel}`}
                value={filesSlot.value}
                onChange={(next) => setSlotValue(member.id, "files", next)}
                onCommit={onCommitField}
                onFocusField={onFocusField}
                multiline
                rows={3}
              />
            </label>
          </>
        ) : null}
      </div>
    </section>
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
        <Tag tone="info">还没有项目</Tag>
        <h2>还没有项目</h2>
        <p>用业务语言办一件长期的事。知识此时锁定。对话在创建页打开。</p>
        <button className="primary-button" type="button" onClick={onCreate}>
          创建项目
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
  onProcess,
  goSettings,
  onCommitField,
  onFocusField,
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
  onProcess: () => void;
  goSettings: () => void;
  onCommitField: (field: string, label: string, next: string) => void;
  onFocusField: (field: string, label: string, value: string) => void;
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
            右侧助手是主入口。画布可改：回车弹出确认，确认后以你的名义写入对话。换行用 Shift+回车。确认本项后才能下一项。
          </p>
          {briefReady ? (
            <Notice title="原型未跑调研" tone="info">
              下列是本地目标态样品 · Requires-backend。不要看成已经调研完。
            </Notice>
          ) : null}
        </div>
        <div className="header-actions">
          <button className="secondary-button" type="button" onClick={onLeaveDraft}>
            离开并保留草稿
          </button>
        </div>
      </section>
      {!providerBound ? (
        <Notice title="还没有绑定助手" tone="warn">
          右侧对话只会请你去设置连接模型。不会静默绑定，也不会在聊天里要密钥。
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
                      <SyncedField
                        field={`wizard:${item.id}`}
                        label={item.label}
                        value={itemValue}
                        onChange={(next) => onEditValue(item.id, next)}
                        onCommit={onCommitField}
                        onFocusField={onFocusField}
                        multiline
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
          ) : last && confirmed ? null : (
            <button
              className="primary-button"
              type="button"
              disabled={!canConfirm}
              onClick={confirmCurrent}
            >
              确认本项
            </button>
          )}
          {last && allConfirmed ? (
            <button
              className="primary-button"
              type="button"
              disabled={!providerBound}
              onClick={onProcess}
            >
              总预览后进入 ② 流程初始化
            </button>
          ) : last ? (
            <span className="flow-end">先确认总预览。项目仍未上线。</span>
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
  onTest,
  onRequestCreate,
  onRequestGenerate,
  onConfirmMember,
  onRefuseMember,
  onBackToProcess,
  activeMemberId,
  setActiveMemberId,
  onViewConfig,
  generatingSlotIndex,
}: {
  members: readonly MemberDraft[];
  setModel: (id: string, model: string) => void;
  confirmRoster: () => void;
  onTest: () => void;
  onRequestCreate: () => void;
  onRequestGenerate: (id: string) => void;
  onConfirmMember: (id: string) => void;
  onRefuseMember: (id: string) => void;
  onBackToProcess: () => void;
  activeMemberId: string | null;
  setActiveMemberId: (id: string) => void;
  onViewConfig: (id: string) => void;
  generatingSlotIndex: number | null;
}) {
  const seatedCount = members.filter((member) => memberSeated(member)).length;
  const allSeated = members.length > 0 && seatedCount === members.length;
  const active =
    members.find((member) => member.id === activeMemberId) ?? members[0] ?? null;
  const activeIndex = active ? members.findIndex((member) => member.id === active.id) : -1;
  const priorConfirmed =
    activeIndex <= 0 || members.slice(0, activeIndex).every((member) => memberSeated(member));
  const canGenerate = Boolean(active && active.model !== "unselected" && priorConfirmed);
  const canConfirmPerson = Boolean(
    active && active.model !== "unselected" && runtimeComplete(active.runtime) && active.initStatus !== "confirmed",
  );
  const readyCount = active ? active.runtime.filter((slot) => slot.status === "ready" || slot.status === "needs-grant").length : 0;
  const headline = active ? currentInitHeadline(active, generatingSlotIndex) : null;
  const slotTotal = RUNTIME_SLOT_DEFS.length;
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>③ 创建岗位，再逐人就位</h2>
          <p>
            先按流程创建班子并选定模型，再一个人一个人生成执行方式。进度可见。业务说法在前；提示词 / Skill / MCP 是第二层。无安装按钮。
          </p>
        </div>
      </section>
      <section className="work-surface">
        <Heading title="按已确认流程建班子" meta="改环节回 ②。完整配方在配置页。" />
        <div className="flow-actions">
          <button className="text-button" type="button" onClick={onBackToProcess}>
            回 ② 改流程
          </button>
          <button
            className={members.length > 0 ? "secondary-button" : "primary-button"}
            type="button"
            onClick={onRequestCreate}
          >
            {members.length > 0 ? "按当前流程重做岗位" : "创建成员"}
          </button>
        </div>
      </section>
      <section className="work-surface">
        <Heading
          title="岗位名单"
          meta={
            members.length === 0
              ? "还没有岗位。"
              : `就位 ${seatedCount} / ${members.length}。没选模型 = 待定。`
          }
        />
        {members.length === 0 ? (
          <Notice title="还没有岗位" tone="info">
            确认「创建成员」后出现名单。然后选定模型，再逐人初始化。
          </Notice>
        ) : (
          <>
            <div className="init-progress" aria-live="polite">
              <strong>
                初始化进度 {seatedCount} / {members.length}
              </strong>
              <progress
                max={members.length}
                value={seatedCount}
                aria-label={`已就位 ${seatedCount} / ${members.length}`}
              />
              <span>{allSeated ? "全员已就位。" : "按顺序确认当前人，才能下一位。"}</span>
            </div>
            <div className="staff-table-wrap" tabIndex={0} aria-label="岗位名单">
              <table>
                <caption>按顺序就位。职责和交出在当前人或配置页改。</caption>
                <thead>
                  <tr>
                    <th scope="col">岗位</th>
                    <th scope="col">模型</th>
                    <th scope="col">就位</th>
                  </tr>
                </thead>
                <tbody>
                  {members.map((member, index) => {
                    const reachable =
                      index === 0 || members.slice(0, index).every((prior) => memberSeated(prior));
                    return (
                      <tr key={member.id}>
                        <th scope="row">
                          <button
                            className="text-button"
                            type="button"
                            disabled={!reachable}
                            onClick={() => {
                              if (reachable) setActiveMemberId(member.id);
                            }}
                          >
                            {index + 1}. {member.name}
                          </button>
                        </th>
                        <td>
                          {member.id === active?.id ? (
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
                          ) : (
                            <span>{member.model === "unselected" ? "未选" : "已选"}</span>
                          )}
                        </td>
                        <td>
                          <Tag tone={initStatusTone(member.initStatus)}>{initStatusLabel(member.initStatus)}</Tag>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </>
        )}
      </section>
      {active && headline ? (
        <section className="work-surface">
          <Heading
            title={`当前初始化 · ${activeIndex + 1} / ${members.length} · ${active.name}`}
            meta="一次只开一个人。这里只显示当前项。全文在配置页。"
          />
          <div className="wizard-dots" role="tablist" aria-label="成员初始化顺序">
            {members.map((member, index) => {
              const reachable =
                index === 0 || members.slice(0, index).every((prior) => memberSeated(prior));
              return (
                <button
                  key={member.id}
                  type="button"
                  role="tab"
                  className="wizard-dot"
                  aria-selected={member.id === active.id}
                  aria-label={`${member.name} · ${initStatusLabel(member.initStatus)}`}
                  disabled={!reachable || active.initStatus === "generating"}
                  onClick={() => setActiveMemberId(member.id)}
                />
              );
            })}
          </div>
          {active.model === "unselected" ? (
            <Notice title="先选模型" tone="warn">
              未选模型不能生成执行方式，也不会静默绑定。
            </Notice>
          ) : !priorConfirmed ? (
            <Notice title="先完成上一位" tone="warn">
              请按顺序确认上一位已就位，再初始化这一位。
            </Notice>
          ) : null}
          <div className="init-now" aria-live="polite">
            <div className="init-progress">
              <strong>
                本项 {readyCount} / {slotTotal}
              </strong>
              <progress
                max={slotTotal}
                value={Math.min(readyCount, slotTotal)}
                aria-label={`执行方式 ${readyCount} / ${slotTotal}。当前：${headline.title}`}
              />
            </div>
            <p className="init-kicker">{headline.kicker}</p>
            <p className="init-current-title">{headline.title}</p>
            <p className="init-hint">{headline.hint}</p>
          </div>
          <div className="flow-actions">
            <button
              className="primary-button"
              type="button"
              disabled={!canGenerate || active.initStatus === "confirmed" || active.initStatus === "generating"}
              onClick={() => onRequestGenerate(active.id)}
            >
              请助手生成本人执行方式
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onViewConfig(active.id)}
            >
              查看完整配置
            </button>
            <button
              className="secondary-button"
              type="button"
              disabled={!canConfirmPerson || active.initStatus === "generating"}
              onClick={() => onConfirmMember(active.id)}
            >
              确认此人已就位
            </button>
            <button
              className="secondary-button"
              type="button"
              disabled={active.initStatus === "generating" || memberSeated(active)}
              onClick={() => onRefuseMember(active.id)}
            >
              拒绝加入
            </button>
            <button
              className="secondary-button"
              type="button"
              disabled={
                activeIndex < 0 ||
                activeIndex >= members.length - 1 ||
                !memberSeated(active) ||
                active.initStatus === "generating"
              }
              onClick={() => {
                const next = members[activeIndex + 1];
                if (next) setActiveMemberId(next.id);
              }}
            >
              下一位
            </button>
          </div>
        </section>
      ) : null}
      <section className="work-surface">
        <div className="flow-actions">
          <button
            className="primary-button"
            type="button"
            disabled={!allSeated}
            onClick={() => {
              confirmRoster();
              onTest();
            }}
          >
            全员就位，进入测试
          </button>
          <span className="flow-end">
            {members.length === 0
              ? "先创建成员。"
              : allSeated
                ? "可以进入分环节测试。"
                : "每人要选模型并确认执行方式。"}
          </span>
        </div>
      </section>
      <Gap>成员 Runtime 写入、Skill/MCP 授权和权限落地需要 daemon。无静默扩权。这里只改变本地原型状态。</Gap>
    </div>
  );
}

function MemberConfigScene({
  member,
  project,
  stageDrafts,
  fromLive,
  onBack,
  setModel,
  setMemberText,
  setSlotValue,
  onCommitField,
  onFocusField,
}: {
  member: MemberDraft | null;
  project: SampleProject | null;
  stageDrafts: Record<string, StageDraft>;
  fromLive: boolean;
  onBack: () => void;
  setModel: (id: string, model: string) => void;
  setMemberText: (id: string, key: "duty" | "handoff", value: string) => void;
  setSlotValue: (id: string, slotId: RuntimeSlotId, value: string) => void;
  onCommitField: (field: string, label: string, next: string) => void;
  onFocusField: (field: string, label: string, value: string) => void;
}) {
  const [tab, setTab] = useState<MemberConfigTab>("duty");
  if (!member) {
    return (
      <div className="scene-stack">
        <section className="work-surface">
          <Notice title="还没有这个岗位" tone="info">
            先创建成员。维护时从项目画布打开配置。
          </Notice>
          <button className="secondary-button" type="button" onClick={onBack}>
            返回
          </button>
        </section>
      </div>
    );
  }
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>{member.name} · 执行方式</h2>
          <p>
            {fromLive ? "上线后维护同一份配置。" : "创建时与上线后看同一页。"}用标签检查职责、输入、输出、技能、工具。回车同步到对话。无安装按钮。
          </p>
        </div>
        <div className="header-actions">
          <Tag tone={initStatusTone(member.initStatus)}>{initStatusLabel(member.initStatus)}</Tag>
          <button className="secondary-button" type="button" onClick={onBack}>
            {fromLive ? "返回成员管理" : "返回成员初始化"}
          </button>
        </div>
      </section>
      <MemberConfigPanel
        member={member}
        project={project}
        stageDrafts={stageDrafts}
        tab={tab}
        setTab={setTab}
        setModel={setModel}
        setMemberText={setMemberText}
        setSlotValue={setSlotValue}
        onCommitField={onCommitField}
        onFocusField={onFocusField}
      />
      <Gap>真实 Skill/MCP 授权、文件权限和提示词版本需要 daemon 预览。这里只演示可检查的配置页。</Gap>
    </div>
  );
}

function CreateProcessScene({
  stageId,
  setStageId,
  confirmedStages,
  confirmStage,
  onMembers,
  stageDrafts,
  setStageDraft,
  onCommitField,
  onFocusField,
}: {
  stageId: string;
  setStageId: (id: string) => void;
  confirmedStages: readonly string[];
  confirmStage: (id: string) => void;
  onMembers: () => void;
  stageDrafts: Record<string, StageDraft>;
  setStageDraft: (id: string, key: keyof StageDraft, value: string) => void;
  onCommitField: (field: string, label: string, next: string) => void;
  onFocusField: (field: string, label: string, value: string) => void;
}) {
  const stage = PROCESS_STAGES.find((item) => item.id === stageId) ?? PROCESS_STAGES[0];
  const draft = stageDrafts[stage.id] ?? defaultStageDrafts()[stage.id];
  const last = stageId === PROCESS_STAGES[PROCESS_STAGES.length - 1].id;
  const [gappedStages, setGappedStages] = useState<readonly string[]>([]);
  const [goalReady, setGoalReady] = useState(false);
  const resolved = (id: string) => confirmedStages.includes(id) || gappedStages.includes(id);
  const allResolved = PROCESS_STAGES.every((item) => resolved(item.id));
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>② 一条流程轴，一次只开一环</h2>
          <p>轴头：总目标每周可打开的经营周报 · 总周期周一。轴确认后再按流程创建成员。意向岗位用岗位名。改输入/执行方式/权限后回车，以你的名义告诉助手。</p>
        </div>
      </section>
      <div className="process-axis" role="list">
        {PROCESS_STAGES.map((item, index) => {
          const reachable =
            index === 0 || PROCESS_STAGES.slice(0, index).every((prior) => resolved(prior.id));
          const gap = gappedStages.includes(item.id);
          const done = confirmedStages.includes(item.id);
          return (
            <button
              key={item.id}
              type="button"
              role="listitem"
              aria-current={item.id === stageId ? "step" : undefined}
              disabled={!reachable}
              onClick={() => {
                if (reachable) setStageId(item.id);
              }}
            >
              <strong>{item.label}</strong>
              <small>{gap ? "留缺口" : done ? "已确认这一环" : reachable ? "待确认" : "先完成前环"}</small>
            </button>
          );
        })}
      </div>
      <section className="work-surface">
        <Heading title={`这一环 · ${stage.label}`} meta={`意向岗位：${stage.owner}。③ 会按此环输入产出配岗。回车告知助手，Shift+回车换行。`} />
        <label className="field">
          <span>输入</span>
          <SyncedField
            field={`process:${stage.id}:input`}
            label={`${stage.label} · 输入`}
            value={draft.input}
            onChange={(next) => setStageDraft(stage.id, "input", next)}
            onCommit={onCommitField}
            onFocusField={onFocusField}
            multiline
            rows={3}
          />
        </label>
        <label className="field">
          <span>执行方式</span>
          <SyncedField
            field={`process:${stage.id}:method`}
            label={`${stage.label} · 执行方式`}
            value={draft.method}
            onChange={(next) => setStageDraft(stage.id, "method", next)}
            onCommit={onCommitField}
            onFocusField={onFocusField}
            multiline
            rows={3}
          />
        </label>
        <label className="field">
          <span>权限后果</span>
          <SyncedField
            field={`process:${stage.id}:rights`}
            label={`${stage.label} · 权限后果`}
            value={draft.rights}
            onChange={(next) => setStageDraft(stage.id, "rights", next)}
            onCommit={onCommitField}
            onFocusField={onFocusField}
            multiline
            rows={3}
          />
        </label>
        <div className="flow-actions">
          <button
            className="primary-button"
            type="button"
            disabled={gappedStages.includes(stage.id)}
            onClick={() => {
              confirmStage(stage.id);
              if (gappedStages.includes(stage.id)) {
                setGappedStages(gappedStages.filter((id) => id !== stage.id));
              }
              if (!last) {
                const index = PROCESS_STAGES.findIndex((item) => item.id === stage.id);
                const next = PROCESS_STAGES[index + 1];
                if (next) setStageId(next.id);
              }
            }}
          >
            确认这一环
          </button>
          <button
            className="secondary-button"
            type="button"
            onClick={() => {
              if (!gappedStages.includes(stage.id)) setGappedStages([...gappedStages, stage.id]);
              if (!last) {
                const index = PROCESS_STAGES.findIndex((item) => item.id === stage.id);
                const next = PROCESS_STAGES[index + 1];
                if (next) setStageId(next.id);
              }
            }}
          >
            本环留缺口
          </button>
          {last && allResolved && !goalReady ? (
            <button className="primary-button" type="button" onClick={() => setGoalReady(true)}>
              确认总目标与项目触发
            </button>
          ) : null}
          {last && goalReady ? (
            <button className="primary-button" type="button" onClick={onMembers}>
              进入 ③
            </button>
          ) : null}
          <span className="flow-end">
            {goalReady
              ? "总目标已确认。下一步才进入成员初始化。"
              : allResolved
                ? "各环已处理。先确认总目标与项目触发，不要同一点击跳进 ③。"
                : "按顺序确认。缺口不能标已确认。"}
          </span>
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
  onBackToProcess,
  testNote,
  setTestNote,
  members,
  onInitMember,
  onViewConfig,
  onCommitField,
  onFocusField,
  hostState = "working",
}: {
  stageId: string;
  setStageId: (id: string) => void;
  testState: "idle" | "running" | "pass" | "fail" | "unknown";
  setTestState: (value: "idle" | "running" | "pass" | "fail" | "unknown") => void;
  onJoint: () => void;
  onBackToProcess: () => void;
  testNote: string;
  setTestNote: (value: string) => void;
  members: readonly MemberDraft[];
  onInitMember: (id: string) => void;
  onViewConfig: (id: string) => void;
  onCommitField: (field: string, label: string, next: string) => void;
  onFocusField: (field: string, label: string, value: string) => void;
  hostState?: StateKey;
}) {
  const stage = PROCESS_STAGES.find((item) => item.id === stageId) ?? PROCESS_STAGES[0];
  const owner = memberForStage(stage, members);
  const seated = memberSeated(owner);
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>④ 测这一环，直到子产出可打开</h2>
          <p>先检查负责人是否就位。未知不能通过。离线不能开测。</p>
          {hostState === "offline" ? (
            <Notice title="离线 · 过时" tone="warn">
              不能开始测。数字与就位检查都可能过时。
            </Notice>
          ) : null}
        </div>
      </section>
      <div className="process-axis" role="list">
        {PROCESS_STAGES.map((item) => {
          const itemOwner = memberForStage(item, members);
          return (
            <button
              key={item.id}
              type="button"
              role="listitem"
              aria-current={item.id === stageId ? "step" : undefined}
              onClick={() => setStageId(item.id)}
            >
              <strong>{item.label}</strong>
              <small>
                {item.owner} · {memberSeated(itemOwner) ? "已就位" : "未就位"}
              </small>
            </button>
          );
        })}
      </div>
      <section className="work-surface">
        <Heading title="成员是否就位" meta={`负责人：${stage.owner}。检查工作说明、工具、能力包、周期与触发、外部连接、文档范围。`} />
        {!owner ? (
          <Notice title="没有对应成员" tone="bad">
            回 ③ 按流程创建岗位。不能开测。
          </Notice>
        ) : !seated ? (
          <Notice title="成员未就位" tone="warn">
            {`${owner.name} 的执行方式还没确认。不能把未就位当成可测。聊天不能批准。`}
          </Notice>
        ) : (
          <Notice title="成员已就位" tone="good">
            {`${owner.name} 已确认执行方式。可以开测这一环。`}
          </Notice>
        )}
        {owner ? (
          <ul className="seat-check">
            {owner.runtime.map((slot) => (
              <li key={slot.id}>
                <span>{slot.businessLabel}</span>
                <Tag tone={slotTone(slot.status)}>{slotStatusLabel(slot.status)}</Tag>
              </li>
            ))}
            <li>
              <span>模型</span>
              <Tag tone={owner.model === "unselected" ? "warn" : "good"}>
                {owner.model === "unselected" ? "未选" : "已选"}
              </Tag>
            </li>
          </ul>
        ) : null}
        <div className="flow-actions">
          {owner ? (
            <>
              <button className="secondary-button" type="button" onClick={() => onInitMember(owner.id)}>
                回 ③ 初始化此人
              </button>
              <button className="secondary-button" type="button" onClick={() => onViewConfig(owner.id)}>
                查看完整配置
              </button>
            </>
          ) : (
            <button className="secondary-button" type="button" onClick={() => onInitMember("")}>
              回 ③ 创建成员
            </button>
          )}
        </div>
      </section>
      <section className="work-surface">
        <Heading title={`正在测 · ${stage.label}`} meta="打开结果 + 是否达标。未就位不能开测。" />
        {!seated ? (
          <p>成员未就位，这一环不能开始测。</p>
        ) : testState === "idle" ? (
          <p>还没开始测这一环。</p>
        ) : testState === "running" ? (
          <p>正在跑这一环。进行中不是完成。</p>
        ) : testState === "fail" ? (
          <Notice title="不通过" tone="bad">
            回到 ② 改这一环。不跳下一环。
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
        <label className="field">
          <span>观察记录</span>
          <SyncedField
            field="test:note"
            label={`${stage.label} · 观察记录`}
            value={testNote}
            onChange={setTestNote}
            onCommit={onCommitField}
            onFocusField={onFocusField}
            multiline
            rows={3}
          />
        </label>
        {testState === "pass" ? (
          <button className="secondary-button" type="button">
            打开这一环结果样品
          </button>
        ) : null}
        <div className="flow-actions">
          <button
            className="primary-button"
            type="button"
            disabled={!seated || testState !== "pass"}
            onClick={onJoint}
          >
            {stage.id === PROCESS_STAGES[PROCESS_STAGES.length - 1]?.id
              ? "末环通过，进入 ⑤"
              : "通过，下一环"}
          </button>
          <button
            className="secondary-button"
            type="button"
            disabled={!seated || hostState === "offline"}
            onClick={() => setTestState("running")}
          >
            开始测（原型）
          </button>
          {testState === "fail" ? (
            <button className="secondary-button" type="button" onClick={onBackToProcess}>
              回 ② 改这一环
            </button>
          ) : null}
        </div>
        {testState === "running" ? (
          <div className="prototype-outcomes" role="group" aria-label="原型结果（不是真测）">
            <span>演示本环结果</span>
            <button className="inline-button" type="button" onClick={() => setTestState("pass")}>
              达标
            </button>
            <button className="inline-button" type="button" onClick={() => setTestState("fail")}>
              不通过
            </button>
            <button className="inline-button" type="button" onClick={() => setTestState("unknown")}>
              说不清
            </button>
          </div>
        ) : null}
      </section>
      <Gap environment>真实测试执行需要后端与合格环境。就位检查在原型里是本地状态。这里只切换样品。</Gap>
    </div>
  );
}

function CreateJointScene({
  jointState,
  setJointState,
  onAccept,
  onBackToTest,
  onBackToProcess,
  onBackToMembers,
  jointNote,
  setJointNote,
  onCommitField,
  onFocusField,
  hostState = "working",
}: {
  jointState: TestOutcome;
  setJointState: (value: TestOutcome) => void;
  onAccept: () => void;
  onBackToTest: () => void;
  onBackToProcess: () => void;
  onBackToMembers: () => void;
  jointNote: string;
  setJointNote: (value: string) => void;
  onCommitField: (field: string, label: string, next: string) => void;
  onFocusField: (field: string, label: string, value: string) => void;
  hostState?: StateKey;
}) {
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>⑤ 联合调试 · 第一次成功</h2>
          <p>打开总成果 + 核对状态。观察记录回车后告诉助手。无假发布。未知不能验收。离线不能开始联调。</p>
          {hostState === "offline" ? (
            <Notice title="离线 · 过时" tone="warn">
              不能开始联调。过时结果不能当验收。
            </Notice>
          ) : null}
        </div>
      </section>
      <section className="work-surface">
        <Heading title="全流程走到哪" meta="失败会指出环节并回 ④ / ②" />
        <ol className="run-steps">
          {PROCESS_STAGES.map((item) => (
            <li
              key={item.id}
              data-state={
                jointState === "pass"
                  ? "done"
                  : jointState === "fail" && item.id === PROCESS_STAGES[PROCESS_STAGES.length - 1]?.id
                    ? "blocked"
                    : jointState === "running" || jointState === "idle"
                      ? "waiting"
                      : "current"
              }
            >
              <strong>{item.label}</strong>
              <span>
                {jointState === "pass"
                  ? "可打开"
                  : jointState === "fail"
                    ? "失败时回 ④ 测该环"
                    : "联调未验收，不能写死已完成"}
              </span>
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
        <label className="field">
          <span>总成果观察</span>
          <SyncedField
            field="joint:note"
            label="联合调试 · 总成果观察"
            value={jointNote}
            onChange={setJointNote}
            onCommit={onCommitField}
            onFocusField={onFocusField}
            multiline
            rows={3}
          />
        </label>
        <div className="flow-actions">
          <button
            className="secondary-button"
            type="button"
            disabled={hostState === "offline"}
            onClick={() => setJointState("running")}
          >
            开始联调（原型）
          </button>
          {jointState === "pass" ? (
            <button className="secondary-button" type="button">
              打开总成果样品
            </button>
          ) : null}
          <button
            className="primary-button"
            type="button"
            disabled={jointState !== "pass"}
            onClick={onAccept}
          >
            验收，进入今日
          </button>
          {jointState === "fail" ? (
            <>
              <button className="secondary-button" type="button" onClick={onBackToTest}>
                回 ④ 测失败环节
              </button>
              <button className="text-button" type="button" onClick={onBackToProcess}>
                回 ② 改流程
              </button>
              <button className="text-button" type="button" onClick={onBackToMembers}>
                回 ③ 改成员
              </button>
            </>
          ) : null}
          <span className="flow-end">没有 Publish 按钮。</span>
        </div>
        {jointState === "running" ? (
          <div className="prototype-outcomes" role="group" aria-label="原型结果（不是真联调）">
            <span>演示联调结果</span>
            <button className="inline-button" type="button" onClick={() => setJointState("pass")}>
              核对通过
            </button>
            <button className="inline-button" type="button" onClick={() => setJointState("fail")}>
              失败
            </button>
            <button className="inline-button" type="button" onClick={() => setJointState("unknown")}>
              说不清
            </button>
          </div>
        ) : null}
      </section>
      <Gap>验收写入权威、独立核对和回执需要 daemon。聊天不能 验收。</Gap>
    </div>
  );
}

function TodayIncompleteScene({
  onContinue,
  gate,
}: {
  onContinue: () => void;
  gate: number;
}) {
  return (
    <div className="scene-stack">
      <section className="today-header">
        <div>
          <h2>创建还没走完</h2>
          <p>日常决策包要等 ⑤ 验收。现在今日只留这一件。停在第 {gate} 段。</p>
        </div>
      </section>
      <section className="work-surface">
        <Heading title="继续未完成的创建" meta="不是日常拍板，不是决策包。" />
        <p>五段向导还在进行。不要把卡片摆在中间当成已经成功。</p>
        <div className="packet-actions">
          <button className="primary-button" type="button" onClick={onContinue}>
            继续第 {gate} 段
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
  onDefer,
  hasDecision,
  hostState = "working",
}: {
  period: Period;
  setPeriod: (value: Period) => void;
  selectedRun: string | null;
  setSelectedRun: (value: string | null) => void;
  onDecision: () => void;
  onProject: () => void;
  onDefer: () => void;
  hasDecision: boolean;
  hostState?: StateKey;
}) {
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
      {hostState === "offline" ? (
        <Notice title="过时" tone="warn">
          主机离线。下面是上次已知事实，不能当当前成功。
        </Notice>
      ) : null}
      {hostState === "unknown" ? (
        <Notice title="说不清" tone="bad">
          未知不是 0，也不是成功。禁止盲着重试。
        </Notice>
      ) : null}
      {hostState === "error" ? (
        <Notice title="读取失败" tone="bad">
          已输入和上次事实保留。决策包仍可点。
        </Notice>
      ) : null}
      {hostState === "blocked" ? (
        <Notice title="停在依赖上" tone="bad">
          已做的工作安全。去该项目运行处理。不是未完成创建。
        </Notice>
      ) : null}
      {hostState === "loading" ? (
        <Notice title="正在读取" tone="info">
          决策包仍可点。上次投影可见。
        </Notice>
      ) : null}
      {hasDecision ? (
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
            <button className="primary-button" type="button" onClick={onDecision} disabled={hostState === "offline"}>
              去处理这一件拍板
            </button>
            <button className="text-button" type="button" onClick={onDefer}>
              以后再说（仍留在今日）
            </button>
          </div>
        </section>
      ) : (
        <section className="work-surface">
          <Heading title="此刻没有要拍板的事" meta="决策包收起。运行概览仍在。" />
          <p>上线项目还在跑。有拍板时会回到这一屏的决策包。</p>
        </section>
      )}
      <section className="run-counts" aria-label="项目计数">
        <div>
          <span>创建的项目</span>
          <strong>{period === "month" ? "说不清" : "2"}</strong>
          <small>样品计数 · 含未上线草稿则另计</small>
        </div>
        <div>
          <span>已上线</span>
          <strong>2</strong>
          <small>无示范项目</small>
        </div>
        <div>
          <span>发生阻塞</span>
          <button className="text-button" type="button" onClick={onProject}>
            <strong>1</strong>
          </button>
          <small>点进该项目运行</small>
        </div>
      </section>
      <section className="work-surface">
        <Heading title={`${period === "today" ? "今日" : period === "week" ? "本周" : "本月"}运行概览`} meta="按已上线项目行：状态 / 完成次数 / 当前环节 / 时长" />
        <ul className="result-list">
          <li>
            <button className="text-button" type="button" onClick={() => { setSelectedRun("weekly"); onProject(); }}>
              <strong>周报与客户跟进</strong>
              <span>
                进行中 · {period === "today" ? "今日 2 次" : period === "week" ? "本周 9 次" : "本月说不清"} · 收集事实 · 41 分钟
              </span>
            </button>
          </li>
          <li>
            <button className="text-button" type="button" onClick={() => { setSelectedRun("site"); onProject(); }}>
              <strong>设备现场周检</strong>
              <span>
                阻塞 · {period === "today" ? "今日 0 次完成" : period === "week" ? "本周 1 次" : "本月说不清"} · 核对 · 过时需处理
              </span>
            </button>
          </li>
        </ul>
        <p>费用未知的格子写「说不清」，不写 0。{hostState === "offline" ? " 数字已过时。" : ""}</p>
      </section>
    </div>
  );
}

function ProjectsScene({
  lifecycle,
  projects,
  copied,
  onCreate,
  onCopy,
  onContinue,
  onDetail,
  onMembers,
  onRuns,
  onOutputs,
}: {
  lifecycle: Lifecycle;
  projects: readonly SampleProject[];
  copied: boolean;
  onCreate: () => void;
  onCopy: (id: SampleProjectId) => void;
  onContinue: () => void;
  onDetail: (id: SampleProjectId) => void;
  onMembers: (id: SampleProjectId) => void;
  onRuns: (id: SampleProjectId) => void;
  onOutputs: (id: SampleProjectId) => void;
}) {
  if (lifecycle === "empty") {
    return (
      <div className="scene-stack">
        <section className="today-header">
          <div>
            <h2>还没有项目</h2>
            <p>创建只从今日空首页开始。这里不再并列创建按钮。没有示范项目。</p>
          </div>
          <button className="text-button" type="button" onClick={onCreate}>
            回今日
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
            <p>项目列表现在只露出这一份草稿。继续创建，不要进详情/成员/运行/产出。</p>
          </div>
          <button className="primary-button" type="button" onClick={onContinue}>
            继续创建
          </button>
        </section>
        <section className="work-surface">
          <Heading title="创建中 · 周报与客户跟进" meta="未上线。不能当已上线项目跑。" />
          <div className="project-row-actions">
            <button className="secondary-button" type="button" onClick={onContinue}>
              继续这份草稿
            </button>
          </div>
        </section>
      </div>
    );
  }
  return (
    <div className="scene-stack">
      <section className="today-header">
        <div>
          <h2>项目列表</h2>
          <p>打开一个项目后，用左侧详情 / 成员 / 运行 / 产出工作。副本不带密钥、进行中任务、对外回执、本周不再问。第二份工作请复制，不要在已上线列表另开看不见的 ①。</p>
        </div>
      </section>
      {copied ? (
        <div className="copy-banner">
          <strong>周报与客户跟进（副本）</strong>
          <p>未激活草稿。先打开总预览。④⑤ 可抽检或跳过。不从 ① 重来。不带密钥、在途任务、外部回执、时间盒跳过。</p>
          <details>
            <summary>总预览（项目仍未上线）</summary>
            <p>目标、周期、流程轴与权限与源项目相同。这是本地样品，不是 daemon 写入。</p>
            <button className="secondary-button" type="button">
              总预览已读 · 激活仍需 daemon
            </button>
          </details>
        </div>
      ) : null}
      {projects.map((item) => (
        <section key={item.id} className="work-surface">
          <Heading title={item.name} meta={`${item.industry} · ${item.statusLine}`} />
          <dl className="definition-list">
            <div>
              <dt>目标</dt>
              <dd>{item.goal}</dd>
            </div>
            <div>
              <dt>周期</dt>
              <dd>{item.cycle}</dd>
            </div>
            <div>
              <dt>费用</dt>
              <dd>{item.costLine}</dd>
            </div>
          </dl>
          <div className="project-row-actions">
            <button className="primary-button" type="button" onClick={() => onDetail(item.id)}>
              打开
            </button>
            <button className="text-button" type="button" onClick={() => onMembers(item.id)}>
              成员
            </button>
            <button className="text-button" type="button" onClick={() => onRuns(item.id)}>
              运行
            </button>
            <button className="text-button" type="button" onClick={() => onOutputs(item.id)}>
              产出
            </button>
            {item.kind === "live" && item.id === "weekly" ? (
              <button className="text-button" type="button" onClick={() => onCopy(item.id)}>
                复制为草稿
              </button>
            ) : null}
          </div>
        </section>
      ))}
    </div>
  );
}

function ProjectDetailScene({
  project,
  switcherProjects,
  onSwitch,
  name,
  setName,
  goal,
  setGoal,
  cycle,
  setCycle,
  onMembers,
  onRuns,
  onOutputs,
  onCommitField,
  onFocusField,
}: {
  project: SampleProject;
  switcherProjects: readonly SampleProject[];
  onSwitch: (id: SampleProjectId) => void;
  name: string;
  setName: (value: string) => void;
  goal: string;
  setGoal: (value: string) => void;
  cycle: string;
  setCycle: (value: string) => void;
  onMembers: () => void;
  onRuns: () => void;
  onOutputs: () => void;
  onCommitField: (field: string, label: string, next: string) => void;
  onFocusField: (field: string, label: string, value: string) => void;
}) {
  return (
    <div className="scene-stack">
      <section className="project-header">
        <div>
          <h2>项目详情</h2>
          <p>只读章程与流程轴。改章程走预览确认，不当表单页。去成员 / 运行 / 产出。</p>
        </div>
        <ProjectSwitcher projects={switcherProjects} currentId={project.id} onChange={onSwitch} />
      </section>
      <section className="work-surface">
        <Heading title={name} meta={`${project.industry} · ${project.statusLine}`} />
        <dl className="definition-list">
          <div>
            <dt>名称</dt>
            <dd>{name}</dd>
          </div>
          <div>
            <dt>目标</dt>
            <dd>{goal}</dd>
          </div>
          <div>
            <dt>周期</dt>
            <dd>{cycle}</dd>
          </div>
          <div>
            <dt>状态</dt>
            <dd>{project.statusLine}</dd>
          </div>
          <div>
            <dt>费用</dt>
            <dd>{project.costLine}</dd>
          </div>
          <div>
            <dt>流程环节</dt>
            <dd>{project.stages.length} 环 · 执行进度在运行管理</dd>
          </div>
          <div>
            <dt>成员</dt>
            <dd>
              {project.kind === "live"
                ? `${project.participants.length} 人 · 不跨项目共享`
                : "未激活副本不带成员。"}
            </dd>
          </div>
        </dl>
        <div className="project-row-actions">
          <button className="text-button" type="button" onClick={onMembers}>
            成员管理
          </button>
          <button className="text-button" type="button" onClick={onRuns}>
            运行管理
          </button>
          <button className="text-button" type="button" onClick={onOutputs}>
            产出管理
          </button>
        </div>
      </section>
      {project.stages.length > 0 ? (
        <>
          <p className="sample-caption">流程轴 · 只读章程。点运行管理看当前步骤。</p>
          <div className="process-axis" role="list">
            {project.stages.map((item) => (
              <div key={item.id} className="process-node" role="listitem">
                <strong>{item.label}</strong>
                <small>负责人 {item.owner}</small>
              </div>
            ))}
          </div>
        </>
      ) : null}
      <Gap>项目权威写入需要 daemon。这里只演示本地样品与确认协议。</Gap>
    </div>
  );
}

function ProjectMembersScene({
  project,
  switcherProjects,
  onSwitch,
  members,
  stageDrafts,
  onAdd,
  setModel,
  setMemberText,
  setSlotValue,
  onCommitField,
  onFocusField,
}: {
  project: SampleProject;
  switcherProjects: readonly SampleProject[];
  onSwitch: (id: SampleProjectId) => void;
  members: readonly MemberDraft[];
  stageDrafts: Record<string, StageDraft>;
  onAdd: () => void;
  setModel: (id: string, model: string) => void;
  setMemberText: (id: string, key: "duty" | "handoff", value: string) => void;
  setSlotValue: (id: string, slotId: RuntimeSlotId, value: string) => void;
  onCommitField: (field: string, label: string, next: string) => void;
  onFocusField: (field: string, label: string, value: string) => void;
}) {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [tab, setTab] = useState<MemberConfigTab>("duty");
  useEffect(() => {
    setSelectedId(null);
    setTab("duty");
  }, [project.id]);
  useEffect(() => {
    setTab("duty");
  }, [selectedId]);
  if (project.kind !== "live") {
    return (
      <div className="scene-stack">
        <section className="project-header">
          <div>
            <h2>成员管理</h2>
            <p>{project.kind === "creating" ? "创建未完成，还没有可维护的班子。" : "未激活副本不带成员。"}</p>
          </div>
          <ProjectSwitcher projects={switcherProjects} currentId={project.id} onChange={onSwitch} />
        </section>
        <section className="work-surface">
          <p>诚实空。不发明示范成员。先完成创建或激活副本。</p>
        </section>
      </div>
    );
  }
  const selected = members.find((member) => member.id === selectedId) ?? null;
  return (
    <div className="scene-stack">
      <section className="project-header">
        <div>
          <h2>成员管理</h2>
          <p>先选左侧一个人，再用标签查看职责、输入、输出、技能、工具。改完回车 → 确认框 → 对话确认。聊天不能批准。</p>
        </div>
        <div className="header-actions">
          <ProjectSwitcher
            projects={switcherProjects}
            currentId={project.id}
            onChange={(id) => {
              setSelectedId(null);
              onSwitch(id);
            }}
          />
          <button className="secondary-button" type="button" onClick={onAdd}>
            加人
          </button>
        </div>
      </section>
      {members.length === 0 ? (
        <section className="work-surface">
          <Heading title="成员名单" meta="成员不跨项目共享。" />
          <p>还没有可检查的岗位。加人后再初始化。</p>
          <button className="primary-button" type="button" onClick={onAdd}>
            加人
          </button>
        </section>
      ) : (
        <section className="work-surface">
          <Heading
            title="成员名单与配置"
            meta="未选人时右侧保持空。就位状态不是 daemon 权威。无安装按钮。"
          />
          <div className="people-layout">
            <div className="member-list" role="listbox" aria-label="成员名单">
              {members.map((member) => (
                <button
                  key={member.id}
                  type="button"
                  role="option"
                  aria-selected={member.id === selectedId}
                  onClick={() => setSelectedId(member.id)}
                >
                  <span>
                    <strong>{member.name}</strong>
                    <small>
                      {initStatusLabel(member.initStatus)}
                      {member.model === "unselected" ? " · 模型待选" : ""}
                    </small>
                  </span>
                </button>
              ))}
            </div>
            {selected ? (
              <MemberConfigPanel
                member={selected}
                project={project}
                stageDrafts={stageDrafts}
                tab={tab}
                setTab={setTab}
                setModel={setModel}
                setMemberText={setMemberText}
                setSlotValue={setSlotValue}
                onCommitField={onCommitField}
                onFocusField={onFocusField}
              />
            ) : (
              <section className="member-detail member-detail-empty" aria-live="polite">
                <h3>还没选人</h3>
                <p>从左侧选一个人，再看他的职责、输入、输出、技能和工具。不默认打开第一人。</p>
              </section>
            )}
          </div>
        </section>
      )}
      <Gap>成员权威写入、Skill/MCP 授权和权限落地需要 daemon。标签页只改本地原型状态。无安装按钮。</Gap>
    </div>
  );
}

function ProjectRunsScene({
  project,
  switcherProjects,
  onSwitch,
  stageId,
  setStageId,
  onHitl,
  onClose,
}: {
  project: SampleProject;
  switcherProjects: readonly SampleProject[];
  onSwitch: (id: SampleProjectId) => void;
  stageId: string;
  setStageId: (id: string) => void;
  onHitl: () => void;
  onClose: () => void;
}) {
  if (project.kind !== "live") {
    return (
      <div className="scene-stack">
        <section className="project-header">
          <div>
            <h2>运行管理</h2>
            <p>{project.kind === "creating" ? "未上线，没有今日执行。" : "未激活副本不能当已上线项目跑。"}</p>
          </div>
          <ProjectSwitcher projects={switcherProjects} currentId={project.id} onChange={onSwitch} />
        </section>
        <section className="work-surface">
          <p>{project.todayOk}。未知和未激活都不写成 0。</p>
        </section>
        <Gap>复制与运行投影需要 daemon。这里只切换本地样品。</Gap>
      </div>
    );
  }
  const stage = project.stages.find((item) => item.id === stageId) ?? project.stages[0];
  const needsOwner = stage.mark !== "none";
  const lastStage = project.stages[project.stages.length - 1];
  const canClose = Boolean(lastStage && stage.id === lastStage.id);
  return (
    <div className="scene-stack">
      <section className="project-header">
        <div>
          <h2>运行管理</h2>
          <p>当前业务流程、每环正在执行的具体步骤、今日执行情况。进行中不是完成。</p>
        </div>
        <ProjectSwitcher projects={switcherProjects} currentId={project.id} onChange={onSwitch} />
      </section>
      <section className="run-counts" aria-label="今日执行情况">
        <div>
          <span>成功次数</span>
          <strong>{project.todayOk}</strong>
          <small>样品 · 来源：本机运行投影</small>
        </div>
        <div>
          <span>失败次数</span>
          <strong>{project.todayFail}</strong>
          <small>样品 · 未知不写 0</small>
        </div>
        <div>
          <span>平均执行时长</span>
          <strong>{project.todayAvg}</strong>
          <small>样品 · 进行中另计</small>
        </div>
      </section>
      <div className="process-axis" role="list">
        {project.stages.map((item) => (
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
        <p className="current-step">当前步骤：{stage.currentStep}</p>
        <div className="packet-marks">
          {stage.mark === "auth" ? <Tag tone="warn">要你授权</Tag> : null}
          {stage.mark === "verify" ? <Tag tone="info">要你核对</Tag> : null}
          {stage.mark === "none" ? <Tag>无需你现在出手</Tag> : null}
        </div>
        <dl className="ledger-facts">
          <div>
            <dt>今日成功</dt>
            <dd>{stage.todayOk}</dd>
          </div>
          <div>
            <dt>今日失败</dt>
            <dd>{stage.todayFail}</dd>
          </div>
          <div>
            <dt>平均时长</dt>
            <dd>{stage.todayAvg}</dd>
          </div>
        </dl>
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
        {canClose ? (
          <button className="text-button" type="button" onClick={onClose}>
            打开成果并验收，回今日
          </button>
        ) : (
          <p className="sample-caption">验收在末环。这一环做完后沿轴前进，不要在中途标完成。</p>
        )}
      </section>
      <Gap>环节状态、授权和回执需要 daemon。没有假发布。</Gap>
    </div>
  );
}

function OutputComposition({
  output,
  format,
  onHitl,
}: {
  output: SampleOutput;
  format: OutputFormat;
  onHitl: () => void;
}) {
  if (format === "checklist") {
    return (
      <article className="output-composition" aria-label={formatLabel(format)}>
        <p>{output.job}</p>
        <ul className="output-checklist">
          <li>来源已标 · 本周客户跟进.md</li>
          <li>未知项单独列出，没有写成 0</li>
          <li>未完成项没有勾成通过</li>
        </ul>
      </article>
    );
  }
  if (format === "packet") {
    return (
      <article className="output-composition" aria-label={formatLabel(format)}>
        <p>{output.job}</p>
        <dl className="definition-list">
          <div>
            <dt>将做什么</dt>
            <dd>把本周摘要发给已选客户。不是已经发出。</dd>
          </div>
          <div>
            <dt>附件</dt>
            <dd>周报 Markdown · 客户名单（本地样品）</dd>
          </div>
        </dl>
        {output.needsHitl ? (
          <div className="packet-actions">
            <button className="primary-button" type="button" onClick={onHitl}>
              去画布预览拍板
            </button>
          </div>
        ) : null}
      </article>
    );
  }
  if (format === "article") {
    const site = output.id.startsWith("inspect");
    return (
      <article className="output-composition" aria-label={formatLabel(format)}>
        <p className="output-lede">{output.job}</p>
        <p>
          {site
            ? "3 号机房本周未见超温。这是本地目标态样品，不是 daemon 写出的文件。"
            : "本周跟进 4 家，2 家待回复。这是本地目标态样品，不是 daemon 写出的文件。"}
        </p>
        <figure className="media-sample">
          <div className="media-frame" aria-hidden="true" />
          <figcaption>{site ? "配图样品 · 3 号机房照片（未嵌入真实图）" : "配图样品 · 跟进口径截图（未嵌入真实图）"}</figcaption>
        </figure>
        <figure className="media-sample">
          <div className="media-frame media-frame-video" role="img" aria-label="视频样品（未播放）">
            视频样品（未播放）
          </div>
          <figcaption>不是播放器。真实视频编解码与打开需要后端。</figcaption>
        </figure>
        <Gap>真实配图/视频打开需要 daemon。禁止假播放器。</Gap>
      </article>
    );
  }
  if (format === "link") {
    return (
      <article className="output-composition" aria-label={formatLabel(format)}>
        <p>{output.job}</p>
        <dl className="definition-list">
          <div>
            <dt>位置</dt>
            <dd>本机项目目录 · 周报.md（样品路径，不是公开网址）</dd>
          </div>
          <div>
            <dt>状态</dt>
            <dd>可打开样品。不是已公开发布。</dd>
          </div>
        </dl>
      </article>
    );
  }
  return (
    <article className="output-composition output-document" aria-label={formatLabel(format)}>
      <p className="output-lede">{output.job}</p>
      <p>
        本周跟进 4 家，2 家待回复。周报可打开。这是本地目标态样品，不是 daemon
        写出的文件。
      </p>
      <dl className="definition-list">
        <div>
          <dt>附件</dt>
          <dd>本周客户跟进.md · 核对记录（本地样品）</dd>
        </div>
        <div>
          <dt>新鲜度</dt>
          <dd>样品标注为当前稿。离线时不能当新成功。</dd>
        </div>
      </dl>
    </article>
  );
}

function ProjectOutputsScene({
  project,
  switcherProjects,
  onSwitch,
  sample,
  setSample,
  selectedId,
  setSelectedId,
  formatOverride,
  onAskFormat,
  onWork,
  onHitl,
}: {
  project: SampleProject;
  switcherProjects: readonly SampleProject[];
  onSwitch: (id: SampleProjectId) => void;
  sample: OutputSampleKey;
  setSample: (value: OutputSampleKey) => void;
  selectedId: string | null;
  setSelectedId: (id: string) => void;
  formatOverride: OutputFormat | null;
  onAskFormat: () => void;
  onWork: () => void;
  onHitl: () => void;
}) {
  const ready = project.outputs.filter((item) => item.accepted);
  const showEmpty =
    sample === "empty" || project.kind !== "live" || (sample !== "working" && sample !== "unknown" && sample !== "partial" && ready.length === 0);
  const visibleOutputs =
    sample === "packet"
      ? ready.filter((item) => item.format === "packet" || item.needsHitl)
      : sample === "document"
        ? ready
        : ready;
  const selected = visibleOutputs.find((item) => item.id === selectedId) ?? null;
  const format = selected ? (formatOverride ?? selected.format) : "document";

  return (
    <div className="scene-stack">
      <section className="project-header">
        <div>
          <h2>产出管理</h2>
          <p>无固定模板。先选一份产出，再看文稿、清单、链接、配图或交付包。</p>
        </div>
        <div className="header-actions">
          <ProjectSwitcher
            projects={switcherProjects}
            currentId={project.id}
            onChange={(id) => {
              setSelectedId("");
              onSwitch(id);
            }}
          />
          <button className="secondary-button" type="button" onClick={onWork}>
            回运行管理
          </button>
        </div>
      </section>
      <details className="trace-fold">
        <summary>原型样品 · 不是 daemon 产出目录</summary>
        <Segmented
          label="原型样品"
          value={sample}
          items={[
            { id: "empty", label: "空" },
            { id: "document", label: "文稿" },
            { id: "packet", label: "交付包" },
            { id: "unknown", label: "未知" },
          ]}
          onChange={setSample}
        />
      </details>
      {sample === "working" ? (
        <section className="work-surface" aria-live="polite">
          <p>正在编排这份产出的展示… 进行中不是完成。</p>
        </section>
      ) : sample === "unknown" ? (
        <section className="work-surface" aria-live="polite">
          <p>说不清这份产出是否可打开。未知不是成功，也不是空画廊。</p>
          <button className="secondary-button" type="button" onClick={onWork}>
            回运行管理
          </button>
        </section>
      ) : sample === "partial" ? (
        <section className="work-surface">
          <p>有一份草稿，还不能当已验收成果打开。缺口不写成已就绪。</p>
          <button className="secondary-button" type="button" onClick={onWork}>
            回运行管理
          </button>
        </section>
      ) : showEmpty ? (
        <section className="work-surface">
          <p>还没有可打开的成果。先把这一环做完，或继续创建。</p>
          <button className="primary-button" type="button" onClick={onWork}>
            {project.kind === "creating" ? "继续创建" : "回运行管理"}
          </button>
        </section>
      ) : (
        <section className="work-surface">
          <Heading title="已验收产出" meta="先选左侧一份。未选不默认打开第一份。" />
          <div className="people-layout">
            <div className="member-list" role="listbox" aria-label="产出名单">
              {visibleOutputs.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  role="option"
                  aria-selected={item.id === selectedId}
                  onClick={() => setSelectedId(item.id)}
                >
                  <span>
                    <strong>{item.title}</strong>
                    <small>{formatLabel(item.format)}{item.needsHitl ? " · 要拍板" : ""}</small>
                  </span>
                </button>
              ))}
            </div>
            {selected ? (
              <section className="member-detail" aria-label={`${selected.title} · 编排`}>
                <p className="output-kicker">
                  助手按这份产出编排 · {formatLabel(format)}
                </p>
                <OutputComposition output={selected} format={format} onHitl={onHitl} />
                <div className="packet-actions">
                  <button className="secondary-button" type="button" onClick={onAskFormat}>
                    请助手换一种展示
                  </button>
                </div>
                <details className="trace-fold">
                  <summary>编排说明 · 默认收起</summary>
                  <p>
                    助手按这份产出选择文稿、清单、链接、正文配图或交付包。没有固定模板。这是本地建议。你在对话里确认后画布才换。
                    需要拍板的成果仍走画布 HITL。聊天没有批准。
                  </p>
                </details>
              </section>
            ) : (
              <section className="member-detail member-detail-empty" aria-live="polite">
                <h3>还没选产出</h3>
                <p>从左侧选一份已验收成果，再看编排。不默认打开第一份。</p>
              </section>
            )}
          </div>
        </section>
      )}
      <Gap>真实产出与打开文件需要 daemon。这里只演示助手编排与 Owner 确认。</Gap>
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
  members,
  onJoin,
  onRefuse,
  onOpenConfig,
  onSettings,
  onCommitField,
  onFocusField,
}: {
  name: string;
  setName: (value: string) => void;
  duty: string;
  setDuty: (value: string) => void;
  model: string;
  setModel: (value: string) => void;
  joined: boolean;
  members: readonly MemberDraft[];
  onJoin: () => void;
  onRefuse: () => void;
  onOpenConfig: () => void;
  onSettings: () => void;
  onCommitField: (field: string, label: string, next: string) => void;
  onFocusField: (field: string, label: string, value: string) => void;
}) {
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>给已上线项目补一个岗位</h2>
          <p>右侧助手是改岗位的主入口。岗位名 / 职责回车后以你的名义写入对话。模型必选。</p>
        </div>
      </section>
      <section className="work-surface">
        <Heading title="现有班子" meta="成员不跨项目共享。进程退出不会删掉人。" />
        {members.length === 0 ? (
          <p>当前项目还没有成员。不发明示范名单。</p>
        ) : (
          <ul className="result-list">
            {members.map((member) => (
              <li key={member.id}>
                <div>
                  <strong>{member.name}</strong>
                  <span>{member.duty}</span>
                </div>
                <div>
                  <Tag tone={initStatusTone(member.initStatus)}>{initStatusLabel(member.initStatus)}</Tag>
                </div>
              </li>
            ))}
          </ul>
        )}
      </section>
      <section className="work-surface">
        <Heading title="新岗位" meta="空 = 这个岗位还不存在。拒绝 = 未加入。没模型 = pending，去设置。" />
        {name.trim().length === 0 && duty.trim().length === 0 ? (
          <Notice title="这个岗位还不存在" tone="info">
            右侧助手可以建议岗位。不要预填示范岗。确认加入后才进名单。
          </Notice>
        ) : null}
        <label className="field">
          <span>岗位名</span>
          <SyncedField
            field="add-member:name"
            label="新岗位 · 岗位名"
            value={name}
            onChange={setName}
            onCommit={onCommitField}
            onFocusField={onFocusField}
          />
        </label>
        <label className="field">
          <span>做什么、交出什么</span>
          <SyncedField
            field="add-member:duty"
            label="新岗位 · 做什么、交出什么"
            value={duty}
            onChange={setDuty}
            onCommit={onCommitField}
            onFocusField={onFocusField}
            multiline
            rows={3}
          />
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
            disabled={name.trim().length === 0}
            onClick={onJoin}
          >
            确认加入
          </button>
          <button className="secondary-button" type="button" onClick={onRefuse}>
            拒绝
          </button>
          {model === "unselected" ? (
            <button className="text-button" type="button" onClick={onSettings}>
              去设置选模型
            </button>
          ) : null}
          <span className="flow-end">无模型也可加入为 pending。加入后改流程/权限要再批。</span>
        </div>
        {joined ? (
          <Notice title="已加入（原型）" tone="good">
            {model === "unselected" ? "pending：还没模型。去设置。执行方式仍要初始化。" : "岗位已进名单。下一步初始化执行方式。不是先装 MCP。"}
            <div className="flow-actions">
              <button className="primary-button" type="button" onClick={onOpenConfig}>
                初始化执行方式
              </button>
            </div>
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
  pending = true,
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
  pending?: boolean;
}) {
  const canApprove = pending && previewAge === "fresh" && !executing && fate !== "narrowed";
  if (!pending) {
    return (
      <div className="scene-stack">
        <section className="work-surface">
          <Heading title="没有待拍板的预览" meta="empty = 无待批。" />
          <p>有对外或关键动作时，预览会出现在这里。聊天只有链接，没有批准。</p>
          <button className="secondary-button" type="button" onClick={onBack}>
            回到运行
          </button>
        </section>
      </div>
    );
  }
  return (
    <div className="scene-stack">
      <section className="setup-header">
        <div>
          <h2>画布预览 · 聊天不能批</h2>
          <p>将做什么 + 完整预览 + 批准 / 改窄 / 拒绝。执行中第四个行动是停。过期或未知不能批。</p>
        </div>
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
          {" "}本周此类不再问（到期失效，设置里可收回）
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
            onClick={() => {
              setFate("narrowed");
              setPreviewAge("stale");
            }}
          >
            改窄
          </button>
          <button
            className="secondary-button"
            type="button"
            onClick={() => {
              setFate("rejected");
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
        {previewAge === "unknown" ? (
          <Notice title="说不清" tone="bad">
            预览是否仍有效说不清。不能批准。不是过期，也不是成功。禁止盲重试。
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
        {fate === "rejected" ? (
          <Notice title="未发出" tone="warn">
            拒绝不会发送。回执留在这一环。
            <div className="flow-actions">
              <button className="secondary-button" type="button" onClick={onBack}>
                回到该环
              </button>
            </div>
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
  hostState = "working",
  showImportDemo = false,
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
  hostState?: StateKey;
  showImportDemo?: boolean;
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
        { id: "site", label: "设备现场周检" },
      ];
  if (locked) {
    return (
      <div className="scene-stack">
        <section className="today-header">
          <div>
            <h2>知识已锁定</h2>
            <p>没有项目时不能进。创建到 ② 流程需要输入时，只为当前草稿打开。</p>
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
          label="知识"
          value={tab}
          items={[
            { id: "files", label: "项目资料" },
            { id: "import", label: "导入" },
            { id: "why", label: "为什么用这段" },
            { id: "memory", label: "记忆" },
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
              disabled={importPhase === "importing" || hostState === "offline"}
              onClick={() => setImportPhase("importing")}
            >
              开始导入（原型，不写磁盘）
            </button>
            <span className="flow-end">
              {hostState === "offline"
                ? "离线 · 只读上次索引。不能开始导入。"
                : hostState === "unknown"
                  ? "资料条数说不清，不写 0。"
                : importPhase === "idle"
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
                    ? "凭证改走批准的 SecretStore 交接。不进知识库、聊天、上下文或记忆。"
                    : "这是目标态样品行，不是文件系统或 daemon 回执。"}
            </Notice>
          )}
          {showImportDemo && importPhase === "importing" ? (
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
          <Heading title="对话自动进入可检查记忆" meta="可改、可忘。跨项目提升要确认。Codex 是记忆架构，不是引擎商店。" />
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
  hostState = "working",
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
  hostState?: StateKey;
}) {
  const custom = provider === "custom";
  return (
    <div className="scene-stack">
      <section className="settings-header">
        <div>
          <h2>设置</h2>
          <p>连接模型 · 收回本周不再问 · 通知恢复。无账单、无引擎商店、无收件箱。</p>
        </div>
      </section>
      <section className="work-surface">
        <Heading title="模型连接" meta="主流下拉 + 自定义 URL / 兼容 / 模型。Owner 输入密钥。" />
        <label className="field">
          <span>供应商模板</span>
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
            placeholder="输入后交接，界面不回显…"
          />
          <small>A5：单向交给 SecretStore。DOM、聊天、git 都不保留明文。此原型在交接后清空输入。</small>
        </label>
        <div className="flow-actions">
          <button
            className="primary-button"
            type="button"
            disabled={keyDraft.trim().length === 0 || hostState === "offline"}
            onClick={handoff}
          >
            交接密钥（原型，不联网）
          </button>
          <span className="flow-end">
            {hostState === "offline"
              ? "离线不能交接。"
              : status === "connected"
              ? "已交接样品 · Requires-backend。没有真连上 Provider。"
              : status === "failed"
                ? "失败 · 点名：SecretStore 不可用（样品）。不是已连接。"
                : hostState === "unknown"
                  ? "费用与额度说不清，不写 0。"
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
          <h2>状态实验室渲染覆盖。不是「Designed」矩阵。</h2>
          <p>下面是该表面在该状态下的真实版式。画布运行时、NVDA、对比度和 200% 布局仍是 not-run。</p>
        </div>
        <div className="state-lab-controls">
          <label>
            <span>表面</span>
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
            <span>状态</span>
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
  activeField,
  thread,
  onSendToAssistant,
  onApplyProposal,
  onDismissProposal,
  participants,
  memberConfigFromLive,
}: {
  scene: Scene;
  providerBound: boolean;
  drafts: string;
  setDrafts: (value: string) => void;
  status: string;
  setStatus: (value: string) => void;
  onOpenHitl: () => void;
  activeField: { field: string; label: string; value: string } | null;
  thread: readonly ChatTurn[];
  onSendToAssistant: () => void;
  onApplyProposal: (turnId: number) => void;
  onDismissProposal: (turnId: number) => void;
  participants: readonly string[];
  memberConfigFromLive: boolean;
}) {
  const setup = isSetupChat(scene, memberConfigFromLive);
  const outputs = scene === "project-outputs";
  const project = isLiveProjectChat(scene, memberConfigFromLive);
  const title = setup ? "个人助手" : project ? "项目群" : "个人助手";
  const proposalThread = setup || outputs;
  const addMention = (mention: string) => {
    const space = drafts.length > 0 && !drafts.endsWith(" ") ? " " : "";
    setDrafts(`${drafts}${space}${mention} `);
    setStatus(`${mention} 只进未发送草稿，不绕过任务权威。`);
  };
  const renderProposal = (item: ChatTurn) => (
    <article key={item.id} data-author={item.author}>
      <span>{item.label}</span>
      <p>{item.text}</p>
      {item.proposal ? (
        <div className="proposal-card">
          <small>
            {item.proposal.status === "pending"
              ? `建议改「${item.proposal.label}」。确认后才写回画布。`
              : item.proposal.status === "applied"
                ? `已按你的确认写回画布「${item.proposal.label}」。`
                : "这条建议未采用。"}
          </small>
          {item.proposal.status === "pending" ? (
            <div className="flow-actions">
              <button
                className="primary-button"
                type="button"
                onClick={() => onApplyProposal(item.id)}
              >
                确认，写回画布
              </button>
              <button
                className="secondary-button"
                type="button"
                onClick={() => onDismissProposal(item.id)}
              >
                不用
              </button>
            </div>
          ) : null}
        </div>
      ) : null}
    </article>
  );
  return (
    <aside className="conversation" id="opc-conversation" aria-label={title}>
      <header>
        <div>
          <span>
            {setup
              ? "创建与编辑的主入口 · 画布同步 · 写入仍要你确认"
              : outputs
                ? "产出怎么呈现由助手编排 · 换展示要你确认"
                : project
                  ? "Owner / 经理 / 成员"
                  : "全局助手 · 最高 UX 特权，写入仍要预览"}
          </span>
          <h2>{title}</h2>
        </div>
      </header>
      {project ? (
        <div className="participants" role="group" aria-label="项目群成员">
          <span>Owner</span>
          {participants.map((item) => (
            <span key={item}>{item}</span>
          ))}
        </div>
      ) : null}
      <div className="messages" role="region" aria-label="原型对话样品">
        {setup && activeField ? (
          <article data-author="system" className="canvas-mirror">
            <span>画布当前项 · {activeField.label}</span>
            <p>{activeField.value.trim().length > 0 ? activeField.value : "（还没有内容）"}</p>
            <small>画布里回车会先弹确认，再以你的名义发到这里。聊天不能批准、验收或安装。</small>
          </article>
        ) : null}
        {setup && !providerBound ? (
          <>
            <article data-author="assistant">
              <span>助手 · 尚未绑定</span>
              <p>还没有模型。请去设置连接 Provider 并绑定助手。我不会在聊天里收密钥。</p>
            </article>
            <article data-author="system">
              <span>无静默绑定</span>
              <p>连接失败会说出问题所在。没有 Connect 假按钮。</p>
            </article>
          </>
        ) : null}
        {outputs && thread.length === 0 ? (
          <article data-author="assistant">
            <span>助手 · 候选</span>
            <p>这份产出怎么呈现由我按内容编排。换展示是建议，要你在对话里确认才改画布。聊天不能批准。</p>
          </article>
        ) : null}
        {proposalThread ? thread.map(renderProposal) : null}
        {!setup && scene === "today" ? (
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
        ) : null}
        {!setup && project && !outputs ? (
          <>
            <article data-author="owner">
              <span>Owner</span>
              <p>@林 现在停在哪一步？</p>
            </article>
            <article data-author="manager">
              <span>林 · 默认发言</span>
              <p>停在当前环节。交给 Owner 的那一环钉了授权。聊天不能批。</p>
            </article>
            <article data-author="system" className="approval-card">
              <span>HITL 只在画布</span>
              <p>成员只在被 @、交产出、交接、阻塞或要决策时主动说话。</p>
              <button className="inline-button" type="button" onClick={onOpenHitl}>
                打开画布预览
              </button>
            </article>
          </>
        ) : null}
        {!setup && !project && scene !== "today" ? (
          <article data-author="assistant">
            <span>助手</span>
            <p>我可以解释、调研、起草并发起流程。写入必须经过预览 → 你确认 → 回执。</p>
          </article>
        ) : null}
      </div>
      <div className="composer">
        {project && !outputs ? (
          <div className="mention-buttons" role="group" aria-label="写入未发送草稿">
            {participants.map((item) => {
              const mention = `@${item.split(" · ")[0] ?? item}`;
              return (
                <button key={item} type="button" onClick={() => addMention(mention)}>
                  {mention}
                </button>
              );
            })}
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
              setup
                ? "用自然语言改当前项。发给助手后，建议要你在对话里确认才写回画布…"
                : outputs
                  ? "请助手换成清单、文稿、链接、正文配图或交付包…"
                  : project
                    ? "问经理或有界地改成员工作…"
                    : "问运行情况，或描述一件要办的事…"
            }
          />
        </label>
        <div className="composer-actions">
          {proposalThread ? (
            <button
              className="primary-button"
              type="button"
              disabled={drafts.trim().length === 0 || (setup && !providerBound)}
              onClick={onSendToAssistant}
            >
              发给助手（原型）
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
        <Gap>真实发送与任务翻译需要 daemon。这里只演示本地对话与画布同步。聊天不能批准。</Gap>
      </div>
    </aside>
  );
}

export default function Personal20OpcE2eOptimizedV9() {
  const theme = useHostTheme();
  const [scene, setScene] = useState<Scene>("empty-home");
  const [wizardIndex, setWizardIndex] = useState(0);
  const [wizardValues, setWizardValues] = useState<Record<WizardId, string>>(() =>
    blankWizardValues(DEFAULT_BRIEF),
  );
  const [wizardConfirmed, setWizardConfirmed] = useState<Record<WizardId, boolean>>(blankWizardFlags);
  const [wizardStale, setWizardStale] = useState<Record<WizardId, boolean>>(blankWizardFlags);
  const [providerBound, setProviderBound] = useState(false);
  const [members, setMembers] = useState<MemberDraft[]>([]);
  const [weeklyLiveMembers, setWeeklyLiveMembers] = useState<MemberDraft[]>(() =>
    WEEKLY_SAMPLE_MEMBERS.map((item) => ({ ...item, runtime: item.runtime.map((slot) => ({ ...slot })) })),
  );
  const [siteMembers, setSiteMembers] = useState<MemberDraft[]>(() =>
    SITE_SAMPLE_MEMBERS.map((item) => ({ ...item, runtime: item.runtime.map((slot) => ({ ...slot })) })),
  );
  const [processStageId, setProcessStageId] = useState("collect");
  const [confirmedStages, setConfirmedStages] = useState<string[]>([]);
  const [stageDrafts, setStageDrafts] = useState<Record<string, StageDraft>>(defaultStageDrafts);
  const [testNote, setTestNote] = useState(TEST_NOTE_DEFAULT);
  const [jointNote, setJointNote] = useState(JOINT_NOTE_DEFAULT);
  const [testByStage, setTestByStage] = useState<Record<string, TestOutcome>>({});
  const [jointState, setJointState] = useState<TestOutcome>("idle");
  const [period, setPeriod] = useState<Period>("today");
  const [selectedRun, setSelectedRun] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [hasDecision, setHasDecision] = useState(true);
  const [currentProjectId, setCurrentProjectId] = useState<SampleProjectId>("weekly");
  const [detailName, setDetailName] = useState(() => projectById("weekly").name);
  const [detailGoal, setDetailGoal] = useState(() => projectById("weekly").goal);
  const [detailCycle, setDetailCycle] = useState(() => projectById("weekly").cycle);
  const [outputSample, setOutputSample] = useState<OutputSampleKey>("empty");
  const [selectedOutputId, setSelectedOutputId] = useState<string | null>(null);
  const [outputFormatOverride, setOutputFormatOverride] = useState<OutputFormat | null>(null);
  const [newName, setNewName] = useState("");
  const [newDuty, setNewDuty] = useState("");
  const [newModel, setNewModel] = useState("unselected");
  const [joined, setJoined] = useState(false);
  const [previewAge, setPreviewAge] = useState<PreviewAge>("fresh");
  const [executing, setExecuting] = useState(false);
  const [fate, setFate] = useState<HitlFate>("idle");
  const [skipWeek, setSkipWeek] = useState(false);
  const [knowledgeTab, setKnowledgeTab] = useState<KnowledgeTab>("files");
  const [importPhase, setImportPhase] = useState<ImportPhase>("idle");
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
  const [thread, setThread] = useState<ChatTurn[]>([]);
  const [pendingCommit, setPendingCommit] = useState<PendingCommit | null>(null);
  const [pendingRosterCreate, setPendingRosterCreate] = useState(false);
  const [pendingRuntimeMemberId, setPendingRuntimeMemberId] = useState<string | null>(null);
  const [runtimeGen, setRuntimeGen] = useState<{ memberId: string; index: number } | null>(null);
  const [activeMemberId, setActiveMemberId] = useState<string | null>(null);
  const [configMemberId, setConfigMemberId] = useState<string | null>(null);
  const [memberConfigFromLive, setMemberConfigFromLive] = useState(false);
  const [focusedField, setFocusedField] = useState<{ field: string; label: string } | null>(null);
  const lastSynced = useRef<Record<string, string>>({});

  useEffect(() => {
    setFocusedField(null);
    setPendingCommit(null);
    setPendingRosterCreate(false);
    setPendingRuntimeMemberId(null);
  }, [scene]);

  useEffect(() => {
    if (!isSetupChat(scene)) return;
    setThread((current) => {
      if (current.length > 0) return current;
      return [
        {
          id: 1,
          author: "assistant",
          label: "助手 · 候选",
          text: SETUP_ASSISTANT_INTRO,
        },
      ];
    });
  }, [scene]);

  useEffect(() => {
    if (!runtimeGen) return;
    const { memberId, index } = runtimeGen;
    const sample = sampleRuntime(memberId);
    const reduced =
      typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    if (reduced) {
      setMembers((current) =>
        current.map((item) =>
          item.id === memberId ? { ...item, runtime: sample, initStatus: "ready" } : item,
        ),
      );
      for (const slot of sample) {
        lastSynced.current[`runtime:${memberId}:${slot.id}`] = slot.value;
      }
      setRuntimeGen(null);
      setComposerStatus("样品已齐。请确认此人已就位。没有 daemon 写入。");
      return;
    }

    const timer = window.setTimeout(() => {
      const filled = sample[index];
      if (!filled) {
        setRuntimeGen(null);
        return;
      }
      const last = index >= sample.length - 1;
      setMembers((current) =>
        current.map((item) => {
          if (item.id !== memberId) return item;
          return {
            ...item,
            runtime: item.runtime.map((slot) => (slot.id === filled.id ? { ...filled } : slot)),
            initStatus: last ? "ready" : "generating",
          };
        }),
      );
      lastSynced.current[`runtime:${memberId}:${filled.id}`] = filled.value;
      if (last) {
        setRuntimeGen(null);
        setComposerStatus("样品已齐。请确认此人已就位。没有 daemon 写入。");
      } else {
        setRuntimeGen({ memberId, index: index + 1 });
      }
    }, 420);
    return () => window.clearTimeout(timer);
  }, [runtimeGen]);

  const chatHidden = scene === "empty-home";
  const currentWizard = WIZARD_STEPS[wizardIndex] ?? WIZARD_STEPS[0];
  const appendTurns = (...turns: Array<Omit<ChatTurn, "id">>) => {
    setThread((current) => {
      let id = current[current.length - 1]?.id ?? 0;
      return [...current, ...turns.map((item) => ({ ...item, id: ++id }))];
    });
  };
  const setMemberText = (id: string, key: "duty" | "handoff", value: string) => {
    const patch = (list: MemberDraft[]) =>
      list.map((member) => (member.id === id ? { ...member, [key]: value } : member));
    setMembers(patch);
    setWeeklyLiveMembers(patch);
    setSiteMembers(patch);
  };
  const setMemberModel = (id: string, model: string) => {
    const patch = (current: MemberDraft[]) =>
      current.map((member) => {
        if (member.id !== id) return member;
        if (model === "unselected") {
          return { ...member, model, initStatus: "blocked" as const, joined: false };
        }
        const initStatus =
          member.initStatus === "confirmed"
            ? "partial"
            : member.initStatus === "blocked"
              ? "idle"
              : member.initStatus;
        return { ...member, model, initStatus };
      });
    setMembers(patch);
    setWeeklyLiveMembers(patch);
    setSiteMembers(patch);
  };
  const setSlotValue = (id: string, slotId: RuntimeSlotId, value: string) => {
    const patch = (current: MemberDraft[]) =>
      current.map((member) => {
        if (member.id !== id) return member;
        return {
          ...member,
          initStatus: nextInitAfterRuntimeEdit(member.initStatus),
          runtime: member.runtime.map((slot) =>
            slot.id === slotId
              ? {
                  ...slot,
                  value,
                  status: (value.trim().length === 0 ? "empty" : "draft") as SlotFill,
                }
              : slot,
          ),
        };
      });
    setMembers(patch);
    setWeeklyLiveMembers(patch);
    setSiteMembers(patch);
  };
  const openMemberConfig = (id: string, fromLive: boolean) => {
    setConfigMemberId(id);
    setMemberConfigFromLive(fromLive);
    if (id) setActiveMemberId(id);
    setScene("member-config");
  };
  const setStageDraft = (id: string, key: keyof StageDraft, value: string) => {
    setStageDrafts((current) => ({
      ...current,
      [id]: { ...(current[id] ?? defaultStageDrafts()[id]), [key]: value },
    }));
  };
  const liveFieldValue = (field: string): string => {
    if (field.startsWith("wizard:")) {
      return wizardValues[field.slice("wizard:".length) as WizardId] ?? "";
    }
    if (field.startsWith("member:")) {
      const [, id, key] = field.split(":");
      const member =
        members.find((item) => item.id === id) ??
        weeklyLiveMembers.find((item) => item.id === id) ??
        siteMembers.find((item) => item.id === id);
      if (!member) return "";
      return key === "handoff" ? member.handoff : member.duty;
    }
    if (field.startsWith("process:")) {
      const [, id, key] = field.split(":");
      const draft = stageDrafts[id];
      if (!draft || !isStageDraftKey(key)) return "";
      return draft[key];
    }
    if (field.startsWith("runtime:")) {
      const parts = field.split(":");
      const id = parts[1];
      const slotId = parts[2];
      const member =
        members.find((item) => item.id === id) ??
        weeklyLiveMembers.find((item) => item.id === id) ??
        siteMembers.find((item) => item.id === id);
      if (!member || !isRuntimeSlotId(slotId)) return "";
      return member.runtime.find((slot) => slot.id === slotId)?.value ?? "";
    }
    if (field === "test:note") return testNote;
    if (field === "joint:note") return jointNote;
    if (field === "add-member:name") return newName;
    if (field === "add-member:duty") return newDuty;
    if (field === "detail:name") return detailName;
    if (field === "detail:goal") return detailGoal;
    if (field === "detail:cycle") return detailCycle;
    if (field === "output:format") return outputFormatOverride ?? "";
    return "";
  };
  const applyFieldValue = (field: string, value: string) => {
    if (field.startsWith("wizard:")) {
      onEditWizardValue(field.slice("wizard:".length) as WizardId, value);
      return;
    }
    if (field.startsWith("member:")) {
      const [, id, key] = field.split(":");
      if (key === "duty" || key === "handoff") setMemberText(id, key, value);
      return;
    }
    if (field.startsWith("process:")) {
      const [, id, key] = field.split(":");
      if (isStageDraftKey(key)) setStageDraft(id, key, value);
      return;
    }
    if (field.startsWith("runtime:")) {
      const parts = field.split(":");
      const id = parts[1];
      const slotId = parts[2];
      if (id && isRuntimeSlotId(slotId)) setSlotValue(id, slotId, value);
      return;
    }
    if (field === "test:note") setTestNote(value);
    if (field === "joint:note") setJointNote(value);
    if (field === "add-member:name") setNewName(value);
    if (field === "add-member:duty") setNewDuty(value);
    if (field === "detail:name") setDetailName(value);
    if (field === "detail:goal") setDetailGoal(value);
    if (field === "detail:cycle") setDetailCycle(value);
    if (field === "output:format" && isOutputFormat(value)) {
      setOutputFormatOverride(value);
      setOutputSample((current) =>
        current === "empty" || current === "unknown" || current === "working" || current === "partial"
          ? value === "packet"
            ? "packet"
            : "document"
          : current,
      );
    }
  };
  const onFocusField = (field: string, label: string, value: string) => {
    setFocusedField({ field, label });
    if (!(field in lastSynced.current)) lastSynced.current[field] = value;
  };
  const requestCanvasCommit = (field: string, label: string, next: string) => {
    const previous = lastSynced.current[field] ?? next;
    if (previous === next) {
      setComposerStatus("没有改动，未发给助手。");
      return;
    }
    setPendingCommit({ field, label, previous, next });
  };
  const confirmPendingCommit = () => {
    if (!pendingCommit) return;
    lastSynced.current[pendingCommit.field] = pendingCommit.next;
    const suggestion = suggestRevision(pendingCommit.label, pendingCommit.next);
    appendTurns(
      {
        author: "owner",
        label: "你",
        text: `我把「${pendingCommit.label}」改成了：\n${pendingCommit.next}`,
      },
      {
        author: "assistant",
        label: "助手 · 候选",
        text: `收到。建议把「${pendingCommit.label}」收成更可核对的说法。确认后才写回画布。`,
        proposal: {
          field: pendingCommit.field,
          label: pendingCommit.label,
          value: suggestion,
          status: "pending",
        },
      },
    );
    setPendingCommit(null);
    setComposerStatus("已以你的名义写入对话。助手建议要你确认才改画布。");
  };
  const cancelPendingCommit = () => {
    if (!pendingCommit) return;
    applyFieldValue(pendingCommit.field, pendingCommit.previous);
    setPendingCommit(null);
    setComposerStatus("已还原画布，没有发给助手。");
  };
  const confirmRosterCreate = () => {
    const nextMembers = proposeMembersFromProcess(stageDrafts);
    setMembers(nextMembers);
    setActiveMemberId(nextMembers[0]?.id ?? null);
    setRuntimeGen(null);
    for (const member of nextMembers) {
      lastSynced.current[`member:${member.id}:duty`] = member.duty;
      lastSynced.current[`member:${member.id}:handoff`] = member.handoff;
      for (const slot of member.runtime) {
        lastSynced.current[`runtime:${member.id}:${slot.id}`] = slot.value;
      }
    }
    appendTurns(
      { author: "owner", label: "你", text: "根据业务流程创建成员" },
      { author: "assistant", label: "助手 · 候选", text: rosterAssistantText(nextMembers) },
    );
    setPendingRosterCreate(false);
    setPendingRuntimeMemberId(null);
    setComposerStatus("已按流程创建岗位。请选定模型，再逐人生成执行方式。没有 daemon 写入。");
  };
  const confirmGenerateRuntime = () => {
    const id = pendingRuntimeMemberId;
    if (!id) return;
    const member = members.find((item) => item.id === id);
    const name = member?.name ?? "当前岗位";
    setMembers((current) =>
      current.map((item) =>
        item.id === id ? { ...item, runtime: emptyRuntime(), initStatus: "generating" } : item,
      ),
    );
    for (const def of RUNTIME_SLOT_DEFS) {
      lastSynced.current[`runtime:${id}:${def.id}`] = "";
    }
    appendTurns(
      {
        author: "owner",
        label: "你",
        text: `请为「${name}」生成工作说明、工具、能力包、周期与触发、外部连接和文档范围`,
      },
      {
        author: "assistant",
        label: "助手 · 候选",
        text: `正在逐项写入「${name}」的目标态样品。画布只显示当前项标题。全文在配置页。没有安装。聊天不能批准。`,
      },
    );
    setPendingRuntimeMemberId(null);
    setRuntimeGen({ memberId: id, index: 0 });
    setComposerStatus("正在逐项写入本地样品。确认就位前不算权威。");
  };
  const confirmMemberSeated = (id: string) => {
    const member = members.find((item) => item.id === id);
    if (!member) return;
    setMembers((current) =>
      current.map((item) =>
        item.id === id
          ? {
              ...item,
              initStatus: "confirmed",
              joined: true,
              runtime: grantRuntimeOnConfirm(item.runtime),
            }
          : item,
      ),
    );
    appendTurns({
      author: "system",
      label: "画布回执",
      text: `「${member.name}」已就位。工作说明、工具、能力包、周期与触发、外部连接、文档范围已确认。聊天不能批准。`,
    });
    const index = members.findIndex((item) => item.id === id);
    const next = members[index + 1];
    if (next) setActiveMemberId(next.id);
    setComposerStatus("此人已就位。下一位可初始化。没有 daemon 写入。");
  };
  const sendToAssistant = () => {
    const text = drafts.trim();
    if (text.length === 0) return;
    if (scene === "project-outputs") {
      const project = projectById(currentProjectId);
      const selected = project.outputs.find((item) => item.id === selectedOutputId) ?? project.outputs[0];
      const currentFormat = outputFormatOverride ?? selected?.format ?? "document";
      const next = formatFromAsk(text, currentFormat);
      appendTurns(
        { author: "owner", label: "你", text },
        {
          author: "assistant",
          label: "助手 · 候选",
          text: `按这份产出，建议改成「${formatLabel(next)}」。确认后才换画布。聊天不能批准。`,
          proposal: {
            field: "output:format",
            label: "产出展示",
            value: next,
            status: "pending",
          },
        },
      );
      setDrafts("");
      setComposerStatus("已写入本地对话。换展示要你确认。没有 daemon 发送。");
      return;
    }
    const target =
      focusedField ??
      (scene === "create-init"
        ? { field: `wizard:${currentWizard.id}`, label: currentWizard.label }
        : null);
    appendTurns({ author: "owner", label: "你", text });
    if (target) {
      appendTurns({
        author: "assistant",
        label: "助手 · 候选",
        text: `按你的说法，建议改「${target.label}」。确认后写回画布。`,
        proposal: {
          field: target.field,
          label: target.label,
          value: suggestRevision(target.label, text),
          status: "pending",
        },
      });
    } else {
      appendTurns({
        author: "assistant",
        label: "助手 · 候选",
        text: "请先点画布上要改的字段。我不会在聊天里批准、验收或安装。",
      });
    }
    setDrafts("");
    setComposerStatus("已写入本地对话。没有 daemon 发送，没有权威。");
  };
  const applyProposal = (turnId: number) => {
    const turn = thread.find((item) => item.id === turnId);
    if (!turn?.proposal || turn.proposal.status !== "pending") return;
    applyFieldValue(turn.proposal.field, turn.proposal.value);
    lastSynced.current[turn.proposal.field] = turn.proposal.value;
    setThread((current) =>
      current.map((item) =>
        item.id === turnId && item.proposal
          ? { ...item, proposal: { ...item.proposal, status: "applied" } }
          : item,
      ),
    );
    appendTurns({
      author: "system",
      label: "画布已同步",
      text: `「${turn.proposal.label}」已按助手建议更新。聊天不能批准。`,
    });
    setComposerStatus("已按你的确认写回画布。");
  };
  const dismissProposal = (turnId: number) => {
    setThread((current) =>
      current.map((item) =>
        item.id === turnId && item.proposal
          ? { ...item, proposal: { ...item.proposal, status: "dismissed" } }
          : item,
      ),
    );
    setComposerStatus("未采用这条建议。画布未改。");
  };
  const pinnedField = (() => {
    if (focusedField) {
      return { ...focusedField, value: liveFieldValue(focusedField.field) };
    }
    if (scene === "create-init") {
      return {
        field: `wizard:${currentWizard.id}`,
        label: currentWizard.label,
        value: wizardValues[currentWizard.id],
      };
    }
    return null;
  })();
  const moveWizard = (index: number) => {
    if (index === wizardIndex || index < 0 || index >= WIZARD_STEPS.length) return;
    const label = WIZARD_STEPS[index].label;
    setWizardIndex(index);
    appendTurns({
      author: "system",
      label: "画布回执",
      text: index < wizardIndex ? `回到「${label}」。` : `进入「${label}」。`,
    });
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
    appendTurns({
      author: "system",
      label: "画布回执",
      text:
        step.id === "preview"
          ? `已确认「${step.label}」。项目仍未上线。`
          : `已确认「${step.label}」。下一项可用。`,
    });
  };

  const projectsCurrent =
    scene === "projects" ||
    CREATE_SCENES.includes(scene) ||
    scene === "project-detail" ||
    scene === "project-members" ||
    scene === "project-runs" ||
    scene === "project-outputs" ||
    scene === "add-member" ||
    scene === "member-config" ||
    scene === "hitl";
  const visibleProjectList = listedProjects(lifecycle, copied);
  const switcherProjects = visibleProjectList.filter((item) => item.kind === "live" || item.kind === "copy-draft");
  const resolvedProjectId: SampleProjectId =
    lifecycle === "creating"
      ? "creating-draft"
      : currentProjectId === "creating-draft" || (currentProjectId === "weekly-copy" && !copied)
        ? "weekly"
        : currentProjectId;
  const currentProject = projectById(resolvedProjectId);
  const workMembers = membersForProject(currentProject, members, weeklyLiveMembers, siteMembers);
  const sceneLabel =
    scene === "project-runs" ||
    scene === "project-members" ||
    scene === "project-detail" ||
    scene === "add-member" ||
    scene === "hitl" ||
    scene === "project-outputs"
      ? `${currentProject.name} · ${SCENE_TITLES[scene]}`
      : SCENE_TITLES[scene];
  const knowledgeOk = lifecycle === "live" || (lifecycle === "creating" && createGate >= 2);
  const locationLabel = (() => {
    if (scene === "member-config") {
      return memberConfigFromLive ? `项目 / ${currentProject.name}` : "项目 / 创建中";
    }
    if (CREATE_SCENES.includes(scene)) return "项目 / 创建中";
    if (
      scene === "project-runs" ||
      scene === "project-members" ||
      scene === "project-detail" ||
      scene === "add-member" ||
      scene === "hitl" ||
      scene === "project-outputs"
    ) {
      return `项目 / ${currentProject.name}`;
    }
    if (scene === "projects") return "项目";
    if (scene === "settings") return "设置";
    if (scene === "knowledge") return "知识";
    if (scene === "state-lab") return "原型质检";
    return "今日";
  })();

  const applyProjectId = (id: SampleProjectId) => {
    const next = projectById(id);
    setCurrentProjectId(id);
    setProcessStageId(next.currentStageId);
    setDetailName(next.name);
    setDetailGoal(next.goal);
    setDetailCycle(next.cycle);
    setOutputFormatOverride(null);
    setSelectedOutputId(null);
  };

  const openProjectWork = (id: SampleProjectId, dest: ProjectWorkScene) => {
    const next = projectById(id);
    applyProjectId(id);
    if (next.kind === "creating") {
      setScene(sceneForCreateGate(createGate));
      return;
    }
    if (dest === "project-outputs" && next.kind === "live") setOutputSample("document");
    setScene(dest);
  };

  const askOutputFormatChange = () => {
    const selected =
      currentProject.outputs.find((item) => item.id === selectedOutputId) ?? currentProject.outputs[0];
    const currentFormat = outputFormatOverride ?? selected?.format ?? "document";
    const next = nextOutputFormat(currentFormat);
    appendTurns(
      { author: "owner", label: "你", text: "请换一种展示。" },
      {
        author: "assistant",
        label: "助手 · 候选",
        text: `按这份产出，建议改成「${formatLabel(next)}」。确认后才换画布。聊天不能批准。`,
        proposal: {
          field: "output:format",
          label: "产出展示",
          value: next,
          status: "pending",
        },
      },
    );
    setComposerStatus("已写入本地对话。换展示要你确认。没有 daemon 发送。");
  };

  const applyScenario = (next: Scene) => {
    setScene(next);
    if (next === "empty-home") {
      setLifecycle("empty");
      setCreateGate(1);
      return;
    }
    if (CREATE_SCENES.includes(next) || next === "today-incomplete") {
      setLifecycle("creating");
      setCurrentProjectId("creating-draft");
      if (next === "create-init") setCreateGate(Math.max(createGate, 1));
      if (next === "create-process") setCreateGate(Math.max(createGate, 2));
      if (next === "create-members") setCreateGate(Math.max(createGate, 3));
      if (next === "create-test") setCreateGate(Math.max(createGate, 4));
      if (next === "create-joint") setCreateGate(Math.max(createGate, 5));
      return;
    }
    if (next === "settings" || next === "state-lab" || next === "member-config") return;
    setLifecycle("live");
    if (next === "project-outputs") setOutputSample("document");
    if (currentProjectId === "creating-draft") applyProjectId("weekly");
    else applyProjectId(currentProjectId);
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
          onProcess={() => { setCreateGate(2); setScene("create-process"); }}
          goSettings={() => setScene("settings")}
          onCommitField={requestCanvasCommit}
          onFocusField={onFocusField}
        />
      );
    }
    if (active === "create-members") {
      return (
        <CreateMembersScene
          members={members}
          setModel={setMemberModel}
          confirmRoster={() =>
            setMembers(members.map((member) => ({ ...member, joined: member.model !== "unselected" })))
          }
          onTest={() => { setCreateGate(4); setScene("create-test"); }}
          onRequestCreate={() => setPendingRosterCreate(true)}
          onRequestGenerate={(id) => setPendingRuntimeMemberId(id)}
          onConfirmMember={confirmMemberSeated}
          onRefuseMember={(id) => {
            setMembers((current) =>
              current.map((item) =>
                item.id === id
                  ? { ...item, joined: false, initStatus: "idle" as const }
                  : item,
              ),
            );
            setComposerStatus("已拒绝加入。此人未加入本项目。");
          }}
          onBackToProcess={() => { setCreateGate(2); setScene("create-process"); }}
          activeMemberId={activeMemberId}
          setActiveMemberId={setActiveMemberId}
          onViewConfig={(id) => openMemberConfig(id, false)}
          generatingSlotIndex={
            runtimeGen && runtimeGen.memberId === (activeMemberId ?? members[0]?.id)
              ? runtimeGen.index
              : null
          }
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
          onMembers={() => { setCreateGate(3); setScene("create-members"); }}
          stageDrafts={stageDrafts}
          setStageDraft={setStageDraft}
          onCommitField={requestCanvasCommit}
          onFocusField={onFocusField}
        />
      );
    }
    if (active === "create-test") {
      return (
        <CreateTestScene
          stageId={processStageId}
          setStageId={setProcessStageId}
          testState={testByStage[processStageId] ?? "idle"}
          setTestState={(value) =>
            setTestByStage((current) => ({ ...current, [processStageId]: value }))
          }
          onJoint={() => {
            const index = PROCESS_STAGES.findIndex((item) => item.id === processStageId);
            const next = PROCESS_STAGES[index + 1];
            if (next) setProcessStageId(next.id);
            else {
              setCreateGate(5);
              setScene("create-joint");
            }
          }}
          onBackToProcess={() => { setCreateGate(2); setScene("create-process"); }}
          testNote={testNote}
          setTestNote={setTestNote}
          members={members}
          onInitMember={(id) => {
            if (id) setActiveMemberId(id);
            setCreateGate(3);
            setScene("create-members");
          }}
          onViewConfig={(id) => openMemberConfig(id, false)}
          onCommitField={requestCanvasCommit}
          onFocusField={onFocusField}
        />
      );
    }
    if (active === "create-joint") {
      return (
        <CreateJointScene
          jointState={jointState}
          setJointState={setJointState}
          onAccept={() => { setLifecycle("live"); setScene("today"); }}
          onBackToTest={() => { setCreateGate(4); setScene("create-test"); }}
          onBackToProcess={() => { setCreateGate(2); setScene("create-process"); }}
          onBackToMembers={() => { setCreateGate(3); setScene("create-members"); }}
          jointNote={jointNote}
          setJointNote={setJointNote}
          onCommitField={requestCanvasCommit}
          onFocusField={onFocusField}
        />
      );
    }
    if (active === "today-incomplete") {
      return <TodayIncompleteScene gate={createGate} onContinue={() => setScene(sceneForCreateGate(createGate))} />;
    }
    if (active === "today") {
      return (
        <TodayLiveScene
          period={period}
          setPeriod={setPeriod}
          selectedRun={selectedRun}
          setSelectedRun={setSelectedRun}
          onDecision={() => setScene("hitl")}
          onProject={() => setScene("project-runs")}
          onDefer={() => {
            setHasDecision(true);
            setComposerStatus("这一件仍留在今日。没有消失。以后再说不是完成。");
          }}
          hasDecision={hasDecision}
        />
      );
    }
    if (active === "projects") {
      return (
        <ProjectsScene
          lifecycle={lifecycle}
          projects={visibleProjectList}
          copied={copied}
          onCreate={() => setScene("empty-home")}
          onCopy={() => {
            setCopied(true);
            applyProjectId("weekly-copy");
          }}
          onContinue={() => setScene(sceneForCreateGate(createGate))}
          onDetail={(id) => openProjectWork(id, "project-detail")}
          onMembers={(id) => openProjectWork(id, "project-members")}
          onRuns={(id) => openProjectWork(id, "project-runs")}
          onOutputs={(id) => openProjectWork(id, "project-outputs")}
        />
      );
    }
    if (active === "project-detail") {
      return (
        <ProjectDetailScene
          project={currentProject}
          switcherProjects={switcherProjects}
          onSwitch={(id) => openProjectWork(id, "project-detail")}
          name={detailName}
          setName={setDetailName}
          goal={detailGoal}
          setGoal={setDetailGoal}
          cycle={detailCycle}
          setCycle={setDetailCycle}
          onMembers={() => openProjectWork(currentProject.id, "project-members")}
          onRuns={() => openProjectWork(currentProject.id, "project-runs")}
          onOutputs={() => openProjectWork(currentProject.id, "project-outputs")}
          onCommitField={requestCanvasCommit}
          onFocusField={onFocusField}
        />
      );
    }
    if (active === "project-members") {
      return (
        <ProjectMembersScene
          project={currentProject}
          switcherProjects={switcherProjects}
          onSwitch={(id) => openProjectWork(id, "project-members")}
          members={workMembers}
          stageDrafts={stageDrafts}
          onAdd={() => setScene("add-member")}
          setModel={setMemberModel}
          setMemberText={setMemberText}
          setSlotValue={setSlotValue}
          onCommitField={requestCanvasCommit}
          onFocusField={onFocusField}
        />
      );
    }
    if (active === "project-runs") {
      return (
        <ProjectRunsScene
          project={currentProject}
          switcherProjects={switcherProjects}
          onSwitch={(id) => openProjectWork(id, "project-runs")}
          stageId={processStageId}
          setStageId={setProcessStageId}
          onHitl={() => setScene("hitl")}
          onClose={() => {
            setOutputSample("document");
            setScene("project-outputs");
          }}
        />
      );
    }
    if (active === "project-outputs") {
      return (
        <ProjectOutputsScene
          project={currentProject}
          switcherProjects={switcherProjects}
          onSwitch={(id) => openProjectWork(id, "project-outputs")}
          sample={outputSample}
          setSample={(value) => {
            setOutputSample(value);
            setOutputFormatOverride(null);
          }}
          selectedId={selectedOutputId}
          setSelectedId={(id) => {
            setSelectedOutputId(id);
            setOutputFormatOverride(null);
          }}
          formatOverride={outputFormatOverride}
          onAskFormat={askOutputFormatChange}
          onWork={() =>
            setScene(currentProject.kind === "creating" ? sceneForCreateGate(createGate) : "project-runs")
          }
          onHitl={() => setScene("hitl")}
        />
      );
    }
    if (active === "member-config") {
      const configMember =
        workMembers.find((item) => item.id === configMemberId) ??
        members.find((item) => item.id === configMemberId) ??
        workMembers.find((item) => item.id === activeMemberId) ??
        members.find((item) => item.id === activeMemberId) ??
        null;
      return (
        <MemberConfigScene
          member={configMember}
          project={currentProject}
          stageDrafts={stageDrafts}
          fromLive={memberConfigFromLive}
          onBack={() => setScene(memberConfigFromLive ? "project-members" : "create-members")}
          setModel={setMemberModel}
          setMemberText={setMemberText}
          setSlotValue={setSlotValue}
          onCommitField={requestCanvasCommit}
          onFocusField={onFocusField}
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
          members={workMembers}
          onJoin={() => {
            setJoined(true);
            const name = newName.trim();
            const added: MemberDraft = {
              id: `added-${name}`,
              name,
              duty: newDuty,
              handoff: "跟进记录",
              model: newModel,
              joined: true,
              initStatus: newModel === "unselected" ? "blocked" : "idle",
              runtime: emptyRuntime(),
            };
            const merge = (current: MemberDraft[]) => {
              const existing = current.find((item) => item.name === name);
              if (existing) {
                return current.map((item) =>
                  item.id === existing.id
                    ? { ...item, duty: newDuty, model: newModel, joined: true }
                    : item,
                );
              }
              return [...current, added];
            };
            setMembers(merge);
            if (currentProject.id === "site") setSiteMembers(merge);
            else setWeeklyLiveMembers(merge);
          }}
          onRefuse={() => {
            setJoined(false);
            setNewName("");
            setNewDuty("");
            setComposerStatus("已拒绝。新岗位未加入。");
          }}
          onOpenConfig={() => {
            const name = newName.trim();
            const id = `added-${name}`;
            openMemberConfig(id, true);
          }}
          onSettings={() => setScene("settings")}
          onCommitField={requestCanvasCommit}
          onFocusField={onFocusField}
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
          onBack={() => setScene("project-runs")}
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
          if (surface === "today" && (state === "working" || state === "partial")) {
            return (
              <TodayLiveScene
                period={period}
                setPeriod={setPeriod}
                selectedRun={selectedRun}
                setSelectedRun={setSelectedRun}
                onDecision={() => setScene("hitl")}
                onProject={() => setScene("project-runs")}
                onDefer={() => setComposerStatus("这一件仍留在今日。没有消失。")}
                hasDecision={true}
                hostState={state}
              />
            );
          }
          if (surface === "today" && state === "success") {
            return (
              <TodayLiveScene
                period={period}
                setPeriod={setPeriod}
                selectedRun={selectedRun}
                setSelectedRun={setSelectedRun}
                onDecision={() => setScene("hitl")}
                onProject={() => setScene("project-runs")}
                onDefer={() => undefined}
                hasDecision={false}
                hostState="success"
              />
            );
          }
          if (surface === "today" && (state === "loading" || state === "error" || state === "unknown" || state === "offline")) {
            return (
              <TodayLiveScene
                period={period}
                setPeriod={setPeriod}
                selectedRun={selectedRun}
                setSelectedRun={setSelectedRun}
                onDecision={() => setScene("hitl")}
                onProject={() => setScene("project-runs")}
                onDefer={() => undefined}
                hasDecision={true}
                hostState={state}
              />
            );
          }
          if (surface === "today" && state === "blocked") {
            return (
              <TodayLiveScene
                period={period}
                setPeriod={setPeriod}
                selectedRun={selectedRun}
                setSelectedRun={setSelectedRun}
                onDecision={() => setScene("hitl")}
                onProject={() => setScene("project-runs")}
                onDefer={() => undefined}
                hasDecision={true}
                hostState="blocked"
              />
            );
          }
          if (surface === "create") {
            if (state === "partial" || state === "working") {
              return (
                <CreateProcessScene
                  stageId={processStageId}
                  setStageId={setProcessStageId}
                  confirmedStages={confirmedStages}
                  confirmStage={(id) =>
                    setConfirmedStages(confirmedStages.includes(id) ? confirmedStages : [...confirmedStages, id])
                  }
                  onMembers={() => { setCreateGate(3); setScene("create-members"); }}
                  stageDrafts={stageDrafts}
                  setStageDraft={setStageDraft}
                  onCommitField={requestCanvasCommit}
                  onFocusField={onFocusField}
                />
              );
            }
            if (state === "blocked") {
              return (
                <CreateMembersScene
                  members={members}
                  setModel={setMemberModel}
                  confirmRoster={() =>
                    setMembers(members.map((member) => ({ ...member, joined: member.model !== "unselected" })))
                  }
                  onTest={() => { setCreateGate(4); setScene("create-test"); }}
                  onRequestCreate={() => setPendingRosterCreate(true)}
                  onRequestGenerate={(id) => setPendingRuntimeMemberId(id)}
                  onConfirmMember={confirmMemberSeated}
                  onRefuseMember={(id) => {
                    setMembers((current) =>
                      current.map((item) =>
                        item.id === id
                          ? { ...item, joined: false, initStatus: "idle" as const }
                          : item,
                      ),
                    );
                  }}
                  onBackToProcess={() => { setCreateGate(2); setScene("create-process"); }}
                  activeMemberId={activeMemberId}
                  setActiveMemberId={setActiveMemberId}
                  onViewConfig={(id) => openMemberConfig(id, false)}
                  generatingSlotIndex={
                    runtimeGen && runtimeGen.memberId === (activeMemberId ?? members[0]?.id)
                      ? runtimeGen.index
                      : null
                  }
                />
              );
            }
            if (state === "error" || state === "unknown" || state === "offline") {
              return (
                <CreateTestScene
                  stageId={processStageId}
                  setStageId={setProcessStageId}
                  testState={state === "unknown" ? "unknown" : state === "error" ? "fail" : "idle"}
                  setTestState={(value) =>
                    setTestByStage((current) => ({ ...current, [processStageId]: value }))
                  }
                  onJoint={() => setScene("create-joint")}
                  onBackToProcess={() => setScene("create-process")}
                  testNote={testNote}
                  setTestNote={setTestNote}
                  members={members}
                  onInitMember={(id) => {
                    if (id) setActiveMemberId(id);
                    setScene("create-members");
                  }}
                  onViewConfig={(id) => openMemberConfig(id, false)}
                  onCommitField={requestCanvasCommit}
                  onFocusField={onFocusField}
                  hostState={state === "offline" ? "offline" : state === "unknown" ? "unknown" : "error"}
                />
              );
            }
            if (state === "success") {
              return (
                <CreateJointScene
                  jointState="pass"
                  setJointState={setJointState}
                  onAccept={() => { setLifecycle("live"); setScene("today"); }}
                  onBackToTest={() => setScene("create-test")}
                  onBackToProcess={() => setScene("create-process")}
                  onBackToMembers={() => setScene("create-members")}
                  jointNote={jointNote}
                  setJointNote={setJointNote}
                  onCommitField={requestCanvasCommit}
                  onFocusField={onFocusField}
                />
              );
            }
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
                onProcess={() => { setCreateGate(2); setScene("create-process"); }}
                goSettings={() => setScene("settings")}
                onCommitField={requestCanvasCommit}
                onFocusField={onFocusField}
              />
            );
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
                showImportDemo
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
          if (surface === "knowledge" && (state === "error" || state === "unknown" || state === "offline" || state === "blocked")) {
            return (
              <KnowledgeScene
                locked={false}
                tab="import"
                setTab={setKnowledgeTab}
                memoryForgotten={memoryForgotten}
                forgetMemory={() => setMemoryForgotten(true)}
                draftOnly={false}
                importPhase={importPhase}
                setImportPhase={setImportPhase}
                hostState={state}
                showImportDemo={state === "error"}
              />
            );
          }
          if (surface === "projects") {
            return (
              <ProjectsScene
                lifecycle={state === "empty" ? "empty" : "live"}
                projects={state === "empty" ? [] : listedProjects("live", false)}
                copied={false}
                onCreate={() => setScene("empty-home")}
                onCopy={() => setCopied(true)}
                onContinue={() => setScene(sceneForCreateGate(createGate))}
                onDetail={(id) => openProjectWork(id, "project-detail")}
                onMembers={(id) => openProjectWork(id, "project-members")}
                onRuns={(id) => openProjectWork(id, "project-runs")}
                onOutputs={(id) => openProjectWork(id, "project-outputs")}
              />
            );
          }
          if (surface === "members") {
            return (
              <ProjectMembersScene
                project={state === "empty" ? projectById("weekly-copy") : projectById("weekly")}
                switcherProjects={listedProjects("live", true)}
                onSwitch={(id) => openProjectWork(id, "project-members")}
                members={state === "empty" ? [] : workMembers}
                stageDrafts={stageDrafts}
                onAdd={() => setScene("add-member")}
                setModel={setMemberModel}
                setMemberText={setMemberText}
                setSlotValue={setSlotValue}
                onCommitField={requestCanvasCommit}
                onFocusField={onFocusField}
              />
            );
          }
          if (surface === "runs") {
            return (
              <ProjectRunsScene
                project={state === "empty" || state === "offline" ? projectById("weekly-copy") : projectById("weekly")}
                switcherProjects={listedProjects("live", true)}
                onSwitch={(id) => openProjectWork(id, "project-runs")}
                stageId={processStageId}
                setStageId={setProcessStageId}
                onHitl={() => setScene("hitl")}
                onClose={() => {
            setOutputSample("document");
            setScene("project-outputs");
          }}
              />
            );
          }
          if (surface === "outputs") {
            const labSample: OutputSampleKey =
              state === "empty"
                ? "empty"
                : state === "working"
                  ? "working"
                  : state === "unknown"
                    ? "unknown"
                    : state === "partial"
                      ? "partial"
                      : state === "blocked" || state === "offline"
                        ? "packet"
                        : "document";
            return (
              <ProjectOutputsScene
                project={projectById("weekly")}
                switcherProjects={listedProjects("live", false)}
                onSwitch={(id) => openProjectWork(id, "project-outputs")}
                sample={labSample}
                setSample={setOutputSample}
                selectedId={selectedOutputId}
                setSelectedId={setSelectedOutputId}
                formatOverride={outputFormatOverride}
                onAskFormat={askOutputFormatChange}
                onWork={() => setScene("project-runs")}
                onHitl={() => setScene("hitl")}
              />
            );
          }
          if (surface === "hitl") {
            return (
              <HitlScene
                previewAge={state === "unknown" ? "unknown" : state === "offline" || state === "error" ? "stale" : "fresh"}
                setPreviewAge={setPreviewAge}
                executing={state === "working"}
                setExecuting={setExecuting}
                fate={state === "success" ? "approved" : "idle"}
                setFate={setFate}
                skipWeek={skipWeek}
                setSkipWeek={setSkipWeek}
                onBack={() => setScene("project-runs")}
                pending={state !== "empty"}
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
                hostState={state}
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
              <p>本组合尚未挂 native 真版式。unknown 不是 0 也不是成功。禁止把占位段当成已覆盖。</p>
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
        .opc-e2e .projects-submenu {
          display: flex;
          flex-direction: column;
          gap: 2px;
          min-width: 0;
          margin: 2px 0 10px 8px;
          padding: 4px 0 4px 8px;
          border-inline-start: 1px solid var(--line);
        }
        .opc-e2e .projects-submenu button {
          min-height: 44px;
          align-items: flex-start;
        }
        .opc-e2e .projects-submenu button > span {
          min-width: 0;
          display: -webkit-box;
          -webkit-line-clamp: 2;
          -webkit-box-orient: vertical;
          overflow: hidden;
        }
        .opc-e2e .submenu-empty {
          margin: 0;
          padding: 8px 10px;
          color: var(--muted);
          font-size: 12px;
        }
        .opc-e2e .project-switcher {
          display: grid;
          gap: 4px;
          min-width: 180px;
        }
        .opc-e2e .project-switcher span {
          color: var(--muted);
          font-size: 12px;
          font-weight: 650;
        }
        .opc-e2e .project-row-actions {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
          margin-block-start: 12px;
        }
        .opc-e2e .current-step {
          margin: 0 0 10px;
          font-weight: 650;
        }
        .opc-e2e .io-block {
          display: grid;
          gap: 8px;
          margin-block-start: 12px;
          padding-block-start: 12px;
          border-block-start: 1px solid var(--line);
        }
        .opc-e2e .io-block h3 {
          margin: 0;
          font-size: 14px;
        }
        .opc-e2e .media-sample {
          margin: 12px 0 0;
        }
        .opc-e2e .media-frame {
          min-height: 96px;
          border: 1px dashed var(--line-strong);
          border-radius: 6px;
          background: var(--fill);
        }
        .opc-e2e .media-frame-video {
          display: grid;
          place-items: center;
          color: var(--muted);
          font-size: 13px;
          font-weight: 650;
        }
        .opc-e2e .media-sample figcaption {
          margin: 6px 0 0;
          color: var(--muted);
          font-size: 12px;
        }
        .opc-e2e .sample-caption {
          margin: 0 0 10px;
          color: var(--muted);
          font-size: 12px;
        }
        .opc-e2e .output-kicker {
          margin: 0 0 10px;
          color: var(--muted);
          font-size: 12px;
        }
        .opc-e2e .output-chooser {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
          margin: 0 0 12px;
        }
        .opc-e2e .output-chooser button {
          min-height: 44px;
          border: 1px solid var(--line);
          border-radius: 6px;
          background: var(--surface);
          padding: 8px 10px;
        }
        .opc-e2e .output-chooser button[aria-current="true"] {
          border-color: var(--accent);
          background: var(--fill-strong);
          font-weight: 720;
        }
        .opc-e2e .output-chooser button:hover { background: var(--fill); }
        .opc-e2e .output-composition { min-width: 0; }
        .opc-e2e .output-lede { font-weight: 650; }
        .opc-e2e .output-document p { max-width: 62ch; }
        .opc-e2e .output-checklist {
          margin: 0;
          padding-inline-start: 1.2em;
        }
        .opc-e2e .output-checklist li { margin-block-end: 6px; }
        .opc-e2e .settings-nav {
          border-block-start: 1px solid var(--line);
          border-radius: 0;
          margin-block-start: 8px;
          padding-block-start: 13px;
        }
        .opc-e2e .main-column { min-width: 0; position: relative; }
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
        .opc-e2e .init-progress {
          display: grid;
          gap: 6px;
          margin: 0 0 12px;
        }
        .opc-e2e .init-progress strong { font-size: 13px; font-variant-numeric: tabular-nums; }
        .opc-e2e .init-progress span { color: var(--muted); font-size: 12px; }
        .opc-e2e .init-progress progress {
          width: 100%;
          height: 8px;
          accent-color: var(--accent);
        }
        .opc-e2e .init-now {
          display: grid;
          gap: 6px;
          margin: 8px 0 4px;
          min-width: 0;
        }
        .opc-e2e .init-kicker {
          margin: 0;
          color: var(--muted);
          font-size: 12px;
          font-weight: 680;
        }
        .opc-e2e .init-current-title {
          margin: 0;
          font-size: 22px;
          font-weight: 720;
          line-height: 1.25;
          text-wrap: pretty;
        }
        .opc-e2e .init-hint {
          margin: 0;
          color: var(--muted);
          font-size: 13px;
        }
        .opc-e2e .runtime-slots,
        .opc-e2e .seat-check {
          list-style: none;
          margin: 10px 0 0;
          padding: 0;
          display: grid;
          gap: 8px;
        }
        .opc-e2e .runtime-slots li,
        .opc-e2e .seat-check li {
          display: grid;
          grid-template-columns: minmax(0, 1fr) auto;
          gap: 6px 12px;
          align-items: start;
          border: 1px solid var(--line);
          border-radius: 7px;
          padding: 10px 12px;
        }
        .opc-e2e .runtime-slots li p {
          grid-column: 1 / -1;
          margin: 0;
          color: var(--muted);
          font-size: 13px;
        }
        .opc-e2e .runtime-slots small { display: block; color: var(--muted); font-weight: 400; }
        .opc-e2e .seat-check li { align-items: center; }
        .opc-e2e .field > span small { color: var(--muted); font-weight: 400; }
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
        .opc-e2e .field input,
        .opc-e2e .field textarea {
          width: 100%;
          border: 1px solid var(--line-strong);
          border-radius: 6px;
          background: var(--surface);
          padding: 9px 10px;
        }
        .opc-e2e .field textarea { min-height: 150px; resize: vertical; }
        .opc-e2e .table-field { margin-block-start: 0; }
        .opc-e2e .table-field input,
        .opc-e2e .table-field textarea { min-height: 72px; }
        .opc-e2e .visually-hidden {
          position: absolute;
          width: 1px;
          height: 1px;
          padding: 0;
          margin: -1px;
          overflow: hidden;
          clip: rect(0, 0, 0, 0);
          white-space: nowrap;
          border: 0;
        }
        .opc-e2e .edit-dialog-scrim {
          position: absolute;
          inset: 0;
          z-index: 6;
          display: grid;
          place-items: center;
          padding: 24px;
          background: color-mix(in srgb, var(--bg) 58%, transparent);
          overscroll-behavior: contain;
        }
        .opc-e2e .edit-dialog {
          width: min(100%, 480px);
          border: 1px solid var(--line-strong);
          border-radius: 10px;
          background: var(--surface);
          padding: 18px 18px 16px;
          box-shadow: 0 16px 40px color-mix(in srgb, var(--text) 12%, transparent);
          overscroll-behavior: contain;
        }
        .opc-e2e .edit-dialog h3 { margin-block-end: 8px; }
        .opc-e2e .edit-dialog p { color: var(--muted); }
        .opc-e2e .proposal-card {
          margin-block-start: 8px;
          border: 1px solid var(--line);
          border-radius: 6px;
          background: var(--fill);
          padding: 8px;
        }
        .opc-e2e .proposal-card small { margin-block-end: 8px; }
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
        .opc-e2e .flow-actions { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 12px; }
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
          grid-template-columns: minmax(168px, 32%) minmax(0, 1fr);
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
        .opc-e2e .member-list button:last-child { border-block-end-color: transparent; }
        .opc-e2e .member-list button:hover { background: var(--fill); }
        .opc-e2e .member-list button[aria-current="page"],
        .opc-e2e .member-list button[aria-selected="true"] { border-color: var(--line-strong); background: var(--fill-strong); }
        .opc-e2e .member-list strong,
        .opc-e2e .member-list small { display: block; }
        .opc-e2e .member-list small { color: var(--muted); }
        .opc-e2e .member-list button > span { min-width: 0; overflow-wrap: anywhere; }
        .opc-e2e .member-detail {
          display: grid;
          align-content: start;
          gap: 12px;
          min-width: 0;
          border: 1px solid var(--line-strong);
          border-radius: 7px;
          background: var(--surface);
          padding: 12px;
        }
        .opc-e2e .member-detail-empty {
          place-content: start;
          min-height: 220px;
        }
        .opc-e2e .member-detail-empty h3 { margin: 0; font-size: 16px; }
        .opc-e2e .member-detail-empty p { margin: 8px 0 0; color: var(--muted); }
        .opc-e2e .member-detail-head {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 10px;
          flex-wrap: wrap;
        }
        .opc-e2e .member-detail-head h3 { margin: 0; font-size: 16px; }
        .opc-e2e .member-detail-meta {
          margin: 4px 0 0;
          color: var(--muted);
          font-size: 12px;
        }
        .opc-e2e .member-detail .field { margin-block-start: 0; }
        .opc-e2e .member-detail .runtime-slots { margin: 0; }
        .opc-e2e .member-tab-panel { min-width: 0; }
        .opc-e2e .io-stack { display: grid; gap: 10px; }
        .opc-e2e .io-block h4 { margin: 0 0 6px; font-size: 13px; }
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
          justify-items: start;
          align-content: start;
          min-height: 0;
          text-align: start;
          padding: 8px 0 24px;
          max-width: 42rem;
        }
        .opc-e2e .empty-home h2 { margin: 8px 0; font-size: 22px; letter-spacing: -.02em; }
        .opc-e2e .empty-home p { margin: 0 0 14px; color: var(--muted); }
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
        .opc-e2e .process-axis button,
        .opc-e2e .process-axis .process-node {
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
        .opc-e2e .process-axis .process-node { min-height: 96px; }
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
        .opc-e2e .run-counts strong { display: block; margin: 4px 0 2px; font-size: 20px; font-variant-numeric: tabular-nums; }
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
        .opc-e2e .wizard-dot:disabled {
          opacity: .35;
          cursor: not-allowed;
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
          .opc-e2e .process-axis .process-node,
          .opc-e2e .wizard-card,
          .opc-e2e .output-composition,
          .opc-e2e .output-chooser button { border-color: var(--text); }
        }`}</style>

      <a className="skip-link" href="#opc-main">跳到主工作区</a>

      <header className="prototype-bar">
        <div className="prototype-title">
          <h1>个人 2.0 · OPC 端到端原型 · v9</h1>
          <span>三栏锁定。先选人再看配置。项目工作：详情 / 成员 / 运行 / 产出。</span>
        </div>
        <label className="scenario-select">
          <span>原型场景</span>
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
        <nav className="primary-nav" aria-label="一级导航">
          <div className="brand">个人</div>
          <button
            type="button"
            aria-current={scene === "today" || scene === "empty-home" || scene === "today-incomplete" ? "page" : undefined}
            onClick={onNavToday}
          >
            今日
            {scene === "today" && hasDecision ? <Tag tone="warn">1</Tag> : null}
          </button>
          <button
            type="button"
            aria-current={projectsCurrent ? "page" : undefined}
            onClick={() => setScene("projects")}
          >
            项目
          </button>
          {projectsCurrent && lifecycle === "live" ? (
            <nav className="projects-submenu" aria-label="项目去向">
              {PROJECT_SUBNAV.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  aria-current={
                    item.id === "project-detail"
                      ? scene === "project-detail"
                        ? "page"
                        : undefined
                      : item.id === "project-members"
                        ? scene === "project-members" ||
                            scene === "add-member" ||
                            scene === "member-config"
                          ? "page"
                          : undefined
                        : scene === item.id
                          ? "page"
                          : undefined
                  }
                  onClick={() => {
                    if (item.id === "project-outputs") setOutputSample("document");
                    setScene(item.id);
                  }}
                >
                  <span>{item.label}</span>
                </button>
              ))}
            </nav>
          ) : null}
          <button
            type="button"
            aria-current={scene === "knowledge" ? "page" : undefined}
            onClick={() => setScene("knowledge")}
          >
            知识
            {!knowledgeOk ? <Tag>锁</Tag> : null}
          </button>
          <div className="nav-space" />
          <button
            className="settings-nav"
            type="button"
            aria-current={scene === "settings" ? "page" : undefined}
            onClick={() => setScene("settings")}
          >
            设置
          </button>
        </nav>

        <main className="main-column" id="opc-main">
          <header className="context-header">
            <div>
              <p>{locationLabel}</p>
              <p className="scene-label">{sceneLabel}</p>
            </div>
            <div className="context-tools">
              <Tag
                tone={
                  scene === "state-lab" && (labState === "offline" || labState === "unknown")
                    ? "warn"
                    : scene === "state-lab" && labState === "error"
                      ? "bad"
                      : "neutral"
                }
              >
                {`Windows 本机 · ${
                  scene === "state-lab" && labState === "offline"
                    ? "离线 · 过时"
                    : scene === "state-lab" && labState === "unknown"
                      ? "说不清"
                      : scene === "state-lab" && labState === "error"
                        ? "连接失败"
                        : "在线时工作"
                }`}
              </Tag>
              <Tag tone="info">{chatHidden ? "对话已隐藏" : projectGroup(scene, memberConfigFromLive)}</Tag>
            </div>
          </header>
          <div className="main-content">{renderMain(scene)}</div>
          {pendingCommit ? (
            <EditConfirmDialog
              pending={pendingCommit}
              onConfirm={confirmPendingCommit}
              onCancel={cancelPendingCommit}
            />
          ) : pendingRosterCreate ? (
            <CreateRosterDialog
              replacing={members.length > 0}
              onConfirm={confirmRosterCreate}
              onCancel={() => setPendingRosterCreate(false)}
            />
          ) : pendingRuntimeMemberId ? (
            <GenerateRuntimeDialog
              memberName={members.find((item) => item.id === pendingRuntimeMemberId)?.name ?? "当前岗位"}
              onConfirm={confirmGenerateRuntime}
              onCancel={() => setPendingRuntimeMemberId(null)}
            />
          ) : null}
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
            activeField={pinnedField}
            thread={thread}
            onSendToAssistant={sendToAssistant}
            onApplyProposal={applyProposal}
            onDismissProposal={dismissProposal}
            participants={currentProject.participants}
            memberConfigFromLive={memberConfigFromLive}
          />
        )}
      </div>
    </div>
  );
}

function projectGroup(scene: Scene, memberConfigFromLive = false): string {
  if (isLiveProjectChat(scene, memberConfigFromLive)) return "项目群";
  return "个人助手";
}

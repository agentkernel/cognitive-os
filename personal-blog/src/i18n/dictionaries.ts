import type { Locale } from "@/i18n/config";

const dictionaries = {
  zh: {
    siteName: "CognitiveOS Research",
    siteShortName: "CognitiveOS Research",
    siteDescription:
      "关于可验证 Agent、确定性 authority 与工程证据边界的双语独立研究刊物。",
    skipToContent: "跳到正文",
    openMenu: "打开导航",
    closeMenu: "关闭导航",
    languageSwitch: "Read in English",
    sample: "示例内容",
    sampleNotice: "这是结构样例，不代表真实经历、客户、实现或业绩。",
    evidenceRail: "证据侧栏",
    source: "来源",
    sourceSnapshot: "研究快照",
    futureIdea: "后续选题",
    noLiveStatus: "静态语义图，不表示实时进度",
    nav: {
      home: "首页",
      articles: "Essays",
      projects: "Lab",
      cognitiveos: "Research",
      about: "Method",
      lab: "Lab",
      sources: "Sources",
    },
    home: {
      eyebrow: "CognitiveOS Research · 可验证 Agent 系统",
      title: "Agent 可以提出完成，但不能自行决定完成。",
      thesis:
        "CognitiveOS 研究如何用确定性授权、独立状态机、Effect 对账与 Verification/Acceptance，把概率推理约束进可检查的工程边界。",
      primaryAction: "阅读完整设计说明",
      secondaryAction: "查看研究总览",
      featuredMeta: "独立研究 · 双语 · 快照 b626e88",
      questionsHeading: "四个不能交给模型自证的问题",
      questionsDescription:
        "每个问题都落到独立 authority、固定状态版本和可定位证据，而不是提示词里的自我声明。",
      questions: [
        ["它能看到什么？", "ContextView 是受治理投影，不是事实或权限。"],
        ["它被允许做什么？", "Operation 描述与 capability 授权必须分离。"],
        ["动作真的发生了吗？", "Effect 保留未知结果，并要求先对账。"],
        [
          "任务真的完成了吗？",
          "Verification 与 Acceptance 分别给出证明和最终决定。",
        ],
      ],
      articleHeading: "从完整论证开始",
      articleDescription:
        "旗舰长文固定来源快照、开放差异和术语边界；任何“完成”都必须说明由谁、依据什么后态接受。",
      snapshotHeading: "研究状态不是产品仪表盘",
      snapshotDescription:
        "这些数字只描述 b626e88 快照中的规范登记和证据缺口，不表示实时进度，也不构成符合性声明。",
      sourcesAction: "检查来源与修订账本",
      methodHeading: "研究方法",
      methodDescription:
        "先区分事实、提议与 authority，再固定失败语义和证据口径；作者身份保持低调，来源边界保持公开。",
      methodAction: "阅读研究方法",
      articlesHeading: "已发布研究",
      articlesDescription: "只展示真实研究，不与结构样例混排。",
      allArticles: "查看全部文章",
      projectsHeading: "Lab",
      projectsDescription: "结构样例已从主旅程移出。",
      allProjects: "进入 Lab",
      aboutHeading: "研究方法",
      aboutDescription: "作者保持低调，来源与边界保持公开。",
      aboutLink: "阅读研究方法",
    },
    articles: {
      title: "Essays",
      description:
        "只收录可公开追溯的研究文章；结构样例已移入 Lab，不与真实研究混排。",
      featuredHeading: "已发布研究",
      featuredDescription: "完整论证、固定来源快照和明确的证据边界。",
      sampleHeading: "结构样例",
      sampleDescription: "仅在 Lab 中用于验证阅读系统。",
      emptyTitle: "下一篇文章仍在研究中",
      emptyBody: "未来文章会在来源、双语配对和证据边界都通过门禁后发布。",
      read: "阅读文章",
    },
    projects: {
      title: "Lab",
      description:
        "用于验证内容结构和交互的隔离样例区，不构成项目、履历或实现证据。",
      read: "查看结构样例",
    },
    lab: {
      title: "Lab",
      description:
        "这里保存双语文章和案例结构样例。它们全部 noindex，不进入 RSS 或 sitemap，也不代表真实经历、客户、实现或结果。",
      notice: "样例区 · 非研究发布面",
      articlesHeading: "文章结构样例",
      projectsHeading: "案例结构样例",
    },
    cognitiveos: {
      title: "CognitiveOS",
      description:
        "一份关于如何把概率推理约束进确定性授权、状态迁移、Effect 对账与验收边界的独立研究入口。",
      audience:
        "从这里建立共同词汇，再进入完整论证或来源账本。适合 Agent 平台、可靠性、安全和系统架构读者。",
      snapshot:
        "研究快照：commit b626e88，M1 进行中。273 条要求已登记；Lane-CTR 契约批已交付，但 REQ 级实现声明仍为 0，行为测试已执行 0，符合 Profile 0，76 个向量均为 not-run。",
      flagship: "阅读完整双语长文",
      sources: "检查来源与修订账本",
      diagramHeading: "视觉地图",
      futureHeading: "后续选题，不是已完成文章",
      ideas: [
        "Context cache 的治理维度如何进入可观测性",
        "Effect 对账与不可查询执行器的准入边界",
        "验证带宽如何成为部署一级约束",
      ],
    },
    about: {
      title: "Research Method",
      description:
        "CognitiveOS Research 由独立作者维护。个人身份有意保持简略，研究来源、失败边界和修订状态保持可检查。",
      methodHeading: "先建立不能越过的边界",
      method:
        "先分离事实、提议与 authority，再固定失败语义、状态版本和证据口径，最后才讨论界面、性能与产品叙事。",
      principlesHeading: "公开研究原则",
      principles: [
        "规范已登记、实现已提供、测试已执行和 Profile 已符合必须分别表述。",
        "远端 completed、receipt 或模型自述都不是最终验收。",
        "来源冲突保留差异，采用不扩大权限、范围、风险或完成声明的解释。",
      ],
      authorHeading: "作者位置",
      author:
        "作者以研究维护者身份低调存在，不使用虚构姓名、履历、客户或合作结果建立可信度。",
      boundaryHeading: "当前边界",
      boundary:
        "本刊物发布研究综合与结构样例，不代表 CognitiveOS 参考实现已经达到行为符合或生产就绪。",
      timelineHeading: "研究原则",
      contactHeading: "当前边界",
      contact: "本刊物不使用虚构身份或联系地址。",
    },
    notFound: {
      title: "没有找到这页",
      body: "地址可能无效，或对应语言的内容尚未发布。",
      action: "返回首页",
    },
    error: {
      title: "页面无法完成渲染",
      body: "请重新尝试；如果问题持续存在，请返回文章索引。",
      retry: "重新尝试",
      back: "返回文章",
    },
    footer:
      "CognitiveOS Research · 可验证 Agent 系统的独立双语研究。静态快照，不表示实时进度。",
  },
  en: {
    siteName: "CognitiveOS Research",
    siteShortName: "CognitiveOS Research",
    siteDescription:
      "An independent bilingual research publication about verifiable agents, deterministic authority, and evidence boundaries.",
    skipToContent: "Skip to main content",
    openMenu: "Open navigation",
    closeMenu: "Close navigation",
    languageSwitch: "中文阅读",
    sample: "Sample content",
    sampleNotice:
      "This is a structure sample, not a real role, client, implementation, or outcome.",
    evidenceRail: "Evidence rail",
    source: "Source",
    sourceSnapshot: "Research snapshot",
    futureIdea: "Future topic",
    noLiveStatus: "Static semantics, not live progress",
    nav: {
      home: "Home",
      articles: "Essays",
      projects: "Lab",
      cognitiveos: "Research",
      about: "Method",
      lab: "Lab",
      sources: "Sources",
    },
    home: {
      eyebrow: "CognitiveOS Research · verifiable agent systems",
      title: "Agents may propose completion. They may not decide it.",
      thesis:
        "CognitiveOS studies how deterministic authorization, independent state machines, Effect reconciliation, and Verification/Acceptance constrain probabilistic reasoning with inspectable engineering boundaries.",
      primaryAction: "Read the full design guide",
      secondaryAction: "Explore the research",
      featuredMeta: "Independent research · bilingual · snapshot b626e88",
      questionsHeading: "Four questions a model cannot answer for itself",
      questionsDescription:
        "Each question resolves to an independent authority, a fixed state version, and locatable evidence—not a claim inside a prompt.",
      questions: [
        [
          "What may it see?",
          "ContextView is a governed projection, not fact or permission.",
        ],
        [
          "What may it do?",
          "Operation description and capability authorization stay separate.",
        ],
        [
          "Did the action happen?",
          "Effect preserves unknown outcomes and requires reconciliation.",
        ],
        [
          "Is the task complete?",
          "Verification supplies proof; Acceptance makes the final decision.",
        ],
      ],
      articleHeading: "Start with the complete argument",
      articleDescription:
        "The flagship essay fixes a source snapshot, preserves open discrepancies, and names who may accept a completion claim against which post-state.",
      snapshotHeading: "Research status is not a product dashboard",
      snapshotDescription:
        "These figures describe specification registration and evidence gaps at b626e88. They are not live progress or a conformance claim.",
      sourcesAction: "Inspect sources and revisions",
      methodHeading: "Research method",
      methodDescription:
        "Separate fact, proposal, and authority first; fix failure semantics and evidence language next. The author stays understated while source boundaries stay public.",
      methodAction: "Read the research method",
      articlesHeading: "Published research",
      articlesDescription: "Only real research appears here.",
      allArticles: "Browse all essays",
      projectsHeading: "Lab",
      projectsDescription: "Structure samples are outside the primary journey.",
      allProjects: "Enter Lab",
      aboutHeading: "Research method",
      aboutDescription: "The author stays understated; sources and boundaries stay public.",
      aboutLink: "Read the research method",
    },
    articles: {
      title: "Essays",
      description:
        "Only traceable research appears here. Structural samples live in Lab and never mix with published work.",
      featuredHeading: "Published research",
      featuredDescription:
        "Complete arguments with fixed source snapshots and explicit evidence boundaries.",
      sampleHeading: "Structure samples",
      sampleDescription: "Available only in Lab to exercise the reading system.",
      emptyTitle: "The next essay is still under research",
      emptyBody:
        "Future work publishes only after sources, bilingual pairing, and evidence boundaries pass their gates.",
      read: "Read essay",
    },
    projects: {
      title: "Lab",
      description:
        "An isolated sample area for testing content structures and interactions—not project, résumé, or implementation evidence.",
      read: "View structure sample",
    },
    lab: {
      title: "Lab",
      description:
        "This area preserves bilingual article and case-study structure samples. Every entry is noindex, excluded from RSS and the sitemap, and makes no claim about real roles, clients, implementations, or outcomes.",
      notice: "Sample area · outside the research publication surface",
      articlesHeading: "Article structure samples",
      projectsHeading: "Case-study structure samples",
    },
    cognitiveos: {
      title: "CognitiveOS",
      description:
        "An independent research entry on constraining probabilistic reasoning with deterministic authorization, state transitions, Effect reconciliation, and acceptance.",
      audience:
        "Start here for the shared vocabulary, then enter the complete argument or the source ledger. Written for agent-platform, reliability, security, and systems readers.",
      snapshot:
        "Research snapshot: commit b626e88, M1 in progress. 273 requirements are specified; the Lane-CTR contract batch is delivered, but REQ-level implementation claims remain 0, behavior-executed 0, conformant profiles 0, and all 76 vectors are not-run.",
      flagship: "Read the complete bilingual article",
      sources: "Inspect sources and revisions",
      diagramHeading: "Visual atlas",
      futureHeading: "Future topics, not completed articles",
      ideas: [
        "Making governance dimensions observable in Context caches",
        "Effect reconciliation and admission for non-queryable executors",
        "Treating verification bandwidth as a deployment constraint",
      ],
    },
    about: {
      title: "Research Method",
      description:
        "CognitiveOS Research is maintained independently. Personal identity stays deliberately minimal; sources, failure boundaries, and revision state remain inspectable.",
      methodHeading: "Establish the boundaries first",
      method:
        "Separate fact, proposal, and authority first; fix failure semantics, state versions, and evidence language next; discuss interfaces, performance, and product narrative last.",
      principlesHeading: "Public research principles",
      principles: [
        "Specified, implementation-provided, behavior-executed, and Profile-conformant remain distinct claims.",
        "Remote completed, receipts, and model self-reports are never final acceptance.",
        "Source discrepancies remain visible; interpretation never enlarges authority, scope, risk, or completion claims.",
      ],
      authorHeading: "Author position",
      author:
        "The author appears only as the research maintainer and does not use invented identity, résumé, clients, or collaboration outcomes as credibility.",
      boundaryHeading: "Current boundary",
      boundary:
        "This publication contains research synthesis and isolated structure samples. It does not claim that the CognitiveOS reference implementation is behavior-conformant or production-ready.",
      timelineHeading: "Research principles",
      contactHeading: "Current boundary",
      contact: "This publication does not invent an identity or contact address.",
    },
    notFound: {
      title: "This page was not found",
      body: "The address may be invalid, or its translation has not been published.",
      action: "Return home",
    },
    error: {
      title: "The page could not finish rendering",
      body: "Try again. If the problem continues, return to the essay index.",
      retry: "Try again",
      back: "Back to essays",
    },
    footer:
      "CognitiveOS Research · independent bilingual work on verifiable agent systems. Static snapshots, never live progress.",
  },
} as const;

export type Dictionary = (typeof dictionaries)[Locale];

export function getDictionary(locale: Locale): Dictionary {
  return dictionaries[locale] as Dictionary;
}

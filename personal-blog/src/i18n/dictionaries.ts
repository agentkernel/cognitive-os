import type { Locale } from "@/i18n/config";

const dictionaries = {
  zh: {
    siteName: "未署名的系统设计笔记",
    siteShortName: "系统设计笔记",
    siteDescription: "关于可验证 Agent、可靠系统与工程边界的双语个人开发者博客。",
    skipToContent: "跳到正文",
    openMenu: "打开导航",
    closeMenu: "关闭导航",
    languageSwitch: "Read in English",
    sample: "示例内容",
    sampleNotice: "这是用于展示信息结构的示例内容，不代表真实经历、客户或业绩。",
    evidenceRail: "证据侧栏",
    source: "来源",
    sourceSnapshot: "研究快照",
    futureIdea: "后续选题",
    noLiveStatus: "静态语义图，不表示实时进度",
    nav: {
      home: "首页",
      articles: "文章",
      projects: "项目",
      cognitiveos: "CognitiveOS",
      about: "关于",
    },
    home: {
      eyebrow: "系统设计 · 可靠性 · Agent 治理",
      title: "把 Agent 的不确定行动，收敛为可检查的工程边界。",
      thesis:
        "我写系统设计、可靠性与开发工具，重点不是让模型说得更像完成，而是让状态、授权、Effect 与验收留下可复核的证据。",
      primaryAction: "阅读 CognitiveOS 代表作",
      featuredLabel: "代表作",
      featuredMeta: "代表作 · CognitiveOS · 研究快照 b626e88",
      projectsHeading: "案例与工具",
      projectsDescription: "示例案例按问题、约束、方案、结果与反思组织。",
      allProjects: "查看全部案例",
      articlesHeading: "近期文章",
      articlesDescription: "长文提供完整论证；短记只处理一个问题，并明确证据边界。",
      allArticles: "查看全部文章",
      aboutHeading: "写作者说明",
      aboutDescription: "当前身份与经历均为占位；方法与内容边界保持公开。",
      aboutLink: "查看方法与示例时间线",
    },
    articles: {
      title: "文章",
      description: "一篇可追溯的 CognitiveOS 长文，以及三篇明确标记的技术写作示例。",
      featuredHeading: "研究长文",
      featuredDescription: "完整论证、来源快照与可展开的证据边界。",
      sampleHeading: "示例短记",
      sampleDescription: "每篇只处理一个工程问题，用于验证阅读与内容结构。",
      emptyTitle: "这里还没有真实短文",
      emptyBody: "示例条目不会进入 RSS 或 sitemap；真实内容会在来源和身份就绪后发布。",
      read: "阅读文章",
    },
    projects: {
      title: "项目",
      description: "四个用于验证案例结构的双语示例；不包含公司、客户、收入或性能声明。",
      read: "查看案例",
    },
    cognitiveos: {
      title: "CognitiveOS",
      description:
        "一份关于如何把概率推理约束进确定性授权、状态迁移、Effect 对账与验收边界的原创研究入口。",
      snapshot:
        "研究快照：commit b626e88，M1 进行中。273 条要求已登记；Lane-CTR 契约批已交付，但 REQ 级实现声明仍为 0，行为测试已执行 0，符合 Profile 0，76 个向量均为 not-run。",
      flagship: "阅读完整双语长文",
      diagramHeading: "五张语义图",
      futureHeading: "后续选题，不是已完成文章",
      ideas: [
        "Context cache 的治理维度如何进入可观测性",
        "Effect 对账与不可查询执行器的准入边界",
        "验证带宽如何成为部署一级约束",
      ],
    },
    about: {
      title: "关于",
      description: "身份资料仍是占位。这里展示写作方法、合作边界与一条明确标注的示例时间线。",
      methodHeading: "工作方法",
      method:
        "先分离事实、提议与 authority，再固定失败语义和证据口径，最后才讨论界面与性能。",
      timelineHeading: "示例时间线",
      contactHeading: "联系",
      contact: "联系地址尚未配置。请勿将此页面当作真实履历。",
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
    footer: "本地静态博客原型。身份与案例区域均明确标注为示例内容。",
  },
  en: {
    siteName: "Unsigned System Design Notes",
    siteShortName: "System Design Notes",
    siteDescription:
      "A bilingual personal developer blog about verifiable agents, reliable systems, and engineering boundaries.",
    skipToContent: "Skip to main content",
    openMenu: "Open navigation",
    closeMenu: "Close navigation",
    languageSwitch: "中文阅读",
    sample: "Sample content",
    sampleNotice:
      "This is sample content for demonstrating the information structure. It does not represent real employment, clients, or results.",
    evidenceRail: "Evidence rail",
    source: "Source",
    sourceSnapshot: "Research snapshot",
    futureIdea: "Future topic",
    noLiveStatus: "Static semantics, not live progress",
    nav: {
      home: "Home",
      articles: "Articles",
      projects: "Projects",
      cognitiveos: "CognitiveOS",
      about: "About",
    },
    home: {
      eyebrow: "System design · reliability · agent governance",
      title: "Turn uncertain agent actions into inspectable engineering boundaries.",
      thesis:
        "I write about system design, reliability, and developer tools. The goal is not to make a model sound finished, but to leave reviewable evidence across state, authorization, Effects, and acceptance.",
      primaryAction: "Read the CognitiveOS flagship",
      featuredLabel: "Flagship",
      featuredMeta: "Featured essay · CognitiveOS · research snapshot b626e88",
      projectsHeading: "Cases and tools",
      projectsDescription:
        "Sample cases are structured as problem, constraints, approach, outcome, and reflection.",
      allProjects: "Browse all cases",
      articlesHeading: "Recent writing",
      articlesDescription:
        "Long-form work carries the full argument; each short note handles one question and names its evidence boundary.",
      allArticles: "Browse all writing",
      aboutHeading: "About this writer",
      aboutDescription:
        "Identity and experience remain placeholders; the working method and content boundaries stay explicit.",
      aboutLink: "See the method and sample timeline",
    },
    articles: {
      title: "Articles",
      description:
        "One traceable CognitiveOS long-form article and three clearly marked technical writing samples.",
      featuredHeading: "Research essay",
      featuredDescription:
        "A complete argument with a source snapshot and inspectable evidence boundaries.",
      sampleHeading: "Sample notes",
      sampleDescription:
        "Each note handles one engineering question and exists to exercise the reading system.",
      emptyTitle: "No real short notes yet",
      emptyBody:
        "Sample entries stay out of RSS and the sitemap. Real work will publish once sources and identity are ready.",
      read: "Read article",
    },
    projects: {
      title: "Projects",
      description:
        "Four bilingual samples used to verify the case-study structure, with no company, client, revenue, or performance claims.",
      read: "View case study",
    },
    cognitiveos: {
      title: "CognitiveOS",
      description:
        "An original research entry on constraining probabilistic reasoning with deterministic authorization, state transitions, Effect reconciliation, and acceptance.",
      snapshot:
        "Research snapshot: commit b626e88, M1 in progress. 273 requirements are specified; the Lane-CTR contract batch is delivered, but REQ-level implementation claims remain 0, behavior-executed 0, conformant profiles 0, and all 76 vectors are not-run.",
      flagship: "Read the complete bilingual article",
      diagramHeading: "Five semantic diagrams",
      futureHeading: "Future topics, not completed articles",
      ideas: [
        "Making governance dimensions observable in Context caches",
        "Effect reconciliation and admission for non-queryable executors",
        "Treating verification bandwidth as a deployment constraint",
      ],
    },
    about: {
      title: "About",
      description:
        "Identity details remain placeholders. This page demonstrates the working method, collaboration boundaries, and a clearly labeled sample timeline.",
      methodHeading: "Working method",
      method:
        "Separate facts, proposals, and authority first; fix failure semantics and evidence language next; discuss interfaces and performance last.",
      timelineHeading: "Sample timeline",
      contactHeading: "Contact",
      contact: "No contact address is configured. Do not treat this page as a real résumé.",
    },
    notFound: {
      title: "This page was not found",
      body: "The address may be invalid, or its translation has not been published.",
      action: "Return home",
    },
    error: {
      title: "The page could not finish rendering",
      body: "Try again. If the problem continues, return to the article index.",
      retry: "Try again",
      back: "Back to articles",
    },
    footer:
      "Local static blog prototype. Identity and case-study areas are explicitly marked as sample content.",
  },
} as const;

export type Dictionary = (typeof dictionaries)[Locale];

export function getDictionary(locale: Locale): Dictionary {
  return dictionaries[locale] as Dictionary;
}

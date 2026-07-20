import type { Locale } from "@/i18n/config";

type LocalizedText = Record<Locale, string>;

export const authorProfile = {
  id: "placeholder-author",
  placeholder: true,
  initials: "SD",
  name: {
    zh: "示例作者",
    en: "Sample Author",
  },
  title: {
    zh: "独立开发者与系统设计写作者（示例）",
    en: "Independent developer and system design writer (sample)",
  },
  bio: {
    zh: "关注 Agent 治理、可靠性与开发者工具。身份、所在地与合作经历尚未由站点所有者提供。",
    en: "Focused on agent governance, reliability, and developer tools. Identity, location, and collaboration history have not yet been provided by the site owner.",
  },
  location: {
    zh: "所在地未提供",
    en: "Location not provided",
  },
  contact: null,
} satisfies {
  id: string;
  placeholder: true;
  initials: string;
  name: LocalizedText;
  title: LocalizedText;
  bio: LocalizedText;
  location: LocalizedText;
  contact: null;
};

export const sampleTimeline = [
  {
    id: "timeline-foundations",
    placeholder: true,
    period: "20XX",
    title: {
      zh: "示例：建立系统设计写作方法",
      en: "Sample: established a system-design writing method",
    },
    detail: {
      zh: "从来源层级、边界条件和反例出发组织技术论证。",
      en: "Structured technical arguments around source tiers, boundary conditions, and counterexamples.",
    },
  },
  {
    id: "timeline-agent-systems",
    placeholder: true,
    period: "20XX",
    title: {
      zh: "示例：研究可验证 Agent 工作流",
      en: "Sample: researched verifiable agent workflows",
    },
    detail: {
      zh: "练习把模型提议与授权、Effect、验证和验收分开表达。",
      en: "Practised separating model proposals from authorization, Effects, verification, and acceptance.",
    },
  },
  {
    id: "timeline-public-notes",
    placeholder: true,
    period: "20XX",
    title: {
      zh: "示例：整理双语公共笔记",
      en: "Sample: assembled bilingual public notes",
    },
    detail: {
      zh: "在不虚构履历或结果的前提下展示写作与信息架构。",
      en: "Demonstrated writing and information architecture without inventing a résumé or outcomes.",
    },
  },
] as const;

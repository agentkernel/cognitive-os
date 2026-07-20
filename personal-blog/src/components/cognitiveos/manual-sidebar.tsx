import Link from "next/link";
import { cognitiveOsSnapshot } from "@/data/cognitiveos";
import type { Locale } from "@/i18n/config";
import {
  cognitiveOsSourcesPath,
  flagshipPath,
  pagePath,
} from "@/i18n/routes";

type ManualSidebarProps = {
  locale: Locale;
  active: "overview" | "flagship" | "sources";
};

type SidebarItem = {
  label: string;
  href: string;
  current?: boolean;
};

type SidebarGroup = {
  label: string;
  items: SidebarItem[];
};

function groups(locale: Locale, active: ManualSidebarProps["active"]): SidebarGroup[] {
  const overview = pagePath(locale, "cognitiveos");
  const flagship = flagshipPath(locale);
  const sources = cognitiveOsSourcesPath(locale);
  const zh = locale === "zh";

  return [
    {
      label: zh ? "开始阅读" : "Start here",
      items: [
        {
          label: zh ? "专题总览" : "Overview",
          href: overview,
          current: active === "overview",
        },
        {
          label: zh ? "完整设计说明" : "Full design guide",
          href: flagship,
          current: active === "flagship",
        },
        {
          label: zh ? "来源与修订账本" : "Sources and revisions",
          href: sources,
          current: active === "sources",
        },
      ],
    },
    {
      label: zh ? "核心机制" : "Core mechanisms",
      items: [
        {
          label: zh ? "Authority 边界" : "Authority boundary",
          href: `${flagship}#authority-boundary`,
        },
        {
          label: "ContextView",
          href: `${flagship}#context-view`,
        },
        {
          label: zh ? "五个生命周期" : "Five lifecycles",
          href: `${flagship}#lifecycle-domains`,
        },
        {
          label: zh ? "Intent、Effect 与对账" : "Intent, Effect, reconciliation",
          href: `${flagship}#intent-effect`,
        },
        {
          label: zh ? "验证与验收" : "Verification and Acceptance",
          href: `${flagship}#verification-acceptance`,
        },
      ],
    },
    {
      label: zh ? "图解与参考" : "Diagrams and reference",
      items: [
        {
          label: zh ? "受治理任务线" : "Governed Flow Thread",
          href: `${overview}#cos-flow`,
        },
        {
          label: zh ? "五张架构图" : "Five architecture diagrams",
          href: `${overview}#cos-diagrams`,
        },
        {
          label: zh ? "后续选题" : "Future topics",
          href: `${overview}#cos-future`,
        },
      ],
    },
  ];
}

function SidebarNavigation({
  locale,
  active,
}: ManualSidebarProps) {
  const zh = locale === "zh";
  return (
    <nav
      className="manual-sidebar__navigation"
      aria-label={zh ? "CognitiveOS 说明书目录" : "CognitiveOS manual"}
    >
      {groups(locale, active).map((group) => (
        <section className="manual-sidebar__group" key={group.label}>
          <p>{group.label}</p>
          <ul>
            {group.items.map((item) => (
              <li key={item.href}>
                <Link
                  href={item.href}
                  aria-current={item.current ? "page" : undefined}
                >
                  {item.label}
                </Link>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </nav>
  );
}

export function CognitiveOsManualSidebar({
  locale,
  active,
}: ManualSidebarProps) {
  const zh = locale === "zh";
  const title = zh ? "CognitiveOS 设计说明" : "CognitiveOS design manual";

  return (
    <>
      <aside className="manual-sidebar">
        <p className="manual-sidebar__eyebrow">
          {zh ? "说明书 / 草案快照" : "Manual / draft snapshot"}
        </p>
        <strong>{title}</strong>
        <SidebarNavigation locale={locale} active={active} />
        <div className="manual-sidebar__status">
          <span>{cognitiveOsSnapshot.milestone}</span>
          <span>{cognitiveOsSnapshot.vectorsNotRun} vectors · not-run</span>
          <span>{cognitiveOsSnapshot.commit.slice(0, 7)}</span>
        </div>
      </aside>

      <details className="manual-sidebar-mobile">
        <summary>{title}</summary>
        <SidebarNavigation locale={locale} active={active} />
      </details>
    </>
  );
}

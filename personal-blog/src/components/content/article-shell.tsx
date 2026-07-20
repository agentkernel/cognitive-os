import Image from "next/image";
import Link from "next/link";
import { cognitiveOsSnapshot } from "@/data/cognitiveos";
import { CognitiveOsManualSidebar } from "@/components/cognitiveos/manual-sidebar";
import { ArticleInteractions } from "@/components/content/article-interactions";
import { JsonLd } from "@/components/seo/json-ld";
import { PageScaffold } from "@/components/layout/page-scaffold";
import type { Locale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { formatDate } from "@/i18n/format";
import {
  cognitiveOsSourcesPath,
  contentPath,
  pagePath,
} from "@/i18n/routes";
import type { ArticleEntry, ProjectEntry } from "@/lib/content/registry";
import { isPublishableFrontmatter } from "@/lib/content/publication";
import { absoluteUrl } from "@/lib/seo/metadata";

type ArticleShellProps = {
  locale: Locale;
  entry: ArticleEntry | ProjectEntry;
  alternatePath: string;
  currentPage: "articles" | "lab" | "cognitiveos";
};

const anchorLabels: Record<Locale, Record<string, string>> = {
  zh: {
    "completion-is-a-claim": "完成首先是待验证声明",
    "architecture-view": "总体架构是解释视图",
    "authority-boundary": "概率提议与 authority 边界",
    "context-view": "ContextView 不是事实捷径",
    "descriptor-capability": "操作描述与授权能力",
    "lifecycle-domains": "五个正交生命周期",
    "intent-effect": "Intent、Effect 与对账",
    "verification-acceptance": "验证与验收",
    "evidence-status": "当前证据状态",
    "reading-limits": "如何安全引用",
    premise: "先固定失败前提",
    unknown: "未知结果与重试",
    review: "人工介入与复核",
    identity: "翻译键与内容身份",
    anchors: "共享锚点",
    "build-gate": "构建门禁",
    "hard-soft": "硬边界与软信号",
    loss: "损失声明",
    questions: "设计问题",
    problem: "问题",
    constraints: "约束",
    approach: "方案",
    outcome: "结果",
    reflection: "反思",
  },
  en: {
    "completion-is-a-claim": "Completion as a claim",
    "architecture-view": "Architecture as explanation",
    "authority-boundary": "Probabilistic proposals and authority",
    "context-view": "ContextView is not a shortcut",
    "descriptor-capability": "Descriptor versus capability",
    "lifecycle-domains": "Five orthogonal lifecycles",
    "intent-effect": "Intent, Effect, and reconciliation",
    "verification-acceptance": "Verification and Acceptance",
    "evidence-status": "Current evidence status",
    "reading-limits": "How to cite the design",
    premise: "Start with the failure premise",
    unknown: "Unknown outcomes and retries",
    review: "Human review",
    identity: "Translation identity",
    anchors: "Shared anchors",
    "build-gate": "Build gate",
    "hard-soft": "Hard and soft constraints",
    loss: "Loss declaration",
    questions: "Design questions",
    problem: "Problem",
    constraints: "Constraints",
    approach: "Approach",
    outcome: "Outcome",
    reflection: "Reflection",
  },
};

function anchorLabel(anchor: string, locale: Locale): string {
  return (
    anchorLabels[locale][anchor] ||
    anchor
      .split("-")
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(" ")
  );
}

function ArticleTocLinks({
  anchors,
  locale,
  alternatePath,
}: {
  anchors: readonly string[];
  locale: Locale;
  alternatePath: string;
}) {
  return (
    <ol>
      {anchors.map((anchor) => (
        <li key={anchor}>
          <a href={`#${anchor}`}>{anchorLabel(anchor, locale)}</a>
          <Link
            href={`${alternatePath}#${anchor}`}
            hrefLang={locale === "zh" ? "en" : "zh-CN"}
            lang={locale === "zh" ? "en" : "zh-CN"}
            aria-label={
              locale === "zh"
                ? `在英文文章中打开 ${anchorLabel(anchor, locale)}`
                : `Open ${anchorLabel(anchor, locale)} in the Chinese article`
            }
          >
            {locale === "zh" ? "EN" : "中"}
          </Link>
        </li>
      ))}
    </ol>
  );
}

export function ArticleShell({
  locale,
  entry,
  alternatePath,
  currentPage,
}: ArticleShellProps) {
  const dictionary = getDictionary(locale);
  const { frontmatter, Component } = entry;
  const sectionLabel =
    currentPage === "lab"
      ? dictionary.nav.lab
      : currentPage === "cognitiveos"
        ? dictionary.nav.cognitiveos
        : dictionary.nav.articles;
  const sectionPath = pagePath(locale, currentPage);
  const canonicalPath = contentPath(locale, frontmatter);
  const contentStatus = frontmatter.placeholder
    ? dictionary.sample
    : locale === "zh"
      ? "研究综合"
      : "Research synthesis";

  const structuredData =
    isPublishableFrontmatter(frontmatter) && frontmatter.kind !== "project"
      ? {
          "@context": "https://schema.org",
          "@type": "TechArticle",
          headline: frontmatter.title,
          description: frontmatter.description,
          datePublished: frontmatter.publishedAt,
          dateModified: frontmatter.updatedAt || frontmatter.publishedAt,
          inLanguage: frontmatter.locale,
          mainEntityOfPage: absoluteUrl(canonicalPath),
          isPartOf: {
            "@type": "WebSite",
            name: dictionary.siteName,
            url: absoluteUrl(`/${locale}`),
          },
        }
      : null;

  const articleContent = (
    <ArticleInteractions>
        <nav className="breadcrumbs" aria-label={locale === "zh" ? "面包屑" : "Breadcrumbs"}>
          <Link href={pagePath(locale, "home")}>{dictionary.nav.home}</Link>
          <span aria-hidden="true">/</span>
          <Link href={sectionPath}>{sectionLabel}</Link>
          <span aria-hidden="true">/</span>
          <span aria-current="page">{frontmatter.title}</span>
        </nav>
        <header className="article-header">
          <div className="article-header__copy">
            <div className="article-header__meta">
              <span>{sectionLabel}</span>
              <time dateTime={frontmatter.publishedAt}>
                {formatDate(frontmatter.publishedAt, locale)}
              </time>
              <span>{contentStatus}</span>
            </div>
            <h1>{frontmatter.title}</h1>
            <p className="article-deck">{frontmatter.description}</p>
            {frontmatter.kind !== "project" ? (
              <ul
                className="article-tags"
                aria-label={locale === "zh" ? "主题标签" : "Topic tags"}
              >
                {frontmatter.tags.map((tag) => (
                  <li key={tag}>{tag}</li>
                ))}
              </ul>
            ) : null}
          </div>
          <div className="article-hero">
            <Image
              src={frontmatter.hero.src}
              alt={frontmatter.hero.alt}
              width={1600}
              height={900}
              preload={frontmatter.kind === "cognitiveos"}
              sizes={
                currentPage === "cognitiveos"
                  ? "(max-width: 1100px) calc(100vw - 32px), (max-width: 1500px) calc(100vw - 320px), 1100px"
                  : "(max-width: 1100px) calc(100vw - 32px), 34vw"
              }
            />
          </div>
        </header>
        <div className="article-grid">
          <div className="article-body prose">
            <details className="mobile-article-toc">
              <summary>{locale === "zh" ? "本页目录" : "On this page"}</summary>
              <ArticleTocLinks
                anchors={frontmatter.anchors}
                locale={locale}
                alternatePath={alternatePath}
              />
            </details>
            <Component />
          </div>
          <aside className="evidence-rail" aria-label={dictionary.evidenceRail}>
            <div className="evidence-rail__block">
              <span>{dictionary.sourceSnapshot}</span>
              <strong>
                {frontmatter.kind === "cognitiveos"
                  ? `${cognitiveOsSnapshot.commit.slice(0, 7)} · ${cognitiveOsSnapshot.milestone}`
                  : frontmatter.pairingSnapshot}
              </strong>
            </div>
            <dl>
              <div>
                <dt>{locale === "zh" ? "发布日期" : "Published"}</dt>
                <dd>
                  <time dateTime={frontmatter.publishedAt}>
                    {formatDate(frontmatter.publishedAt, locale)}
                  </time>
                </dd>
              </div>
              {frontmatter.updatedAt ? (
                <div>
                  <dt>{locale === "zh" ? "更新日期" : "Updated"}</dt>
                  <dd>
                    <time dateTime={frontmatter.updatedAt}>
                      {formatDate(frontmatter.updatedAt, locale)}
                    </time>
                  </dd>
                </div>
              ) : null}
              <div>
                <dt>{locale === "zh" ? "内容状态" : "Content status"}</dt>
                <dd>{contentStatus}</dd>
              </div>
              {frontmatter.kind !== "project" ? (
                <div>
                  <dt>{locale === "zh" ? "主题" : "Topic"}</dt>
                  <dd>{frontmatter.primaryTopic}</dd>
                </div>
              ) : null}
              <div>
                <dt>{locale === "zh" ? "许可" : "License"}</dt>
                <dd>{frontmatter.hero.license}</dd>
              </div>
            </dl>
            {frontmatter.kind === "cognitiveos" ? (
              <Link
                className="evidence-rail__source"
                href={cognitiveOsSourcesPath(locale)}
              >
                {dictionary.cognitiveos.sources}
              </Link>
            ) : null}
            <nav aria-label={locale === "zh" ? "文章目录" : "On this page"}>
              <span>{locale === "zh" ? "共享锚点" : "Shared anchors"}</span>
              <ArticleTocLinks
                anchors={frontmatter.anchors}
                locale={locale}
                alternatePath={alternatePath}
              />
            </nav>
          </aside>
        </div>
        <nav
          className="article-footer-navigation"
          aria-label={locale === "zh" ? "文章后续导航" : "Article navigation"}
        >
          <Link href={sectionPath}>
            <span aria-hidden="true">←</span>
            {locale === "zh" ? `返回${sectionLabel}` : `Back to ${sectionLabel}`}
          </Link>
          {frontmatter.kind === "cognitiveos" ? (
            <Link href={cognitiveOsSourcesPath(locale)}>
              {dictionary.nav.sources}
              <span aria-hidden="true">↗</span>
            </Link>
          ) : (
            <Link href={pagePath(locale, "cognitiveos")}>
              {dictionary.nav.cognitiveos}
              <span aria-hidden="true">↗</span>
            </Link>
          )}
          <Link
            href={alternatePath}
            hrefLang={locale === "zh" ? "en" : "zh-CN"}
            lang={locale === "zh" ? "en" : "zh-CN"}
          >
            {dictionary.languageSwitch}
          </Link>
          <a href="#article-top">
            {locale === "zh" ? "返回顶部" : "Back to top"}
            <span aria-hidden="true">↑</span>
          </a>
        </nav>
    </ArticleInteractions>
  );

  return (
    <PageScaffold
      locale={locale}
      currentPage={currentPage}
      alternatePath={alternatePath}
    >
      {structuredData ? <JsonLd data={structuredData} /> : null}
      {currentPage === "cognitiveos" ? (
        <div className="manual-layout">
          <CognitiveOsManualSidebar locale={locale} active="flagship" />
          <div className="manual-content">{articleContent}</div>
        </div>
      ) : (
        articleContent
      )}
    </PageScaffold>
  );
}

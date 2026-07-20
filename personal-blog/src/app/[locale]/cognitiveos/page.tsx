import type { Metadata } from "next";
import Link from "next/link";
import { CognitiveOsManualSidebar } from "@/components/cognitiveos/manual-sidebar";
import { GovernedFlowThread } from "@/components/content/governed-flow-thread";
import {
  AuthorityBoundaryDiagram,
  ContextPipelineDiagram,
  GovernedFlowDiagram,
  LifecycleDomainsDiagram,
  OverallArchitectureDiagram,
} from "@/components/diagrams/governance-diagrams";
import { PageScaffold } from "@/components/layout/page-scaffold";
import { JsonLd } from "@/components/seo/json-ld";
import { otherLocale, requireLocale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import {
  cognitiveOsSourcesPath,
  flagshipPath,
  pagePath,
} from "@/i18n/routes";
import { getFlagshipSummary } from "@/lib/content/registry";
import { absoluteUrl, createLocalizedMetadata } from "@/lib/seo/metadata";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const locale = requireLocale((await params).locale);
  const dictionary = getDictionary(locale);
  return createLocalizedMetadata({
    locale,
    title: dictionary.cognitiveos.title,
    description: dictionary.cognitiveos.description,
    path: pagePath(locale, "cognitiveos"),
    alternatePath: pagePath(otherLocale(locale), "cognitiveos"),
  });
}

export default async function CognitiveOsPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const locale = requireLocale((await params).locale);
  const dictionary = getDictionary(locale);
  const flagship = getFlagshipSummary(locale);
  const alternatePath = pagePath(otherLocale(locale), "cognitiveos");

  return (
    <PageScaffold
      locale={locale}
      currentPage="cognitiveos"
      alternatePath={alternatePath}
    >
      <JsonLd
        data={{
          "@context": "https://schema.org",
          "@type": "CollectionPage",
          name: dictionary.cognitiveos.title,
          description: dictionary.cognitiveos.description,
          inLanguage: locale === "zh" ? "zh-CN" : "en",
          url: absoluteUrl(pagePath(locale, "cognitiveos")),
          hasPart: [
            {
              "@type": "TechArticle",
              headline: flagship.frontmatter.title,
              url: absoluteUrl(flagshipPath(locale)),
            },
            {
              "@type": "Dataset",
              name:
                locale === "zh"
                  ? "CognitiveOS 研究资料簿"
                  : "CognitiveOS Research Sourcebook",
              url: absoluteUrl(cognitiveOsSourcesPath(locale)),
            },
          ],
        }}
      />
      <div className="manual-layout manual-layout--atlas">
        <CognitiveOsManualSidebar locale={locale} active="overview" />
        <div className="manual-content">
          <header className="page-intro">
            <div>
              <p className="eyebrow">
                {locale === "zh" ? "研究总览" : "RESEARCH OVERVIEW"}
              </p>
              <h1>{dictionary.cognitiveos.title}</h1>
            </div>
            <div>
              <p>{dictionary.cognitiveos.description}</p>
              <p className="page-intro__supporting">
                {dictionary.cognitiveos.audience}
              </p>
            </div>
          </header>
          <p className="snapshot-strip">{dictionary.cognitiveos.snapshot}</p>

          <section className="research-entrypoints" aria-labelledby="start-here">
            <header>
              <p className="eyebrow">{locale === "zh" ? "开始阅读" : "START HERE"}</p>
              <h2 id="start-here">
                {locale === "zh" ? "选择证据深度" : "Choose the evidence depth"}
              </h2>
            </header>
            <div>
              <article>
                <span aria-hidden="true">01</span>
                <h3>{flagship.frontmatter.title}</h3>
                <p>{flagship.frontmatter.description}</p>
                <Link className="text-link" href={flagshipPath(locale)}>
                  {dictionary.cognitiveos.flagship}
                </Link>
              </article>
              <article>
                <span aria-hidden="true">02</span>
                <h3>
                  {locale === "zh"
                    ? "来源、差异与修订状态"
                    : "Sources, discrepancies, and revision state"}
                </h3>
                <p>{dictionary.home.snapshotDescription}</p>
                <Link className="text-link" href={cognitiveOsSourcesPath(locale)}>
                  {dictionary.cognitiveos.sources}
                </Link>
              </article>
            </div>
          </section>

          <GovernedFlowThread locale={locale} />

          <section className="visual-atlas prose" aria-labelledby="cos-diagrams">
            <header className="visual-atlas__header">
              <p className="eyebrow">
                {locale === "zh" ? "宽版图解" : "FULL-WIDTH DIAGRAMS"}
              </p>
              <h2 id="cos-diagrams">{dictionary.cognitiveos.diagramHeading}</h2>
              <p>
                {locale === "zh"
                  ? "图解只表达结构关系；颜色、位置和路径都不代表实时运行状态。"
                  : "These diagrams explain structural relationships only; color, position, and paths never represent live runtime state."}
              </p>
            </header>
            <OverallArchitectureDiagram locale={locale} />
            <AuthorityBoundaryDiagram locale={locale} />
            <GovernedFlowDiagram locale={locale} />
            <LifecycleDomainsDiagram locale={locale} />
            <ContextPipelineDiagram locale={locale} />
          </section>

          <section className="future-research" aria-labelledby="cos-future">
            <header>
              <p className="eyebrow">
                {locale === "zh" ? "研究队列" : "RESEARCH QUEUE"}
              </p>
              <h2 id="cos-future">{dictionary.cognitiveos.futureHeading}</h2>
            </header>
            <ul>
              {dictionary.cognitiveos.ideas.map((idea) => (
                <li key={idea}>{idea}</li>
              ))}
            </ul>
          </section>
        </div>
      </div>
    </PageScaffold>
  );
}

import type { Metadata } from "next";
import Image from "next/image";
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
import { flagshipPath, pagePath } from "@/i18n/routes";
import { getFlagship } from "@/lib/content/registry";
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
  const flagship = getFlagship(locale);
  const alternatePath = pagePath(otherLocale(locale), "cognitiveos");

  return (
    <PageScaffold locale={locale} currentPage="cognitiveos" alternatePath={alternatePath}>
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
          ],
        }}
      />
      <div className="manual-layout">
        <CognitiveOsManualSidebar locale={locale} active="overview" />
        <div className="manual-content">
          <header className="page-intro">
            <h1>{dictionary.cognitiveos.title}</h1>
            <p>{dictionary.cognitiveos.description}</p>
          </header>
          <p className="snapshot-strip">{dictionary.cognitiveos.snapshot}</p>
          <figure className="topic-visual">
            <Image
              src="/images/ai/orthogonal-lifecycles-hero.avif"
              alt={
                locale === "zh"
                  ? "五条独立石墨轨道绕过铜色未知区域，在两道门槛前保持彼此分离"
                  : "Five independent graphite tracks route around a copper unknown region and remain separate before two gates"
              }
              width={1600}
              height={900}
              sizes="(max-width: 720px) 100vw, 86vw"
              priority
            />
            <figcaption>
              {locale === "zh"
                ? "五个生命周期保持正交；抽象图只表达关系，不表示实时状态。"
                : "Five lifecycles remain orthogonal; this abstraction expresses relationships, not live state."}
            </figcaption>
          </figure>

          <section className="home-section" aria-labelledby="cos-flagship">
            <header>
              <span>{locale === "zh" ? "代表作" : "FLAGSHIP ESSAY"}</span>
              <h2 id="cos-flagship">
                {locale === "zh" ? "完整论证" : "Complete argument"}
              </h2>
            </header>
            <div className="home-about">
              <p className="eyebrow">
                {locale === "zh" ? "真实研究内容" : "Research content"}
              </p>
              <h3>{flagship.frontmatter.title}</h3>
              <p>{flagship.frontmatter.description}</p>
              <Link className="text-link" href={flagshipPath(locale)}>
                {dictionary.cognitiveos.flagship}
              </Link>
            </div>
          </section>

          <section aria-labelledby="cos-flow">
            <h2 id="cos-flow" className="eyebrow">
              {locale === "zh" ? "受治理任务线" : "GOVERNED FLOW THREAD"}
            </h2>
            <GovernedFlowThread locale={locale} />
          </section>

          <section className="prose" aria-labelledby="cos-diagrams">
            <h2 id="cos-diagrams">{dictionary.cognitiveos.diagramHeading}</h2>
            <OverallArchitectureDiagram locale={locale} />
            <AuthorityBoundaryDiagram locale={locale} />
            <GovernedFlowDiagram locale={locale} />
            <LifecycleDomainsDiagram locale={locale} />
            <ContextPipelineDiagram locale={locale} />
          </section>

          <section className="home-section" aria-labelledby="cos-future">
            <header>
              <span>{locale === "zh" ? "后续选题" : "FUTURE TOPICS"}</span>
              <h2 id="cos-future">{dictionary.cognitiveos.futureHeading}</h2>
            </header>
            <ul className="future-list">
              {dictionary.cognitiveos.ideas.map((idea) => (
                <li key={idea}>
                  <span>
                    <strong>{dictionary.futureIdea}</strong>
                    <br />
                    {idea}
                  </span>
                </li>
              ))}
            </ul>
          </section>
        </div>
      </div>
    </PageScaffold>
  );
}

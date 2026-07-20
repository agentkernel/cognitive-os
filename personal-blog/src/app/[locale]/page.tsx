import type { Metadata } from "next";
import Link from "next/link";
import { ContentList } from "@/components/content/content-list";
import { GovernedFlowThread } from "@/components/content/governed-flow-thread";
import { PageScaffold } from "@/components/layout/page-scaffold";
import { JsonLd } from "@/components/seo/json-ld";
import { cognitiveOsSnapshot } from "@/data/cognitiveos";
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
    title: dictionary.siteName,
    description: dictionary.siteDescription,
    path: pagePath(locale, "home"),
    alternatePath: pagePath(otherLocale(locale), "home"),
  });
}

export default async function HomePage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const locale = requireLocale((await params).locale);
  const dictionary = getDictionary(locale);
  const flagship = getFlagshipSummary(locale);
  const alternatePath = pagePath(otherLocale(locale), "home");

  return (
    <PageScaffold
      locale={locale}
      currentPage="home"
      alternatePath={alternatePath}
    >
      <JsonLd
        data={{
          "@context": "https://schema.org",
          "@type": "WebSite",
          name: dictionary.siteName,
          description: dictionary.siteDescription,
          inLanguage: locale === "zh" ? "zh-CN" : "en",
          url: absoluteUrl(pagePath(locale, "home")),
        }}
      />

      <section className="home-hero" aria-labelledby="home-title">
        <div className="home-hero__copy">
          <p className="eyebrow">{dictionary.home.eyebrow}</p>
          <h1 id="home-title">{dictionary.home.title}</h1>
          <p className="home-hero__thesis">{dictionary.home.thesis}</p>
          <p className="home-hero__meta">{dictionary.home.featuredMeta}</p>
          <div className="hero-actions">
            <Link className="primary-action" href={flagshipPath(locale)}>
              {dictionary.home.primaryAction}
              <span aria-hidden="true">→</span>
            </Link>
            <Link
              className="secondary-action"
              href={pagePath(locale, "cognitiveos")}
            >
              {dictionary.home.secondaryAction}
            </Link>
          </div>
        </div>
        <aside className="home-hero__aside">
          <p>{locale === "zh" ? "研究前提" : "RESEARCH PREMISE"}</p>
          <strong>
            {locale === "zh"
              ? "完成是一项待验证、待验收的声明。"
              : "Completion is a claim awaiting verification and acceptance."}
          </strong>
          <span>
            {locale === "zh"
              ? "静态语义 · 非实时状态"
              : "Static semantics · never live status"}
          </span>
        </aside>
      </section>

      <section
        className="home-flow-section"
        aria-label={locale === "zh" ? "受治理任务线摘要" : "Governed flow summary"}
      >
        <GovernedFlowThread locale={locale} variant="compact" />
      </section>

      <section className="research-questions" aria-labelledby="research-questions">
        <header>
          <p className="eyebrow">{locale === "zh" ? "研究地图" : "RESEARCH MAP"}</p>
          <h2 id="research-questions">{dictionary.home.questionsHeading}</h2>
          <p>{dictionary.home.questionsDescription}</p>
        </header>
        <ol>
          {dictionary.home.questions.map(([question, answer], index) => (
            <li key={question}>
              <span aria-hidden="true">{String(index + 1).padStart(2, "0")}</span>
              <div>
                <h3>{question}</h3>
                <p>{answer}</p>
              </div>
            </li>
          ))}
        </ol>
      </section>

      <section className="home-section home-section--feature" aria-labelledby="home-essay">
        <header>
          <span>{locale === "zh" ? "旗舰研究" : "FLAGSHIP RESEARCH"}</span>
          <h2 id="home-essay">{dictionary.home.articleHeading}</h2>
          <p>{dictionary.home.articleDescription}</p>
          <Link className="section-index-link" href={pagePath(locale, "articles")}>
            {dictionary.home.allArticles}
            <span aria-hidden="true">→</span>
          </Link>
        </header>
        <ContentList
          entries={[flagship]}
          resolveHref={() => flagshipPath(locale)}
          actionLabel={dictionary.articles.read}
          sampleLabel={dictionary.sample}
          headingLevel="h3"
        />
      </section>

      <section className="research-status" aria-labelledby="research-status">
        <header>
          <p className="eyebrow">{locale === "zh" ? "证据状态" : "EVIDENCE STATUS"}</p>
          <h2 id="research-status">{dictionary.home.snapshotHeading}</h2>
          <p>{dictionary.home.snapshotDescription}</p>
          <Link className="text-link" href={cognitiveOsSourcesPath(locale)}>
            {dictionary.home.sourcesAction}
          </Link>
        </header>
        <dl>
          <div>
            <dt>{locale === "zh" ? "规范已登记" : "Specified"}</dt>
            <dd>{cognitiveOsSnapshot.requirementsSpecified}</dd>
          </div>
          <div>
            <dt>{locale === "zh" ? "实现声明" : "Implementation claims"}</dt>
            <dd>{cognitiveOsSnapshot.implementationProvidedRequirements}</dd>
          </div>
          <div>
            <dt>{locale === "zh" ? "行为已执行" : "Behavior executed"}</dt>
            <dd>{cognitiveOsSnapshot.behaviorExecuted}</dd>
          </div>
          <div>
            <dt>{locale === "zh" ? "符合 Profile" : "Conformant profiles"}</dt>
            <dd>{cognitiveOsSnapshot.conformantProfiles}</dd>
          </div>
        </dl>
      </section>

      <section className="home-section" aria-labelledby="home-method">
        <header>
          <span>{locale === "zh" ? "方法" : "METHOD"}</span>
          <h2 id="home-method">{dictionary.home.methodHeading}</h2>
        </header>
        <div className="home-about">
          <p>{dictionary.home.methodDescription}</p>
          <Link className="text-link" href={pagePath(locale, "about")}>
            {dictionary.home.methodAction}
          </Link>
        </div>
      </section>
    </PageScaffold>
  );
}

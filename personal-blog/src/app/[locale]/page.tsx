import type { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";
import { authorProfile } from "@content/data/profile";
import { ContentList } from "@/components/content/content-list";
import { PageScaffold } from "@/components/layout/page-scaffold";
import { JsonLd } from "@/components/seo/json-ld";
import { otherLocale, requireLocale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import {
  articlePath,
  flagshipPath,
  pagePath,
  projectPath,
} from "@/i18n/routes";
import {
  listArticles,
  listProjects,
  type ArticleEntry,
  type ProjectEntry,
} from "@/lib/content/registry";
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
  const recentArticles = listArticles(locale)
    .filter((entry) => entry.frontmatter.kind === "article")
    .slice(0, 2);
  const projects = listProjects(locale).slice(0, 2);
  const alternatePath = pagePath(otherLocale(locale), "home");

  return (
    <PageScaffold locale={locale} currentPage="home" alternatePath={alternatePath}>
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
          <Link className="primary-link" href={flagshipPath(locale)}>
            {dictionary.home.primaryAction}
            <svg viewBox="0 0 20 20" aria-hidden="true">
              <path d="M3 10h13M11 5l5 5-5 5" />
            </svg>
          </Link>
        </div>
        <figure className="home-hero__visual">
          <Image
            src="/images/ai/governed-trace-hero.avif"
            alt={
              locale === "zh"
                ? "蓝色证据路径穿过冷白纸面，在铜色断点前转向并通过两道石墨门槛"
                : "A blue evidence trace crosses cold white paper, detours at a copper interruption, and passes two graphite gates"
            }
            width={1600}
            height={900}
            priority
            sizes="(max-width: 980px) 100vw, 34vw"
          />
          <figcaption>
            {locale === "zh"
              ? "本地 AI 生成抽象图 · 无文字、Logo 或假界面 · AVIF/WebP"
              : "Local AI-generated abstraction · no text, logo, or fake UI · AVIF/WebP"}
          </figcaption>
        </figure>
      </section>

      <section className="home-section" aria-labelledby="home-articles">
        <header>
          <span>{locale === "zh" ? "技术写作" : "TECHNICAL WRITING"}</span>
          <h2 id="home-articles">{dictionary.home.articlesHeading}</h2>
          <p>{dictionary.home.articlesDescription}</p>
          <Link className="section-index-link" href={pagePath(locale, "articles")}>
            {dictionary.home.allArticles}
            <span aria-hidden="true">→</span>
          </Link>
        </header>
        <ContentList
          entries={recentArticles}
          resolveHref={(entry: ArticleEntry | ProjectEntry) =>
            articlePath(locale, entry.frontmatter.slug)
          }
          actionLabel={dictionary.articles.read}
          sampleLabel={dictionary.sample}
          headingLevel="h3"
        />
      </section>

      <section className="home-section" aria-labelledby="home-projects">
        <header>
          <span>{locale === "zh" ? "项目证据" : "PROJECT EVIDENCE"}</span>
          <h2 id="home-projects">{dictionary.home.projectsHeading}</h2>
          <p>{dictionary.home.projectsDescription}</p>
          <Link className="section-index-link" href={pagePath(locale, "projects")}>
            {dictionary.home.allProjects}
            <span aria-hidden="true">→</span>
          </Link>
        </header>
        <ContentList
          entries={projects}
          resolveHref={(entry: ArticleEntry | ProjectEntry) =>
            projectPath(locale, entry.frontmatter.slug)
          }
          actionLabel={dictionary.projects.read}
          sampleLabel={dictionary.sample}
          headingLevel="h3"
        />
      </section>

      <section className="home-section" aria-labelledby="home-about">
        <header>
          <span>{locale === "zh" ? "作者资料" : "AUTHOR CONTEXT"}</span>
          <h2 id="home-about">{dictionary.home.aboutHeading}</h2>
          <p>{dictionary.home.aboutDescription}</p>
        </header>
        <div className="home-about">
          <p className="eyebrow">{dictionary.sample}</p>
          <h3>{authorProfile.title[locale]}</h3>
          <p>{authorProfile.bio[locale]}</p>
          <Link className="text-link" href={pagePath(locale, "about")}>
            {dictionary.home.aboutLink}
          </Link>
        </div>
      </section>
    </PageScaffold>
  );
}

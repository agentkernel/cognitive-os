import Image from "next/image";
import Link from "next/link";
import { cognitiveOsSnapshot } from "@/data/cognitiveos";
import { JsonLd } from "@/components/seo/json-ld";
import { PageScaffold } from "@/components/layout/page-scaffold";
import type { Locale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { pagePath } from "@/i18n/routes";
import type { ArticleEntry, ProjectEntry } from "@/lib/content/registry";
import { absoluteUrl } from "@/lib/seo/metadata";

type ArticleShellProps = {
  locale: Locale;
  entry: ArticleEntry | ProjectEntry;
  alternatePath: string;
  currentPage: "articles" | "projects" | "cognitiveos";
};

export function ArticleShell({
  locale,
  entry,
  alternatePath,
  currentPage,
}: ArticleShellProps) {
  const dictionary = getDictionary(locale);
  const { frontmatter, Component } = entry;
  const sectionLabel =
    currentPage === "projects"
      ? dictionary.nav.projects
      : currentPage === "cognitiveos"
        ? dictionary.nav.cognitiveos
        : dictionary.nav.articles;
  const sectionPath = pagePath(locale, currentPage);
  const canonicalPath =
    currentPage === "cognitiveos"
      ? `/${locale}/cognitiveos/${frontmatter.slug}`
      : `/${locale}/${currentPage}/${frontmatter.slug}`;

  const structuredData =
    !frontmatter.placeholder && frontmatter.kind === "cognitiveos"
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

  return (
    <PageScaffold
      locale={locale}
      currentPage={currentPage}
      alternatePath={alternatePath}
    >
      {structuredData ? <JsonLd data={structuredData} /> : null}
      <article className="article-page">
        <nav className="breadcrumbs" aria-label={locale === "zh" ? "面包屑" : "Breadcrumbs"}>
          <Link href={pagePath(locale, "home")}>{dictionary.nav.home}</Link>
          <span aria-hidden="true">/</span>
          <Link href={sectionPath}>{sectionLabel}</Link>
          <span aria-hidden="true">/</span>
          <span aria-current="page">{frontmatter.title}</span>
        </nav>
        <header className="article-header">
          <div className="article-header__copy">
            <p className="eyebrow">
              {frontmatter.placeholder ? dictionary.sample : sectionLabel}
            </p>
            <h1>{frontmatter.title}</h1>
            <p className="article-deck">{frontmatter.description}</p>
          </div>
          <div className="article-hero">
            <Image
              src={frontmatter.hero.src}
              alt={frontmatter.hero.alt}
              width={1600}
              height={900}
              priority={frontmatter.kind === "cognitiveos"}
              sizes="(max-width: 980px) calc(100vw - 48px), 34vw"
            />
          </div>
        </header>
        <div className="article-grid">
          <div className="article-body prose">
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
                    {frontmatter.publishedAt}
                  </time>
                </dd>
              </div>
              <div>
                <dt>{locale === "zh" ? "内容状态" : "Content status"}</dt>
                <dd>{frontmatter.placeholder ? dictionary.sample : "Research synthesis"}</dd>
              </div>
              <div>
                <dt>{locale === "zh" ? "许可" : "License"}</dt>
                <dd>{frontmatter.hero.license}</dd>
              </div>
            </dl>
            <nav aria-label={locale === "zh" ? "文章目录" : "On this page"}>
              <span>{locale === "zh" ? "共享锚点" : "Shared anchors"}</span>
              <ol>
                {frontmatter.anchors.map((anchor) => (
                  <li key={anchor}>
                    <a href={`#${anchor}`}>{anchor.replaceAll("-", " ")}</a>
                    <Link
                      href={`${alternatePath}#${anchor}`}
                      hrefLang={locale === "zh" ? "en" : "zh-CN"}
                      aria-label={
                        locale === "zh"
                          ? `在英文文章中打开 ${anchor}`
                          : `Open ${anchor} in the Chinese article`
                      }
                    >
                      {locale === "zh" ? "EN" : "中"}
                    </Link>
                  </li>
                ))}
              </ol>
            </nav>
          </aside>
        </div>
      </article>
    </PageScaffold>
  );
}

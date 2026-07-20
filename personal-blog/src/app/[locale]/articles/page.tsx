import type { Metadata } from "next";
import { ContentList } from "@/components/content/content-list";
import { PageScaffold } from "@/components/layout/page-scaffold";
import { JsonLd } from "@/components/seo/json-ld";
import { otherLocale, requireLocale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { contentPath, pagePath } from "@/i18n/routes";
import { listPublishedContent } from "@/lib/content/registry";
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
    title: dictionary.articles.title,
    description: dictionary.articles.description,
    path: pagePath(locale, "articles"),
    alternatePath: pagePath(otherLocale(locale), "articles"),
  });
}

export default async function ArticlesPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const locale = requireLocale((await params).locale);
  const dictionary = getDictionary(locale);
  const entries = listPublishedContent(locale);
  const alternatePath = pagePath(otherLocale(locale), "articles");

  return (
    <PageScaffold
      locale={locale}
      currentPage="articles"
      alternatePath={alternatePath}
    >
      <JsonLd
        data={{
          "@context": "https://schema.org",
          "@type": "CollectionPage",
          name: dictionary.articles.title,
          description: dictionary.articles.description,
          inLanguage: locale === "zh" ? "zh-CN" : "en",
          url: absoluteUrl(pagePath(locale, "articles")),
          hasPart: entries.map((entry) => ({
            "@type": "TechArticle",
            headline: entry.frontmatter.title,
            url: absoluteUrl(contentPath(locale, entry.frontmatter)),
          })),
        }}
      />
      <div className="page-shell">
        <header className="page-intro">
          <div>
            <p className="eyebrow">
              {locale === "zh" ? "已发布研究" : "PUBLISHED RESEARCH"}
            </p>
            <h1>{dictionary.articles.title}</h1>
          </div>
          <p>{dictionary.articles.description}</p>
        </header>
        <section className="essay-index" aria-labelledby="published-essays">
          <header>
            <h2 id="published-essays">{dictionary.articles.featuredHeading}</h2>
            <p>{dictionary.articles.featuredDescription}</p>
          </header>
          <ContentList
            entries={entries}
            resolveHref={(entry) => contentPath(locale, entry.frontmatter)}
            actionLabel={dictionary.articles.read}
            sampleLabel={dictionary.sample}
            headingLevel="h3"
          />
        </section>
      </div>
    </PageScaffold>
  );
}

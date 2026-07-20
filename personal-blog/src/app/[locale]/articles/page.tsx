import type { Metadata } from "next";
import { ContentList } from "@/components/content/content-list";
import { PageScaffold } from "@/components/layout/page-scaffold";
import { JsonLd } from "@/components/seo/json-ld";
import { otherLocale, requireLocale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { articlePath, flagshipPath, pagePath } from "@/i18n/routes";
import {
  getFlagship,
  listArticles,
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
  const entries = listArticles(locale);
  const flagship = getFlagship(locale);
  const sampleEntries = entries.filter(
    (entry) => entry.frontmatter.kind === "article",
  );
  const alternatePath = pagePath(otherLocale(locale), "articles");

  return (
    <PageScaffold locale={locale} currentPage="articles" alternatePath={alternatePath}>
      <JsonLd
        data={{
          "@context": "https://schema.org",
          "@type": "CollectionPage",
          name: dictionary.articles.title,
          description: dictionary.articles.description,
          inLanguage: locale === "zh" ? "zh-CN" : "en",
          url: absoluteUrl(pagePath(locale, "articles")),
          hasPart: [
            {
              "@type": "TechArticle",
              headline: flagship.frontmatter.title,
              url: absoluteUrl(flagshipPath(locale)),
            },
          ],
        }}
      />
      <div className="page-shell">
        <header className="page-intro">
          <h1>{dictionary.articles.title}</h1>
          <p>{dictionary.articles.description}</p>
        </header>
        <section className="home-section" aria-labelledby="featured-essay">
          <header>
            <span>{locale === "zh" ? "已发布研究" : "PUBLISHED RESEARCH"}</span>
            <h2 id="featured-essay">{dictionary.articles.featuredHeading}</h2>
            <p>{dictionary.articles.featuredDescription}</p>
          </header>
          <ContentList
            entries={[flagship]}
            resolveHref={() => flagshipPath(locale)}
            actionLabel={dictionary.articles.read}
            sampleLabel={dictionary.sample}
            headingLevel="h3"
          />
        </section>
        <section className="home-section" aria-labelledby="sample-notes">
          <header>
            <span>{locale === "zh" ? "结构示例" : "STRUCTURE SAMPLES"}</span>
            <h2 id="sample-notes">{dictionary.articles.sampleHeading}</h2>
            <p>{dictionary.articles.sampleDescription}</p>
          </header>
          <ContentList
            entries={sampleEntries}
            resolveHref={(entry: ArticleEntry | ProjectEntry) =>
              articlePath(locale, entry.frontmatter.slug)
            }
            actionLabel={dictionary.articles.read}
            sampleLabel={dictionary.sample}
            headingLevel="h3"
          />
        </section>
      </div>
    </PageScaffold>
  );
}

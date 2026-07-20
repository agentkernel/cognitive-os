import type { Metadata } from "next";
import { ContentList } from "@/components/content/content-list";
import { PageScaffold } from "@/components/layout/page-scaffold";
import { otherLocale, requireLocale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { contentPath, pagePath } from "@/i18n/routes";
import { listLabContent } from "@/lib/content/registry";
import { createLocalizedMetadata } from "@/lib/seo/metadata";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const locale = requireLocale((await params).locale);
  const dictionary = getDictionary(locale);
  return createLocalizedMetadata({
    locale,
    title: dictionary.lab.title,
    description: dictionary.lab.description,
    path: pagePath(locale, "lab"),
    alternatePath: pagePath(otherLocale(locale), "lab"),
    noIndex: true,
  });
}

export default async function LabPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const locale = requireLocale((await params).locale);
  const dictionary = getDictionary(locale);
  const entries = listLabContent(locale);
  const articles = entries.filter((entry) => entry.frontmatter.kind === "article");
  const projects = entries.filter((entry) => entry.frontmatter.kind === "project");

  return (
    <PageScaffold
      locale={locale}
      currentPage="lab"
      alternatePath={pagePath(otherLocale(locale), "lab")}
    >
      <div className="page-shell">
        <header className="page-intro page-intro--lab">
          <div>
            <p className="eyebrow">{dictionary.lab.notice}</p>
            <h1>{dictionary.lab.title}</h1>
          </div>
          <p>{dictionary.lab.description}</p>
        </header>

        <section className="lab-section" aria-labelledby="lab-articles">
          <header>
            <span aria-hidden="true">A</span>
            <h2 id="lab-articles">{dictionary.lab.articlesHeading}</h2>
          </header>
          <ContentList
            entries={articles}
            resolveHref={(entry) => contentPath(locale, entry.frontmatter)}
            actionLabel={dictionary.articles.read}
            sampleLabel={dictionary.sample}
            headingLevel="h3"
          />
        </section>

        <section className="lab-section" aria-labelledby="lab-projects">
          <header>
            <span aria-hidden="true">B</span>
            <h2 id="lab-projects">{dictionary.lab.projectsHeading}</h2>
          </header>
          <ContentList
            entries={projects}
            resolveHref={(entry) => contentPath(locale, entry.frontmatter)}
            actionLabel={dictionary.projects.read}
            sampleLabel={dictionary.sample}
            headingLevel="h3"
          />
        </section>
      </div>
    </PageScaffold>
  );
}

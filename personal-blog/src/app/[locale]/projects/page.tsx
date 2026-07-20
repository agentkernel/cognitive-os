import type { Metadata } from "next";
import { ContentList } from "@/components/content/content-list";
import { PageScaffold } from "@/components/layout/page-scaffold";
import { otherLocale, requireLocale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { pagePath, projectPath } from "@/i18n/routes";
import {
  listProjects,
  type ArticleEntry,
  type ProjectEntry,
} from "@/lib/content/registry";
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
    title: dictionary.projects.title,
    description: dictionary.projects.description,
    path: pagePath(locale, "projects"),
    alternatePath: pagePath(otherLocale(locale), "projects"),
    noIndex: true,
  });
}

export default async function ProjectsPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const locale = requireLocale((await params).locale);
  const dictionary = getDictionary(locale);
  const entries = listProjects(locale);
  const alternatePath = pagePath(otherLocale(locale), "projects");

  return (
    <PageScaffold locale={locale} currentPage="projects" alternatePath={alternatePath}>
      <div className="page-shell">
        <header className="page-intro">
          <h1>{dictionary.projects.title}</h1>
          <div>
            <p>{dictionary.projects.description}</p>
            <p className="eyebrow">{dictionary.sample} / Sample content</p>
          </div>
        </header>
        <ContentList
          entries={entries}
          resolveHref={(entry: ArticleEntry | ProjectEntry) =>
            projectPath(locale, entry.frontmatter.slug)
          }
          actionLabel={dictionary.projects.read}
          sampleLabel={dictionary.sample}
        />
      </div>
    </PageScaffold>
  );
}

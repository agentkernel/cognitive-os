import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { ArticleShell } from "@/components/content/article-shell";
import { locales, otherLocale, requireLocale } from "@/i18n/config";
import { projectPath } from "@/i18n/routes";
import { getProject, projectSlugs } from "@/lib/content/registry";
import { createLocalizedMetadata } from "@/lib/seo/metadata";

export const dynamicParams = false;

export function generateStaticParams() {
  return locales.flatMap((locale) =>
    projectSlugs.map((slug) => ({
      locale,
      slug,
    })),
  );
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string; slug: string }>;
}): Promise<Metadata> {
  const { locale: rawLocale, slug } = await params;
  const locale = requireLocale(rawLocale);
  const entry = getProject(locale, slug);
  if (!entry) {
    notFound();
  }

  return createLocalizedMetadata({
    locale,
    title: entry.frontmatter.title,
    description: entry.frontmatter.description,
    path: projectPath(locale, slug),
    alternatePath: projectPath(otherLocale(locale), slug),
    noIndex: true,
  });
}

export default async function ProjectPage({
  params,
}: {
  params: Promise<{ locale: string; slug: string }>;
}) {
  const { locale: rawLocale, slug } = await params;
  const locale = requireLocale(rawLocale);
  const entry = getProject(locale, slug);
  if (!entry) {
    notFound();
  }

  return (
    <ArticleShell
      locale={locale}
      entry={entry}
      alternatePath={projectPath(otherLocale(locale), slug)}
      currentPage="projects"
    />
  );
}

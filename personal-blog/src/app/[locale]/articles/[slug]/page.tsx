import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { ArticleShell } from "@/components/content/article-shell";
import { locales, otherLocale, requireLocale } from "@/i18n/config";
import { contentPath } from "@/i18n/routes";
import {
  articleSlugs,
  getArticle,
  getArticleSummary,
  getTranslationSummary,
} from "@/lib/content/registry";
import { isPublishableFrontmatter } from "@/lib/content/publication";
import { createLocalizedMetadata } from "@/lib/seo/metadata";

export function generateStaticParams() {
  return locales.flatMap((locale) =>
    articleSlugs.map((slug) => ({
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
  const entry = getArticleSummary(locale, slug);
  if (!entry) {
    notFound();
  }
  const alternateLocale = otherLocale(locale);
  const alternate = getTranslationSummary(entry, alternateLocale);
  if (!alternate) {
    notFound();
  }

  return createLocalizedMetadata({
    locale,
    title: entry.frontmatter.title,
    description: entry.frontmatter.description,
    path: contentPath(locale, entry.frontmatter),
    alternatePath: contentPath(alternateLocale, alternate.frontmatter),
    noIndex: !isPublishableFrontmatter(entry.frontmatter),
    type: "article",
  });
}

export default async function ArticlePage({
  params,
}: {
  params: Promise<{ locale: string; slug: string }>;
}) {
  const { locale: rawLocale, slug } = await params;
  const locale = requireLocale(rawLocale);
  const entry = await getArticle(locale, slug);
  if (!entry) {
    notFound();
  }
  const alternateLocale = otherLocale(locale);
  const alternate = getTranslationSummary(entry, alternateLocale);
  if (!alternate) {
    notFound();
  }

  return (
    <ArticleShell
      locale={locale}
      entry={entry}
      alternatePath={contentPath(alternateLocale, alternate.frontmatter)}
      currentPage={
        isPublishableFrontmatter(entry.frontmatter) ? "articles" : "lab"
      }
    />
  );
}

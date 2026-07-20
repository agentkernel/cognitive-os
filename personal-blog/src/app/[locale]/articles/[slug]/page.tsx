import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { ArticleShell } from "@/components/content/article-shell";
import { locales, otherLocale, requireLocale } from "@/i18n/config";
import { articlePath } from "@/i18n/routes";
import { articleSlugs, getArticle } from "@/lib/content/registry";
import { createLocalizedMetadata } from "@/lib/seo/metadata";

export const dynamicParams = false;

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
  const entry = getArticle(locale, slug);
  if (!entry) {
    notFound();
  }

  return createLocalizedMetadata({
    locale,
    title: entry.frontmatter.title,
    description: entry.frontmatter.description,
    path: articlePath(locale, slug),
    alternatePath: articlePath(otherLocale(locale), slug),
    noIndex: true,
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
  const entry = getArticle(locale, slug);
  if (!entry) {
    notFound();
  }

  return (
    <ArticleShell
      locale={locale}
      entry={entry}
      alternatePath={articlePath(otherLocale(locale), slug)}
      currentPage="articles"
    />
  );
}

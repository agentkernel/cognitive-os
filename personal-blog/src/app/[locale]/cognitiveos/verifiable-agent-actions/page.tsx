import type { Metadata } from "next";
import { ArticleShell } from "@/components/content/article-shell";
import { otherLocale, requireLocale } from "@/i18n/config";
import { flagshipPath } from "@/i18n/routes";
import {
  getFlagship,
  getFlagshipSummary,
} from "@/lib/content/registry";
import { createLocalizedMetadata } from "@/lib/seo/metadata";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const locale = requireLocale((await params).locale);
  const entry = getFlagshipSummary(locale);
  return createLocalizedMetadata({
    locale,
    title: entry.frontmatter.title,
    description: entry.frontmatter.description,
    path: flagshipPath(locale),
    alternatePath: flagshipPath(otherLocale(locale)),
    type: "article",
  });
}

export default async function FlagshipArticlePage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const locale = requireLocale((await params).locale);
  const entry = await getFlagship(locale);
  return (
    <ArticleShell
      locale={locale}
      entry={entry}
      alternatePath={flagshipPath(otherLocale(locale))}
      currentPage="cognitiveos"
    />
  );
}

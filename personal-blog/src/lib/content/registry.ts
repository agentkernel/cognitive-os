import type { ComponentType } from "react";
import type { ContentLocale, Locale } from "@/i18n/config";
import { contentLocaleByRoute } from "@/i18n/config";
import { loadContentComponent } from "@/lib/content/loaders";
import {
  contentManifest,
  type ArticleSummary,
  type ContentSummary,
  type ProjectSummary,
} from "@/lib/content/manifest";
import {
  isLabFrontmatter,
  isPublishableFrontmatter,
} from "@/lib/content/publication";

export type ArticleEntry = ArticleSummary & {
  Component: ComponentType;
};

export type ProjectEntry = ProjectSummary & {
  Component: ComponentType;
};

export type ContentEntry = ArticleEntry | ProjectEntry;

function isArticleSummary(entry: ContentSummary): entry is ArticleSummary {
  return entry.frontmatter.kind !== "project";
}

function isProjectSummary(entry: ContentSummary): entry is ProjectSummary {
  return entry.frontmatter.kind === "project";
}

function routeContentLocale(locale: Locale): ContentLocale {
  return contentLocaleByRoute[locale];
}

function byNewest(
  a: ContentSummary,
  b: ContentSummary,
): number {
  return b.frontmatter.publishedAt.localeCompare(a.frontmatter.publishedAt);
}

export function listArticles(locale: Locale): ArticleSummary[] {
  const contentLocale = routeContentLocale(locale);
  return contentManifest
    .filter(isArticleSummary)
    .filter((entry) => entry.frontmatter.locale === contentLocale)
    .toSorted(byNewest);
}

export function listProjects(locale: Locale): ProjectSummary[] {
  const contentLocale = routeContentLocale(locale);
  return contentManifest
    .filter(isProjectSummary)
    .filter((entry) => entry.frontmatter.locale === contentLocale)
    .toSorted(byNewest);
}

export function listPublishedContent(locale: Locale): ContentSummary[] {
  const contentLocale = routeContentLocale(locale);
  return contentManifest
    .filter(
      (entry) =>
        entry.frontmatter.locale === contentLocale &&
        isPublishableFrontmatter(entry.frontmatter),
    )
    .toSorted(byNewest);
}

export function listLabContent(locale: Locale): ContentSummary[] {
  const contentLocale = routeContentLocale(locale);
  return contentManifest
    .filter(
      (entry) =>
        entry.frontmatter.locale === contentLocale &&
        isLabFrontmatter(entry.frontmatter),
    )
    .toSorted(byNewest);
}

export function getArticleSummary(
  locale: Locale,
  slug: string,
): ArticleSummary | undefined {
  const contentLocale = routeContentLocale(locale);
  return contentManifest.filter(isArticleSummary).find(
    (entry) =>
      entry.frontmatter.kind === "article" &&
      entry.frontmatter.locale === contentLocale &&
      entry.frontmatter.slug === slug,
  );
}

export async function getArticle(
  locale: Locale,
  slug: string,
): Promise<ArticleEntry | undefined> {
  const summary = getArticleSummary(locale, slug);
  if (!summary) {
    return undefined;
  }
  return {
    ...summary,
    Component: await loadContentComponent(summary.id),
  };
}

export function getFlagshipSummary(locale: Locale): ArticleSummary {
  const contentLocale = routeContentLocale(locale);
  const entry = contentManifest.filter(isArticleSummary).find(
    (candidate) =>
      candidate.frontmatter.kind === "cognitiveos" &&
      candidate.frontmatter.locale === contentLocale,
  );

  if (!entry) {
    throw new Error(`Missing CognitiveOS flagship for ${locale}`);
  }

  return entry;
}

export async function getFlagship(locale: Locale): Promise<ArticleEntry> {
  const summary = getFlagshipSummary(locale);
  return {
    ...summary,
    Component: await loadContentComponent(summary.id),
  };
}

export function getProjectSummary(
  locale: Locale,
  slug: string,
): ProjectSummary | undefined {
  const contentLocale = routeContentLocale(locale);
  return contentManifest.filter(isProjectSummary).find(
    (entry) =>
      entry.frontmatter.locale === contentLocale &&
      entry.frontmatter.slug === slug,
  );
}

export async function getProject(
  locale: Locale,
  slug: string,
): Promise<ProjectEntry | undefined> {
  const summary = getProjectSummary(locale, slug);
  if (!summary) {
    return undefined;
  }
  return {
    ...summary,
    Component: await loadContentComponent(summary.id),
  };
}

export function getTranslationSummary(
  entry: ContentSummary,
  locale: Locale,
): ContentSummary | undefined {
  const contentLocale = routeContentLocale(locale);
  return contentManifest.find(
    (candidate) =>
      candidate.frontmatter.translationKey ===
        entry.frontmatter.translationKey &&
      candidate.frontmatter.kind === entry.frontmatter.kind &&
      candidate.frontmatter.locale === contentLocale,
  );
}

export const articleSlugs = [
  ...new Set(
    contentManifest
      .filter(isArticleSummary)
      .filter((entry) => entry.frontmatter.kind === "article")
      .map((entry) => entry.frontmatter.slug),
  ),
] as const;

export const projectSlugs = [
  ...new Set(
    contentManifest
      .filter(isProjectSummary)
      .map((entry) => entry.frontmatter.slug),
  ),
] as const;

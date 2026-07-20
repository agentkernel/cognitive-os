import type { ComponentType } from "react";

import ContextBudgetEn, {
  frontmatter as contextBudgetEnFrontmatter,
} from "@content/articles/context-budget-notes/en.mdx";
import ContextBudgetZh, {
  frontmatter as contextBudgetZhFrontmatter,
} from "@content/articles/context-budget-notes/zh-CN.mdx";
import FailureSemanticsEn, {
  frontmatter as failureSemanticsEnFrontmatter,
} from "@content/articles/designing-failure-semantics/en.mdx";
import FailureSemanticsZh, {
  frontmatter as failureSemanticsZhFrontmatter,
} from "@content/articles/designing-failure-semantics/zh-CN.mdx";
import BilingualTestingEn, {
  frontmatter as bilingualTestingEnFrontmatter,
} from "@content/articles/testing-bilingual-content/en.mdx";
import BilingualTestingZh, {
  frontmatter as bilingualTestingZhFrontmatter,
} from "@content/articles/testing-bilingual-content/zh-CN.mdx";
import FlagshipEn, {
  frontmatter as flagshipEnFrontmatter,
} from "@content/cognitiveos/verifiable-agent-actions/en.mdx";
import FlagshipZh, {
  frontmatter as flagshipZhFrontmatter,
} from "@content/cognitiveos/verifiable-agent-actions/zh-CN.mdx";
import AccessibleDocsEn, {
  frontmatter as accessibleDocsEnFrontmatter,
} from "@content/projects/accessible-docs-migration/en.mdx";
import AccessibleDocsZh, {
  frontmatter as accessibleDocsZhFrontmatter,
} from "@content/projects/accessible-docs-migration/zh-CN.mdx";
import EvidenceCliEn, {
  frontmatter as evidenceCliEnFrontmatter,
} from "@content/projects/evidence-first-cli/en.mdx";
import EvidenceCliZh, {
  frontmatter as evidenceCliZhFrontmatter,
} from "@content/projects/evidence-first-cli/zh-CN.mdx";
import GovernedContextEn, {
  frontmatter as governedContextEnFrontmatter,
} from "@content/projects/governed-context-prototype/en.mdx";
import GovernedContextZh, {
  frontmatter as governedContextZhFrontmatter,
} from "@content/projects/governed-context-prototype/zh-CN.mdx";
import IncidentReplayEn, {
  frontmatter as incidentReplayEnFrontmatter,
} from "@content/projects/incident-replay-notebook/en.mdx";
import IncidentReplayZh, {
  frontmatter as incidentReplayZhFrontmatter,
} from "@content/projects/incident-replay-notebook/zh-CN.mdx";
import type { ContentLocale, Locale } from "@/i18n/config";
import { contentLocaleByRoute } from "@/i18n/config";
import {
  parseArticleFrontmatter,
  parseProjectFrontmatter,
  type ArticleFrontmatter,
  type ProjectFrontmatter,
} from "@/lib/content/schema";

export type ArticleEntry = {
  frontmatter: ArticleFrontmatter;
  Component: ComponentType;
};

export type ProjectEntry = {
  frontmatter: ProjectFrontmatter;
  Component: ComponentType;
};

function article(frontmatter: unknown, Component: ComponentType): ArticleEntry {
  return {
    frontmatter: parseArticleFrontmatter(frontmatter),
    Component,
  };
}

function project(frontmatter: unknown, Component: ComponentType): ProjectEntry {
  return {
    frontmatter: parseProjectFrontmatter(frontmatter),
    Component,
  };
}

export const articleEntries = [
  article(flagshipZhFrontmatter, FlagshipZh),
  article(flagshipEnFrontmatter, FlagshipEn),
  article(failureSemanticsZhFrontmatter, FailureSemanticsZh),
  article(failureSemanticsEnFrontmatter, FailureSemanticsEn),
  article(bilingualTestingZhFrontmatter, BilingualTestingZh),
  article(bilingualTestingEnFrontmatter, BilingualTestingEn),
  article(contextBudgetZhFrontmatter, ContextBudgetZh),
  article(contextBudgetEnFrontmatter, ContextBudgetEn),
] as const;

export const projectEntries = [
  project(evidenceCliZhFrontmatter, EvidenceCliZh),
  project(evidenceCliEnFrontmatter, EvidenceCliEn),
  project(governedContextZhFrontmatter, GovernedContextZh),
  project(governedContextEnFrontmatter, GovernedContextEn),
  project(incidentReplayZhFrontmatter, IncidentReplayZh),
  project(incidentReplayEnFrontmatter, IncidentReplayEn),
  project(accessibleDocsZhFrontmatter, AccessibleDocsZh),
  project(accessibleDocsEnFrontmatter, AccessibleDocsEn),
] as const;

function routeContentLocale(locale: Locale): ContentLocale {
  return contentLocaleByRoute[locale];
}

export function listArticles(locale: Locale): ArticleEntry[] {
  const contentLocale = routeContentLocale(locale);
  return articleEntries
    .filter((entry) => entry.frontmatter.locale === contentLocale)
    .toSorted((a, b) =>
      b.frontmatter.publishedAt.localeCompare(a.frontmatter.publishedAt),
    );
}

export function listProjects(locale: Locale): ProjectEntry[] {
  const contentLocale = routeContentLocale(locale);
  return projectEntries
    .filter((entry) => entry.frontmatter.locale === contentLocale)
    .toSorted((a, b) =>
      b.frontmatter.publishedAt.localeCompare(a.frontmatter.publishedAt),
    );
}

export function getArticle(locale: Locale, slug: string): ArticleEntry | undefined {
  const contentLocale = routeContentLocale(locale);
  return articleEntries.find(
    (entry) =>
      entry.frontmatter.kind === "article" &&
      entry.frontmatter.locale === contentLocale &&
      entry.frontmatter.slug === slug,
  );
}

export function getFlagship(locale: Locale): ArticleEntry {
  const contentLocale = routeContentLocale(locale);
  const entry = articleEntries.find(
    (candidate) =>
      candidate.frontmatter.kind === "cognitiveos" &&
      candidate.frontmatter.locale === contentLocale,
  );

  if (!entry) {
    throw new Error(`Missing CognitiveOS flagship for ${locale}`);
  }

  return entry;
}

export function getProject(locale: Locale, slug: string): ProjectEntry | undefined {
  const contentLocale = routeContentLocale(locale);
  return projectEntries.find(
    (entry) =>
      entry.frontmatter.locale === contentLocale &&
      entry.frontmatter.slug === slug,
  );
}

export const articleSlugs = [
  ...new Set(
    articleEntries
      .filter((entry) => entry.frontmatter.kind === "article")
      .map((entry) => entry.frontmatter.slug),
  ),
] as const;

export const projectSlugs = [
  ...new Set(projectEntries.map((entry) => entry.frontmatter.slug)),
] as const;

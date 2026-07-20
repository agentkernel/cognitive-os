import type { MetadataRoute } from "next";
import {
  cognitiveOsSourcesPath,
  contentPath,
  pagePath,
} from "@/i18n/routes";
import {
  getTranslationSummary,
  listPublishedContent,
} from "@/lib/content/registry";
import { absoluteUrl } from "@/lib/seo/metadata";

export default function sitemap(): MetadataRoute.Sitemap {
  const pagePairs = [
    { zh: pagePath("zh", "home"), en: pagePath("en", "home"), priority: 1 },
    {
      zh: pagePath("zh", "cognitiveos"),
      en: pagePath("en", "cognitiveos"),
      priority: 0.9,
    },
    {
      zh: pagePath("zh", "articles"),
      en: pagePath("en", "articles"),
      priority: 0.75,
    },
    {
      zh: cognitiveOsSourcesPath("zh"),
      en: cognitiveOsSourcesPath("en"),
      priority: 0.7,
    },
    {
      zh: pagePath("zh", "about"),
      en: pagePath("en", "about"),
      priority: 0.55,
    },
  ] as const;

  const pages = pagePairs.flatMap((entry) =>
    (["zh", "en"] as const).map((locale) => ({
      url: absoluteUrl(entry[locale]),
      lastModified: new Date("2026-07-20T00:00:00Z"),
      changeFrequency: "monthly" as const,
      priority: entry.priority,
      alternates: {
        languages: {
          "zh-CN": absoluteUrl(entry.zh),
          en: absoluteUrl(entry.en),
          "x-default": absoluteUrl(entry.zh),
        },
      },
    })),
  );

  const content = listPublishedContent("zh").flatMap((zhEntry) => {
    if (zhEntry.frontmatter.kind === "project") {
      return [];
    }
    const enEntry = getTranslationSummary(zhEntry, "en");
    if (!enEntry || enEntry.frontmatter.kind === "project") {
      return [];
    }
    const paths = {
      zh: contentPath("zh", zhEntry.frontmatter),
      en: contentPath("en", enEntry.frontmatter),
    };
    const lastModified =
      zhEntry.frontmatter.updatedAt || zhEntry.frontmatter.publishedAt;
    const priority = zhEntry.frontmatter.featured ? 0.95 : 0.7;
    return (["zh", "en"] as const).map((locale) => ({
      url: absoluteUrl(paths[locale]),
      lastModified: new Date(`${lastModified}T00:00:00Z`),
      changeFrequency: "monthly" as const,
      priority,
      alternates: {
        languages: {
          "zh-CN": absoluteUrl(paths.zh),
          en: absoluteUrl(paths.en),
          "x-default": absoluteUrl(paths.zh),
        },
      },
    }));
  });

  return [...pages, ...content];
}

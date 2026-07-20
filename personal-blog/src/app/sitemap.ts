import type { MetadataRoute } from "next";
import { flagshipPath, pagePath } from "@/i18n/routes";
import { absoluteUrl } from "@/lib/seo/metadata";

export default function sitemap(): MetadataRoute.Sitemap {
  const paths = [
    {
      zh: pagePath("zh", "home"),
      en: pagePath("en", "home"),
      priority: 1,
    },
    {
      zh: pagePath("zh", "articles"),
      en: pagePath("en", "articles"),
      priority: 0.7,
    },
    {
      zh: pagePath("zh", "cognitiveos"),
      en: pagePath("en", "cognitiveos"),
      priority: 0.8,
    },
    {
      zh: flagshipPath("zh"),
      en: flagshipPath("en"),
      priority: 0.9,
    },
  ] as const;

  return paths.flatMap((entry) =>
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
}

import type { Metadata } from "next";
import { getSiteOrigin, hasPublishableOrigin } from "@content/data/site";
import type { Locale } from "@/i18n/config";

type LocalizedMetadataInput = {
  locale: Locale;
  title: string;
  description: string;
  path: string;
  alternatePath: string;
  noIndex?: boolean;
  type?: "website" | "article";
};

export function createLocalizedMetadata({
  locale,
  title,
  description,
  path,
  alternatePath,
  noIndex = false,
  type = "website",
}: LocalizedMetadataInput): Metadata {
  const origin = getSiteOrigin();
  const alternateLocale = locale === "zh" ? "en" : "zh";
  const effectiveNoIndex = noIndex || !hasPublishableOrigin();

  return {
    metadataBase: origin,
    title,
    description,
    alternates: {
      canonical: path,
      languages: {
        [locale === "zh" ? "zh-CN" : "en"]: path,
        [alternateLocale === "zh" ? "zh-CN" : "en"]: alternatePath,
        "x-default": locale === "zh" ? path : alternatePath,
      },
    },
    robots: {
      index: !effectiveNoIndex,
      follow: true,
      googleBot: {
        index: !effectiveNoIndex,
        follow: true,
      },
    },
    openGraph: {
      type,
      locale: locale === "zh" ? "zh_CN" : "en_US",
      alternateLocale: locale === "zh" ? ["en_US"] : ["zh_CN"],
      title,
      description,
      url: path,
      siteName: locale === "zh" ? "未署名的系统设计笔记" : "Unsigned System Design Notes",
      images: [
        {
          url: "/images/og/system-notebook.png",
          width: 1200,
          height: 630,
          alt:
            locale === "zh"
              ? "抽象系统设计笔记封面"
              : "Abstract system design notebook cover",
        },
      ],
    },
    twitter: {
      card: "summary_large_image",
      title,
      description,
      images: ["/images/og/system-notebook.png"],
    },
  };
}

export function absoluteUrl(path: string): string {
  return new URL(path, getSiteOrigin()).toString();
}

import type { Metadata, Viewport } from "next";
import type { ReactNode } from "react";
import "@/styles/globals.css";
import { locales, requireLocale, htmlLanguageByRoute } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { pagePath } from "@/i18n/routes";
import { createLocalizedMetadata } from "@/lib/seo/metadata";

export const dynamicParams = false;

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const locale = requireLocale((await params).locale);
  const dictionary = getDictionary(locale);
  const alternate = locale === "zh" ? "en" : "zh";
  return createLocalizedMetadata({
    locale,
    title: dictionary.siteName,
    description: dictionary.siteDescription,
    path: pagePath(locale, "home"),
    alternatePath: pagePath(alternate, "home"),
  });
}

export const viewport: Viewport = {
  colorScheme: "light",
  themeColor: "#F4F7F8",
  width: "device-width",
  initialScale: 1,
};

export default async function LocaleLayout({
  children,
  params,
}: {
  children: ReactNode;
  params: Promise<{ locale: string }>;
}) {
  const locale = requireLocale((await params).locale);
  return (
    <html lang={htmlLanguageByRoute[locale]}>
      <body>{children}</body>
    </html>
  );
}

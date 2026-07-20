import { notFound } from "next/navigation";

export const locales = ["zh", "en"] as const;
export type Locale = (typeof locales)[number];
export type ContentLocale = "zh-CN" | "en";

export const contentLocaleByRoute: Record<Locale, ContentLocale> = {
  zh: "zh-CN",
  en: "en",
};

export const htmlLanguageByRoute: Record<Locale, string> = {
  zh: "zh-CN",
  en: "en",
};

export function isLocale(value: string): value is Locale {
  return locales.includes(value as Locale);
}

export function requireLocale(value: string): Locale {
  if (!isLocale(value)) {
    notFound();
  }

  return value;
}

export function otherLocale(locale: Locale): Locale {
  return locale === "zh" ? "en" : "zh";
}

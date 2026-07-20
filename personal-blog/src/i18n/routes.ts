import type { Locale } from "@/i18n/config";

export type PageKey =
  | "home"
  | "articles"
  | "projects"
  | "cognitiveos"
  | "about"
  | "lab";

const pageSegments: Record<PageKey, string> = {
  home: "",
  articles: "articles",
  projects: "projects",
  cognitiveos: "cognitiveos",
  about: "about",
  lab: "lab",
};

export function pagePath(locale: Locale, page: PageKey): string {
  const segment = pageSegments[page];
  return segment ? `/${locale}/${segment}` : `/${locale}`;
}

export function articlePath(locale: Locale, slug: string): string {
  return `/${locale}/articles/${slug}`;
}

export function projectPath(locale: Locale, slug: string): string {
  return `/${locale}/projects/${slug}`;
}

export function flagshipPath(locale: Locale): string {
  return `/${locale}/cognitiveos/verifiable-agent-actions`;
}

export function cognitiveOsSourcesPath(locale: Locale): string {
  return `/${locale}/cognitiveos/sources`;
}

export function contentPath(
  locale: Locale,
  content: {
    kind: "article" | "project" | "cognitiveos";
    slug: string;
  },
): string {
  if (content.kind === "cognitiveos") {
    return flagshipPath(locale);
  }
  return content.kind === "project"
    ? projectPath(locale, content.slug)
    : articlePath(locale, content.slug);
}

export function localizedPath(
  locale: Locale,
  route:
    | { kind: "page"; page: PageKey }
    | { kind: "article"; slug: string }
    | { kind: "project"; slug: string }
    | { kind: "flagship" },
  anchorKey?: string,
): string {
  const path =
    route.kind === "page"
      ? pagePath(locale, route.page)
      : route.kind === "article"
        ? articlePath(locale, route.slug)
        : route.kind === "project"
          ? projectPath(locale, route.slug)
          : flagshipPath(locale);

  return anchorKey ? `${path}#${anchorKey}` : path;
}

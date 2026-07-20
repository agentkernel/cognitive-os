import { locales, isLocale } from "@/i18n/config";
import { contentPath, pagePath } from "@/i18n/routes";
import { listPublishedContent } from "@/lib/content/registry";
import { absoluteUrl } from "@/lib/seo/metadata";

export const dynamic = "force-static";
export const revalidate = false;

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

function escapeXml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&apos;");
}

export async function GET(
  _request: Request,
  { params }: { params: Promise<{ locale: string }> },
) {
  const { locale: rawLocale } = await params;
  if (!isLocale(rawLocale)) {
    return new Response("Not found", { status: 404 });
  }

  const locale = rawLocale;
  const entries = listPublishedContent(locale).filter(
    (entry) => entry.frontmatter.kind !== "project",
  );
  const title = "CognitiveOS Research";
  const description =
    locale === "zh"
      ? "关于可验证 Agent、可靠系统与工程边界的文章。"
      : "Writing about verifiable agents, reliable systems, and engineering boundaries.";
  const selfUrl = absoluteUrl(`/${locale}/rss.xml`);
  const homeUrl = absoluteUrl(pagePath(locale, "home"));
  const items = entries
    .map((entry) => {
      const url = absoluteUrl(contentPath(locale, entry.frontmatter));
      return [
        "<item>",
        `<title>${escapeXml(entry.frontmatter.title)}</title>`,
        `<link>${escapeXml(url)}</link>`,
        `<guid isPermaLink="true">${escapeXml(url)}</guid>`,
        `<description>${escapeXml(entry.frontmatter.description)}</description>`,
        `<pubDate>${new Date(`${entry.frontmatter.publishedAt}T00:00:00Z`).toUTCString()}</pubDate>`,
        "</item>",
      ].join("");
    })
    .join("");

  const xml = [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">',
    "<channel>",
    `<title>${escapeXml(title)}</title>`,
    `<link>${escapeXml(homeUrl)}</link>`,
    `<description>${escapeXml(description)}</description>`,
    `<language>${locale === "zh" ? "zh-CN" : "en"}</language>`,
    `<atom:link href="${escapeXml(selfUrl)}" rel="self" type="application/rss+xml" />`,
    items,
    "</channel>",
    "</rss>",
  ].join("");

  return new Response(xml, {
    headers: {
      "Content-Type": "application/rss+xml; charset=utf-8",
      "Cache-Control": "public, max-age=3600, s-maxage=86400",
    },
  });
}

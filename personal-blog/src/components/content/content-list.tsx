import Link from "next/link";
import { formatDate } from "@/i18n/format";
import type { ContentSummary } from "@/lib/content/manifest";

type ContentListProps = {
  entries: ReadonlyArray<ContentSummary>;
  resolveHref: (entry: ContentSummary) => string;
  actionLabel: string;
  sampleLabel: string;
  headingLevel?: "h2" | "h3";
};

export function ContentList({
  entries,
  resolveHref,
  actionLabel,
  sampleLabel,
  headingLevel = "h2",
}: ContentListProps) {
  const Heading = headingLevel;
  return (
    <ul className="content-list">
      {entries.map((entry) => (
        <li key={`${entry.frontmatter.translationKey}-${entry.frontmatter.locale}`}>
          <Link className="content-list__link" href={resolveHref(entry)}>
            <div className="content-list__body">
              <div className="content-list__meta">
                <time dateTime={entry.frontmatter.publishedAt}>
                  {formatDate(
                    entry.frontmatter.publishedAt,
                    entry.frontmatter.locale === "zh-CN" ? "zh" : "en",
                  )}
                </time>
                {entry.frontmatter.placeholder ? <span>{sampleLabel}</span> : null}
              </div>
              <Heading>{entry.frontmatter.title}</Heading>
              <p>{entry.frontmatter.description}</p>
              <ul
                className="content-list__topics"
                aria-label={
                  entry.frontmatter.locale === "zh-CN" ? "主题" : "Topics"
                }
              >
                {(entry.frontmatter.kind === "project"
                  ? entry.frontmatter.stack
                  : entry.frontmatter.tags
                ).map((topic) => (
                  <li key={topic}>{topic}</li>
                ))}
              </ul>
              <span className="content-list__action">
                {actionLabel}
                <svg viewBox="0 0 20 20" aria-hidden="true">
                  <path d="M3 10h13M11 5l5 5-5 5" />
                </svg>
              </span>
            </div>
          </Link>
        </li>
      ))}
    </ul>
  );
}

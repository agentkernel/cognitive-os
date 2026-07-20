import Link from "next/link";
import type { ArticleEntry, ProjectEntry } from "@/lib/content/registry";

type ContentListProps = {
  entries: ReadonlyArray<ArticleEntry | ProjectEntry>;
  resolveHref: (entry: ArticleEntry | ProjectEntry) => string;
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
                  {entry.frontmatter.publishedAt}
                </time>
                {entry.frontmatter.placeholder ? <span>{sampleLabel}</span> : null}
              </div>
              <Heading>{entry.frontmatter.title}</Heading>
              <p>{entry.frontmatter.description}</p>
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

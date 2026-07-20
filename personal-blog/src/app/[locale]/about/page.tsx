import type { Metadata } from "next";
import Link from "next/link";
import { PageScaffold } from "@/components/layout/page-scaffold";
import { otherLocale, requireLocale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { cognitiveOsSourcesPath, pagePath } from "@/i18n/routes";
import { createLocalizedMetadata } from "@/lib/seo/metadata";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const locale = requireLocale((await params).locale);
  const dictionary = getDictionary(locale);
  return createLocalizedMetadata({
    locale,
    title: dictionary.about.title,
    description: dictionary.about.description,
    path: pagePath(locale, "about"),
    alternatePath: pagePath(otherLocale(locale), "about"),
  });
}

export default async function AboutPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const locale = requireLocale((await params).locale);
  const dictionary = getDictionary(locale);
  const alternatePath = pagePath(otherLocale(locale), "about");

  return (
    <PageScaffold
      locale={locale}
      currentPage="about"
      alternatePath={alternatePath}
    >
      <div className="page-shell">
        <header className="page-intro">
          <div>
            <p className="eyebrow">
              {locale === "zh" ? "方法与边界" : "METHOD & BOUNDARIES"}
            </p>
            <h1>{dictionary.about.title}</h1>
          </div>
          <p>{dictionary.about.description}</p>
        </header>

        <section className="method-lead" aria-labelledby="method-first">
          <p className="method-lead__index" aria-hidden="true">
            01
          </p>
          <div>
            <h2 id="method-first">{dictionary.about.methodHeading}</h2>
            <p>{dictionary.about.method}</p>
          </div>
        </section>

        <section className="method-grid" aria-labelledby="method-principles">
          <header>
            <p className="eyebrow">
              {locale === "zh" ? "公开承诺" : "PUBLIC COMMITMENTS"}
            </p>
            <h2 id="method-principles">{dictionary.about.principlesHeading}</h2>
          </header>
          <ol>
            {dictionary.about.principles.map((principle, index) => (
              <li key={principle}>
                <span aria-hidden="true">{String(index + 1).padStart(2, "0")}</span>
                <p>{principle}</p>
              </li>
            ))}
          </ol>
        </section>

        <section className="method-notes">
          <article aria-labelledby="author-position">
            <p className="eyebrow">{locale === "zh" ? "作者" : "AUTHOR"}</p>
            <h2 id="author-position">{dictionary.about.authorHeading}</h2>
            <p>{dictionary.about.author}</p>
          </article>
          <article aria-labelledby="current-boundary">
            <p className="eyebrow">
              {locale === "zh" ? "研究边界" : "RESEARCH BOUNDARY"}
            </p>
            <h2 id="current-boundary">{dictionary.about.boundaryHeading}</h2>
            <p>{dictionary.about.boundary}</p>
            <Link className="text-link" href={cognitiveOsSourcesPath(locale)}>
              {dictionary.home.sourcesAction}
            </Link>
          </article>
        </section>
      </div>
    </PageScaffold>
  );
}

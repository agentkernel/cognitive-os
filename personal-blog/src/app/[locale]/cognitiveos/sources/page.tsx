import type { Metadata } from "next";
import SourcebookEn from "@content/research/cognitiveos-sourcebook.en.md";
import SourcebookZh from "@content/research/cognitiveos-sourcebook.zh-CN.md";
import { CognitiveOsManualSidebar } from "@/components/cognitiveos/manual-sidebar";
import { PageScaffold } from "@/components/layout/page-scaffold";
import { JsonLd } from "@/components/seo/json-ld";
import { cognitiveOsSnapshot } from "@/data/cognitiveos";
import { otherLocale, requireLocale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { cognitiveOsSourcesPath } from "@/i18n/routes";
import { absoluteUrl, createLocalizedMetadata } from "@/lib/seo/metadata";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const locale = requireLocale((await params).locale);
  const dictionary = getDictionary(locale);
  const title =
    locale === "zh" ? "来源与修订账本" : "Sources and revision ledger";
  return createLocalizedMetadata({
    locale,
    title,
    description: dictionary.home.snapshotDescription,
    path: cognitiveOsSourcesPath(locale),
    alternatePath: cognitiveOsSourcesPath(otherLocale(locale)),
  });
}

export default async function CognitiveOsSourcesPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const locale = requireLocale((await params).locale);
  const dictionary = getDictionary(locale);
  const Sourcebook = locale === "zh" ? SourcebookZh : SourcebookEn;

  return (
    <PageScaffold
      locale={locale}
      currentPage="cognitiveos"
      alternatePath={cognitiveOsSourcesPath(otherLocale(locale))}
    >
      <JsonLd
        data={{
          "@context": "https://schema.org",
          "@type": "Dataset",
          name:
            locale === "zh"
              ? "CognitiveOS 研究资料簿"
              : "CognitiveOS Research Sourcebook",
          description: dictionary.home.snapshotDescription,
          inLanguage: locale === "zh" ? "zh-CN" : "en",
          dateModified: cognitiveOsSnapshot.capturedAt,
          url: absoluteUrl(cognitiveOsSourcesPath(locale)),
        }}
      />
      <div className="manual-layout manual-layout--sourcebook">
        <CognitiveOsManualSidebar locale={locale} active="sources" />
        <div className="manual-content">
          <header className="sourcebook-intro">
            <p className="eyebrow">
              {locale === "zh" ? "来源与修订账本" : "SOURCES & REVISIONS"}
            </p>
            <p>{dictionary.home.snapshotDescription}</p>
          </header>
          <p className="snapshot-strip">
            {cognitiveOsSnapshot.commit.slice(0, 7)} ·{" "}
            {cognitiveOsSnapshot.capturedAt} · {cognitiveOsSnapshot.vectorsNotRun}{" "}
            vectors not-run
          </p>
          <article className="sourcebook prose">
            <Sourcebook />
          </article>
        </div>
      </div>
    </PageScaffold>
  );
}

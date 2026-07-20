import type { Metadata } from "next";
import { authorProfile, sampleTimeline } from "@content/data/profile";
import { PageScaffold } from "@/components/layout/page-scaffold";
import { otherLocale, requireLocale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { pagePath } from "@/i18n/routes";
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
    noIndex: true,
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
    <PageScaffold locale={locale} currentPage="about" alternatePath={alternatePath}>
      <div className="page-shell">
        <header className="page-intro">
          <h1>{dictionary.about.title}</h1>
          <div>
            <p>{dictionary.about.description}</p>
            <p className="eyebrow">{dictionary.sample} / Sample content</p>
          </div>
        </header>

        <section className="home-section" aria-labelledby="about-profile">
          <header>
            <span>{locale === "zh" ? "资料" : "PROFILE"}</span>
            <h2 id="about-profile">{authorProfile.name[locale]}</h2>
          </header>
          <div className="profile-panel">
            <div className="profile-initials" aria-hidden="true">
              {authorProfile.initials}
            </div>
            <div>
              <p className="eyebrow">{dictionary.sample} / Sample content</p>
              <h2>{authorProfile.title[locale]}</h2>
              <p>{authorProfile.bio[locale]}</p>
              <p>{authorProfile.location[locale]}</p>
            </div>
          </div>
        </section>

        <section className="home-section" aria-labelledby="about-method">
          <header>
            <span>{locale === "zh" ? "工作方法" : "WORKING METHOD"}</span>
            <h2 id="about-method">{dictionary.about.methodHeading}</h2>
          </header>
          <div className="home-about">
            <p>{dictionary.about.method}</p>
          </div>
        </section>

        <section className="home-section" aria-labelledby="about-timeline">
          <header>
            <span>{locale === "zh" ? "示例时间线" : "SAMPLE TIMELINE"}</span>
            <h2 id="about-timeline">{dictionary.about.timelineHeading}</h2>
          </header>
          <ol className="timeline">
            {sampleTimeline.map((entry) => (
              <li key={entry.id}>
                <time>{entry.period}</time>
                <div>
                  <p className="eyebrow">{dictionary.sample} / Sample content</p>
                  <h3>{entry.title[locale]}</h3>
                  <p>{entry.detail[locale]}</p>
                </div>
              </li>
            ))}
          </ol>
        </section>

        <section className="home-section" aria-labelledby="about-contact">
          <header>
            <span>{locale === "zh" ? "联系占位" : "CONTACT PLACEHOLDER"}</span>
            <h2 id="about-contact">{dictionary.about.contactHeading}</h2>
          </header>
          <div className="home-about">
            <p>{dictionary.about.contact}</p>
          </div>
        </section>
      </div>
    </PageScaffold>
  );
}

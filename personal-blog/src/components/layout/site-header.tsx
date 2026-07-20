import Link from "next/link";
import { MobileNavigation } from "@/components/navigation/mobile-navigation";
import type { Locale } from "@/i18n/config";
import { otherLocale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { pagePath, type PageKey } from "@/i18n/routes";

type SiteHeaderProps = {
  locale: Locale;
  currentPage: PageKey;
  alternatePath: string;
};

export function SiteHeader({ locale, currentPage, alternatePath }: SiteHeaderProps) {
  const dictionary = getDictionary(locale);
  const links: Array<{ key: PageKey; label: string; href: string }> = [
    {
      key: "cognitiveos",
      label: dictionary.nav.cognitiveos,
      href: pagePath(locale, "cognitiveos"),
    },
    {
      key: "articles",
      label: dictionary.nav.articles,
      href: pagePath(locale, "articles"),
    },
    { key: "about", label: dictionary.nav.about, href: pagePath(locale, "about") },
  ];

  return (
    <header className="site-header">
      <div className="site-header__inner">
        <Link className="site-mark" href={pagePath(locale, "home")} aria-label={dictionary.siteName}>
          <span aria-hidden="true">CO</span>
          <strong>{dictionary.siteShortName}</strong>
        </Link>
        <nav className="desktop-navigation" aria-label={locale === "zh" ? "主导航" : "Primary"}>
          {links.map((link) => (
            <Link
              key={link.key}
              href={link.href}
              aria-current={link.key === currentPage ? "page" : undefined}
            >
              {link.label}
            </Link>
          ))}
        </nav>
        <div className="site-header__actions">
          <Link
            className="language-switch"
            href={alternatePath}
            hrefLang={otherLocale(locale) === "zh" ? "zh-CN" : "en"}
            lang={otherLocale(locale) === "zh" ? "zh-CN" : "en"}
          >
            {dictionary.languageSwitch}
          </Link>
          <MobileNavigation
            locale={locale}
            links={links}
            currentPage={currentPage}
            alternatePath={alternatePath}
          />
        </div>
      </div>
    </header>
  );
}

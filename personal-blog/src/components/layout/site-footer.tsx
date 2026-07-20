import Link from "next/link";
import type { Locale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { pagePath } from "@/i18n/routes";

export function SiteFooter({ locale }: { locale: Locale }) {
  const dictionary = getDictionary(locale);
  return (
    <footer className="site-footer">
      <div>
        <p>{dictionary.footer}</p>
        <nav aria-label={locale === "zh" ? "页脚导航" : "Footer navigation"}>
          <Link href={pagePath(locale, "articles")}>{dictionary.nav.articles}</Link>
          <Link href={pagePath(locale, "cognitiveos")}>{dictionary.nav.cognitiveos}</Link>
          <Link href={pagePath(locale, "about")}>{dictionary.nav.about}</Link>
          <Link href={pagePath(locale, "lab")}>{dictionary.nav.lab}</Link>
          <Link href={`/${locale}/rss.xml`}>RSS</Link>
        </nav>
      </div>
      <small>© 2026 · CognitiveOS Research · UNLICENSED</small>
    </footer>
  );
}

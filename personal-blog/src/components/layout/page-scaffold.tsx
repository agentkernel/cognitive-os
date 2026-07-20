import { SiteFooter } from "@/components/layout/site-footer";
import { SiteHeader } from "@/components/layout/site-header";
import type { Locale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import type { PageKey } from "@/i18n/routes";

type PageScaffoldProps = {
  locale: Locale;
  currentPage: PageKey;
  alternatePath: string;
  children: React.ReactNode;
};

export function PageScaffold({
  locale,
  currentPage,
  alternatePath,
  children,
}: PageScaffoldProps) {
  const dictionary = getDictionary(locale);
  return (
    <>
      <a className="skip-link" href="#main-content">
        {dictionary.skipToContent}
      </a>
      <SiteHeader
        locale={locale}
        currentPage={currentPage}
        alternatePath={alternatePath}
      />
      <main id="main-content" tabIndex={-1}>
        {children}
      </main>
      <SiteFooter locale={locale} />
    </>
  );
}

import Link from "next/link";
import type { Locale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { flagshipPath } from "@/i18n/routes";

export function RealArticlesEmptyState({ locale }: { locale: Locale }) {
  const dictionary = getDictionary(locale);
  return (
    <aside className="empty-state" aria-labelledby={`empty-real-articles-${locale}`}>
      <svg viewBox="0 0 64 64" aria-hidden="true">
        <path d="M8 12h48v40H8zM18 24h28M18 34h18M18 44h24" />
        <path className="unresolved" d="M43 34h3v10h-4" />
      </svg>
      <div>
        <h2 id={`empty-real-articles-${locale}`}>{dictionary.articles.emptyTitle}</h2>
        <p>{dictionary.articles.emptyBody}</p>
        <Link href={flagshipPath(locale)}>{dictionary.home.primaryAction}</Link>
      </div>
    </aside>
  );
}

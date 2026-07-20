"use client";

import Link from "next/link";
import { useParams } from "next/navigation";
import { isLocale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import { pagePath } from "@/i18n/routes";

export default function LocalizedError({
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  const params = useParams<{ locale?: string }>();
  const locale = params.locale && isLocale(params.locale) ? params.locale : "en";
  const dictionary = getDictionary(locale);

  return (
    <main className="error-panel" role="alert">
      <div>
        <p className="eyebrow">RENDER ERROR</p>
        <h1>{dictionary.error.title}</h1>
        <p>{dictionary.error.body}</p>
        <div className="error-panel__actions">
          <button type="button" onClick={reset}>
            {dictionary.error.retry}
          </button>
          <Link href={pagePath(locale, "articles")}>{dictionary.error.back}</Link>
        </div>
      </div>
    </main>
  );
}

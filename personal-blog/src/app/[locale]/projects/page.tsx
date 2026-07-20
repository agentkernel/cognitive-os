import { permanentRedirect } from "next/navigation";
import { requireLocale } from "@/i18n/config";
import { pagePath } from "@/i18n/routes";

export default async function LegacyProjectsPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const locale = requireLocale((await params).locale);
  permanentRedirect(pagePath(locale, "lab"));
}

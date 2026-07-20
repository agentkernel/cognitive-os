"use client";

import Link from "next/link";
import { useEffect, useId, useRef, useState } from "react";
import type { Locale } from "@/i18n/config";
import { otherLocale } from "@/i18n/config";
import { getDictionary } from "@/i18n/dictionaries";
import type { PageKey } from "@/i18n/routes";

type MobileNavigationProps = {
  locale: Locale;
  links: Array<{ key: PageKey; label: string; href: string }>;
  currentPage: PageKey;
  alternatePath: string;
};

export function MobileNavigation({
  locale,
  links,
  currentPage,
  alternatePath,
}: MobileNavigationProps) {
  const [open, setOpen] = useState(false);
  const panelId = useId();
  const triggerRef = useRef<HTMLButtonElement>(null);
  const closeRef = useRef<HTMLButtonElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const dictionary = getDictionary(locale);

  useEffect(() => {
    if (!open) {
      return;
    }

    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    closeRef.current?.focus();

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
        triggerRef.current?.focus();
        return;
      }

      if (event.key !== "Tab" || !panelRef.current) {
        return;
      }

      const focusable = Array.from(
        panelRef.current.querySelectorAll<HTMLElement>(
          'button:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])',
        ),
      );
      const first = focusable[0];
      const last = focusable.at(-1);

      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last?.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first?.focus();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.body.style.overflow = previousOverflow;
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [open]);

  const close = () => {
    setOpen(false);
  };

  return (
    <div className="mobile-navigation">
      <button
        ref={triggerRef}
        type="button"
        className="mobile-navigation__trigger"
        aria-expanded={open}
        aria-controls={panelId}
        aria-label={dictionary.openMenu}
        onClick={() => setOpen(true)}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M4 7h16M4 12h16M4 17h16" />
        </svg>
      </button>
      {open ? (
        <div className="mobile-navigation__layer">
          <button
            type="button"
            className="mobile-navigation__backdrop"
            aria-label={dictionary.closeMenu}
            onClick={close}
          />
          <div
            ref={panelRef}
            id={panelId}
            className="mobile-navigation__panel"
            role="dialog"
            aria-modal="true"
            aria-label={locale === "zh" ? "移动导航" : "Mobile navigation"}
          >
            <div className="mobile-navigation__heading">
              <span>{dictionary.siteShortName}</span>
              <button
                ref={closeRef}
                type="button"
                aria-label={dictionary.closeMenu}
                onClick={close}
              >
                <svg viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M5 5l14 14M19 5L5 19" />
                </svg>
              </button>
            </div>
            <nav aria-label={locale === "zh" ? "主导航" : "Primary"}>
              {links.map((link) => (
                <Link
                  key={link.key}
                  href={link.href}
                  aria-current={link.key === currentPage ? "page" : undefined}
                  onClick={close}
                >
                  {link.label}
                </Link>
              ))}
            </nav>
            <Link
              className="mobile-navigation__language"
              href={alternatePath}
              hrefLang={otherLocale(locale) === "zh" ? "zh-CN" : "en"}
              onClick={close}
            >
              {dictionary.languageSwitch}
            </Link>
          </div>
        </div>
      ) : null}
    </div>
  );
}

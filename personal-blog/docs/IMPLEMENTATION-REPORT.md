# CognitiveOS Research implementation report

- Refactor completed: 2026-07-20
- Site root: `personal-blog/`
- Parent research snapshot: `b626e88be3b985399051e6e7624223b9cb38e7c6`
- Deployment performed: no
- CognitiveOS normative assets changed: none

## Delivered

- Repositioned the site from an unsigned portfolio prototype to a bilingual
  CognitiveOS research publication with an understated author position.
- Rebuilt the primary journey around one thesis, one Governed Flow Thread
  signature, the flagship essay, a visual research atlas, and a public source
  ledger.
- Removed placeholder identity, timeline, articles, and projects from the
  primary experience. All seven paired sample sets now live under noindex Lab;
  the legacy Projects index redirects there.
- Added `/{locale}/cognitiveos/sources`, publishing both 18-fact sourcebooks,
  snapshot hashes, source tiers, open discrepancies, and wording guardrails.
- Split the former 2,286-line stylesheet into Tailwind v4 semantic tokens,
  base, shell, content, and diagram layers. Restored list markers, persistent
  prose links, fluid type/spacing, touch targets, dark-surface focus, and
  readable diagram fallbacks.
- Kept full diagrams in the wide Research atlas. Long-form MDX uses compact
  summaries linked to the corresponding atlas figure.
- Replaced the all-MDX registry with a metadata-only manifest and a static,
  per-entry loader map. Publication, localized content paths, RSS, sitemap,
  metadata, and translation pairing share one contract.
- Added locale-aware dates, visible topic tags, source links, improved article
  follow-up navigation, stricter ISO date/origin validation, and article-level
  JSON-LD for every publishable research item.
- Hardened the mobile drawer with viewport-correct backdrop behavior, short
  screen scrolling, background inerting, focus trap, Escape support, and focus
  restoration for every close path.
- Localized code/table accessible names and added hydration-gated copy
  enhancement. Static content stays usable before JavaScript.
- Added CSP, frame denial, nosniff, referrer, permissions, COOP/CORP, and
  conditional HSTS response headers.
- Reduced the Chinese font path from two global full faces to one 400 face used
  only by long-form prose; shell and navigation use system CJK fonts.

## Executed checks

`pnpm check` / its constituent commands:

- ESLint: passed with no warnings.
- strict `tsc --noEmit`: passed.
- Vitest: 2 files, 14 tests passed.
- content contract: 4 paired article sets, 4 paired project sets, 18 facts per
  sourcebook.
- boundary check: 16 statically mapped MDX loaders and exactly 4 intentional
  Client Components.
- Next.js production build: passed; 38 generated static/SSG pages.

`pnpm test:e2e:against-build`:

- 22/22 Chromium scenarios passed in the production build.
- Covered root redirect, primary product journey, Lab isolation, keyboard
  navigation, shared bilingual anchors, source/manual navigation, mobile
  drawer close paths, copy feedback, responsive diagrams, reduced motion,
  forced colors, security headers, RSS/sitemap/robots, and invalid routes.
- Automated axe scans passed for Home, Essays, Research, Sources, Method, Lab,
  flagship, and sample-detail templates with WCAG 2.0/2.1/2.2 A/AA tags.
- Responsive overflow/screenshots passed at 375, 768, 1024, and 1440 CSS
  pixels.

Generated evidence remains ignored:

- `artifacts/evidence/screenshots/`
- `artifacts/evidence/playwright-results/`
- `artifacts/evidence/playwright-report/`

This is executed local evidence, not Profile-conformance evidence.

## Publication and evidence boundary

The site still reports the pinned snapshot:

- 273 specified requirements
- 55 registered error codes
- 56 schemas
- 76 declarative vectors, all `not-run`
- 0 REQ-level implementation claims
- 0 behavior-executed vectors
- 0 conformant Profiles

The UI refactor does not change those facts and does not read the parent
repository at build or runtime. Sourcebooks remain local, pinned research
copies.

## Deployment

1. Use `personal-blog` as the deployment root.
2. Use Node.js 22 and pinned pnpm 10.33.2.
3. Install with `pnpm install --frozen-lockfile`.
4. Build with `pnpm build`.
5. Set `NEXT_PUBLIC_SITE_URL` to the final clean HTTPS origin.
6. Leave analytics disabled unless explicitly approved.

Without a valid HTTPS origin, metadata remains `noindex` and robots disallows
crawling.

## Remaining risk and manual work

- The project is `UNLICENSED`; choose public code/content terms before release.
- Confirm image-generation publication rights.
- Execute the documented NVDA/real-browser, 200–400% zoom, pronunciation, and
  human bilingual review; automated axe does not replace them.
- Root CI still excludes this isolated workspace. `.github/workflows/` belongs
  to Lane-CFR and was intentionally not changed.
- `globalNotFound` remains an experimental Next.js convention and should be
  rechecked during framework upgrades.
- Next 16.2.10 currently carries a moderate indirect PostCSS advisory; no
  untrusted CSS ingestion path exists, but the dependency should be updated
  with the next safe Next.js patch.

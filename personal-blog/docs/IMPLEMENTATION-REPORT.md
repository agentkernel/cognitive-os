# Personal blog implementation report

- Completed: 2026-07-20
- Site root: `personal-blog/`
- Parent source snapshot: `b626e88be3b985399051e6e7624223b9cb38e7c6`
- Deployment performed: no

## Delivered

- Next.js 16.2 App Router site with complete `/zh` and `/en` navigation.
- Local, Zod-validated MDX with explicit bilingual registry and shared anchors.
- Home, article, project, CognitiveOS, and about pages; RSS, sitemap, robots,
  metadata, hreflang, Open Graph, JSON-LD, 404, error, and empty states.
- One complete bilingual CognitiveOS article, three paired sample articles,
  four paired sample projects, a placeholder profile, and a sample timeline.
- Two complete CognitiveOS research sourcebooks with 18 traced facts each.
- Five responsive semantic React/SVG diagrams with captions and text
  alternatives.
- Two locally stored AI-generated abstract visuals in AVIF/WebP plus a local
  PNG Open Graph asset and provenance records.
- Locally bundled OFL-1.1 Source Serif 4, Recursive, and Noto Serif SC fonts.
- Responsive code blocks, GFM tables, footnotes, and mobile diagram summaries.

## Executed checks

`pnpm check` passed:

- ESLint CLI
- strict `tsc --noEmit`
- Vitest: 2 files, 9 tests
- content checks: 4 paired article entries, 4 paired projects, 18 traced facts
  per sourcebook
- static import/boundary checks: 16 MDX imports, 3 intentional Client
  Components
- Next.js production build: 34 generated static/SSG routes

`pnpm test:e2e` passed 15 Chromium scenarios:

- permanent root redirect
- keyboard skip link and primary navigation
- equivalent-language routing and anchors
- flagship semantics and all five diagrams
- local generated images and local font requests
- mobile-menu focus trap, Escape, and focus return
- 375, 768, and 1440 viewport screenshots and overflow checks
- enforced long-form line measure, font size, and line-height checks
- browser console/page-error checks
- reduced-motion and forced-colors behavior
- RSS, sitemap, robots, sample noindex, and invalid-route 404s
- mobile GFM table, code block, and footnote containment
- axe WCAG A/AA scans for home and flagship routes

Screenshots were generated under the ignored directory:

- `artifacts/evidence/screenshots/home-375.png`
- `artifacts/evidence/screenshots/home-768.png`
- `artifacts/evidence/screenshots/home-1440.png`
- `artifacts/evidence/screenshots/article-375.png`
- `artifacts/evidence/screenshots/article-1440.png`

## Independent-root proof

The source tree was copied to a newly created directory outside the repository.
From that directory, the following completed successfully:

- `pnpm install --frozen-lockfile`
- workspace-root resolution to the isolated directory
- lint
- strict typecheck
- 9 Vitest tests
- content and import-boundary checks
- 33-route production build

The temporary copy was removed after the successful run. The parent workspace
listing did not include `personal-blog`, and the final parent-only Git status
was clean.

## Placeholder material

The following remain intentionally marked `placeholder: true`, visibly labeled
as “示例内容 / Sample content,” `noindex`, and excluded from RSS/sitemap:

- author identity, location, contact, and timeline
- three ordinary technical articles
- four project case studies

No real employer, client, award, revenue, user-count, or performance claim was
invented. No `Person` structured data is emitted.

## CognitiveOS evidence boundary

The content is pinned to the parent snapshot above:

- 273 specified requirements
- 55 registered errors
- 56 schemas with filename `$id` values
- 76 declarative vectors, all `not-run`
- 0 REQ-level implementation claims
- 0 behavior-executed vectors
- 0 conformant Profiles

M1 is in progress. Lane-CTR contract artifacts exist, but they are not promoted
to REQ-level implementation, executed-vector, or Profile-conformance claims.

## Vercel setup

1. Import the parent repository without changing its workspace files.
2. Set Root Directory to `personal-blog`.
3. Use Node.js 22 and the package-pinned pnpm 10.33.2.
4. Install with `pnpm install --frozen-lockfile`.
5. Build with `pnpm build`.
6. Set `NEXT_PUBLIC_SITE_URL` to the final HTTPS origin.
7. Leave Vercel Analytics disabled unless the owner explicitly opts in.

Until a valid HTTPS origin is configured, metadata remains `noindex` and
`robots.txt` disallows crawling.

## Remaining risks and manual work

- Replace all placeholder identity and project material with owner-approved
  facts before treating the site as a real portfolio.
- Choose a repository/content license; the project currently remains private
  and `UNLICENSED`.
- Confirm publication rights under the image-generation service terms.
- Complete the documented NVDA/real-browser, 200–400% zoom, pronunciation, and
  final human translation review after identity replacement.
- Next.js currently logs its internal `NoFallbackError` on deliberately rejected
  `dynamicParams=false` requests even though it correctly returns 404. Browser
  console checks are clean; this upstream server-log issue is tracked by
  vercel/next.js#90537.
- `globalNotFound` is a documented Next.js experimental convention and should
  be rechecked during future Next upgrades.

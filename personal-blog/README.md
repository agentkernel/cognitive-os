# CognitiveOS Research

An isolated bilingual Next.js publication about verifiable agents,
deterministic authority, and inspectable evidence boundaries. The author stays
deliberately understated; source snapshots and research limits remain public.

## Product surface

- `/` permanently redirects to `/zh`; equivalent Chinese and English routes
  use explicit static params and shared translation identities.
- The primary journey is Home → Research → flagship essay / source ledger.
  Essays contains publishable research only. Method explains the publication
  discipline without inventing identity or résumé claims.
- `/{locale}/lab` contains all structural article and case-study samples. Lab
  and every sample detail remain `noindex` and are excluded from RSS/sitemap.
  The old `/{locale}/projects` index permanently redirects to Lab.
- `/{locale}/cognitiveos/sources` publishes the bilingual 18-fact sourcebook,
  snapshot hashes, discrepancies, and wording guardrails.

## Architecture

- Next.js 16 App Router and React Server Components remain the default. Only
  mobile navigation, article enhancement, and error recovery are client-side.
- Trusted local MDX frontmatter is parsed and validated with Zod through a
  metadata-only manifest. List, sitemap, RSS, and SEO code do not import MDX
  components; detail routes load only the statically mapped matching module.
- One publication/path contract controls indexing, canonical URLs, hreflang,
  RSS GUIDs, sitemap entries, Lab isolation, and translated-slug lookup.
- The research snapshot is pinned to parent commit
  `b626e88be3b985399051e6e7624223b9cb38e7c6`: 273 requirements specified,
  76 vectors `not-run`, zero REQ-level implementation claims, zero
  behavior-executed vectors, and zero conformant Profiles.
- The visual system is an Asymmetric Evidence Notebook: cold canvas, evidence
  blue, unknown-outcome copper, and a single Governed Flow Thread signature.
  CSS is split into tokens, base, shell, content, and diagram layers.
- Source Serif 4 and Recursive are locally bundled. The full Chinese 400 face
  is scoped to long-form prose; interface text uses the system CJK stack.
- Static security headers include CSP, frame denial, nosniff, referrer policy,
  permissions policy, COOP, and CORP. HSTS is emitted only for a clean,
  configured HTTPS publication origin.

## Commands

Run from `personal-blog/`:

```powershell
pnpm install
pnpm lint
pnpm typecheck
pnpm test
pnpm test:content
pnpm check
pnpm exec playwright install chromium
pnpm test:e2e
pnpm verify
```

`pnpm verify` runs the complete static check/build sequence and then the
production Playwright suite on isolated port 3101. The nested
`pnpm-workspace.yaml` is the workspace boundary; do not install from the parent
repository.

The executed evidence and deployment notes are in
[`docs/IMPLEMENTATION-REPORT.md`](docs/IMPLEMENTATION-REPORT.md).

## Publication blockers

- Configure a clean HTTPS `NEXT_PUBLIC_SITE_URL`; until then all pages remain
  `noindex` and `robots.txt` disallows crawling.
- Choose a public content/code license. The project remains private and
  `UNLICENSED`.
- Confirm image-service publication rights.
- Complete and retain the manual assistive-technology checks in
  [`tests/MANUAL-ACCESSIBILITY.md`](tests/MANUAL-ACCESSIBILITY.md).
- Add an independent blog CI job through Lane-CFR ownership; this change does
  not modify the root workflow.

Generated browser reports and screenshots belong under ignored
`artifacts/evidence/`.

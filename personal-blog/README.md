# Personal developer blog

An isolated bilingual Next.js App Router site for evidence-led systems writing. The project is private and unlicensed until the owner supplies a license and real identity details.

## Architecture

- `/` permanently redirects to `/zh`; `zh` and `en` use explicit static route params with no locale middleware.
- Official local `@next/mdx` compiles trusted files. YAML frontmatter is exported by `remark-mdx-frontmatter`, validated with Zod at build time, and imported through one explicit registry.
- React Server Components are the default. Only the mobile navigation and error boundaries are client components.
- Sample articles, projects, profile, and timeline are always marked placeholder, `noindex`, and excluded from RSS/sitemap data.
- The CognitiveOS research ledger is pinned to parent commit `b626e88be3b985399051e6e7624223b9cb38e7c6` (M1 in progress, 76 vectors all `not-run`). Lane-CTR contract artifacts exist, but the snapshot records zero REQ-level implementation claims, zero behavior-executed vectors, and zero conformant Profiles.
- The CognitiveOS article uses child-local facts and diagrams. Build and runtime never read the parent repository.
- Metadata helpers emit exact canonical and language alternates. Without a configured, non-placeholder HTTPS origin, all pages remain `noindex` and `robots.txt` disallows crawling.
- Two text-free AI-generated abstract visuals are stored locally as AVIF/WebP
  with provenance notes; no image hotlink or runtime generation is used.
- Source Serif 4, Recursive, and Noto Serif SC are bundled locally from pinned
  OFL-1.1 Fontsource packages; license texts ship with the site.

## Commands

Run every command from this directory:

```powershell
pnpm install
pnpm lint
pnpm typecheck
pnpm test
pnpm test:content
pnpm build
pnpm exec playwright install chromium
pnpm test:e2e
```

The nested `pnpm-workspace.yaml` is the workspace boundary. Do not install from the parent repository.

The executed verification record, isolated-build proof, placeholder inventory,
and deployment steps are in
[`docs/IMPLEMENTATION-REPORT.md`](docs/IMPLEMENTATION-REPORT.md).

## Publication blockers

- Replace the placeholder identity, timeline, contact, and sample case studies with owner-approved facts.
- Add a real HTTPS `NEXT_PUBLIC_SITE_URL`; until then indexing remains blocked.
- Complete the manual accessibility checks in `tests/MANUAL-ACCESSIBILITY.md` after final assets and fonts are chosen.

Generated browser reports and screenshots belong under ignored `artifacts/evidence/`.

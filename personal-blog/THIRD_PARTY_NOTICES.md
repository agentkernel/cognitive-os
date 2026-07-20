# Third-party notices

This private project is `UNLICENSED`. Dependency licenses remain governed by their respective packages and lockfile versions.

Runtime and build dependencies include Next.js, React, MDX, remark, Zod, Tailwind CSS, TypeScript, ESLint, Vitest, Playwright, axe-core, and YAML. Before redistribution, generate a dependency license report from the final lockfile and have the owner choose a project license.

Locally bundled fonts:

- Source Serif 4 Variable, SIL Open Font License 1.1.
- Recursive Variable/Mono, SIL Open Font License 1.1.
- Noto Serif SC, SIL Open Font License 1.1. This is the locally packaged
  Simplified Chinese web distribution used instead of a remote font service.

The full license texts are distributed under `public/fonts/`. Font files come
from the pinned Fontsource 5.3.0 packages; there is no font CDN or runtime
download.

`sharp` is used only to create local AVIF/WebP derivatives. The generated
image provenance and publication-rights caveat are recorded in
`ASSET_PROVENANCE.md` and `public/images/ai/README.md`.

No remote image, icon, analytics, or content service is used at build or
runtime.

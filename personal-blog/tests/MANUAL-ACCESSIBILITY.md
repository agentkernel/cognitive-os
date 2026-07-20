# Manual accessibility checks

Automated axe checks cover WCAG 2.0/2.1/2.2 A/AA rules that can be evaluated from the rendered DOM. Before any public release, complete and record these manual checks at 375, 768, 1024, and 1440 CSS pixels:

- Read both home pages, Research, Sources, Method, Lab, and the flagship with NVDA + Firefox or NVDA + Chromium. Confirm heading order, figure titles, captions, source notes, and expanded text alternatives are announced in a useful sequence.
- Traverse every interactive element using only Tab, Shift+Tab, Enter, Space, and Escape. Confirm the skip link is first, focus remains visible, the mobile drawer traps and returns focus on every close path, and no content is unreachable.
- Zoom to 200% and use Windows text scaling at 200%. Confirm reflow does not create page-level horizontal scrolling.
- Use forced-colors mode. Confirm focus, links, the unresolved branch, and diagram boundaries remain distinguishable without relying on color alone.
- Verify Chinese and English pronunciation, especially `CANDIDATE_COMPLETE`, `OUTCOME_UNKNOWN`, Intent, Effect, Verification, and Acceptance.
- Confirm reduced-motion mode removes the drawer animation and smooth scrolling without removing state changes.
- Inspect the five compact article summaries against their wide Research-atlas SVGs and long-text alternatives. No meaning may exist only in SVG position or color.
- Activate every code-copy control before and after hydration. Confirm its localized accessible name and success/failure announcement remain useful.
- Check the active generated research hero in context. Confirm it does not imply live authority state and that its alternative identifies the abstract evidence path.

Record evidence under `artifacts/evidence/accessibility/`; that path is ignored and must not be treated as a committed certification.

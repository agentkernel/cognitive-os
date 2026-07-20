# Manual accessibility checks

Automated axe checks cover WCAG A/AA rules that can be evaluated from the rendered DOM. Before any public release, complete and record these manual checks at 375, 768, and 1440 CSS pixels:

- Read both home pages and the flagship with NVDA + Firefox or NVDA + Chromium. Confirm heading order, figure titles, captions, source notes, and expanded text alternatives are announced in a useful sequence.
- Traverse every interactive element using only Tab, Shift+Tab, Enter, Space, and Escape. Confirm the skip link is first, focus remains visible, the mobile drawer traps and returns focus, and no content is unreachable.
- Zoom to 200% and use Windows text scaling at 200%. Confirm reflow does not create page-level horizontal scrolling.
- Use forced-colors mode. Confirm focus, links, the unresolved branch, and diagram boundaries remain distinguishable without relying on color alone.
- Verify Chinese and English pronunciation, especially `CANDIDATE_COMPLETE`, `OUTCOME_UNKNOWN`, Intent, Effect, Verification, and Acceptance.
- Confirm reduced-motion mode removes the drawer animation and smooth scrolling without removing state changes.
- Inspect the five mobile diagram summaries against their SVG and long-text alternatives. No meaning may exist only in SVG position or color.
- Check both generated abstract-image alternatives in context. Confirm they do not
  imply live authority state and that their captions identify them as abstractions.

Record evidence under `artifacts/evidence/accessibility/`; that path is ignored and must not be treated as a committed certification.

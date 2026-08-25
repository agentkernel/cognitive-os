---
name: control-plane-redesign-workflow
description: >
  Orchestrates CognitiveOS Control Plane WebUI redesign for Cursor development.
  Use when the user asks to redesign Control Plane, rethink IA, redesign pages,
  Apple-style WebUI, Agent/Task/Resource/Activity UX, or prepare WebUI
  implementation. Enforces Product → UX/IA → Apple → Agent UX → Frontend order.
  Does NOT modify UI code until an explicit implementation phase is authorized.
  Does NOT invent backend capabilities. Cursor design-tooling only — not a
  CognitiveOS runtime skill.
---

# Control Plane Redesign Workflow

You are coordinating the **CognitiveOS Control Plane Design Stack** for Cursor
development. These skills are **external design tooling**. They must never be
injected into CognitiveOS runtime, Agent Skill Registry, or daemon behavior.

## Authority order (conflict resolution)

When skills disagree, resolve in this order:

1. **CognitiveOS Reality** — existing APIs, contracts, route inventory, and
   honestly unavailable capabilities (show Unavailable / Not supported; never
   fake buttons or lifecycle).
2. **Apple Product Design** — `/apple-design` (PRIMARY DESIGN AUTHORITY).
3. **UX / IA** — `/ux-design` (PRIMARY UX / IA AUTHORITY).
4. **Agent UX** — `/ai-agent-ux` (+ `/ai-trust-transparency`, `/ai-error-resilience`).
5. **Frontend aesthetics** — `/frontend-design` (implementation polish only;
   must defer to Apple restraint and Control Plane density).

Product skills (`/jtbd-analysis`, `/problem-validation`, `/opportunity-mapping`,
`/scope-cutting`, `/prd-writing`, `/feature-prioritization`) own **why / who /
what / priority** before visual work.

## Apple style (non-negotiable)

Apple style ≠ glassmorphism, Liquid Glass everywhere, blur, giant gradients,
oversized rounded cards, marketing landing pages, or empty whitespace.

Core: Clarity, Deference, Depth, Hierarchy, Consistency, Direct Manipulation,
Feedback, Restraint, Accessibility.

Target feel for Control Plane: **Calm, Dense, Precise, Professional**.

## Mandatory phase gate

Until the user explicitly authorizes implementation:

1. Product discovery (product skills)
2. Current system audit (read existing WebUI / contracts — no code edits)
3. Product model
4. Information architecture (`/ux-design`)
5. User flows
6. Apple interaction design (`/apple-design`)
7. Agent UX (`/ai-agent-ux`)
8. Visual design (Apple-led)
9. Component architecture
10. **Only then** Frontend implementation (`/frontend-design`)
11. Accessibility / Apple / UX review
12. Implementation QA

**Do not write React/CSS or refactor the WebUI in the same turn as design
discovery.** Stop at design specs unless implementation is explicitly requested.

## Capability honesty

If backend lacks cancel/pause/resume/stop/restart/quarantine, SecretStore in
browser, shell, filesystem, or Provider direct access — UI must not pretend.
Record Backend Capability Gaps; do not invent APIs.

## Where the product lives

- Personal Control Plane SPA: `cognitiveos-clients` `pc/web/` (not kernel SPA tree).
- Kernel serves static bundle at `/ui/`; contracts and inventory live in this repo.
- Native `cognitive dsh web` panel is a separate surface — do not conflate.

## Invocation cheat sheet

| Need | Skill |
|------|-------|
| Why / priority / PRD | product skills listed above |
| IA / nav / workbench / master-detail | `ux-design` |
| Apple philosophy / motion / type / restraint | `apple-design` |
| Agent supervision UX | `ai-agent-ux` |
| Trust / errors | `ai-trust-transparency`, `ai-error-resilience` |
| Frontend polish after design lock | `frontend-design` |

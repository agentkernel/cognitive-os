# CognitiveOS Control Plane Design Stack — Skill Manifest

**Scope:** Cursor **project-local** design / product / UX skills for Control Plane
WebUI development.  
**Not** CognitiveOS runtime skills, Agent Skill Registry entries, or daemon
capabilities.

**Installation date:** 2026-08-24  
**Install method:** File-level copy from pinned GitHub commit zipballs / raw
files (no `npx skills`, no global install).  
**Install root:** `.cursor/skills/`

## Personal 2.0 design-corpus use

The installed skill provenance and security record below are unchanged. For the
adopted Personal 2.0 Control Plane target, the stack is applied in this order:

1. current implementation evidence and immutable authority/secret boundaries;
2. adopted product model and six-space IA;
3. UX flows and state/recovery coverage;
4. Apple restraint and accessibility;
5. Agent supervision/trust/error resilience;
6. frontend aesthetics only after the operational model is stable.

The adopted target output is the design corpus `01`–`25` as amended for:

- `Home / Agents / Work / Library / Activity / Settings`;
- desktop primary three-region shell and global Agent Shell;
- Adapter-backed native conversation/history with common projection and native
  slots;
- explicit Manage with Personal into daemon Goal/Plan
  revisions/Tasks/attempts and multi-Agent orchestration;
- seven-family task-oriented placement: Library has Memory/Skills/Tools/MCP,
  Work has Context/Task, and Agents has Runtime/Process;
- Settings Account Hub with ADR-0055 consent and SecretStore/proxy boundaries;
- federated observation and governed writeback;
- one Native/Observed/Governed/Verified timeline;
- calm, dense, precise, professional visuals with full keyboard and reduced
  motion/transparency.

The stack may specify target capabilities, but every capability lacking verified
backend support is labeled **Requires-backend**. It must not invent an API,
active-looking control, progress signal, Provider secret path, MCP host-session
authority, or completion claim. Reality/audit documents `26`–`41`,
[Current State Map](control-plane-current-state.md), and
[Capability Inventory](control-plane-capability-inventory.md) remain the
P7-T05 current-state evidence baseline.

---

## Priority / conflict rules

1. CognitiveOS Reality (APIs, contracts, honest unavailability)
2. Apple Product Design (`apple-design`)
3. UX / IA (`ux-design`)
4. Agent UX (`ai-agent-ux` + trust / error)
5. Frontend aesthetics (`frontend-design`)

Orchestrator: `control-plane-redesign-workflow`

---

## Installed skills

| Skill | Role | Repository | Commit SHA | Source path | Local path | License | Network | Security |
|-------|------|------------|------------|-------------|------------|---------|---------|----------|
| `apple-design` | PRIMARY DESIGN AUTHORITY | [emilkowalski/skills](https://github.com/emilkowalski/skills) | `d23d7f88a2e21c9e4b1418c7abe420f5c1052ba7` | `skills/apple-design/` | `.cursor/skills/apple-design/` | MIT | None | PASS (markdown-only) |
| `ux-design` | PRIMARY UX / IA AUTHORITY | [f0d010c/stark](https://github.com/f0d010c/stark) | `ff94e5b4e1c98d259f3cde9f806406c4528deed4` | `skills/ux-design/` + trimmed `references/` | `.cursor/skills/_vendor/stark/skills/ux-design/` | Apache-2.0 | None | PASS (no scripts copied) |
| `jtbd-analysis` | PRODUCT | [assimovt/productskills](https://github.com/assimovt/productskills) | `66f9cee5868d6daf9cf106b4a74090428d6fa83e` | `skills/jtbd-analysis/` | `.cursor/skills/jtbd-analysis/` | MIT | None | PASS |
| `problem-validation` | PRODUCT (discovery) | assimovt/productskills | `66f9cee5868d6daf9cf106b4a74090428d6fa83e` | `skills/problem-validation/` | `.cursor/skills/problem-validation/` | MIT | None | PASS |
| `opportunity-mapping` | PRODUCT | assimovt/productskills | `66f9cee5868d6daf9cf106b4a74090428d6fa83e` | `skills/opportunity-mapping/` | `.cursor/skills/opportunity-mapping/` | MIT | None | PASS |
| `scope-cutting` | PRODUCT | assimovt/productskills | `66f9cee5868d6daf9cf106b4a74090428d6fa83e` | `skills/scope-cutting/` | `.cursor/skills/scope-cutting/` | MIT | None | PASS |
| `prd-writing` | PRODUCT | assimovt/productskills | `66f9cee5868d6daf9cf106b4a74090428d6fa83e` | `skills/prd-writing/` | `.cursor/skills/prd-writing/` | MIT | None | PASS |
| `feature-prioritization` | PRODUCT | assimovt/productskills | `66f9cee5868d6daf9cf106b4a74090428d6fa83e` | `skills/feature-prioritization/` | `.cursor/skills/feature-prioritization/` | MIT | None | PASS |
| `ai-agent-ux` | AGENT UX AUTHORITY | [varunk130/ai-ux-skill-library](https://github.com/varunk130/ai-ux-skill-library) | `8b0617bcc48d2602d98d6dc585909e9751b4046d` | `skills/ai-agent-ux/` | `.cursor/skills/ai-agent-ux/` | MIT | None | PASS |
| `ai-trust-transparency` | AGENT UX | varunk130/ai-ux-skill-library | `8b0617bcc48d2602d98d6dc585909e9751b4046d` | `skills/ai-trust-transparency/` | `.cursor/skills/ai-trust-transparency/` | MIT | None | PASS |
| `ai-error-resilience` | AGENT UX | varunk130/ai-ux-skill-library | `8b0617bcc48d2602d98d6dc585909e9751b4046d` | `skills/ai-error-resilience/` | `.cursor/skills/ai-error-resilience/` | MIT | None | PASS |
| `frontend-design` | FRONTEND DESIGN AUTHORITY | [anthropics/skills](https://github.com/anthropics/skills) | `3b3fad96af16a10759d930941b4520ba0c40edae` | `skills/frontend-design/` | `.cursor/skills/frontend-design/` | Apache-2.0 (repo README) | None | PASS (markdown-only; whole repo not installed) |
| `control-plane-redesign-workflow` | ORCHESTRATOR | project-local | n/a (authored 2026-08-24) | n/a | `.cursor/skills/control-plane-redesign-workflow/` | project | None | PASS |

### Stark vendor layout note

`ux-design` keeps relative paths `../../references/...`. Files live under:

```text
.cursor/skills/_vendor/stark/
  LICENSE
  NOTICE
  skills/ux-design/SKILL.md
  references/ux-patterns/...
  references/ui-patterns/...   # trimmed to navigation / forms / desktop archetypes
```

Cursor discovers skills recursively; identity is the folder containing `SKILL.md`
(`ux-design`).

---

## Explicitly NOT installed

| Candidate | Reason |
|-----------|--------|
| vercel-labs deploy / claimable skills | Project upload / deploy risk |
| vercel `web-design-guidelines` | Deferred (network-required review); not in this deploy set |
| naplesblue / s1gmamale1 apple packs | Marketing / Liquid Glass bias; prior audit Tier 2 only |
| dickwu / ehmo / ebuntario HIG packs | License / Apple HIG redistribution risk |
| podo design-agent-skills | On-demand install / supply chain |
| nextlevelbuilder ui-ux-pro-max | CLI surface / landing bias |
| heyman333 atelier-ui | License unclear |
| Full anthropics/skills tree | Out of scope; only `frontend-design` |
| Full stark platform skills (`apple-design`, `web-design`, …) | Avoid conflict with emilkowalski `apple-design`; UX-only |
| Global `~/.cursor` / `~/.claude` installs | Forbidden by deploy policy |

---

## Skill roles

| Role | Skills |
|------|--------|
| Product Manager | `jtbd-analysis`, `problem-validation`, `opportunity-mapping`, `scope-cutting`, `prd-writing`, `feature-prioritization` |
| UX Architect | `ux-design` |
| Apple Product Designer | `apple-design` |
| AI Agent UX Designer | `ai-agent-ux`, `ai-trust-transparency`, `ai-error-resilience` |
| Frontend Design Engineer | `frontend-design` |
| Workflow router | `control-plane-redesign-workflow` |

---

## Future redesign workflow

```text
Product Discovery
  → Current System Audit (read-only)
  → Product Model
  → Information Architecture
  → User Flow
  → Apple Interaction Design
  → Agent UX
  → Visual Design
  → Component Architecture
  → Frontend Implementation   ← only when explicitly authorized
  → Accessibility Review
  → Apple Design Review
  → UX Review
  → Implementation QA
```

---

## Discovery verification checklist

After Cursor reload / new Agent chat:

1. Open **Customize → Skills** (or type `/` in Agent chat).
2. Confirm project skills appear: `apple-design`, `ux-design`, product subset,
   `ai-agent-ux`, `frontend-design`, `control-plane-redesign-workflow`.
3. Invoke `/control-plane-redesign-workflow` — should describe phase gates, not
   write React.
4. Confirm `.cursor/rules/*` unchanged by this install.

---

## Security declaration (install session)

- No secrets / `.env` / credentials accessed
- No project upload
- No `npx skills` / npm / pip install into the product tree
- No CognitiveOS runtime skill / registry / backend / API / DB modified
- No `.cursor/rules` modified or overwritten
- No `package.json` modified
- No git commit / push performed by this deployment

---
doc_id: meta.sync-policy
locale: en
kind: meta
audience: [developer, ai]
generated: false
---

# Handbook synchronization policy

（locale-neutral 元规范。）How the handbook stays true as the repository evolves.

## Obligations on every change

1. **Route the impact**: match changed paths against
   [`source-map.json`](source-map.json). Mapped hand-written pages are updated
   and re-fingerprinted (`node tools/src/fill-handbook-fingerprints.mjs`); mapped
   generated pages are regenerated (`node tools/src/generate-handbook.mjs`).
2. **Classify new files**: any new tracked file must match a
   [`source-coverage.json`](source-coverage.json) rule (HB009) — either an
   owning-doc rule or an excluded category with a reason. HTTP route generation
   also consumes `personal/apps/kernel-server/src/personal/tool_lifecycle.rs`,
   `personal/apps/kernel-server/src/personal/pinned_https.rs`, and
   `personal/apps/kernel-server/src/personal/observation.rs` so annotated Tool
   lifecycle, pinned-HTTPS, and observation-plane paths stay bidirectional with
   [`annotations/http-routes.json`](annotations/http-routes.json).
3. **User-visible changes** (CLI, config, errors, security, install, recovery)
   update the user + reference trees; **architectural changes** (data, protocol,
   authority, environments) update the developer + AI trees.
4. **Same change set, enforced timing**: synchronization happens after each
   development step and **before every commit, push, and merge** — not "later in
   the PR". The docs-sync gate fails a commit/push whose mapped source changes
   carry no handbook update; the only escape is an explicit
   `DOCS_IMPACT_NONE="<concrete reason>"` acknowledgment, recorded in the
   commit/PR description (blank or trivial reasons are rejected). Tools fixtures
   such as the P7-T05 Web UI route inventory
   (`tools/src/p7_t05_web_ui_inventory.mjs`) and the Personal Linux RC binder
   (`tools/src/personal-rc-gate.mjs`) follow this same routing: they are
   not a reason to skip handbook review.
5. **Checks**: `node tools/src/check-handbook.mjs`,
   `node tools/src/generate-handbook.mjs --check`, and
   `pnpm run check:consistency` must pass; CI runs them on every PR. Path
   existence in the consistency and agent-rule checkers is Git-tracked-only
   (`git ls-files`, P0-T09): linking a file that exists only in your working
   tree fails locally exactly as it fails in CI.
   Measurement-only C1/C2 paired instruments live in
   `tools/personal/c1-c2-paired/` and are covered by
   `tools/test/c1_c2_paired_p_arm.test.mjs` (broker, Secret Service get helper,
   fairness checker, freeze ledger, frozen-system-task-prompt.txt, live
   `--append-system-prompt` command manifests, live `runLivePairedCell`
   (injected `executeArm`; counted b1/b2 only), and P-arm unified-diff
   WorkspacePatch). They are not Gate evidence.

## Enforcement layers

| Layer | Mechanism |
|---|---|
| Canonical obligation | [`docs/standards/docs-sync-contract.md`](../../../docs/standards/docs-sync-contract.md) §2 handbook block (all change classes) + §5 items 16–17 |
| Pre-commit / pre-push gate | `tools/src/docs-sync-gate.mjs` (`--staged` / `--push`): source-map routing, conditional handbook check set, fail-closed on unsynced mapped changes; wired through repo `.githooks/` — enable once per clone with `pnpm run hooks:install` |
| Editor/AI guidance | `AGENTS.md` §5 quick table, the always-applied rule `.cursor/rules/10-…` (checkpoint/closure obligations) and the glob-attached rule `.cursor/rules/20-cognitiveos-personal-handbook-sync.mdc` (attached whenever `core/`, `personal/`, `tools/`, `.github/`, `.githooks/` or root manifests are edited; adapters only — this file + the contract own the policy) |
| Non-Cursor AI tools | root `llms.txt` + [`personal/handbook/en/ai/README.md`](../en/ai/README.md) |
| Machine gate (CI, unconditional) | `check-handbook.mjs` rules HB001–HB016 (manifest, pairing, links, sources, symbols, fingerprints, coverage, generated equality, forbidden content, source-set reproducibility, and HB016: a `source-map.json` rule that declares `symbols` — e.g. `pi-official-package-pin` on `installer.rs` `OFFICIAL_PI_PACKAGE` — must be pinned by every routed hand-written page in every locale) + generator `--check` byte equality |
| Task-closure gate | `check-handbook.mjs --diff-base <rev>` proves legacy docs changed only on the allowlist ([`legacy-change-allowlist.json`](legacy-change-allowlist.json)) |

## Failure semantics

A red handbook check is a build failure equal to any other: fix the page, the
mapping, or the generator input — never bypass, and never "fix" a canonical
source to match documentation. Checker changes themselves require rerunning the
negative fixtures (`tools/test/handbook-check.test.mjs`).

## Baseline advancement

`source-set.json` records the implementation baseline revision the content was
authored against. Advancing it is a deliberate act: read the increment, update
affected pages, regenerate, refresh fingerprints, and record the new revision in
the same PR.

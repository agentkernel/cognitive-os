## 20260726 Current Project Close-Out Handoff

> **SUPERSEDED (2026-07-26).** This note describes the working tree as it stood
> *before* the owner extracted the `clients/` documentation domain into
> <https://github.com/agentkernel/cognitiveos-clients> and before the Linux
> toolchain was restored in this environment. Change set **C** below no longer
> applies to this repository — those files now live in the external repo.
> Its packaging recommendation was carried out (batches landed as two commits).
> The authoritative continuation record is
> [20260726-toolchain-recovery-and-worktree-landing-handoff.md](20260726-toolchain-recovery-and-worktree-landing-handoff.md).
> Retained unchanged below as a point-in-time record.

### 1. Purpose

This handoff closes the current session by consolidating the active working tree
into a single operator-facing summary: what changed, what was validated, what is
still blocked, and how the current edits should be packaged for submission.

It does **not** change any task status, Gate outcome, Profile claim, release
claim, or readiness conclusion on its own. It is an informative close-out note
for the current working tree.

### 2. Working-tree scope at close-out

The uncommitted working tree currently groups into three coherent change sets.

#### A. Personal runtime and evidence plumbing

- `apps/pi-agent-adapter/src/main.rs`
- `crates/cognitive-runtime/src/perf.rs`
- `crates/cognitive-conformance/src/main.rs`
- `scripts/v01-auto-run.ps1`
- `scripts/v01-auto-run.sh`
- `tools/src/validate-manifest.mjs`
- `tools/test/check.test.mjs`

Scope summary:

- adds a pinned `extension-load` path for `P0-T06` local evidence collection,
  still candidate-only and still bounded by the ADR-0018 local-development
  exception;
- hardens V01 local verification/performance plumbing so the release-candidate
  manifest validates real local result/performance references instead of relying
  on implicit sample assumptions;
- adds regression coverage in repo-tools for the evidence-graph / orchestrator
  guardrails.

Important boundary:

- this batch does **not** complete `P0-T06` by itself;
- no Profile, containment, Gate, or release claim is added;
- Windows GNU linker `exit 121` remains the documented unsupported baseline in
  this environment.

#### B. Personal planning, ADR, and progress synchronization

- `docs/adr/0026-personal-trust-profile-low-friction-authorization.md`
- `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`
- `docs/plan/PI-AGENT-INTEGRATION-PLAN.md`
- `docs/plan/PERSONAL-SUPPORT-MATRIX.md`
- `docs/plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md`
- `docs/plan/personal-trace.yaml`
- `docs/plan/PROGRESS.md`
- `docs/plan/plan.md`
- `docs/checkpoints/20260726-personal-p0-t06-extension-poc-handoff.md`
- `docs/checkpoints/20260726-personal-p2-cards-expansion-handoff.md`
- `docs/plan/AUTOPILOT-PROMPT.md`
- `docs/research/20260726-frontier-review-and-environment-perception.md`

Scope summary:

- records ADR-0026 (low-friction authorization / trust profile) and syncs its
  implications into the Personal planning surface;
- expands the `P2-T01..P2-T08` task cards into full `docs/plan/plan.md` field sets;
- aligns Personal progress text, critical-path wording, and traceability with
  the current task inventory, including the `P7-T07` destination and ADR-0018
  expiration check;
- preserves the explicit non-claim boundary for Personal Gates / Profile /
  release.

Important boundary:

- these edits are predominantly planning/governance synchronization;
- they do not by themselves create new executed evidence beyond what is
  explicitly cited in `PROGRESS.md` and the referenced handoffs.

#### C. `clients/` governance, review, and execution-plan cleanup

- `clients/README.md`
- `clients/GOVERNANCE.md`
- `clients/READINESS.md`
- `clients/plan/README.md`
- `clients/plan/milestones.md`
- `clients/plan/progress.md`
- `clients/plan/development-plan.md`
- `clients/prompts/README.md`
- `clients/prompts/continuous-development-execution.md`
- `clients/review/2026-07-26-clients-design-review.md`
- `clients/review/2026-07-26-clients-development-plan.md`
- `clients/governance/decision-log.md`
- `clients/governance/evidence-index.md`
- `clients/shared/docs/test-strategy.md`
- `clients/pc/app/README.md`
- `clients/pc/docs/platforms/desktop-parity-matrix.md`
- `clients/pc/docs/product/requirements-traceability.md`
- `clients/pc/docs/quality/README.md`
- `clients/mobile/android/docs/android-product-design.md`
- `clients/mobile/ios/docs/ios-product-design.md`
- `clients/mobile/shared/docs/mobile-parity-matrix.md`
- `clients/mobile/shared/docs/mobile-platform-decision-log.md`
- `clients/agent-hub/docs/README.md`
- `clients/agent-hub/docs/decisions/decision-log.md`
- `clients/agent-hub/docs/planning/README.md`
- `clients/agent-hub/docs/progress.md`
- `clients/agent-hub/docs/traceability/evidence-index.md`
- `clients/agent-hub/plan/README.md`
- `clients/agent-hub/plan/progress.md`
- `clients/agent-hub/prompts/README.md`

Scope summary:

- introduces a formal design review and an operative client development plan;
- removes stale hard-coded global counts from `clients/**` and replaces them
  with `PROGRESS.md` pointers and dated snapshots;
- clarifies `clients/` governance responsibilities, document-system layering,
  and the currently executable Phase 0/1 work surface;
- registers/propagates the PoC-code exemption path so `clients/**` can be kept
  documentation-only while PoC harness work lands outside the client tree.

Important boundary:

- `clients/**` remains documentation/governance/planning only;
- no client implementation, test, or Profile claim is introduced;
- gate state remains blocked exactly where the canonical readiness sources say
  it is blocked.

### 3. Validation performed in this close-out session

Executed from `D:\agent-kernel`:

1. `git diff --check`
   - **result:** passed
   - notes: CRLF-to-LF warnings only on several Markdown/test files; no
     whitespace error or conflict marker was reported.

2. `pnpm run check:consistency`
   - **result:** passed
   - summary: `273 requirements, 55 error codes, 63 schemas, 85 vectors` and
     markdown/traceability verification OK.

3. `pnpm --filter @cognitiveos/repo-tools test`
   - **result:** passed (`4 passed / 0 failed`)
   - scope: includes the new evidence-graph / orchestrator safeguard coverage.

4. `cargo test -p pi-agent-adapter`
   - **result:** blocked in this host baseline
   - detail: dependency fetch completed, then Rust build failed on
     `x86_64-w64-mingw32-gcc` linker `exit 121`.
   - interpretation: consistent with the repository's already-documented
     unsupported Windows GNU baseline; this is not a new regression claim.

### 4. Submission packaging recommendation

Do **not** squash the whole working tree into one opaque change. The current
state is materially easier to review if split into the following commits/PR
chunks.

#### Recommended batch 1 - clients documentation close-out

Suggested title:

- `docs(clients): add design review and operative recovery plan`

Include:

- `clients/**` documentation/governance/plan/review changes only.

Why separate it:

- documentation-only, low-risk, reviewable by Lane-CON/Lane-DOC readers;
- independent from Rust/tooling execution changes.

#### Recommended batch 2 - Personal planning and ADR synchronization

Suggested title:

- `docs(personal): sync trust-profile ADR and Personal planning`

Include:

- `docs/adr/0026-*`
- Personal planning/progress/trace files
- `docs/plan/plan.md`
- related Personal handoff/progress note updates

Why separate it:

- keeps planning/governance intent distinct from executable code behavior;
- makes the ADR-0026 adoption trail easier to audit.

#### Recommended batch 3 - Pi evidence mode and V01 verification hardening

Suggested title:

- `feat(pi): add local extension-load evidence mode`

Suggested companion title if split again:

- `fix(verify): harden local evidence manifest validation`

Include:

- adapter/runtime/conformance/script/tooling/test changes.

Why separate it:

- code-bearing change with targeted runtime/test implications;
- easiest batch to validate independently once a supported Rust baseline is
  available.

### 5. Remaining blockers after close-out

1. **Personal `P0-T06` is still not complete.**
   - missing: isolated, redacted, real Pi Extension session/RPC load evidence
     on an eligible Linux-native host;
   - WSL remains intentionally ineligible for the ADR-0018 exception path.

2. **Windows GNU Rust baseline is still unsupported here.**
   - the current `cargo test -p pi-agent-adapter` result is environment-limited
     by linker `exit 121`.

3. **Client program remains planning/gate-bound.**
   - the new client review/plan set improves navigability and truthfulness, but
     does not unblock implementation by itself.

4. **Owner/environment-provided prerequisites still matter.**
   - Linux-native host / secret-store route for real Pi load evidence;
   - remaining external/legal/platform items already tracked by the canonical
     plans.

### 6. Recommended next action

Choose one of these two next entries instead of mixing them again:

1. **Docs-first path**
   - submit batches 1 and 2 first, because they already have successful
     consistency/test coverage and no code-runtime dependency on a supported
     Rust linker baseline.

2. **Runtime-first path**
   - move to an eligible supported Rust/Linux-native environment, execute the
     real `P0-T06` Extension load evidence step, then submit batch 3 with the
     resulting evidence-linked docs updates.

### 7. Session-end boundary

- No commit was created in this close-out session.
- No push or PR was created in this close-out session.
- No existing user changes were reverted.
- This handoff is the recommended operator entry point for packaging the
  current working tree into reviewable submissions.

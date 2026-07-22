# 20260722 Lane-CTR V02 CA SIG Design Handoff

## 1. TARGET merge and SIG execution gate

- TARGET PR [#52](https://github.com/agentkernel/cognitive-os/pull/52) was
  reverified `MERGED`, head
  `18e66bd171c1c0284f05f179307609b0907b4aee`, base `main`, merge commit
  `42d609b2f49e2db641f46aa99b6cc9a538a7f4fd`, merged at
  `2026-07-22T13:07:36Z`.
- PR #52 changed exactly 11 docs paths. Reviews, review decision, and requested
  reviewers were empty. No review exception from PR #50 or #51 applied to
  TARGET, and no TARGET merge/review behavior applies to SIG.
- Merge-triggered main CI run
  [29922529556](https://github.com/agentkernel/cognitive-os/actions/runs/29922529556)
  is a `push` run at the merge commit. `verify (ubuntu-latest)` and
  `verify (windows-latest)` both completed `success`.
- Remote `main` was reverified at the same merge commit with zero later commits
  and no contrary governance decision. The authenticated repository owner
  account `agentkernel` had admin/maintain/push permission; no revocation or
  contrary owner ruling was found.
- TARGET remained docs-only: all three configure candidates blocked, no machine
  registration, no implementation, no new behavior execution, no Profile
  claim, D-016 open, D-022 blocking, and CA-1 through CA-8 blocked.

## 2. Branch and protection record

- SIG branch: `lane/ctr-v02-ca-sig-design`.
- Created directly from verified
  `origin/main@42d609b2f49e2db641f46aa99b6cc9a538a7f4fd`, not from the TARGET
  branch.
- Before creation and immediately after switching, tracked worktree and index
  were clean.
- Existing untracked bypass set: 40 paths; path-set SHA-256
  `719a1de0e0c5ffeecf442d01605fdae48400980ac3247d6daaf6b842f8da5f79`
  before and after branch creation. Paths were listed only and their business
  content was not read.
- `History/**` and `personal-blog/**` were not read, accessed, modified,
  staged, or committed.

## 3. SIG design completed

- Added `V02-CA-SIG-01`, a docs-only source audit, binding decision,
  verification order, error responsibility table, migration plan, and 40-case
  planned negative matrix for session/approval signatures.
- Added proposed ADR-0012 for detached-signature profile governance.
- Structural decision: a reusable detached-signature envelope/profile family
  is viable, but session and approval require independent object-specific
  profiles. A generic or cross-object signature domain is forbidden.
- Proposed session profile/domain:
  `cognitiveos.signature.management-session-authority/0.2` /
  `management-session-authority/0.2`.
- Proposed approval profile/domain:
  `cognitiveos.signature.management-approval-authority/0.2` /
  `management-approval-authority/0.2`.
- All profile/family versions are proposed `0.2.0-draft.1`; every digest is
  `unresolved/not computed`. Nothing is registered.

## 4. Machine and implementation audit result

- `PrivilegedManagementSession.authority_signature` remains a required
  `string(minLength=16)`.
- `ManagementApprovalDecision.authority_signature` remains a required
  `string(minLength=16)`.
- No registered detached-signature schema, algorithm set, key ID/resolver,
  signature domain, signed schema/projection/exclusion contract, encoding,
  trust root, rotation/revocation profile, verification receipt, or complete
  crypto-error mapping exists.
- `AuthorizationCapability.signature` is also an open string and was not used
  as precedent or redesigned.
- `MGMT-CONFIG-001.authority_signature_valid: true` and
  `MGMT-APPROVAL-005.independent_signed_decision_required` are vector facts,
  not verification profiles.
- Rust/TypeScript canonical helpers construct section 12 preimages only. The
  session parser shape/length-checks its signature, and the approval test helper
  emits a literal fixture signature. Those implementation-private facts are
  not cryptographic verification or authority.

## 5. Shared family and object-specific bindings

The proposed shared envelope carries only profile triple, algorithm, key ID,
signed domain, signed schema digest, signed content digest, negotiation epoch
digest, and encoded signature bytes. Algorithm, key, resolver, trust root,
projection, and exclusions cannot be caller supplied.

Object-specific differences remain mandatory:

- independent profile ID/version/digest and signature domain;
- independent signed schema/projection and exact exclusions;
- session-signing versus approval-signing key usage;
- session issuance/renewal/revocation versus approval challenge/independence/
  single-use business rules;
- independent replay keys, lifetime checks, receipts, and error responsibilities.

Digest integrity, signature validity, signer/key authorization, trust-root
validity, rotation/revocation status, and current business authorization are
separate checks. No one check implies another.

## 6. Algorithm, key, and trust alternatives

No unique selection was possible from current digest-pinned facts. The design
records these bounded alternatives for owner/security review:

- algorithms: Ed25519 with fixed raw unpadded-base64url signature; fixed-width
  low-S P-256 ECDSA; or a strictly pinned two-algorithm set only if a proven
  compatibility need justifies the downgrade surface;
- key resolution: governed CognitiveOS authority-key registry or immutable
  external KMS/PKI resolver profile;
- trust: platform-rooted authority, with bounded tenant delegation only if a
  tenant-scoped authority requirement is proven.

Both profiles remain blocked until the owner/security and identity/KMS reviewers
select the allowed set, identifiers/encoding, key ID and resolver, trust roots,
usage, rotation, revocation, freshness/cache/outage rules, and exact errors.

## 7. Signed projections

### Session

- content-digest domain proposal: `management-session-content/0.2`;
- `session_digest` projection excludes exactly `/session_digest` and
  `/authority_signature`;
- subject projection excludes exactly `/authority_signature` and includes the
  recomputed `session_digest` plus every present schema-known field;
- signed projection is a closed binding record adding exact profile, algorithm,
  key, signed-schema, canonical-profile, specification-set, and negotiation
  epoch facts; signed-projection digest domain is
  `management-session-signed-projection/0.2`;
- any renewal, activity/expiry, scope/risk, policy/revocation, state, or other
  signed-field change requires a new object version, digest, and signature.

### Approval

- content-digest domain proposal:
  `management-approval-decision-content/0.2`;
- `decision_digest` projection excludes exactly `/decision_digest` and
  `/authority_signature`;
- subject projection excludes exactly `/authority_signature` and includes the
  recomputed `decision_digest` plus every present conditional/optional field;
- signed projection is a closed binding record adding exact profile, algorithm,
  key, signed-schema, specification/negotiation, revocation, session
  version/digest, and approval-request schema/content-digest facts;
  signed-projection digest domain is
  `management-approval-signed-projection/0.2`;
- proposal, request/challenge, session, decision, authority, approver/
  ActorChain, independence, policy, risk, step-up, decision/expiry, and
  single-use bindings remain in the signed projection where applicable.

Current v0.1 schemas cannot be the final signed schemas because they contain
only string signatures and do not register these projections. Future v0.2
schema/profile identities and digests remain a machine-registration task.

## 8. Verification, errors, receipt, and audit

- G0: framing/channel/authenticated peer.
- G1: negotiation epoch/specification set/signature critical extension.
- G2: profile/algorithm/signed schema/domain.
- G3: projection/digests/encoding/signature bytes.
- G4: resolver/trust/key status/usage/signer authority.
- G5: session/approval/capability/risk/target/independence authorization.
- G6: verification receipt/authoritative audit/commit.

All G1-G4 signature checks finish before business authorization. Signature
success is followed by current business authorization. Receipt or audit failure
cannot report success.

Exact existing-code reuse is limited to the registered condition: critical
extension, lossy mapping, supported version window, pinned protocol schema,
schema shape after registration, declared digest mismatch, self-authorization,
missing independent approval, session expiry/revocation/scope/step-up, and
authoritative persistence failure.

Unknown profile/algorithm/key, downgrade, malformed signature, wrong domain or
projection, revoked/rotated key, unauthorized usage, trust mismatch, invalid
signature, replay, and general object/policy/revocation/negotiation mismatch do
not have complete exact registered error closure. No nearby code was repurposed.

SIG owns verification facts and a logical receipt slot. AUDIT still owns the
authoritative carrier/profile/persistence port and atomic audit slot. Event open
payload, transition record, outbox, SQLite row, and AKP `audit_ref` remain
insufficient.

## 9. Planned tests and evidence boundary

- The SIG matrix contains 40 negatives, all `planned/not executed`.
- Common rejection oracle is dispatches/effects/business mutations/commits/
  success receipts all zero.
- Future vectors must be new assets; existing `expected` values are unchanged.
- No behavior vector was executed and no conformance evidence was created or
  changed.
- Static checks, builds, and ordinary unit tests are repository-integrity
  evidence only; they do not prove cryptographic behavior.

## 10. Docs sync and validation

- Synchronized OPS release notes, finite compatibility window, migration plan,
  PROGRESS, PARALLEL-LANES, POST-v0.1 plan, findings ledger, and Lane-CTR prompt.
- Impact scan covered D-016, D-022, IMP-01, Configuration Authority, signature
  profiles/envelopes, session/approval, algorithm/key/trust/rotation/revocation,
  domains/projections/exclusions, canonical encoding/digest, specification/
  schema/operation sets, critical extensions/epochs, authorization
  non-expansion, OPS/TARGET/SIG/AUDIT boundaries, and four-state terminology.
  Scanned tracked `docs`, `specs`, `conformance`, `crates`, `packages`, `apps`,
  `tests`, and `.cursor`; explicitly excluded `History/**`, `personal-blog/**`,
  `target/**`, `node_modules/**`, `dist/**`, and untracked bypass content.
- `pnpm run check:consistency`: pass (273 requirements / 55 errors / 61
  schemas / 84 vectors; Markdown links and traceability verified).
- `node tools/src/gen-matrix.mjs --check`: pass.
- `git diff --check`: pass.
- `pnpm -r build`: pass.
- `pnpm -r test`: pass (contracts-ts 38; tools 2; sdk-ts 69 pass / 3 skip;
  agent-shell 13).
- Additional local `cargo test --workspace` did not enter test execution: the
  known Windows GNU environment still lacks linker libraries `libgcc_eh` and
  `libgcc`. This is an environment limitation, not a SIG document failure;
  SIG PR Ubuntu/Windows CI is the required Rust build/test gate.
- These checks are repository-integrity evidence only and are not new
  behavior-vector execution.

## 11. Commit, PR, CI, and review snapshot

- SIG primary design commit: pending.
- SIG PR: pending; base must be `main`, docs-only, and independent from TARGET.
- SIG head checks: pending.
- Owner/security reviewer status: pending. No reviewer request is created without
  an explicit user instruction naming the reviewer.
- SIG PR must not be auto-merged.

## 12. Status and pins

- D-016: OPS/TARGET merged; SIG design materialized; registration pending; not
  closed.
- D-022: OPS/TARGET merged; SIG design materialized; AUDIT and four machine
  registrations pending; remains a blocker for CA-1 through CA-8.
- IMP-01: v0.1 freeze unchanged; this is a docs-only v0.2 structural design and
  registers no proposed structure.
- Pins remain 273 REQ / 55 errors / 61 schemas / 84 vectors / 59 pass / 25
  not-run / self-check 40 / matrix impl 70 / Profile implemented 0.
- Machine contracts remain unregistered; Configuration Authority implementation
  not provided; new behavior not executed.

## 13. Next unique entry

1. wait for SIG owner/security review and ordinary merge;
2. wait for merge-triggered main CI Ubuntu/Windows success;
3. AUDIT design;
4. OPS/TARGET/SIG/AUDIT four independent machine-registration batches;
5. CA-0 re-review;
6. explicit CA-0 GO;
7. implementation;
8. Management CFR.

Suggested continuation prompt: `docs/prompts/lane-ctr.md`.

Final status: SIG design materialized for owner review; machine contracts
remain unregistered.

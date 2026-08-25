# 32 — Provider / Session / Security Map (Authority Model Verification)

- Phase 2.5 (audit only)
- Date: 2026-08-24
- Covers brief §8 (authority model), §11 (provider reality), §12 (session reality). Sources: `apps/kernel-server/src/personal/{server.rs,auth.rs,provider_control_plane.rs,provider_proxy.rs,bounds.rs}`, ADR-0053, `pc/web/src/{channels.ts,session.ts,api.ts,policy.ts}`.

---

## 1. Authority chain (verified)

```text
Browser (untrusted client, static SPA from GET /ui/)
  │  Authorization: Bearer <channel-token>   (no cookies — hard-rejected)
  │  Origin/Referer = daemon loopback origin (when present)
  ▼
Front door (server.rs:443-453 loopback bind; 589-598 cookie reject;
            2781-2802 Host check; 2853-2894 Origin/Referer allowlist;
            2685-2756 bounded read; 481-503 connection/in-flight caps)
  │  channel-scoped session auth (auth.rs: task vs management; mgmt == daemon-admin)
  ▼
Typed route handlers (task_api / resource_* / provider_control_plane / …)
  │  preview → admission (CAS/epoch/principal) → scheduler → executor
  ▼
Authority stores (SQLite via cognitive-store; event log; SecretStore via approved backend)
```

**The daemon is the only authority writer — confirmed in implementation**, not just documentation: all mutation routes are daemon handlers; the SPA's only write path is `fetch` with a channel bearer; no browser-reachable route touches SQLite/SecretStore/filesystem/provider egress directly.

## 2. Violation scan (browser privilege boundaries)

| Check | Result | Evidence |
|---|---|---|
| Browser writes state directly (non-HTTP)? | **None found** — SPA has no storage/db/filesystem access paths; tokens memory-only with a self-check that throws on web-storage token material (`session.ts:9-30`) | PASS |
| Browser accesses secrets? | Key material only ever in the `POST …/accounts/key` request body; responses redacted via `redactSecrets` (`policy.ts:37-59`); daemon stores into SecretStore (`bind_key_and_discover`, `provider_control_plane.rs:478-553`) | PASS |
| Browser accesses SQLite? | No route exposes raw SQLite; backup excludes it (`user_backup.rs` module docstring) | PASS |
| Browser accesses filesystem? | None; `/ui/*` has a strict segment allowlist + 1 MiB cap (`server.rs:2943-2989`) | PASS |
| Browser accesses provider credentials? | `secret_ref` is serialized in account responses (opaque identifier — **flagged, R-5**: display presence/absence only); no key material ever returned | PASS with display rule |
| Browser performs privileged lifecycle operations? | None exist over HTTP (agent lifecycle is admin-cli/store-direct, `main.rs:111-128`); forbidden routes are implemented refusals | PASS |

No violations found. Two hygiene flags carried forward: R-1 (200-stub fallthrough), R-5 (`secret_ref` serialized).

## 3. Provider reality (what is authoritative)

| Fact | Authority | Browser can read | Browser can mutate |
|---|---|---|---|
| Accounts (id/name/kind/endpoint/trust/status/catalog_revision) | daemon SQLite | yes (list/inspect) | yes — create/update/delete via typed routes (class A) |
| API keys | **SecretStore only** (daemon resolves; material exists only in daemon memory during egress) | presence/absence inferred from `secret_ref` presence + readiness `secret_ref_resolves` | set/rotate/remove via `accounts/key` body (one-way write; never read back) |
| Model catalog | daemon (discovered snapshot or manual entries) | yes | refresh (bounded probe), add manual, set-price |
| Bindings (agent↔account+model, revision CAS) | daemon | yes | set/remove with CAS |
| Usage events/aggregates | daemon (post-hoc metering; `metering_source` honesty) | yes (no filters) | no |
| Budgets | daemon | yes | set/remove — **observe-only; no enforcement hook in proxy path** |
| Alerts | daemon (80%/100% deduped) | yes | acknowledge |
| Audit (provider plane) | daemon append-only | yes (no filters) | no (written by mutations) |
| Provider egress (chat completions) | daemon proxy (bound binding; legacy fallback; SSE passthrough) | n/a (SPA does not call models) | n/a |

**Daemon-only forever (contract):** key material, SecretStore access, provider network egress, endpoint/DNS/redirect policy, trust enforcement. The UI never receives key material, raw headers, raw prompts/completions, or raw provider responses.

## 4. Session reality (UI session vs daemon authority session)

| Fact | Reality | Evidence |
|---|---|---|
| Creation | `POST /local/session` with bootstrap secret + channel + principal; constant-time bootstrap compare | `auth.rs:276-281`, `server.rs:1020-1059` |
| Token form | `sess-<hex>-<hex>`, 32-byte CSPRNG | `auth.rs:381-418` |
| Storage (daemon) | **in-process `HashMap` only** — daemon restart invalidates all sessions | `auth.rs:155-163` |
| Storage (browser) | JS module memory only; localStorage/sessionStorage/IndexedDB/URL/history forbidden + self-checked | `session.ts:3-30`; ADR-0053 §3 |
| Expiry | absolute 12 h, idle 30 min; touched per authorized request | `bounds.rs:33-34`, `auth.rs:352` |
| Channels | task vs management, disjoint; cross-use → 403 `SHELL_CHANNEL_BINDING_MISMATCH`; management == daemon-admin boundary | `auth.rs:13-34, 340-342, 356-369` |
| Principal | caller-supplied `principal_id` at issuance (≤128 chars); task acceptance is bound to the session principal server-side (`accepted_by` must equal session principal) | `task_api.rs:572-578` |
| Logout/revoke | **no endpoint**; in-process `revoke_all` on shutdown only | `auth.rs:372-375` |
| UI session vs authority session | **not identical and never conflated**: the UI bearer is a channel credential; authority identity is the principal derived from it server-side. The SPA cannot assert a foreign principal. | design + enforcement above |

Browser bootstrap ergonomics (verified): there is **no browser-specific auth flow** — the SPA obtains a bearer exactly like the CLI (bootstrap secret from the local runtime dir, pasted by the owner). No CORS headers exist; cross-origin browser fetches from other loopback ports would fail preflight — only same-origin `/ui/` fetches work. This is intentional per ADR-0053 and is the BD-9 ergonomic pressure point.

## 5. Threat-model binding status (ADR-0053 §5 vs implementation)

| Threat | Control | Verified in code |
|---|---|---|
| CSRF | no cookies; explicit bearer; Origin allowlist | `server.rs:589-598, 2853-2894` |
| DNS rebinding | loopback Host check; same-origin `/ui/`; no CORS | `server.rs:2781-2802` |
| Token theft | memory-only; no URL/storage; idle/absolute expiry; restart revoke | `auth.rs`, `session.ts` |
| Channel confusion | disjoint clients; fail closed | `auth.rs:340-342`; `channels.ts:37-45` |
| XSS / untrusted output | daemon-side redaction + client escaping + CSP `default-src 'self'` | `policy.ts:170-178`; `server.rs` CSP headers |
| Secret leak | key/SecretRef negatives on DOM/URL/storage/logs/errors | `api.ts:12-14, 56-60`; `channels.ts:70-72` |
| Remote exposure | loopback only | `server.rs:443-453` |

## 6. Consequences for the approved design

1. Phase-2 session chrome (`12` §5, `20` §5) matches reality: memory-only, expiry countdown displayable from `absolute_expiry_secs`/`idle_expiry_secs`, no logout button (BD-7 honesty note).
2. Provider spec (`17`) credential rules match implementation exactly; the "secret present/absent/unknown" display rule is enforceable from `secret_ref` presence + readiness resolution facts without exposing the ref value.
3. Principal binding means the UI must surface *which principal* is admitting (session cell) — already in the shell spec.
4. BD-9 (bootstrap ergonomics) is confirmed as a real ergonomic gap with a hard security boundary; any improvement needs owner + security review.

---

*Event/activity/evidence reality is mapped in `33`; agent reality in `31`; work/task/run reality in `30`.*

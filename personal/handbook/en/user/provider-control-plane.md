---
doc_id: user.provider-control-plane
locale: en
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: personal/apps/admin-cli/src/personal_cli/provider.rs
    symbols: ["parse_provider_args", "parse_agent_args", "CONTROL_PLANE_FLAGS"]
  - path: personal/apps/admin-cli/src/personal_cli/mod.rs
    symbols: ["COGNITIVE_USAGE"]
  - path: personal/apps/admin-cli/src/personal_cli/secret_input.rs
    symbols: ["read_api_key_material"]
  - path: personal/apps/kernel-server/src/personal/provider_control_plane.rs
    symbols: ["PI_AGENT", "DSH_AGENT", "set_binding", "query_usage"]
  - path: personal/apps/kernel-server/src/personal/provider_proxy.rs
    symbols: ["BindingMismatch"]
  - path: personal/crates/cognitive-secret/src/endpoint_trust.rs
    symbols: ["TrustedEndpoint", "ProviderKind"]
  - path: personal/crates/cognitive-store/src/provider_control_plane.rs
    symbols: ["USAGE_EVENT_RETENTION_MS", "USAGE_AGGREGATE_RETENTION_MS", "honest_usage_read_model", "labelled_cost_source"]
  - path: personal/docs/product/provider-control-plane.md
  - path: personal/docs/product/account-hub.md
  - path: personal/docs/product/account-hub.zh-CN.md
  - path: docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
tests:
  - personal/apps/kernel-server/tests/p8_t13_provider_control_plane.rs
  - personal/crates/cognitive-secret/tests/p8_t13_endpoint_trust.rs
  - personal/crates/cognitive-store/tests/p8_t13_provider_store.rs
  - personal/crates/cognitive-store/tests/p11_t12_honest_usage.rs
  - personal/apps/admin-cli/src/personal_cli/mod.rs
fingerprint: "sha256:bc83f3d0e77bdb232af7475ef4432af4455ceb71502851f5cb9bbcb648464334"
non_claims:
  - This page documents the shipped daemon API, cognitive CLI, and current localhost Web UI path. It does not claim live Secret Store proof, live Provider/Pi/dsh qualification, Gate, release, Profile, B01, the Personal 2.0 desktop redesign/Account Hub import, or Agent-benefit.
---

# Provider Control Plane

`partial`: the daemon management API and the `cognitive` CLI callers below are
implemented and covered by focused tests. A localhost-only Web UI in this
repository at `clients/pc/web/`, served same-origin from `GET /ui/`, is a
daemon client for the same management routes: named accounts, SecretStore key
handoff, bounded probe, and fixed Agent bindings. The adopted desktop-first
Personal 2.0 redesign is **not implemented**. The
native dsh control panel (`cognitive dsh web`, default `http://127.0.0.1:3080`)
is a separate dsh-owned UI, not this Provider Control Plane surface and not
Personal `/ui/`. Live
Secret Store rotate/remove and live Provider/Pi/dsh qualification remain
fail-closed when the store or upstream is unavailable; they are not Gate
proof.

Exact verb text also appears in the generated
[CLI reference](../reference/cli-cognitive.md). Secret-store mechanics that
`cognitive init` already uses are in
[Provider and secrets](provider-and-secrets.md).

## What it is

The Provider Control Plane is the owner-local way to manage **named LLM
accounts**, store API keys only in an approved OS Secret Store, keep a model
catalog, bind the Pi agent and the DeepSeek harness (`dsh`) to one fixed
account+provider+model each, and query usage, observe-only budgets, alerts, and
a redacted audit log.

The daemon is the only writer. The CLI and the localhost Web UI are
non-authority clients: they never open SQLite or the Secret Store. They send
management-channel HTTP to the daemon. The browser must not keep the API key
in the DOM after submit, URL, or Web storage; SecretRef is shown only as
present/absent. Task-channel copies of these routes return HTTP 403
`PROVIDER_CONTROL_CHANNEL_FORBIDDEN`.

This plane does **not** replace `cognitive init`. First-conversation setup
still writes `provider.json` / `selected-model.json`. Unbound agents still use
that pair. Once you set a control-plane binding for `pi` or `dsh`, that
binding is the only allowed account+model for that agent — `provider.json` is
not a fallback.

Not shipped by the current API: OAuth, browser/Agent credential import,
automatic routing or load balancing, hard budget blocking, third-party
Anthropic-compatible endpoints, background model refresh, and the adopted
desktop-first Account Hub redesign. The first two and the redesign are
Personal 2.0 targets marked `Requires-backend`, not current features. The Web
UI does not invent Task cancel or Agent pause/resume/stop/restart/quarantine
HTTP.

## Personal 2.0 Account Hub target (`Requires-backend`)

The target Account Hub broadens the current named API-key accounts into one
desktop place for Provider accounts, subscriptions, model access, and
installed-Agent bindings. Vendor-specific conversation adapters are intended
to connect each Agent's supported conversation behavior; the current API still
binds only the shipped `pi` and `dsh` identities and qualifies only Pi.

Credential import follows ADR-0055 exactly: the user initiates and consents to
one exact source before it is read; the daemon alone reads that source and
writes an approved SecretStore; source retention is the default and secure
deletion is an explicit per-import choice. Raw material never returns to the UI
or an Agent and never enters argv, environment, CognitiveOS ordinary config,
SQLite, logs, evidence, support output, or chat. Browser-profile, Agent
credential-file, subscription, and OAuth import mechanisms do not exist yet.
See [Provider and secrets](provider-and-secrets.md).

## Prerequisites

1. A running Personal daemon (`cognitive daemon start`, loopback
   `127.0.0.1:48181` by default).
2. A usable management session. The CLI obtains one from the local bootstrap
   secret; you do not paste a bearer on the command line.
3. An approved Secret Store: Linux Secret Service (`secret-tool`, session
   D-Bus) or, on Windows hosts, Windows Credential Manager. There is no
   plaintext fallback. macOS and a locked or missing keyring fail closed.
4. Never place a Provider key in process arguments, ordinary config, SQLite,
   environment variables, service units, logs, evidence, or chat.

Every verb below also accepts `--runtime-root <dir>` (hermetic-test layout
escape hatch) and `--endpoint <host:port>` (daemon address, **not** the
Provider URL). The Provider URL flag is `--endpoint-url`.

Exit codes: `0` success, `1` operational error, `2` usage error. Success output
is JSON. The CLI redacts `sk-`, `Bearer `, `bearer `, and `x-api-key` spans
before printing.

## Accounts

`--name` is the **display name** (ASCII letters, digits, hyphen, underscore;
max 64; unique). The durable account id is generated as `acct-<uuid>` and is
what later commands take as `--id`.

Provider kinds:

| `--provider-kind` | Endpoint | Auth on the wire |
|---|---|---|
| `openai_official` | always `https://api.openai.com/v1` | `Authorization: Bearer` |
| `anthropic_official` | always `https://api.anthropic.com` | `x-api-key` plus `anthropic-version` |
| `openai_compatible` | you must pass `--endpoint-url` | `Authorization: Bearer` only |

Official endpoints are immutable. Passing a different `--endpoint-url` for an
official kind fails with `PROVIDER_ENDPOINT_OFFICIAL_IMMUTABLE`. A custom
endpoint whose host is `api.anthropic.com` is refused
(`PROVIDER_ENDPOINT_ANTHROPIC_COMPATIBLE_FORBIDDEN`). Callers cannot inject
`headers` or `authorization` fields.

Create without a key leaves the account `revoked` (not callable). Create with
`--api-key-file` stores the key in the Secret Store, then runs one foreground
model discovery. Discovery failure leaves the account `degraded`, keeps any
catalog and binding, and records `last_discovery_error`.

```text
cognitive provider account create --name openai-work --provider-kind openai_official --api-key-file ./provider.key

cognitive provider account create --name lan-proxy --provider-kind openai_compatible --endpoint-url https://llm.internal.example/v1 --allow-private-network --api-key-file ./provider.key

cognitive provider account create --name xai-grok --provider-kind openai_compatible --endpoint-url https://api.x.ai/v1 --api-key-file ./provider.key

cognitive provider account list
cognitive provider account show --id acct-YOUR-ID
cognitive provider account update --id acct-YOUR-ID --endpoint-url http://127.0.0.1:8080/v1 --allow-insecure-http --reconfirm
cognitive provider account delete --id acct-YOUR-ID
```

`--api-key-file -` reads the key from stdin (no echo). Omitting the flag on
Unix uses hidden TTY input. On hosts that cannot disable echo, the CLI fails
closed and asks you to pass `--api-key-file`. `--api-key` is not accepted.

Delete fails with `PROVIDER_CONTROL_CONFLICT` while an **active** agent binding
points at the account. Remove the binding first.

List/show projections include `id`, `display_name`, `provider_kind`,
`endpoint`, opaque `secret_ref`, `status` (`active` / `degraded` / `revoked`),
`catalog_revision`, `last_discovery_error`, trust flags, and `network_scope`.
They never include the API key.

## Keys

```text
cognitive provider key set --id acct-YOUR-ID --api-key-file ./provider.key
cognitive provider key rotate --id acct-YOUR-ID --api-key-file ./provider.key
cognitive provider key remove --id acct-YOUR-ID
```

Set and rotate send the key once over loopback HTTP. The daemon puts it in the
Secret Store and persists only an opaque `secret_ref`. Rotate deletes the
previous Secret Store item after the new put. Remove deletes the store item
(best-effort) and marks the account `revoked`.

A missing or unusable Secret Store returns `PROVIDER_SECRET_STORE_UNAVAILABLE`
(HTTP 503). After remove, discovery and bound calls fail until you set a key
again.

## Endpoint trust

Custom OpenAI-compatible URLs may be public HTTPS without extra flags. You
must pass durable account-level grants for anything narrower or clearer:

- `--allow-private-network` — loopback, LAN, or other private ranges (and DNS
  results that resolve into those ranges).
- `--allow-insecure-http` — `http://` instead of HTTPS.

The daemon rejects embedded userinfo, fragments, query strings, redirects,
caller-supplied header templates, and implicit URL rewriting. DNS is checked
again at request time (`PROVIDER_ENDPOINT_DNS_REBINDING` if a name now points
somewhere more private than the grant).

Stored custom endpoints are OpenAI-compatible **API roots** only: empty/`/`,
`/v1`, `/api/v1`, `/openai/v1`, or `/compatible-mode/v1`. The control plane
accepts a pasted chat or models RPC URL (for example
`https://api.x.ai/v1/chat/completions`) and persists the root
(`https://api.x.ai/v1`). Other paths — including the local daemon proxy
`/provider/v1/...` — return HTTP 400 `PROVIDER_ENDPOINT_PATH_FORBIDDEN`.

`--reconfirm` is required on `account update` when authority, DNS/network
scope, or HTTPS→HTTP would change. Without it the daemon returns HTTP 409
`PROVIDER_ENDPOINT_RECONFIRM_REQUIRED`.

HTTP without `--allow-insecure-http` is
`PROVIDER_ENDPOINT_HTTP_REQUIRES_GRANT`. Private/loopback without
`--allow-private-network` is `PROVIDER_ENDPOINT_PRIVATE_REQUIRES_GRANT`.

## Models

Account create (when a key is supplied) and `key set` / `key rotate` each run
**one** foreground discovery. There is no background refresh.

- Official OpenAI and OpenAI-compatible: `GET` `{endpoint}/models`.
- Official Anthropic: `GET` `{endpoint}/v1/models`.

```text
cognitive provider models refresh --account-id acct-YOUR-ID
cognitive provider models list --account-id acct-YOUR-ID
cognitive provider models add --account-id acct-YOUR-ID --model-id my-local-model --price-input-per-million 1.00 --price-output-per-million 2.00
cognitive provider models set-price --account-id acct-YOUR-ID --model-id my-local-model --pricing-version manual --price-input-per-million 1.00 --price-output-per-million 2.00
```

`refresh` and `list` take `--account-id`. Failed refresh is audited, returns
`PROVIDER_DISCOVERY_FAILED` or `PROVIDER_DISCOVERY_MALFORMED` (HTTP 502), sets
status `degraded`, and **preserves** the last catalog and any binding.

Catalog `source` is `provider_discovered` or `manually_configured`. `add`
inserts a manual model (required before you can bind a model the provider did
not list). `set-price` updates prices; if you omit `--pricing-version` the CLI
sends `manual`. Price flags are decimal USD per million tokens:
`--price-input-per-million`, `--price-output-per-million`,
`--price-cache-read-per-million`, `--price-cache-write-per-million`.

A few official model ids carry a built-in versioned price table. Custom and
manual models have no price until you set one. Missing price is
`cost_unavailable` — not a zero cost.

## Agent bindings

Each of the two shipped agents has at most one active binding: a fixed account
+ provider + model. Requests cannot pick another model. There is no fallback
and no cross-agent sharing of Pi evidence.

CLI `--agent` values: `pi` or `dsh` (the daemon stores
`agent://personal/pi` and `agent://personal/dsh`).

```text
cognitive agent binding set --agent pi --account-id acct-YOUR-ID --model-id gpt-4o
cognitive agent binding set --agent dsh --account-id acct-YOUR-ID --model-id deepseek-chat
cognitive agent binding list
cognitive agent binding show --agent pi
cognitive agent binding remove --agent pi
```

`set` fails with `PROVIDER_MODEL_NOT_FOUND` unless that model is already in
the account catalog (discover it or `models add` it). Catalog membership is
not enough: a DeepSeek host only serves `deepseek-*`, and `grok-*` is only
servable on a non-DeepSeek `openai_compatible` account (for example xAI).
`models add` and `binding set` fail closed with
`PROVIDER_MODEL_ENDPOINT_MISMATCH` otherwise. HTTP `POST
/management/agent-bindings` accepts optional integer `expected_revision`
(current binding revision, or `0` when unbound). A mismatch is HTTP 409
`PROVIDER_BINDING_REVISION_STALE`. Changing account or model without
`expected_revision` is HTTP 409 `PROVIDER_SILENT_REBIND_REJECTED` — remove
the binding first, or send a matching `expected_revision`. The CLI `binding
set` omits that field, so a switch is `remove` then `set`. Same
account+model refresh still succeeds. `show` requires `--agent` at parse time but currently calls the same list
endpoint as `list` (it does not filter). Use `list` to inspect both bindings.

Pi traffic uses `POST /provider/v1/chat/completions`. DeepSeek harness traffic
uses the independent `POST /provider/v1/dsh/chat/completions` route. A bound
Pi private-candidate call also uses the binding rather than `provider.json`.
If a **Pi** request `model` does not match the Pi binding, the proxy fails closed with
HTTP 400 `PERSONAL_PROVIDER_BINDING_MISMATCH`. The **dsh** Path B proxy rewrites
the request model to the Cos `agent://personal/dsh` binding so native catalog
ids still chat with the assigned model on the **bound account**. If that
account cannot serve the bound model (grok on `api.deepseek.com`), Path B
fail-closes with HTTP 400 `PERSONAL_PROVIDER_BINDING_MISMATCH` and does not
POST to DeepSeek. Setting, removing, or catalog-changing the dsh binding writes
the native Models overlay from the current dsh-bound account only. Personal
`/ui/` Bindings **Apply to running dsh** (`POST /personal/dsh/runtime`
`op=apply`) republishes selected-model and reloads Cos-installed web so that
list matches; unbinding dsh drops grok (and every other id from that account)
from native Models. A revoked account or missing key is HTTP 409
`PERSONAL_PROVIDER_ACCOUNT_UNAVAILABLE`. Official Anthropic
bindings do not support public SSE (`stream:true`).

Pi and `dsh` bindings are isolated: setting one never copies the other.

## Usage, cost, and audit

Bound proxy calls persist a usage event (no prompt, completion, key, or
reversible payload). Token classes on the ledger are input, output,
cache-read, and cache-write. A missing field stays **unknown**; it is never
stored as `0`. `metering_source` on the ledger is `provider_reported` when
both input and output token counts are present, otherwise `unavailable`.

Cost is `priced` only when every token class that is present has a price;
otherwise `cost_status` is `cost_unavailable` and `cost_micros` is omitted —
that is not a zero bill. Cache hit rate is derived only when the provider
denominator is known; otherwise the ledger keeps raw cache counters and an
unknown rate.

```text
cognitive usage query
cognitive audit query
```

Those two CLI verbs take **no filters** in this phase (the design text that
mentions time-range/account filters is not a shipped flag). `usage query`
dumps `GET /management/usage`: each event has `event_id`, `account_id`,
`cost` (`unknown` or a non-zero micro-USD number), `cost_label`
(`actual` | `estimated` | `unknown`), `cost_micros`, `cost_status`, and
`metering_source`. Unknown cost is never JSON `0` or `"0"`.
`locally_estimated` is mapped to `estimated` only when that metering_source
was recorded. The same body includes `binding_explanation.layers` in order
global → Project → employee → Task (missing layers are `unbound`, not
invented zeros) and `accounts[]` with separate `account` vs `quota` objects
(`quota.status` is `unknown` until a real quota source exists). Secret
handles are omitted. `audit query` returns `audit_id`, `action`, `outcome`, and
a redacted `detail`. Per-call usage events are retained **30 days**;
aggregates **90 days**. Querying usage runs that cleanup.

## Budgets and alerts

Budgets are calendar-month, observe-only. They never block, throttle, or
reroute a Provider call. Scope is `account` (account id) or `agent` (use the
stored agent id, for example `agent://personal/pi`, so it matches usage rows).

```text
cognitive budget set --scope-kind account --scope-id acct-YOUR-ID --token-limit 2000000 --amount-micros-limit 10000000
cognitive budget list
cognitive budget remove --budget-id bud-YOUR-ID
cognitive alerts list
cognitive alerts acknowledge --alert-id YOUR-ALERT-ID
```

`--amount-micros-limit` is integer **micro-USD** (1 USD = 1_000_000).
`--budget-id` is optional on set (the daemon mints `bud-<timestamp>` when
omitted). Remove takes `--budget-id`, not `--id`.

Each period emits at most one `warning_80` at 80% of the token or amount
limit and one `exceeded_100` at 100%, deduplicated per budget. Unavailable
cost is not treated as zero spend. `alerts list` may mint newly crossed
thresholds as it reads. Acknowledge takes `--alert-id`.

## Safety

- Trust flags and DNS pinning exist to contain SSRF. Do not grant
  `--allow-private-network` or `--allow-insecure-http` unless you intend that
  account to reach that network and scheme.
- Prompts, completions, keys, and request headers are not retained in usage,
  audit, or CLI output.
- Management mutations require the management channel. Unauthenticated calls
  fail (typically HTTP 401 on the daemon front door).
- Retention: 30-day events / 90-day aggregates, as shipped.

## Common failures

| What you see | What to do |
|---|---|
| CLI `--api-key is not accepted` | Use `--api-key-file` or `--api-key-file -`. |
| HTTP 401 on a `cognitive provider` verb | Daemon not running, no management session, or bootstrap secret missing. Start the daemon; do not put keys on argv. |
| HTTP 403 `PROVIDER_CONTROL_CHANNEL_FORBIDDEN` | These operations are management-only. Use the `cognitive` product CLI, not a task bearer. |
| `PROVIDER_SECRET_STORE_UNAVAILABLE` | Unlock or install the OS Secret Store. There is no file/env fallback. |
| `PROVIDER_DISCOVERY_FAILED` / detail `upstream 401` or `upstream 403` | Key or account entitlement is wrong. Rotate the key; do not paste it into logs. The previous catalog remains. |
| `PROVIDER_DISCOVERY_FAILED` (transport) or `PROVIDER_DISCOVERY_MALFORMED` | Network, TLS, or unexpected `/models` JSON. Account stays `degraded`; bindings stay. |
| `PROVIDER_KEY_MISSING` | Set a key before refresh or bound calls. |
| `PROVIDER_MODEL_NOT_FOUND` | `models refresh` or `models add` before `agent binding set`. |
| `PROVIDER_MODEL_ENDPOINT_MISMATCH` | Bind grok only on a grok-capable non-DeepSeek `openai_compatible` account. Do not add grok to a DeepSeek catalog. |
| `PERSONAL_PROVIDER_BINDING_MISMATCH` | The request model is not the bound model. Change the binding or send the bound id. No fallback. |
| HTTP 409 `PROVIDER_BINDING_REVISION_STALE` | Re-read the binding revision and retry the confirmed `expected_revision`. |
| `PROVIDER_CONTROL_CONFLICT` on delete | `agent binding remove` first. |
| `PROVIDER_ENDPOINT_RECONFIRM_REQUIRED` | Re-run `account update` with `--reconfirm` if you really want the new host, scheme, or scope. |
| `PROVIDER_ENDPOINT_HTTP_REQUIRES_GRANT` / `PROVIDER_ENDPOINT_PRIVATE_REQUIRES_GRANT` | Pass the matching `--allow-*` flag at create or update (with `--reconfirm` when required). |
| `cost_status: cost_unavailable` | Set prices on custom/manual models. Do not treat missing cost as `$0`. |
| Official Anthropic + `stream:true` | Not supported on the bound path. Pi stays unary regardless. |
| dsh panel reports "API key invalid" after a daemon restart while the account/binding shows persisted `active` state | The new daemon reports dsh `INACTIVE`, so `cognitive dsh apply` is rejected and cannot recover the stale session. Do not extract or probe the bearer. Restart `cognitive dsh web`, then check `cognitive dsh status`. `apply` is only for supported binding/model overlay synchronization when the runtime is already `ACTIVE` and the daemon has not restarted. Persisted account `active` does not prove current SecretStore resolution; live resolution occurs during discovery/proxy use, so a locked or changed store remains a separate possible cause. See the [tracked defect](../../../../docs/bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md). |

## Worked sequence (official OpenAI, then bind Pi)

```text
cognitive daemon start
cognitive provider account create --name openai-work --provider-kind openai_official --api-key-file ./provider.key
cognitive provider account list
cognitive provider models list --account-id acct-YOUR-ID
cognitive agent binding set --agent pi --account-id acct-YOUR-ID --model-id gpt-4o
cognitive agent binding list
cognitive usage query
cognitive budget set --scope-kind account --scope-id acct-YOUR-ID --token-limit 2000000
cognitive alerts list
cognitive audit query
```

Replace `acct-YOUR-ID` with the `id` from create/list. Keep `./provider.key`
mode `0600` and out of Git. This sequence does not prove the upstream key is
valid until discovery or a real call succeeds, and it does not complete a
Task.

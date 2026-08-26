# ADR-0055: Personal Credential-Import Boundary and A5 Revision

- Status: Accepted (owner-directed, 2026-08-26)
- Date: 2026-08-26
- Decision owner: repository owner (CognitiveOS Personal product owner)
- Change class: **normative-semantic** (revises the scope of axiom A5's
  approved input paths; no public machine contract, schema, transition,
  registered error, or negative vector changes)
- Amends: [AXIOMS.md](../governance/AXIOMS.md) §A5 in the same delivery, per
  the ADR-0041 axiom-revision process
- Executed under: `lease/personal/GOV-A5/credential-import-boundary`
- Related: ADR-0016 (external agents are candidate-only), ADR-0018 (Secret
  Store boundary), ADR-0020 (Provider config/secret binding), ADR-0041
  (axiom system)

## Context

On 2026-08-26 the owner-directed Control Plane UX design session established
the product direction that CognitiveOS Personal becomes the desktop account
hub for installed Agents: importing and managing subscriptions (OAuth) and
API keys in the spirit of cc-switch/cockpit-style account switchers, and
routing every installed Agent through daemon-mediated Provider access.

Maximizing that experience requires importing credential material the user
already has on disk: browser cookie/profile stores, third-party Agent CLI
credential files, and existing plain configuration the user points at. Axiom
A5 as written ("Provider and user secrets enter only approved Secret Stores
and approved non-logging input paths") did not define reading such locations,
so the strictest honest reading of the axiom blocked the design session. The
owner paused the UX session and directed this axiom revision as a separate
governance delivery, to be completed before the design work continues.

The owner explicitly accepts, as a product decision, the supplier
terms-of-service and account-policy risk of importing and using third-party
subscription tokens the user already possesses. That decision changes what
the product may build; it does not change the isolation mechanics that keep
secret material out of logs, evidence, and client-visible surfaces.

## Decision

1. **A5 is revised in the same delivery.** The approved non-logging input
   paths now include the *user-directed credential-import boundary* defined
   by this ADR. The revised axiom text lives in
   [AXIOMS.md](../governance/AXIOMS.md) §A5 and is the canonical wording;
   this section defines the boundary semantics.
2. **Boundary definition.** A credential import under this boundary is:
   - *User-initiated and per-source consented.* Every import names the exact
     source and the target Secret Store and shows what will be read before
     reading it. No background, speculative, or bulk scanning of credential
     locations is permitted.
   - *Daemon-owned and audited.* Only the Rust daemon performs the read and
     the Secret Store write; the material exists only in process memory
     between the two. Evidence records only redacted metadata — source kind,
     target store, timestamp, outcome — never the material itself or any
     brute-forceable representation of it.
   - *Non-logging end to end.* No secret material may appear in argv,
     environment variables, ordinary configuration written by CognitiveOS,
     SQLite, logs, CI output, test output, evidence, or chat, and an import
     must never leave a new plaintext copy outside the read source and the
     target Secret Store.
   - *Explicit source disposition.* Retention of the source is the default;
     secure deletion of the source is a per-import user choice.
3. **Runtime usage is unchanged.** Agents, sidecars, UI clients, and other
   clients never receive raw secret material. Daemon-mediated Provider
   proxying and opaque `SecretRef` handles remain the only consumption paths
   (ADR-0018, ADR-0020 stay in force and are extended only by this boundary).
4. **No implementation is authorized.** Any concrete import mechanism —
   browser-cookie/profile decryption, third-party CLI credential-file
   parsing, subscription/OAuth token capture — requires its own future
   formal task with focused negatives. Until such tasks exist, every product
   and design surface marks the capability `Requires-backend`.
5. **Wording alignment.** Relative to the previous axiom text, the revised
   A5 judgment list adds "environment variables" and "chat"; this aligns the
   axiom with the already-stricter operational wording carried by the Cursor
   rules and changes no operational boundary.
6. **Delivery mechanics.** This delivery executes under a new owner-directed
   governance lease class, `lease/personal/GOV-<id>/<purpose>`, registered
   in `tools/src/check-consistency.mjs` with focused positive and negative
   fixtures. The class exists because no previously accepted lease grammar
   could own an axiom revision: evaluation (`EVAL-*`) leases are restricted
   to `docs/evaluation/`, `docs/checkpoints/`, and `docs/plan/PROGRESS.md`,
   and formal-task leases require a registered plan slice. A governance
   lease must name the same snapshot-registered `GOV-<id>` in its
   description and may own only `docs/governance/`, `docs/adr/`,
   `docs/plan/PROGRESS.md`, the lease-grammar checker surface
   (`tools/src/check-consistency.mjs`, `tools/test/check.test.mjs`), and
   mapped handbook pages under `personal/handbook/`.

## Alternatives considered

- **Public-OAuth-only import** (first-party OAuth flows only, never reading
  existing on-disk material): rejected by the owner — insufficient coverage
  for the account-hub experience, since installed third-party Agent CLIs and
  browser sessions hold the subscriptions users actually want to manage.
- **Keep A5 unchanged and grant time-bounded per-feature exceptions**:
  rejected — a standing product capability would churn through expiring
  exception ADRs, and an exception list is a weaker statement than one
  explicit boundary with auditable properties.
- **Unbounded scraping of cookie stores and configuration**: rejected —
  unauditable, incompatible with user consent, and incompatible with A5's
  core purpose.

## Consequences

- The Control Plane UX/product design session may resume and design
  subscription import as a target capability, marked `Requires-backend`
  until formal implementation tasks exist.
- A6 is untouched: no contract, negative, or transition is relaxed. Future
  import implementations must add focused negatives proving: consent is
  required per source, no material reaches logs/evidence/SQLite/ordinary
  configuration, source deletion is honored when chosen, and Secret Stores
  stay isolated from each other.
- A1 and A2 are untouched: import is a daemon operation, no Agent or Adapter
  may perform it, and an imported subscription grants no authority by
  itself.
- The `GOV-*` owner-directed governance lease class now exists in the
  consistency checker with focused fixtures; future owner-directed
  governance deliveries can claim it instead of stretching the evaluation
  lease class or inventing formal plan tasks.
- Claim ceiling `hypothesis`: this ADR creates no Gate, release, Profile,
  benchmark, or Agent-benefit claim.

## Non-goals and non-claims

This ADR authorizes no implementation, no import mechanism, no new Secret
Store backend, and no Provider behavior change. It creates no Gate, release,
Profile, benchmark, or Agent-benefit evidence. Recording the owner's
acceptance of supplier terms-of-service and account-policy risk is a product
decision ledger entry, not a legal assessment. No secret material was read,
copied, or stored by this delivery.

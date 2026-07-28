# ADR-0033: Personal Linux User-Systemd Controller Compensation

- Status: Accepted for the P1-T08 controller-fixture slice
- Date: 2026-07-28
- Decision owners: CognitiveOS repository maintainers
- Classification: Personal distribution implementation decision; it changes no
  registry requirement, schema, transition, conformance vector, Profile, Gate,
  or release claim.
- Related: ADR-0030 service health transaction, ADR-0031 safe extraction,
  ADR-0032 rendered user-service promotion, P1-T08.

## Context

ADR-0032 fixes product-owned candidate and canonical active unit identities and
their intended promotion order, but does not uniquely determine the failure
semantics for publishing units, daemon reload, pointer activation, and old
active recovery. The existing controller is intentionally fail-closed before
any real systemd action and needs a private fake-systemctl fixture boundary
before a Linux-native campaign can be considered.

## Decision

1. The controller owns a product-fixed private or injected unit root, exactly
   two unit identities, a constrained release version, and a product deployment
   root. It accepts no manifest, archive, keyring, health URL, command, unit
   name, port, environment, or arbitrary argument.
2. Every manager invocation has fixed leading arguments `--user`,
   `--no-ask-password`, and `--no-pager`. The only permitted actions are
   `daemon-reload`, `start cognitiveos-personal-candidate.service`,
   `stop cognitiveos-personal-candidate.service`, and
   `restart cognitiveos-personal.service`. A fake executable is injectable
   only through the test fixture boundary.
3. The forward sequence is: publish candidate unit; daemon-reload; start
   candidate; bounded candidate health on `127.0.0.1:48182`; stop candidate;
   publish canonical active unit; daemon-reload; activate pointer; restart
   canonical active unit; confirm pointer, rendered canonical unit, active
   manager action, and bounded health on `127.0.0.1:48181`; issue receipt.
4. Before pointer activation, any failure leaves a healthy old active pointer
   and canonical service untouched. A started candidate is stopped first. Unit
   restoration, where needed, is followed by daemon-reload. Failure of any
   required compensation action returns `RollbackIncomplete` and never a
   receipt.
5. After pointer activation, compensation stops the candidate/new active when
   applicable, restores the previous complete pointer and canonical unit,
   daemon-reloads, restarts the old canonical active service, and confirms
   liveness. First install removes the pointer rather than inventing a prior
   version. Same-version success requires pointer/unit/service consistency;
   pointer equality alone is insufficient.
6. Unit publication rejects unsafe parent chains, symlinks, non-regular target
   files, directory replacement, and unsafe permissions. It uses a private
   synchronized temporary file and atomic rename. Commands are deadline-bound,
   killed and reaped on timeout, fully drained, and fail closed when either
   stdout or stderr exceeds its cap. Outputs, health bodies, bundle material,
   key material, tokens, and user data are never returned in errors or receipts.

## Consequences and non-claims

This decision authorizes a deterministic fake-systemctl harness and focused
implementation-fixture tests only. It does not run native systemd, select a
production user-systemd root, create release material, perform uninstall,
establish B01/Gate/Profile/containment evidence, or change P1-T08's
`in-progress` / `experimental-local-only` status. P1-T09 remains `not-started`.

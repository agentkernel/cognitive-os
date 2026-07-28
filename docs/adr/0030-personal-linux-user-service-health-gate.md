# ADR-0030: Personal Linux User-Service Health Transaction

- Status: Accepted for the P1-T08 service-lifecycle slice
- Date: 2026-07-28
- Decision owners: CognitiveOS repository maintainers
- Classification: Personal distribution implementation decision; it changes no
  registry requirement, schema, transition, conformance vector, Profile,
  Gate, or release claim.
- Related: ADR-0022 local daemon, ADR-0023 readiness projections, ADR-0025
  distribution scope, ADR-0028 offline bundle attestation, ADR-0029 bootstrap
  download trust, P1-T08.

## Context

The P1-T08 bootstrap and verifier stop after offline bundle verification. The
existing installer stages an opaque verified artifact behind an OS-backed
per-deployment-root lifecycle lease, but has no service controller, bounded
daemon liveness probe, or compensation after an active-pointer change.

## Decision

1. A separate `linux_bundle_service` entry point owns the service-aware
   transaction. The generic offline `install_linux_bundle` callback remains
   unchanged and cannot acquire systemd behavior.
2. Complete `verify_linux_bundle` runs before creation of a lease, deployment
   root, unit, service, or any installer state. The existing stable OS file
   lock remains held from deployment open through stage, candidate start,
   candidate liveness, pointer activation, final confirmation, and all
   compensation. No process mutex, TTL, owner metadata, PID policy, stale-file
   removal, or lock stealing is introduced.
3. The service-controller trust boundary receives only a checked target
   version and candidate directory. It cannot select a keyring, manifest,
   executable, unit name, health URL, port, bundle metadata, or command line.
   Receipts and errors are typed and omit secrets, tokens, artifact/bundle
   bytes, key material, service output, and user data.
4. The source-controlled systemd unit is user-service-only. Its checked-in
   template contains unresolved release placeholders and must be rejected
   before installation or start. The controller uses only fixed literal
   `systemctl --user --no-ask-password --no-pager` argument vectors; it does
   not use system scope, sudo, shells, eval, root paths, global enable, or
   metadata-derived command arguments.
5. Candidate liveness uses only unauthenticated loopback `GET
   /personal/health`. A probe requires exact HTTP 200, a bounded
   Content-Length response, no transfer encoding or redirect, a strict closed
   JSON body with `schema_version=1`, `surface="personal-health"`,
   `status="ok"`, `authority_side_effects=false`, and explicit non-claims.
   Connect/read/overall deadlines, maximum response bytes, and a finite retry
   count are mandatory. TCP reachability, `/personal/status`,
   `/personal/readiness`, and `/personal/doctor` are not activation predicates.
6. The fixed transaction is offline verify -> lease -> stage -> candidate
   start -> bounded candidate liveness -> atomic pointer activation -> pointer
   and active-service confirmation -> non-secret receipt. On any candidate
   start, health, activation, pointer, or final-service failure: stop the
   candidate, restore the prior complete pointer and restart/confirm the prior
   service; on first install remove the pointer rather than inventing a prior
   target. If any compensation action fails, return `rollback incomplete` and
   never a success receipt. Staging and user data remain inspectable.

## Current limitation and evidence boundary

The verified artifact is still an opaque archive. No safe archive extraction,
runnable `bin/kernel-server` layout, production unit rendering, or actual
systemd invocation is claimed. The production controller therefore rejects a
missing safe extracted executable before running systemctl. Fake controller,
loopback fixture, WSL, and CI results are implementation/test evidence only;
they are not Linux-native systemd evidence.

## Non-claims

This ADR does not provide a production signing key, release bundle, release
attestation, SBOM/provenance, GitHub Release, Linux-native campaign,
uninstall, B01, Gate, Profile, containment, RC, release claim, product
readiness, Provider/SecretStore/Pi/first-conversation proof, or archive-safe
extraction. P1-T08 remains `in-progress` on the
`experimental-local-only` development track.

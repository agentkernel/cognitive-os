# ADR-0032: Personal Linux Rendered User-Service Promotion

- Status: Accepted for the P1-T08 rendered user-service slice
- Date: 2026-07-28
- Decision owners: CognitiveOS repository maintainers
- Classification: Personal distribution implementation decision. This ADR does
  not add or change a registry requirement, schema, transition, conformance
  vector, Profile, Gate, or release claim.
- Related: ADR-0028 offline bundle attestation, ADR-0030 user-service health
  transaction, ADR-0031 safe extraction, P1-T08.

## Context

ADR-0031 makes the fixed `bin/kernel-server` layout safe to publish at
`staged/<version>`. ADR-0030 intentionally left `start_candidate` fail-closed:
a canonical active unit cannot safely launch a staged executable, and sharing
its liveness address would create an active/candidate ambiguity and port
conflict. The earlier ADR does not uniquely specify candidate identity,
endpoint, runtime root, or promotion order.

## Decision

1. The product owns exactly two user-unit identities:
   `cognitiveos-personal-candidate.service` and
   `cognitiveos-personal.service`. They are neither template instances nor
   metadata-selected names.
2. The candidate unit runs only
   `staged/<version>/bin/kernel-server --personal --bind 127.0.0.1:48182`
   with runtime root `<deployment-root>/runtime/candidate`. The canonical
   active unit runs only
   `versions/<version>/bin/kernel-server --personal --bind 127.0.0.1:48181`
   with runtime root `<deployment-root>/runtime/active`. Both addresses are
   fixed loopback-only product inputs. `/personal/health` remains liveness,
   not readiness.
3. Rendered units use only constrained version text and product-owned paths.
   Manifest, archive, keyring, health URL, bundle metadata, environment, and
   arbitrary arguments cannot select an executable, command, unit name, port,
   runtime root, or trust boundary.
4. The implementation transaction is fixed: complete offline verification,
   per-root OS lease, extracted staging, candidate unit render/install,
   daemon-reload, candidate start, bounded candidate health, candidate stop,
   canonical unit render/install, pointer activation, daemon-reload/restart,
   pointer/service/health confirmation, non-secret receipt.
5. Unit installation must use a product-fixed user-systemd directory (or a
   private injected test directory), reject unsafe file types/paths/modes, and
   use a private temporary file followed by atomic publication. It must not
   follow symlinks or rely on host umask. This ADR fixes the intended boundary;
   its fixture implementation is not a Linux-native systemd campaign.
6. On failure, candidate is stopped if it started. If a previous active version
   exists, the previous complete pointer and canonical service are restored and
   confirmed; first install clears rather than invents a pointer. Any failed
   compensation returns `rollback incomplete`, never a receipt. User data is
   never deleted.

## Rejected alternatives

### One canonical unit for candidate and active

Rejected because it cannot execute `staged/<version>` before pointer activation
without either changing the active identity early or sharing the active port.

### Dynamic template unit names or bundle-provided systemd fields

Rejected because versioned names and free-form unit text broaden the command
and service-manager authority boundary beyond product-owned inputs.

### Candidate health as readiness or release acceptance

Rejected because `/personal/health` only proves bounded daemon liveness. It
does not prove SecretStore, Provider, Pi, user configuration, B01, Gate,
Profile, containment, release, or Linux-native systemd behavior.

## Consequences and non-claims

This ADR supplies an implementation-local promotion model and local fixture
test target only. It does not create production signing material, release
archives, a GitHub Release, uninstall/user-data retention workflow, Linux
native campaign, B01, Gate, Profile, containment, RC, or release evidence.

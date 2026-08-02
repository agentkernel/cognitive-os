# ADR-0036: Personal Linux 1.0 and Official Pi Acquisition

- Status: Accepted
- Date: 2026-08-02
- Decision owners: CognitiveOS Personal product owner
- Classification: product-semantic release, platform and acquisition decision
- Related: ADR-0025, ADR-0027, ADR-0034, ADR-0035, P5-T01, P5-T02,
  P5-T05, P7-T01, P7-T02, P7-T03, P7-T08

## Context

ADR-0034 created `GMVP-LINUX` as the first public Linux product Gate but did
not assign it a product version or include a managed Agent lifecycle. ADR-0025
also required users to install Pi locally. The owner has now selected a clearer
Personal 1.0 product boundary:

- Linux x86_64 is the only 1.0 product platform;
- Pi is the only Agent adapter qualified for 1.0;
- Personal obtains the exact official Pi package from npm and manages its
  lifecycle, while continuing not to vendor Pi or Node in the release bundle;
- the generic adapter framework is delivered, but every later adapter requires
  independent qualification and promotion evidence.

## Decision

### 1. Version and Gate identity

`GMVP-LINUX` is the release Gate for CognitiveOS Personal `1.0.0`. It remains
the existing product-only Gate; no parallel `G1.0` Gate is created. Passing it
permits only the exact Linux 1.0 scope in this ADR and the formal Personal plan.
It is not a CognitiveOS Core Profile claim.

### 2. Linux 1.0 included scope

The release candidate must provide and verify:

1. Linux x86_64 installation as one `cognitiveos-personal.service` user unit on
   `127.0.0.1:48181`;
2. native Secret Service, daemon-owned Provider proxy and exact model snapshot;
3. Pi-hosted Agent Shell with separate task and management sessions;
4. official Pi acquisition, durable installation/registry record, health,
   activation, supervision, suspension/resume, upgrade/rollback and uninstall;
5. one governed single-Agent Task loop with deadline, retry, step and cost
   bounds;
6. one catalog-bound safe operation using persist-before-dispatch Intent/Effect;
7. watch, detach/cancel, recovery, reconciliation, evidence and independent
   completion verification;
8. production signing, SBOM, attestation, transactional product update,
   rollback and uninstall;
9. backup/restore excluding secrets, and redacted doctor/support output.

At this ADR's original decision time, durable Memory and general Context work
were deferred. ADR-0037 partially supersedes that deferral for the Linux 1.0
minimum durable Memory and deterministic Context slices. Advanced Memory
retrieval and complex Context optimization, MCP, Multi-Agent, Web UI, Windows
installer/service and any non-Pi adapter remain deferred and excluded from the
1.0 claim.

### 3. Official Pi acquisition

The product acquisition source is the official npm package
`@earendil-works/pi-coding-agent` at an exact approved version. The initial pin
remains `0.81.1` until a product-semantic update explicitly changes it.

The acquisition transaction must:

1. fetch registry metadata and tarball from the fixed allowed npm origin;
2. require exact package name and version;
3. verify npm SRI and independently compute the package digest;
4. reject redirects/origins, package identity drift, integrity mismatch,
   unsafe paths, lifecycle scripts not explicitly admitted, and incomplete
   dependency locking;
5. stage into a private versioned Personal installation root;
6. verify the executable/API compatibility pin before activation;
7. write a production-signed acquisition lock binding package identity,
   version, source URL/origin policy, SRI, package/dependency digests, adapter
   digest, Node compatibility and installer version;
8. commit installation visibility only after all checks pass.

The CognitiveOS signature means “this exact upstream artifact was reviewed and
admitted by the Personal release process.” It does not claim that npm SRI is
publisher provenance or that CognitiveOS is the upstream publisher.

### 4. No vendoring and Node boundary

Pi and Node remain absent from the CognitiveOS release bundle. The installer
may acquire Pi after explicit user preview and network authorization. It must
fail closed when a supported Node runtime is absent or incompatible; it must
not download an unreviewed Node runtime as a side effect. Provider or npm
credentials, if any, remain in approved secret storage and never enter argv,
ordinary configuration, logs, evidence or acquisition locks.

### 5. Lifecycle and rollback

- Upgrade creates a new immutable Pi installation and adapter binding; it does
  not mutate the active version in place.
- Activation changes only after compatibility/health checks and an epoch-fenced
  daemon commit.
- Failure restores the prior installation and instance binding, or reports a
  durable incomplete rollback; it never emits a success receipt early.
- Uninstall stops new dispatch, suspends/fences instances, reconciles or
  quarantines pending Effects, removes package bytes, and retains required
  audit/evidence according to policy.
- Removing Pi does not remove CognitiveOS user data, Provider secrets or Task
  history unless a separate explicitly confirmed product-data operation says
  so.

### 6. Adapter framework and claim isolation

Personal 1.0 ships the generic package/acquisition/adapter/instance test
framework with Pi as its only product-qualified implementation. OpenClaw,
Hermes, Codex, WorkBuddy and other Agents enter later independent trains. Each
must pin package, adapter and protocol identity; execute its own compatibility,
sandbox, lifecycle, recovery and negative campaigns; and receive an explicit
release inclusion decision.

## Consequences

- ADR-0025 is partially superseded only where it requires the user to install
  Pi manually. Its no-vendoring, no-Node-bundle, license and distribution
  decisions remain in force.
- ADR-0034 is extended: the managed-Pi B09 result becomes a promotion
  dependency of `GMVP-LINUX`/1.0.
- Existing experimental Pi artifacts and signer keys cannot satisfy P7
  production trust or the 1.0 acquisition lock.
- A user-provided Pi path may remain an explicit development/import mode, but
  it is outside the default 1.0 release claim unless separately qualified.

## Rejected alternatives

1. **Bundle Pi or Node in the product archive.** Rejected because it expands
   redistribution, patch and trust obligations and contradicts the lean release
   topology.
2. **Trust npm SRI as publisher signature.** Rejected because integrity from a
   registry response is not independent publisher provenance.
3. **Qualify several Agents in 1.0.** Rejected to preserve a narrow executable
   MVP while still delivering the reusable adapter framework.
4. **Create a second Linux 1.0 Gate.** Rejected because `GMVP-LINUX` already
   owns the release convergence point.

## Non-claims

This ADR does not implement acquisition or lifecycle management, pass B01 or
B09, produce production signing material, release Personal 1.0, or establish
Profile conformance.

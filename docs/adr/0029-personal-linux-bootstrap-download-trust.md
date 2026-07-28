# ADR-0029: Personal Linux Bootstrap Download and Trust Handoff

- Status: Accepted for the P1-T08 bootstrap slice
- Date: 2026-07-28
- Decision owners: CognitiveOS repository maintainers
- Classification: Personal distribution implementation decision. This ADR does
  not add or change a registry requirement, schema, transition, conformance
  vector, Profile claim, Gate claim, or release claim.
- Related: ADR-0025 distribution, ADR-0028 offline bundle attestation,
  P1-T08 Linux installer, P7-T01 release material

## Context

ADR-0028 verifies a directory already available locally. A network bootstrap
must not execute a bundle-selected verifier, trust root, keyring, Pi pin, or
download URL to obtain that directory. No production signing key, release
bundle, release URL, or release digest exists yet.

## Decision

1. `deploy/linux/install.sh` is an inspected, version-specific release
   template. The supported user flow is `curl -o install.sh`, inspect with
   `less`, then `sh install.sh`; `curl | sh` is never recommended or executed.
2. The checked-in source is intentionally unrendered. It fails before calling
   curl if any policy placeholder remains. Release rendering alone binds the
   release version, HTTPS object directory, one redirect host, verifier digest,
   public keyring, and Pi version/integrity. Environment variables and bundle
   metadata cannot replace those values.
3. The shell uses `curl --disable` as its first curl option, HTTPS-only fixed
   arguments, bounded connect/total timeout, retries, per-object byte bounds,
   and `.partial` paths that are renamed only after a complete HTTP response.
   It follows no automatic redirect. One explicit redirect is accepted only if
   it is absolute HTTPS, has no user-info by construction of the host match,
   targets the rendered allowlisted host, and itself returns a direct 200.
4. The bootstrap verifier executable is downloaded to a private `mktemp -d`
   directory and authenticated against the script-bound SHA-256 digest before
   execution. This digest authenticates only the bootstrap executable; it is
   not represented as bundle signing or release attestation.
5. The executable accepts only the local bundle directory and script-bound
   public keyring/Pi pin, then delegates to ADR-0028 `verify_linux_bundle`.
   It has no network, service manager, secret, authority, staging, or
   activation behavior. The shell does not parse bundle JSON or implement
   Ed25519.
6. `umask 077`, invocation-owned marker checks, and EXIT/HUP/INT/TERM traps
   clean only the bootstrap temporary directory. Deployment roots, user data,
   stable lifecycle lease files, and inspectable installer staging are outside
   this cleanup boundary.

## Consequences and non-claims

The bootstrap ends at verified local handoff. It intentionally does not call
`install_linux_bundle`, because P1-T08 has not yet defined a bounded service
health callback. Systemd service start, health-gated activation, rollback,
uninstall, production keys, production release rendering, GitHub Release,
SBOM, provenance, Linux-native campaign, B01, Gate, Profile, containment, RC,
and release claims remain out of scope.

# ADR-0031: Personal Linux Bundle Safe Extraction

- Status: Accepted for the P1-T08 safe-extraction slice
- Date: 2026-07-28
- Decision owners: CognitiveOS repository maintainers
- Classification: Personal distribution implementation decision; it changes no
  registry requirement, schema, transition, conformance vector, Profile,
  Gate, or release claim.
- Related: ADR-0025 distribution scope, ADR-0028 offline bundle attestation,
  ADR-0030 service health transaction, P1-T08.

## Context

ADR-0028 authenticates an artifact as opaque bytes and requires a second
digest check immediately before staging. ADR-0030 deliberately rejects a
candidate until its fixed `bin/kernel-server` executable layout exists. A
verified digest alone does not make archive extraction safe: a hostile archive
can attempt path escape, links, special files, resource exhaustion, metadata
abuse, or an ambiguous executable layout.

## Decision

1. The P1-T08 artifact container is exactly one `tar.gz` stream. Extraction
   uses an in-process, locked Rust implementation; it never invokes host
   `tar`, `gzip`, a shell, PATH lookup, hook, or command embedded in the
   archive. The artifact is still authenticated as exact opaque bytes by
   ADR-0028; this decision does not add release metadata or attestation fields.
2. The only accepted extracted release layout is the direct archive-root path
   `bin/kernel-server`, with its required parent directory. There is no
   top-level release directory, free file discovery, PATH lookup, unit,
   configuration, secret, user-data, Node, Pi, provider payload, hook, or
   arbitrary command supplied by the archive.
3. The extractor accepts only regular-file and directory entries needed by the
   fixed layout. It rejects absolute paths, `.` or `..` components, backslash
   or platform-prefix equivalents, non-UTF-8 paths, duplicate canonical paths,
   file/directory collisions, symbolic links, hard links, device nodes, FIFOs,
   sockets, sparse files, ownership records, setuid/setgid/sticky permissions,
   unsupported archive entry types, multiple gzip members, and trailing bytes.
4. The bounded limits are: at most 512 MiB compressed artifact input, 1 GiB
   expanded regular-file bytes, 512 MiB for one regular file, 1,024 regular
   files, 128 directories, and 4,096 UTF-8 path bytes. Limits are checked while
   streaming and before writing each entry; declared archive sizes never waive
   the streamed-byte bounds.
5. `bin/kernel-server` must be a regular file with owner-execute in its archive
   mode. The extractor rejects privileged mode bits and, on Unix, explicitly
   sets the installed entry to mode `0755`; success never relies on archive
   ownership metadata or the host umask. Other fixed-layout file handling is
   intentionally absent from this slice.
6. Extraction occurs only after complete `verify_linux_bundle`, after the
   per-root OS lease is held, and after `LinuxBundleDeployment` is open. The
   artifact is re-hashed before a private staging subdirectory is created.
   A candidate becomes `staged/<version>` only after extraction and complete
   layout validation succeed. Failed extraction must not alter an active
   pointer, active/previous deployment, canonical active service, or produce a
   receipt. Partial private staging is removed when possible and is never made
   active.
7. A successful extracted candidate is a static prerequisite for the current
   controller's fixed layout preflight only. The controller continues to reject
   the unresolved checked-in unit and does not gain any candidate `systemctl`
   action in this decision.

## Non-claims

This ADR does not render or install a production systemd unit, run a real
service, publish an artifact, create a production key or attestation, add a
downloader release rendering, implement uninstall or upgrade, or establish
Linux-native systemd, B01, Gate, Profile, containment, RC, release, Provider,
Pi, SecretStore, authority, Effect, Task, or capability evidence. P1-T08
remains `in-progress` on `experimental-local-only`.

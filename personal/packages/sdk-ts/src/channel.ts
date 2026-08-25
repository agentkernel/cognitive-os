/**
 * Channel binding, channel-branded credentials, and the channel-partitioned
 * projection store (REQ-SHELL-CHANNEL-001, REQ-AKP-SHELL-001..003,
 * REQ-AKP-MGMT-001..003; vector `shell-channel-isolation-003.json`; rule
 * `.cursor/rules/11-typescript-clients.mdc`).
 *
 * Isolation is enforced twice: at the type level (the `channel` literal on
 * {@link ChannelCredential} makes cross-channel assignment a compile error)
 * and at runtime ({@link ChannelBindingViolation} fails closed before any
 * request leaves the client). Cache keyspaces are partitioned by channel and
 * credential ID; secrets never enter cache keys.
 *
 * The store holds authority projections verbatim (latest by authority
 * version). It never aggregates, recomputes, or promotes state: clients are
 * not an authority (REQ-SHELL-STATUS-001).
 */

/** The two isolated client channels. A client instance binds exactly one. */
export const CLIENT_CHANNELS = ["task", "management"] as const;
export type ClientChannel = (typeof CLIENT_CHANNELS)[number];

/**
 * A credential bound to one channel. `secret` is opaque session material for
 * the transport; it must never appear in cache keys, logs, or envelopes.
 */
export interface ChannelCredential<C extends ClientChannel> {
  readonly channel: C;
  /** Non-secret identifier used for cache partitioning and audit labels. */
  readonly credentialId: string;
  readonly principalRef: string;
  readonly secret: string;
}

export interface CredentialInit {
  readonly credentialId: string;
  readonly principalRef: string;
  readonly secret: string;
}

/** Construct a task-channel credential. */
export function taskCredential(init: CredentialInit): ChannelCredential<"task"> {
  return { channel: "task", ...init };
}

/** Construct a management-channel credential. */
export function managementCredential(init: CredentialInit): ChannelCredential<"management"> {
  return { channel: "management", ...init };
}

/**
 * Client-side fail-closed rejection for mixed channels; mirrors the
 * registered `SHELL_CHANNEL_BINDING_MISMATCH` denial the authority would
 * produce (the client refuses to even send such a request).
 */
export class ChannelBindingViolation extends Error {
  readonly code = "SHELL_CHANNEL_BINDING_MISMATCH";

  constructor(detail: string) {
    super(`channel binding mismatch: ${detail}`);
    this.name = "ChannelBindingViolation";
  }
}

/** Latest stored projection for one key. */
export interface ProjectionEntry<T = unknown> {
  readonly version: number;
  readonly view: T;
}

/**
 * Channel- and credential-partitioned read-only projection cache. Displayed
 * state comes exclusively from entries ingested here, which in turn come
 * only from authority projections (snapshot + watch deltas).
 */
export class ProjectionStore {
  private readonly prefix: string;
  private readonly entries = new Map<string, ProjectionEntry>();

  constructor(credential: ChannelCredential<ClientChannel>) {
    // Partition by channel + credential ID only; never by secret.
    this.prefix = `${credential.channel}\u0000${credential.credentialId}\u0000`;
  }

  /** Runtime guard shared by clients: fail closed on mixed channels. */
  static assertChannel<C extends ClientChannel>(
    credential: ChannelCredential<ClientChannel>,
    expected: C,
  ): asserts credential is ChannelCredential<C> {
    if (credential.channel !== expected) {
      throw new ChannelBindingViolation(
        `credential ${credential.credentialId} is bound to ${credential.channel}, not ${expected}`,
      );
    }
  }

  /**
   * Store an authority projection if it is at least as new as the stored
   * one (monotonic by the authority-declared version; a stale delivery of an
   * at-least-once stream never regresses the displayed state).
   */
  ingest<T>(key: string, version: number, view: T): void {
    const existing = this.entries.get(this.prefix + key);
    if (existing && existing.version > version) {
      return;
    }
    this.entries.set(this.prefix + key, { version, view: deepFreeze(structuredClone(view)) });
  }

  get<T = unknown>(key: string): ProjectionEntry<T> | undefined {
    return this.entries.get(this.prefix + key) as ProjectionEntry<T> | undefined;
  }

  /** Test hook: raw partitioned keys (to prove secrets never leak in). */
  debugKeys(): ReadonlyArray<string> {
    return [...this.entries.keys()];
  }
}

function deepFreeze<T>(value: T): T {
  if (value !== null && typeof value === "object") {
    for (const member of Object.values(value)) {
      deepFreeze(member);
    }
    Object.freeze(value);
  }
  return value;
}

/**
 * Stable, redaction-safe failure codes for the Pi Extension's daemon client.
 *
 * Every failure the Extension can hit is one of these codes. They are stable
 * strings so an operator, a doctor bundle and a test can all name the same
 * fault. `P1-T07` acceptance requires that an unavailable daemon fails
 * explicitly rather than degrading into a session that looks ready.
 *
 * No constructor here ever receives credential material: messages are built
 * from codes, paths and HTTP status numbers only. `daemon-client.test.ts`
 * asserts that neither the bootstrap secret nor a session token appears in any
 * thrown message.
 */

export type DaemonClientErrorCode =
  /** `HOME`/`USERPROFILE` is unset, so the XDG layout cannot be resolved. */
  | "PI_EXTENSION_HOME_MISSING"
  /** `XDG_RUNTIME_DIR` is unset or empty; the layout fails closed (ADR-0019). */
  | "PI_EXTENSION_RUNTIME_DIR_MISSING"
  /** No endpoint file; the daemon was never started via `cognitive daemon start`. */
  | "PI_EXTENSION_ENDPOINT_FILE_MISSING"
  /** Endpoint file present but not a readable v1 endpoint document. */
  | "PI_EXTENSION_ENDPOINT_FILE_CORRUPT"
  /** Bootstrap secret file absent or empty; the daemon is not serving. */
  | "PI_EXTENSION_BOOTSTRAP_SECRET_MISSING"
  /** Connect/read failed, or the request exceeded the client deadline. */
  | "PI_EXTENSION_DAEMON_UNREACHABLE"
  /** The daemon refused the session or the bearer (401/403). */
  | "PI_EXTENSION_DAEMON_AUTH_REFUSED"
  /** The daemon answered, but not with the shape this client requires. */
  | "PI_EXTENSION_DAEMON_PROTOCOL_ERROR";

/** A failure that must be surfaced to the operator, never swallowed. */
export class DaemonClientError extends Error {
  readonly code: DaemonClientErrorCode;
  /** Daemon-side error code when the daemon supplied one, else `undefined`. */
  readonly daemonErrorCode: string | undefined;
  /** Transport status when there was a response, else `undefined`. */
  readonly httpStatus: number | undefined;

  constructor(
    code: DaemonClientErrorCode,
    message: string,
    options: { readonly daemonErrorCode?: string; readonly httpStatus?: number } = {},
  ) {
    super(message);
    this.name = "DaemonClientError";
    this.code = code;
    this.daemonErrorCode = options.daemonErrorCode;
    this.httpStatus = options.httpStatus;
  }
}

/** True when the fault means "no daemon to talk to", as opposed to a refusal. */
export function isDaemonUnavailable(error: unknown): boolean {
  return (
    error instanceof DaemonClientError &&
    (error.code === "PI_EXTENSION_DAEMON_UNREACHABLE" ||
      error.code === "PI_EXTENSION_ENDPOINT_FILE_MISSING" ||
      error.code === "PI_EXTENSION_BOOTSTRAP_SECRET_MISSING" ||
      error.code === "PI_EXTENSION_RUNTIME_DIR_MISSING" ||
      error.code === "PI_EXTENSION_HOME_MISSING")
  );
}

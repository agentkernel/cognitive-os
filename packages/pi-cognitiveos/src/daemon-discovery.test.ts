/**
 * Daemon discovery tests.
 *
 * The Extension must locate the daemon exactly the way `cognitive` does, and it
 * must fail closed everywhere the Rust layout fails closed. Every case here is
 * driven with an injected environment and file reader, so the tests behave
 * identically on Linux and Windows CI.
 */

import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import {
  BOOTSTRAP_SECRET_FILE_NAME,
  ENDPOINT_FILE_NAME,
  PERSONAL_PRODUCT_DIR_NAME,
  isLoopbackEndpoint,
  readBootstrapSecret,
  readDaemonEndpoint,
  resolvePersonalDaemonPaths,
  type EnvironmentSlice,
  type FileReader,
} from "./daemon-discovery.js";
import { DaemonClientError, isDaemonUnavailable } from "./errors.js";
import { captureThrown } from "./test-support.js";

const BASE_ENV: EnvironmentSlice = {
  HOME: path.join(path.sep, "home", "owner"),
  XDG_RUNTIME_DIR: path.join(path.sep, "run", "user", "1000"),
};

function fileReaderFor(files: Readonly<Record<string, string>>): FileReader {
  return {
    readTextFile(filePath: string): string {
      const contents = files[filePath];
      if (contents === undefined) {
        throw new Error(`ENOENT: ${filePath}`);
      }
      return contents;
    },
  };
}

function endpointDocument(endpoint: string): string {
  return JSON.stringify({
    schema_version: 1,
    endpoint,
    surface: "personal-daemon-endpoint",
  });
}

test("paths follow the XDG layout with the cognitiveos product directory", () => {
  const paths = resolvePersonalDaemonPaths(BASE_ENV);
  assert.equal(paths.stateDir, path.join(BASE_ENV["HOME"] ?? "", ".local", "state", PERSONAL_PRODUCT_DIR_NAME));
  assert.equal(
    paths.runtimeDir,
    path.join(BASE_ENV["XDG_RUNTIME_DIR"] ?? "", PERSONAL_PRODUCT_DIR_NAME),
  );
  assert.equal(paths.endpointFile, path.join(paths.stateDir, ENDPOINT_FILE_NAME));
  assert.equal(paths.bootstrapSecretFile, path.join(paths.runtimeDir, BOOTSTRAP_SECRET_FILE_NAME));
});

test("XDG_STATE_HOME overrides the default state root", () => {
  const stateHome = path.join(path.sep, "custom", "state");
  const paths = resolvePersonalDaemonPaths({ ...BASE_ENV, XDG_STATE_HOME: stateHome });
  assert.equal(paths.stateDir, path.join(stateHome, PERSONAL_PRODUCT_DIR_NAME));
});

test("a missing XDG_RUNTIME_DIR fails closed instead of inventing a fallback", () => {
  for (const runtimeDir of [undefined, "", "   "]) {
    const environment: EnvironmentSlice =
      runtimeDir === undefined
        ? { HOME: BASE_ENV["HOME"] }
        : { HOME: BASE_ENV["HOME"], XDG_RUNTIME_DIR: runtimeDir };
    const error = captureThrown(() => resolvePersonalDaemonPaths(environment)) as DaemonClientError;
    assert.equal(error.code, "PI_EXTENSION_RUNTIME_DIR_MISSING");
    assert.ok(isDaemonUnavailable(error));
  }
});

test("a missing HOME fails closed", () => {
  const error = captureThrown(() => resolvePersonalDaemonPaths({ XDG_RUNTIME_DIR: BASE_ENV["XDG_RUNTIME_DIR"] })) as DaemonClientError;
  assert.equal(error.code, "PI_EXTENSION_HOME_MISSING");
});

test("USERPROFILE stands in for HOME", () => {
  const paths = resolvePersonalDaemonPaths({
    USERPROFILE: path.join(path.sep, "users", "owner"),
    XDG_RUNTIME_DIR: BASE_ENV["XDG_RUNTIME_DIR"],
  });
  assert.ok(paths.stateDir.includes("owner"));
});

test("a missing endpoint file reports that the daemon was never started", () => {
  const paths = resolvePersonalDaemonPaths(BASE_ENV);
  const error = captureThrown(() => readDaemonEndpoint(paths, fileReaderFor({}))) as DaemonClientError;
  assert.equal(error.code, "PI_EXTENSION_ENDPOINT_FILE_MISSING");
  assert.match(error.message, /cognitive daemon start/);
  assert.ok(isDaemonUnavailable(error));
});

test("a well-formed loopback endpoint document is accepted", () => {
  const paths = resolvePersonalDaemonPaths(BASE_ENV);
  const files = fileReaderFor({ [paths.endpointFile]: endpointDocument("127.0.0.1:7420") });
  assert.equal(readDaemonEndpoint(paths, files), "127.0.0.1:7420");
});

test("corrupt, mis-versioned, mis-surfaced and non-loopback endpoints are all refused", () => {
  const paths = resolvePersonalDaemonPaths(BASE_ENV);
  const rejected: readonly string[] = [
    "not json at all",
    "[]",
    JSON.stringify({ schema_version: 2, endpoint: "127.0.0.1:7420", surface: "personal-daemon-endpoint" }),
    JSON.stringify({ schema_version: 1, endpoint: "127.0.0.1:7420", surface: "something-else" }),
    JSON.stringify({ schema_version: 1, surface: "personal-daemon-endpoint" }),
    JSON.stringify({ schema_version: 1, endpoint: "   ", surface: "personal-daemon-endpoint" }),
    endpointDocument("10.0.0.5:7420"),
    endpointDocument("evil.example:7420"),
  ];
  for (const document of rejected) {
    const error = captureThrown(() => readDaemonEndpoint(paths, fileReaderFor({ [paths.endpointFile]: document }))) as DaemonClientError;
    assert.equal(error.code, "PI_EXTENSION_ENDPOINT_FILE_CORRUPT");
  }
});

test("loopback classification matches the daemon's own Host rule", () => {
  for (const endpoint of ["127.0.0.1:7420", "localhost:7420", "[::1]:7420", "localhost.:7420"]) {
    assert.ok(isLoopbackEndpoint(endpoint), endpoint);
  }
  for (const endpoint of ["10.0.0.5:7420", "evil.example:7420", "0.0.0.0:7420", "[fe80::1]:7420"]) {
    assert.ok(!isLoopbackEndpoint(endpoint), endpoint);
  }
});

test("a missing or empty bootstrap secret means the daemon is not running", () => {
  const paths = resolvePersonalDaemonPaths(BASE_ENV);
  for (const files of [fileReaderFor({}), fileReaderFor({ [paths.bootstrapSecretFile]: "  \n" })]) {
    const error = captureThrown(() => readBootstrapSecret(paths, files)) as DaemonClientError;
    assert.equal(error.code, "PI_EXTENSION_BOOTSTRAP_SECRET_MISSING");
    assert.ok(isDaemonUnavailable(error));
  }
});

test("the bootstrap secret is trimmed and never embedded in an error message", () => {
  const paths = resolvePersonalDaemonPaths(BASE_ENV);
  const secret = "boot-0123456789abcdef-fedcba9876543210";
  const files = fileReaderFor({ [paths.bootstrapSecretFile]: `${secret}\n` });
  assert.equal(readBootstrapSecret(paths, files), secret);

  const missingError = captureThrown(() => readBootstrapSecret(paths, fileReaderFor({}))) as DaemonClientError;
  assert.ok(!missingError.message.includes(secret));
});

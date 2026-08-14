/**
 * Source-level safety guards (P1-T07 acceptance: no API key, no env key, no
 * SQLite path).
 *
 * Behavioural tests prove what the Extension does; these prove what it cannot
 * do, by scanning its own runtime sources. A future edit that reaches for a
 * Provider key, a database file or a subprocess fails here rather than in
 * review.
 *
 * Comments are stripped before scanning, so documentation is free to name the
 * very things the code may not touch. The scan covers the runtime modules only:
 * tests legitimately read repository files to check the Pi pin.
 */

import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

interface RuntimeSource {
  readonly name: string;
  /** Source text with block comments and whole-line comments removed. */
  readonly code: string;
}

function sourceDirectory(): string {
  // Compiled tests live in dist/; the TypeScript sources are the subject.
  return path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "src");
}

/**
 * Remove `/* ... *\/` blocks and whole-line `//` comments. Trailing comments are
 * left in place deliberately: stripping them would require parsing string
 * literals, and every comment in these modules is on its own line.
 */
function stripComments(text: string): string {
  const withoutBlocks = text.replace(/\/\*[\s\S]*?\*\//g, "");
  return withoutBlocks
    .split("\n")
    .filter((line) => !line.trim().startsWith("//"))
    .join("\n");
}

function runtimeSources(): readonly RuntimeSource[] {
  const directory = sourceDirectory();
  return readdirSync(directory)
    .filter((name) => name.endsWith(".ts"))
    .filter((name) => !name.endsWith(".test.ts"))
    .filter((name) => name !== "test-support.ts")
    .sort()
    .map((name) => ({
      name,
      code: stripComments(readFileSync(path.join(directory, name), "utf8")),
    }));
}

test("the runtime sources are exactly the modules under guard", () => {
  assert.deepEqual(
    runtimeSources().map((source) => source.name),
    [
      "daemon-client.ts",
      "daemon-discovery.ts",
      "daemon-provider.ts",
      "errors.ts",
      "extension.ts",
      "index.ts",
      "pi-api.ts",
      "pi-route-observation.ts",
      "pin.ts",
      "status.ts",
      "tool-policy.ts",
    ],
  );
});

test("no runtime code names a Provider API key or a Provider credential artifact", () => {
  const forbidden = [
    "API_KEY",
    "apiKey",
    "api_key",
    "DEEPSEEK",
    "OPENAI",
    "ANTHROPIC",
    "auth.json",
    "provider.json",
    "SecretRef",
    "secret_ref",
    "secret-tool",
  ];
  for (const source of runtimeSources()) {
    // The structural Pi declaration records the pinned required property name;
    // it is erased at runtime and is not a credential-resolution implementation.
    if (source.name === "pi-api.ts" || source.name === "daemon-provider.ts") continue;
    for (const needle of forbidden) {
      assert.ok(
        !source.code.includes(needle),
        `${source.name} must not reference ${needle}: the Extension holds no Provider credential`,
      );
    }
  }
});

test("no runtime code touches a database, a subprocess or a shell", () => {
  const forbidden = [
    "sqlite",
    "SQLite",
    "child_process",
    "execSync",
    "spawnSync",
    "spawn(",
    ".db",
  ];
  for (const source of runtimeSources()) {
    for (const needle of forbidden) {
      assert.ok(
        !source.code.includes(needle),
        `${source.name} must not reference ${needle}: Pi is a non-authority client`,
      );
    }
  }
});

test("the ambient process environment is read in exactly one place, as an injected default", () => {
  const readers = runtimeSources().filter((source) => source.code.includes("process.env"));
  assert.deepEqual(
    readers.map((source) => source.name),
    ["daemon-client.ts"],
  );
  const client = readers[0]?.code ?? "";
  assert.ok(
    client.includes("options.environment ?? process.env"),
    "process.env may only be the default for the injected environment slice",
  );
  assert.equal((client.match(/process\.env/g) ?? []).length, 1);
});

test("filesystem reads are limited to the two published daemon files", () => {
  const readers = runtimeSources().filter((source) => source.code.includes("node:fs"));
  assert.deepEqual(
    readers.map((source) => source.name),
    ["daemon-discovery.ts"],
  );
  const discovery = readers[0]?.code ?? "";
  assert.ok(discovery.includes('ENDPOINT_FILE_NAME = "daemon-endpoint.json"'));
  assert.ok(discovery.includes('BOOTSTRAP_SECRET_FILE_NAME = "local-bootstrap.secret"'));
  assert.equal(
    (discovery.match(/readFileSync/g) ?? []).length,
    2,
    "readFileSync must appear once as an import and once behind the FileReader port",
  );
});

test("no runtime code writes to the filesystem at all", () => {
  const forbidden = ["writeFileSync", "writeFile", "appendFile", "mkdirSync", "unlinkSync", "rmSync"];
  for (const source of runtimeSources()) {
    for (const needle of forbidden) {
      assert.ok(!source.code.includes(needle), `${source.name} must not write to the filesystem`);
    }
  }
});

test("no runtime code opens a network listener or a non-loopback target", () => {
  const forbidden = ["createServer", "listen(", "https://api.", "0.0.0.0"];
  for (const source of runtimeSources()) {
    for (const needle of forbidden) {
      assert.ok(
        !source.code.includes(needle),
        `${source.name} must not reference ${needle}: the Extension is a loopback client only`,
      );
    }
  }
});

test("Pi is never imported, at type level or runtime; only its pin is recorded", () => {
  for (const source of runtimeSources()) {
    assert.ok(
      !/\bfrom\s+["']@earendil-works\//.test(source.code),
      `${source.name} must not import Pi (ADR-0025: Pi is not vendored, and it is not a dependency)`,
    );
    assert.ok(
      !/\brequire\(["']@earendil-works\//.test(source.code),
      `${source.name} must not require Pi at runtime`,
    );
  }
  // The package name appears exactly once, as pin data rather than an import.
  const mentions = runtimeSources().filter((source) =>
    source.code.includes("@earendil-works/pi-coding-agent"),
  );
  assert.deepEqual(
    mentions.map((source) => source.name),
    ["pin.ts"],
  );
});

/**
 * Campaign-only pure-Pi credential broker (execution plan §2.2 option 2).
 *
 * Not an authority writer. No Context, Tool-as-authority, Memory, Task, retry,
 * cache, or verification. Provider material is never placed in argv, env,
 * logs, or the Pi-facing token.
 */

export const PI_PLACEHOLDER_TOKEN = "campaign-broker-nonsecret-token";
export const LOOPBACK_HOST = "127.0.0.1";

const SECRET_SHAPED =
  /sk-[A-Za-z0-9]{10,}|BEGIN [A-Z ]+PRIVATE KEY|-----BEGIN/;
const SECRET_ENV_NAMES = /^(?:PROVIDER|OPENAI|DEEPSEEK|ANTHROPIC|API|LLM).*KEY$/i;

const FORBIDDEN_CAPABILITIES = [
  "context",
  "memory",
  "skill",
  "task",
  "retry",
  "cache",
  "verify",
  "verification",
  "accept",
  "acceptance",
];

export function isSecretShaped(value) {
  return typeof value === "string" && SECRET_SHAPED.test(value);
}

export function assertSecretFreeProcess({ env = process.env, argv = process.argv } = {}) {
  for (const [name, value] of Object.entries(env)) {
    if (value === undefined || value === "") {
      continue;
    }
    if (SECRET_ENV_NAMES.test(name) || isSecretShaped(String(value))) {
      throw new Error(`secret-shaped process input refused: env ${name}`);
    }
  }
  for (const argument of argv) {
    if (isSecretShaped(String(argument))) {
      throw new Error("secret-shaped process input refused: argv");
    }
  }
}

export function assertLoopbackBind(host) {
  if (host !== LOOPBACK_HOST) {
    throw new Error(`broker bind host must be ${LOOPBACK_HOST}`);
  }
}

export function createPurePiBroker({
  host = LOOPBACK_HOST,
  port,
  getSecret,
} = {}) {
  assertLoopbackBind(host);
  if (!Number.isInteger(port) || port <= 0 || port > 65535) {
    throw new Error("broker port must be a TCP port integer");
  }
  if (typeof getSecret !== "function") {
    throw new Error("broker requires getSecret()");
  }

  let retainedMaterial = false;
  const loadSecretOnce = () => {
    const material = getSecret();
    if (typeof material !== "string" || material.length === 0) {
      throw new Error("broker secret resolution failed");
    }
    retainedMaterial = false;
    return material;
  };

  const broker = {
    kind: "pure-pi-credential-broker",
    host,
    port,
    piPlaceholderToken: PI_PLACEHOLDER_TOKEN,
    bind({ env = process.env, argv = process.argv } = {}) {
      assertSecretFreeProcess({ env, argv });
      const material = loadSecretOnce();
      void material;
      return {
        bind: `${host}:${port}`,
        pi_token: PI_PLACEHOLDER_TOKEN,
        secret_material_written: false,
        retry: 0,
      };
    },
    hasRetainedSecretMaterial() {
      return retainedMaterial;
    },
  };

  for (const name of FORBIDDEN_CAPABILITIES) {
    if (name in broker) {
      throw new Error(`broker must not expose ${name}`);
    }
  }

  return Object.freeze(broker);
}

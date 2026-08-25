export type ChannelClass = "management" | "task";

const memory: {
  management?: string;
  task?: string;
  principal?: string;
} = {};

function assertMemoryOnlyStore(): void {
  if (typeof window === "undefined") {
    return;
  }
  const probes = ["localStorage", "sessionStorage"] as const;
  for (const name of probes) {
    try {
      const store = window[name];
      for (let i = 0; i < store.length; i += 1) {
        const key = store.key(i) ?? "";
        const value = store.getItem(key) ?? "";
        if (/token|bootstrap|secret|bearer/i.test(`${key}\n${value}`)) {
          throw new Error("session material must not persist in Web storage");
        }
      }
    } catch (error) {
      if (error instanceof Error && error.message.includes("must not persist")) {
        throw error;
      }
    }
  }
}

export function rememberBearer(channel: ChannelClass, token: string): void {
  assertMemoryOnlyStore();
  memory[channel] = token;
}

export function rememberPrincipal(principal: string): void {
  assertMemoryOnlyStore();
  memory.principal = principal;
}

export function sessionPrincipal(): string {
  return memory.principal ?? "principal://local/owner";
}

export function bearer(channel: ChannelClass): string | undefined {
  return memory[channel];
}

export function clearSession(): void {
  memory.management = undefined;
  memory.task = undefined;
  memory.principal = undefined;
}

export function exportClientState(): Record<string, never> {
  return {};
}

export function sessionHasChannel(channel: ChannelClass): boolean {
  return typeof memory[channel] === "string" && memory[channel]!.length > 0;
}

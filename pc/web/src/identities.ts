export const AGENT_IDENTITY_KEYS = [
  "package",
  "installation",
  "registration",
  "instance",
  "sidecar",
  "execution",
  "process",
  "task",
  "shell_session",
] as const;

export type AgentIdentityKey = (typeof AGENT_IDENTITY_KEYS)[number];

export type AgentIdentities = Partial<Record<AgentIdentityKey, string>>;

export function emptyIdentities(): AgentIdentities {
  return Object.fromEntries(AGENT_IDENTITY_KEYS.map((key) => [key, "unknown"])) as AgentIdentities;
}

export function mergeIdentities(partial: AgentIdentities): AgentIdentities {
  return { ...emptyIdentities(), ...partial };
}

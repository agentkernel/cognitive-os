export const MEMBER_CONFIG_TABS = [
  { id: "duty", label: "Duty" },
  { id: "input", label: "Input" },
  { id: "output", label: "Output" },
  { id: "skills", label: "Skills" },
  { id: "tools", label: "Tools" },
  { id: "prompt", label: "Brief" },
  { id: "loop", label: "Loop" },
  { id: "perms", label: "Perms" },
] as const;

export type MemberConfigTabId = (typeof MEMBER_CONFIG_TABS)[number]["id"];

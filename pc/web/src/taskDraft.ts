/** Build contracted Task preview drafts. Browser does not mint authority. */

export function uuidV7(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  const ms = BigInt(Date.now());
  bytes[0] = Number((ms >> 40n) & 0xffn);
  bytes[1] = Number((ms >> 32n) & 0xffn);
  bytes[2] = Number((ms >> 24n) & 0xffn);
  bytes[3] = Number((ms >> 16n) & 0xffn);
  bytes[4] = Number((ms >> 8n) & 0xffn);
  bytes[5] = Number(ms & 0xffn);
  bytes[6] = (bytes[6] & 0x0f) | 0x70;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const hex = [...bytes].map((byte) => byte.toString(16).padStart(2, "0")).join("");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

export type WorkspaceSearchDraft = {
  allowed_state_domains: string[];
  allowed_tools: string[];
  budget: { semantic_calls: number; tool_calls: number };
  budget_id: string;
  conditions: Array<{
    description: string;
    id: string;
    kind: "acceptance";
    verifier_ref: string;
  }>;
  deadline: string;
  loop_object_id: string;
  max_iterations: number;
  max_retries: number;
  objective: string;
  scope: { in_scope: string[]; out_of_scope: string[] };
  task_ref: string;
};

export function workspaceSearchDraft(objective: string): WorkspaceSearchDraft {
  const id = uuidV7();
  return {
    allowed_state_domains: ["task", "effect"],
    allowed_tools: ["native.workspace.search"],
    budget: { semantic_calls: 4, tool_calls: 4 },
    budget_id: uuidV7(),
    conditions: [
      {
        description: "independent fixed-effect verification",
        id: "acceptance",
        kind: "acceptance",
        verifier_ref: "verifier://personal/fixed-effect",
      },
    ],
    deadline: "2027-12-31T00:00:00Z",
    loop_object_id: uuidV7(),
    max_iterations: 4,
    max_retries: 0,
    objective,
    scope: {
      in_scope: ["workspace search"],
      out_of_scope: ["bash", "edit", "write"],
    },
    task_ref: `task://personal/web-ui/${id}`,
  };
}

export function interpretCandidate(objective: string): {
  objectives: string[];
  constraints: string[];
  forbidden: string[];
  assumptions: string[];
  ambiguities: string[];
  information_gaps: string[];
} {
  return {
    objectives: [objective],
    constraints: [],
    forbidden: [],
    assumptions: [],
    ambiguities: [],
    information_gaps: [],
  };
}

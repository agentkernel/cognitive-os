/**
 * P14-T02 Dual Track create-wizard draft model. Local only — not Project
 * authority. Activation still requires draft.create → preview.request → confirm.
 */

export const PROCESS_STAGES = [
  { id: "collect", label: "收集本周事实", owner: "收集岗" },
  { id: "analyze", label: "分析与建议", owner: "分析岗" },
  { id: "draft", label: "起草产出", owner: "起草岗" },
] as const;

export type StageId = (typeof PROCESS_STAGES)[number]["id"];

export const RUNTIME_SLOTS = [
  { id: "prompt", label: "工作说明" },
  { id: "tools", label: "工具" },
  { id: "skills", label: "能力包" },
  { id: "loop", label: "周期与触发" },
  { id: "mcp", label: "外部连接" },
  { id: "files", label: "文档范围" },
] as const;

export type SlotId = (typeof RUNTIME_SLOTS)[number]["id"];
export type RingStatus = "pending" | "confirmed" | "gap";
export type SeatStatus = "pending" | "seated" | "refused";
export type TestOutcome = "idle" | "running" | "pass" | "fail" | "unknown";
export type ModelChoice = "unselected" | "draft-bound";

export interface RingDraft {
  id: StageId;
  label: string;
  owner: string;
  input: string;
  method: string;
  rights: string;
  status: RingStatus;
}

export interface MemberDraft {
  id: string;
  name: string;
  stageId: StageId;
  model: ModelChoice;
  slots: Record<SlotId, boolean>;
  status: SeatStatus;
}

export function emptySlots(): Record<SlotId, boolean> {
  return {
    prompt: false,
    tools: false,
    skills: false,
    loop: false,
    mcp: false,
    files: false,
  };
}

export function defaultRings(): RingDraft[] {
  return PROCESS_STAGES.map((stage) => ({
    id: stage.id,
    label: stage.label,
    owner: stage.owner,
    input: "",
    method: "",
    rights: "",
    status: "pending",
  }));
}

export function rosterFromRings(rings: readonly RingDraft[]): MemberDraft[] {
  return rings.map((ring) => ({
    id: `member-${ring.id}`,
    name: ring.owner,
    stageId: ring.id,
    model: "unselected",
    slots: emptySlots(),
    status: "pending",
  }));
}

export function idleTests(): Record<StageId, TestOutcome> {
  return { collect: "idle", analyze: "idle", draft: "idle" };
}

export function ringResolved(status: RingStatus): boolean {
  return status === "confirmed" || status === "gap";
}

export function allRingsResolved(rings: readonly RingDraft[]): boolean {
  return rings.every((ring) => ringResolved(ring.status));
}

export function ringReachable(rings: readonly RingDraft[], index: number): boolean {
  return rings.slice(0, index).every((ring) => ringResolved(ring.status));
}

export function slotsComplete(member: MemberDraft): boolean {
  return RUNTIME_SLOTS.every((slot) => member.slots[slot.id]);
}

export function memberSeated(member: MemberDraft | undefined): boolean {
  return Boolean(member && member.status === "seated" && member.model !== "unselected");
}

export function sequenceResolved(member: MemberDraft): boolean {
  return member.status === "seated" || member.status === "refused";
}

export function memberReachable(members: readonly MemberDraft[], index: number): boolean {
  return members.slice(0, index).every((member) => sequenceResolved(member));
}

export function allMembersSeated(members: readonly MemberDraft[]): boolean {
  return members.length > 0 && members.every((member) => memberSeated(member));
}

export function allTestsPassed(tests: Record<StageId, TestOutcome>): boolean {
  return PROCESS_STAGES.every((stage) => tests[stage.id] === "pass");
}

export function buildCharterBlob(input: {
  title: string;
  charter: string;
  rings: readonly RingDraft[];
  members: readonly MemberDraft[];
  tests: Record<StageId, TestOutcome>;
  joint: TestOutcome;
  goalReady: boolean;
}): string {
  const processLines = input.rings.map((ring) => {
    return `- ${ring.id} (${ring.label}): ${ring.status}; input=${ring.input.trim() || "(none)"}; method=${ring.method.trim() || "(none)"}; rights=${ring.rights.trim() || "(none)"}`;
  });
  const memberLines =
    input.members.length === 0
      ? ["- (none — roster not created)"]
      : input.members.map((member) => {
          const slotBits = RUNTIME_SLOTS.map((slot) => `${slot.id}:${member.slots[slot.id] ? "draft" : "empty"}`).join(
            ",",
          );
          return `- ${member.name} stage=${member.stageId} model=${member.model} seat=${member.status} slots=${slotBits}`;
        });
  const testLines = PROCESS_STAGES.map((stage) => `- ${stage.id}: ${input.tests[stage.id]}`);
  return [
    `title: ${input.title.trim()}`,
    `goal_and_trigger_confirmed: ${input.goalReady ? "yes" : "no"}`,
    `process:\n${processLines.join("\n")}`,
    `members:\n${memberLines.join("\n")}`,
    `stage_tests:\n${testLines.join("\n")}`,
    `joint: ${input.joint}`,
    `charter:\n${input.charter.trim()}`,
    "honesty: owner-recorded Dual Track draft; independent verification is not this wizard; local notes are not Project authority.",
  ].join("\n\n");
}

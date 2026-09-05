import { useMemo, useState, type FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import { readJson } from "../../api";
import { PageHeader } from "../../components/PageHeader";
import { HonestyNote } from "../../state/HonestyNote";
import { CreateAssistantChat } from "./CreateAssistantChat";
import {
  PROCESS_STAGES,
  RUNTIME_SLOTS,
  allMembersSeated,
  allRingsResolved,
  allTestsPassed,
  buildCharterBlob,
  defaultRings,
  idleTests,
  memberReachable,
  memberSeated,
  ringReachable,
  rosterFromRings,
  slotsComplete,
  type MemberDraft,
  type ModelChoice,
  type RingDraft,
  type StageId,
  type TestOutcome,
} from "./createWizardModel";

const STEPS = [
  { id: "create-init", title: "① Charter" },
  { id: "create-process", title: "② 流程初始化" },
  { id: "create-members", title: "③ 成员初始化" },
  { id: "create-test", title: "④ 分环节测试" },
  { id: "create-joint", title: "⑤ 联合调试" },
] as const;

type StepId = (typeof STEPS)[number]["id"];

function errorMessage(status: number, body: unknown): string {
  if (body && typeof body === "object") {
    const record = body as Record<string, unknown>;
    const nested = record.error;
    if (nested && typeof nested === "object") {
      const error = nested as Record<string, unknown>;
      const code = typeof error.code === "string" ? error.code : "error";
      const message = typeof error.message === "string" ? error.message : "";
      return `HTTP ${status} · ${code}${message ? ` — ${message}` : ""}`;
    }
    if (typeof record.code === "string") {
      const message = typeof record.message === "string" ? record.message : "";
      return `HTTP ${status} · ${record.code}${message ? ` — ${message}` : ""}`;
    }
  }
  return `HTTP ${status}`;
}

function field(body: unknown, key: string): string | undefined {
  if (!body || typeof body !== "object") {
    return undefined;
  }
  const value = (body as Record<string, unknown>)[key];
  return typeof value === "string" && value.length > 0 ? value : undefined;
}

function ringStatusLabel(status: RingDraft["status"]): string {
  if (status === "confirmed") {
    return "已确认这一环";
  }
  if (status === "gap") {
    return "留缺口";
  }
  return "待确认";
}

/**
 * Dual Track create wizard (P14-T02). Local ①–⑤ draft is not Project authority.
 * Minting requires management POST draft.create → preview.request → confirm.
 * Labels avoid fake Create project / Activate / Confirm chrome.
 */
export function CreateWizardPage() {
  const navigate = useNavigate();
  const [step, setStep] = useState(0);
  const [title, setTitle] = useState("");
  const [charter, setCharter] = useState("");
  const [rings, setRings] = useState<RingDraft[]>(defaultRings);
  const [ringIndex, setRingIndex] = useState(0);
  const [goalReady, setGoalReady] = useState(false);
  const [members, setMembers] = useState<MemberDraft[]>([]);
  const [activeMember, setActiveMember] = useState(0);
  const [tests, setTests] = useState<Record<StageId, TestOutcome>>(idleTests);
  const [testIndex, setTestIndex] = useState(0);
  const [joint, setJoint] = useState<TestOutcome>("idle");
  const [draftId, setDraftId] = useState<string | undefined>();
  const [previewId, setPreviewId] = useState<string | undefined>();
  const [previewDigest, setPreviewDigest] = useState<string | undefined>();
  const [projectId, setProjectId] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const current: StepId = STEPS[step].id;
  const currentRing = rings[ringIndex] ?? rings[0];
  const currentMember = members[activeMember];
  const currentTestStage = PROCESS_STAGES[testIndex] ?? PROCESS_STAGES[0];
  const testOwner = members.find((member) => member.stageId === currentTestStage.id);
  const testOutcome = tests[currentTestStage.id];
  const ringsReady = allRingsResolved(rings);
  const rosterSeated = allMembersSeated(members);
  const testsPassed = allTestsPassed(tests);
  const canSeat = Boolean(
    currentMember &&
      memberReachable(members, activeMember) &&
      currentMember.model !== "unselected" &&
      slotsComplete(currentMember) &&
      currentMember.status === "pending",
  );
  const charterBlob = useMemo(
    () =>
      buildCharterBlob({
        title,
        charter,
        rings,
        members,
        tests,
        joint,
        goalReady,
      }),
    [title, charter, rings, members, tests, joint, goalReady],
  );

  function goNext(event: FormEvent) {
    event.preventDefault();
    setError(undefined);
    if (step === 0 && (title.trim() === "" || charter.trim() === "")) {
      setError("Charter title and body are required before leaving this step. Nothing is written yet.");
      return;
    }
    setStep((value) => Math.min(value + 1, STEPS.length - 1));
  }

  function resolveRing(status: "confirmed" | "gap") {
    setError(undefined);
    setRings((previous) =>
      previous.map((ring, index) => (index === ringIndex ? { ...ring, status } : ring)),
    );
    setRingIndex((index) => Math.min(index + 1, rings.length - 1));
    setGoalReady(false);
  }

  function patchRing(patch: Partial<RingDraft>) {
    setRings((previous) =>
      previous.map((ring, index) => (index === ringIndex ? { ...ring, ...patch } : ring)),
    );
  }

  function createRoster() {
    setError(undefined);
    setMembers(rosterFromRings(rings));
    setActiveMember(0);
    setTests(idleTests());
    setTestIndex(0);
    setJoint("idle");
    setPreviewId(undefined);
    setPreviewDigest(undefined);
  }

  function patchActiveMember(patch: Partial<MemberDraft>) {
    setMembers((previous) =>
      previous.map((member, index) => (index === activeMember ? { ...member, ...patch } : member)),
    );
  }

  function seatActive() {
    if (!canSeat) {
      setError("Model and the six runtime slots are required before 就位. Missing model is pending, not a silent bind.");
      return;
    }
    setError(undefined);
    setMembers((previous) =>
      previous.map((member, index) =>
        index === activeMember ? { ...member, status: "seated" } : member,
      ),
    );
    setActiveMember((index) => Math.min(index + 1, Math.max(members.length - 1, 0)));
  }

  function refuseActive() {
    if (!currentMember || currentMember.status === "seated") {
      return;
    }
    setError(undefined);
    setMembers((previous) =>
      previous.map((member, index) =>
        index === activeMember ? { ...member, status: "refused" } : member,
      ),
    );
    setActiveMember((index) => Math.min(index + 1, Math.max(members.length - 1, 0)));
  }

  function setTestOutcome(outcome: TestOutcome) {
    setTests((previous) => ({ ...previous, [currentTestStage.id]: outcome }));
  }

  async function requestPreview() {
    if (joint !== "pass" || !testsPassed) {
      setError("⑤ 验收 needs Owner-recorded joint pass after every stage test pass. No Project was minted.");
      return;
    }
    setBusy(true);
    setError(undefined);
    setPreviewId(undefined);
    setPreviewDigest(undefined);
    try {
      const created = await readJson("/management/project/v1/draft.create", "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          payload: title.trim(),
          charter: charterBlob,
        }),
      });
      if (!created.ok) {
        setError(errorMessage(created.status, created.body));
        return;
      }
      const nextDraft = field(created.body, "draft_id");
      if (!nextDraft) {
        setError("draft.create returned no draft_id. No Project was minted.");
        return;
      }
      setDraftId(nextDraft);
      const previewed = await readJson("/management/project/v1/preview.request", "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          subject_kind: "activation",
          subject_ref: nextDraft,
        }),
      });
      if (!previewed.ok) {
        setError(errorMessage(previewed.status, previewed.body));
        return;
      }
      const nextPreview = field(previewed.body, "preview_id");
      const nextDigest = field(previewed.body, "preview_digest");
      if (!nextPreview || !nextDigest) {
        setError("preview.request returned no digest-bound preview. No Project was minted.");
        return;
      }
      setPreviewId(nextPreview);
      setPreviewDigest(nextDigest);
    } finally {
      setBusy(false);
    }
  }

  async function writeProject() {
    if (!previewId || !previewDigest) {
      setError("Request a preview first. This page does not mint a Project without a digest.");
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      const written = await readJson("/management/project/v1/confirm", "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          preview_id: previewId,
          preview_digest: previewDigest,
        }),
      });
      if (!written.ok) {
        setError(errorMessage(written.status, written.body));
        return;
      }
      const nextProject = field(written.body, "new_ref");
      if (!nextProject) {
        setError("confirm returned no new_ref. Treat the Project as not minted.");
        return;
      }
      setProjectId(nextProject);
      navigate("/");
    } finally {
      setBusy(false);
    }
  }

  return (
    <section data-page="opc-create-wizard" data-step={current}>
      <PageHeader
        title="Create Project"
        lede="①–⑤ Dual Track wizard. Local draft is not authority. Activation is digest-bound management HTTP."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. Vite is not the product origin.
        Process axis, seating, and tests stay local until preview.request then confirm.
        Owner-recorded pass/fail is not independent verification. This wizard does
        not Activate, Approve, or write a Project without a digest.
      </HonestyNote>
      <ol className="cp-quiet" aria-label="Create steps">
        {STEPS.map((item, index) => (
          <li key={item.id} data-step-item={item.id} aria-current={index === step ? "step" : undefined}>
            {item.title}
          </li>
        ))}
      </ol>
      {current === "create-init" ? (
        <form onSubmit={goNext}>
          <label className="cp-field">
            Title
            <input
              name="title"
              value={title}
              onChange={(event) => setTitle(event.target.value)}
            />
          </label>
          <label className="cp-field">
            Charter
            <textarea
              name="charter"
              value={charter}
              onChange={(event) => setCharter(event.target.value)}
              rows={8}
            />
          </label>
          <p className="cp-quiet">A draft exists only after preview on the last step.</p>
          <button type="submit" className="cp-button cp-button--primary">
            Continue
          </button>
        </form>
      ) : null}
      {current === "create-process" ? (
        <div>
          <h2 className="cp-title">② 一条流程轴，一次只开一环</h2>
          <p className="cp-quiet">
            「确认这一环」后再开下一环。缺口留在轴上，不标已就绪。最后确认总目标与项目触发，再进入 ③。
          </p>
          <div className="cp-filters" data-process-axis="" role="list" aria-label="流程轴">
            {rings.map((ring, index) => {
              const reachable = ringReachable(rings, index);
              return (
                <button
                  key={ring.id}
                  type="button"
                  className="cp-button"
                  role="listitem"
                  data-ring={ring.id}
                  data-ring-status={ring.status}
                  aria-current={index === ringIndex ? "step" : undefined}
                  disabled={!reachable}
                  onClick={() => {
                    if (reachable) setRingIndex(index);
                  }}
                >
                  {ring.label}
                  <span className="cp-quiet"> · {reachable ? ringStatusLabel(ring.status) : "先完成前环"}</span>
                </button>
              );
            })}
          </div>
          <section className="cp-panel">
            <h3>这一环 · {currentRing.label}</h3>
            <p className="cp-quiet">意向岗位：{currentRing.owner}。③ 按此环配岗。</p>
            <label className="cp-field">
              输入
              <textarea
                name={`ring-${currentRing.id}-input`}
                value={currentRing.input}
                onChange={(event) => patchRing({ input: event.target.value })}
                rows={3}
              />
            </label>
            <label className="cp-field">
              执行方式
              <textarea
                name={`ring-${currentRing.id}-method`}
                value={currentRing.method}
                onChange={(event) => patchRing({ method: event.target.value })}
                rows={3}
              />
            </label>
            <label className="cp-field">
              权限后果
              <textarea
                name={`ring-${currentRing.id}-rights`}
                value={currentRing.rights}
                onChange={(event) => patchRing({ rights: event.target.value })}
                rows={3}
              />
            </label>
            <p>
              <button type="button" className="cp-button" onClick={() => setStep(0)}>
                Back
              </button>{" "}
              <button
                type="button"
                className="cp-button cp-button--primary"
                onClick={() => resolveRing("confirmed")}
              >
                确认这一环
              </button>{" "}
              <button type="button" className="cp-button" onClick={() => resolveRing("gap")}>
                本环留缺口
              </button>
              {ringsReady && !goalReady ? (
                <>
                  {" "}
                  <button
                    type="button"
                    className="cp-button cp-button--primary"
                    onClick={() => setGoalReady(true)}
                  >
                    确认总目标与项目触发
                  </button>
                </>
              ) : null}
              {goalReady ? (
                <>
                  {" "}
                  <button
                    type="button"
                    className="cp-button cp-button--primary"
                    onClick={() => {
                      setError(undefined);
                      setStep(2);
                    }}
                  >
                    进入 ③
                  </button>
                </>
              ) : null}
            </p>
            <p className="cp-quiet">
              {goalReady
                ? "总目标已确认。下一步才进入成员初始化。"
                : ringsReady
                  ? "各环已处理。先确认总目标与项目触发，不要同一点击跳进 ③。"
                  : "按顺序确认。缺口不能标已确认。"}
            </p>
          </section>
        </div>
      ) : null}
      {current === "create-members" ? (
        <div>
          <h2 className="cp-title">③ 创建岗位，再逐人就位</h2>
          <p className="cp-quiet">
            按已确认流程建班子。模型必选，未选不能就位、也不会静默绑定。拒绝 = 未加入。全员就位后才进入 ④。
          </p>
          <p>
            <button type="button" className="cp-button" onClick={() => setStep(1)}>
              回 ② 改流程
            </button>{" "}
            <button type="button" className="cp-button cp-button--primary" onClick={createRoster}>
              创建岗位
            </button>
          </p>
          {members.length === 0 ? (
            <p className="cp-quiet">还没有岗位。确认「创建岗位」后出现名单。</p>
          ) : (
            <section className="cp-panel">
              <p className="cp-quiet">
                初始化进度 {members.filter((member) => memberSeated(member)).length} / {members.length}
              </p>
              <table className="cp-table">
                <caption>按顺序就位。没选模型 = 待定。</caption>
                <thead>
                  <tr>
                    <th scope="col">岗位</th>
                    <th scope="col">模型</th>
                    <th scope="col">就位</th>
                  </tr>
                </thead>
                <tbody>
                  {members.map((member, index) => (
                    <tr key={member.id} data-member-row={member.id}>
                      <th scope="row">{member.name}</th>
                      <td>{member.model === "unselected" ? "未选" : "草稿已选"}</td>
                      <td>
                        {member.status === "seated"
                          ? "已就位"
                          : member.status === "refused"
                            ? "已拒绝"
                            : "待确认"}
                      </td>
                      <td>
                        <button
                          type="button"
                          className="cp-button"
                          disabled={!memberReachable(members, index)}
                          onClick={() => setActiveMember(index)}
                        >
                          打开此人
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {currentMember ? (
                <div data-active-member={currentMember.id}>
                  <h3>
                    当前初始化 · {activeMember + 1} / {members.length} · {currentMember.name}
                  </h3>
                  {currentMember.model === "unselected" ? (
                    <p className="cp-quiet">先选模型。未选模型不能就位，也不会静默绑定。</p>
                  ) : null}
                  <label className="cp-field">
                    岗位模型
                    <select
                      name="member-model"
                      value={currentMember.model}
                      onChange={(event) =>
                        patchActiveMember({ model: event.target.value as ModelChoice })
                      }
                    >
                      <option value="unselected">未选 · 待定</option>
                      <option value="draft-bound">选用已接模型（草稿，非静默绑定）</option>
                    </select>
                  </label>
                  {RUNTIME_SLOTS.map((slot) => (
                    <label key={slot.id} className="cp-field">
                      <input
                        type="checkbox"
                        name={`slot-${slot.id}`}
                        checked={currentMember.slots[slot.id]}
                        onChange={() =>
                          patchActiveMember({
                            slots: {
                              ...currentMember.slots,
                              [slot.id]: !currentMember.slots[slot.id],
                            },
                          })
                        }
                      />{" "}
                      {slot.label}（草稿已填写）
                    </label>
                  ))}
                  <p>
                    <button
                      type="button"
                      className="cp-button cp-button--primary"
                      disabled={!canSeat}
                      onClick={seatActive}
                    >
                      确认就位
                    </button>{" "}
                    <button
                      type="button"
                      className="cp-button"
                      disabled={currentMember.status === "seated"}
                      onClick={refuseActive}
                    >
                      拒绝此岗
                    </button>
                  </p>
                </div>
              ) : null}
            </section>
          )}
          <p>
            <button
              type="button"
              className="cp-button cp-button--primary"
              disabled={!rosterSeated}
              onClick={() => {
                setError(undefined);
                setStep(3);
              }}
            >
              进入 ④
            </button>
            <span className="cp-quiet">
              {members.length === 0
                ? " 先创建岗位。"
                : rosterSeated
                  ? " 可以进入分环节测试。"
                  : " 每人要选模型并确认六个槽位。拒绝 = 未加入。"}
            </span>
          </p>
        </div>
      ) : null}
      {current === "create-test" ? (
        <div>
          <h2 className="cp-title">④ 测这一环，直到子产出可打开</h2>
          <p className="cp-quiet">先检查负责人是否就位。未知不能通过。Owner 记录不是独立核对。</p>
          <div className="cp-filters" data-process-axis="" role="list" aria-label="测试轴">
            {PROCESS_STAGES.map((stage, index) => {
              const owner = members.find((member) => member.stageId === stage.id);
              return (
                <button
                  key={stage.id}
                  type="button"
                  className="cp-button"
                  role="listitem"
                  data-test-stage={stage.id}
                  aria-current={index === testIndex ? "step" : undefined}
                  onClick={() => setTestIndex(index)}
                >
                  {stage.label}
                  <span className="cp-quiet">
                    {" "}
                    · {memberSeated(owner) ? "已就位" : "未就位"} · {tests[stage.id]}
                  </span>
                </button>
              );
            })}
          </div>
          <section className="cp-panel">
            <h3>正在测 · {currentTestStage.label}</h3>
            <p className="cp-quiet">
              负责人：{testOwner?.name ?? "没有对应成员"}。
              {memberSeated(testOwner) ? "已就位，可以开测。" : "未就位，不能开始测。"}
            </p>
            {!memberSeated(testOwner) ? (
              <p>
                <button type="button" className="cp-button" onClick={() => setStep(2)}>
                  回 ③ 初始化此人
                </button>
              </p>
            ) : null}
            {testOutcome === "unknown" ? (
              <p className="cp-stateview" role="status">
                说不清。未知不能通过。
              </p>
            ) : null}
            {testOutcome === "fail" ? (
              <p className="cp-stateview" role="status">
                不通过。回到 ② 或 ③ 改这一环。不跳下一环。
              </p>
            ) : null}
            {testOutcome === "pass" ? (
              <details data-openable-result={currentTestStage.id}>
                <summary>打开这一环结果</summary>
                <p className="cp-quiet">
                  Owner-recorded sample for {currentTestStage.label}. Independent verification is
                  not this wizard. This is not a daemon write.
                </p>
              </details>
            ) : null}
            <p>
              <button type="button" className="cp-button" onClick={() => setStep(2)}>
                Back
              </button>{" "}
              <button
                type="button"
                className="cp-button"
                disabled={!memberSeated(testOwner) || testOutcome === "pass"}
                onClick={() => {
                  if (!memberSeated(testOwner)) {
                    setError("成员未就位，这一环不能开始测。");
                    return;
                  }
                  setError(undefined);
                  setTestOutcome("running");
                }}
              >
                开始测
              </button>{" "}
              <button
                type="button"
                className="cp-button"
                disabled={testOutcome !== "running"}
                onClick={() => setTestOutcome("pass")}
              >
                记录通过
              </button>{" "}
              <button
                type="button"
                className="cp-button"
                disabled={testOutcome !== "running"}
                onClick={() => setTestOutcome("fail")}
              >
                记录不通过
              </button>{" "}
              <button
                type="button"
                className="cp-button"
                disabled={testOutcome !== "running"}
                onClick={() => setTestOutcome("unknown")}
              >
                记录说不清
              </button>{" "}
              <button
                type="button"
                className="cp-button cp-button--primary"
                disabled={testOutcome !== "pass"}
                onClick={() => {
                  if (testIndex >= PROCESS_STAGES.length - 1) {
                    setStep(4);
                    return;
                  }
                  setTestIndex((index) => index + 1);
                }}
              >
                {testIndex >= PROCESS_STAGES.length - 1 ? "末环通过，进入 ⑤" : "通过，下一环"}
              </button>
              {testOutcome === "fail" ? (
                <>
                  {" "}
                  <button
                    type="button"
                    className="cp-button"
                    onClick={() => {
                      const index = rings.findIndex((ring) => ring.id === currentTestStage.id);
                      setRingIndex(index >= 0 ? index : 0);
                      setStep(1);
                    }}
                  >
                    回 ② 改这一环
                  </button>
                </>
              ) : null}
            </p>
          </section>
        </div>
      ) : null}
      {current === "create-joint" ? (
        <div>
          <h2 className="cp-title">⑤ 联合调试 · 第一次成功</h2>
          <p className="cp-quiet">
            打开总成果 + 核对状态。未知不能验收。无假发布。验收接到既有 preview.request → confirm。
          </p>
          <ol className="cp-quiet">
            {PROCESS_STAGES.map((stage) => (
              <li key={stage.id}>
                {stage.label} · {tests[stage.id]}
              </li>
            ))}
          </ol>
          {PROCESS_STAGES.filter((stage) => tests[stage.id] === "pass").map((stage) => (
            <details key={stage.id} data-openable-result={stage.id}>
              <summary>打开 {stage.label} 结果</summary>
              <p className="cp-quiet">Owner-recorded sample. Independent verification is not this wizard.</p>
            </details>
          ))}
          {joint === "unknown" ? (
            <p className="cp-stateview" role="status">
              核对不上。不能验收。
            </p>
          ) : null}
          {joint === "fail" ? (
            <p className="cp-stateview" role="status">
              失败。回到 ④ 测该环，或回 ② / ③。聊天不能当验收。
            </p>
          ) : null}
          {joint === "pass" ? (
            <details data-openable-result="joint">
              <summary>打开总成果</summary>
              <p className="cp-quiet">
                Title: {title.trim() || "(missing)"}. This is ⑤ aha, not publish. Write Project
                still needs a digest-bound preview.
              </p>
            </details>
          ) : (
            <p className="cp-quiet">还没有可打开的总成果。</p>
          )}
          <p className="cp-quiet">
            {draftId ? ` Draft ${draftId}.` : " No draft yet."}
            {previewId ? ` Preview ${previewId}.` : ""}
          </p>
          <p>
            <button type="button" className="cp-button" onClick={() => setStep(3)} disabled={busy}>
              Back
            </button>{" "}
            <button
              type="button"
              className="cp-button"
              disabled={!testsPassed || busy}
              onClick={() => setJoint("running")}
            >
              开始联调
            </button>{" "}
            <button
              type="button"
              className="cp-button"
              disabled={joint !== "running" || busy}
              onClick={() => setJoint("pass")}
            >
              核对通过
            </button>{" "}
            <button
              type="button"
              className="cp-button"
              disabled={joint !== "running" || busy}
              onClick={() => setJoint("fail")}
            >
              联调失败
            </button>{" "}
            <button
              type="button"
              className="cp-button"
              disabled={joint !== "running" || busy}
              onClick={() => setJoint("unknown")}
            >
              联调说不清
            </button>
            {joint === "fail" ? (
              <>
                {" "}
                <button type="button" className="cp-button" onClick={() => setStep(3)}>
                  回 ④ 测失败环节
                </button>{" "}
                <button type="button" className="cp-button" onClick={() => setStep(1)}>
                  回 ② 改流程
                </button>{" "}
                <button type="button" className="cp-button" onClick={() => setStep(2)}>
                  回 ③ 改成员
                </button>
              </>
            ) : null}
          </p>
          <p>
            <button
              type="button"
              className="cp-button"
              onClick={() => void requestPreview()}
              disabled={busy || joint !== "pass"}
            >
              Request preview
            </button>{" "}
            <button
              type="button"
              className="cp-button cp-button--primary"
              onClick={() => void writeProject()}
              disabled={busy || !previewId || !previewDigest}
            >
              Write Project
            </button>
          </p>
          <p className="cp-quiet">
            Request preview mints a digest-bound ApprovalPreview. Write Project posts that digest
            on management confirm. Chat cannot do this.
          </p>
        </div>
      ) : null}
      {error ? (
        <p className="cp-stateview" role="alert" data-wizard-error="true">
          {error} No Project was invented locally.
        </p>
      ) : null}
      {projectId ? (
        <p className="cp-quiet">
          Daemon returned <code className="cp-mono">{projectId}</code>.
        </p>
      ) : null}
      <CreateAssistantChat step={current} title={title} />
      <p>
        <Link to="/">Back to Today</Link>
      </p>
    </section>
  );
}

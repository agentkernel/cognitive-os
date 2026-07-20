import type { Locale } from "@/i18n/config";

const stages = {
  zh: [
    ["01", "Context", "受治理的非 authority 工作投影"],
    ["02", "Proposal", "概率组件产出的候选"],
    ["03", "Persisted Intent", "dispatch 前固定动作与幂等绑定"],
    ["04", "Authorization", "本地 authority 的确定性门禁"],
    ["05", "Effect", "保留 EXECUTED 或 OUTCOME_UNKNOWN"],
    ["06", "Reconcile", "先闭合外部结果；未知则隔离"],
    ["07", "Verification", "对固定后态检查 criteria"],
    ["08", "Acceptance", "验收 authority 推进 Task"],
  ],
  en: [
    ["01", "Context", "governed, non-authority working projection"],
    ["02", "Proposal", "candidate from a probabilistic component"],
    ["03", "Persisted Intent", "action and idempotency fixed before dispatch"],
    ["04", "Authorization", "deterministic local-authority gate"],
    ["05", "Effect", "preserves EXECUTED or OUTCOME_UNKNOWN"],
    ["06", "Reconcile", "closes external outcome; unknown quarantines"],
    ["07", "Verification", "checks criteria against a fixed post-state"],
    ["08", "Acceptance", "acceptance authority advances the Task"],
  ],
} as const;

export function GovernedFlowThread({
  locale,
  variant = "full",
}: {
  locale: Locale;
  variant?: "full" | "compact";
}) {
  const zh = locale === "zh";
  return (
    <aside
      className={`governed-thread governed-thread--${variant}`}
      aria-labelledby={`flow-thread-${locale}-${variant}`}
    >
      <header>
        <p>{zh ? "静态语义模型" : "Static semantic model"}</p>
        <h2 id={`flow-thread-${locale}-${variant}`}>Governed Flow Thread</h2>
        <span>{zh ? "不是实时进度" : "Never live progress"}</span>
      </header>
      <ol>
        {stages[locale].map(([number, title, detail]) => (
          <li key={title}>
            <span aria-hidden="true">{number}</span>
            <div>
              <strong>{title}</strong>
              <p>{detail}</p>
            </div>
          </li>
        ))}
      </ol>
      <p className="governed-thread__exception">
        <strong>OUTCOME_UNKNOWN</strong>
        {" → "}
        Reconcile
        {" → "}
        {zh
          ? "单独授权的 compensation，或 quarantine。"
          : "separately authorized compensation, or quarantine."}
      </p>
    </aside>
  );
}

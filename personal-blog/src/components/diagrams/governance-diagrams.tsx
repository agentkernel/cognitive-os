import {
  coreContextStages,
  governedFlowStages,
  lifecycleDomains,
  standardContextStages,
} from "@/data/cognitiveos";
import type { Locale } from "@/i18n/config";
import { pagePath } from "@/i18n/routes";

type DiagramMode = "full" | "summary";

type DiagramFrameProps = {
  id: string;
  title: string;
  caption: string;
  source: string;
  longText: string;
  mobileItems: readonly string[];
  mode: DiagramMode;
  fullHref: string;
  children: React.ReactNode;
};

function DiagramFrame({
  id,
  title,
  caption,
  source,
  longText,
  mobileItems,
  mode,
  fullHref,
  children,
}: DiagramFrameProps) {
  return (
    <figure
      id={id}
      className={`semantic-diagram semantic-diagram--${mode}`}
      aria-labelledby={`${id}-caption`}
    >
      <div className="diagram-heading">
        <span aria-hidden="true">FIG.</span>
        <h3>{title}</h3>
      </div>
      {mode === "full" ? (
        <div
          className="diagram-canvas"
          role="region"
          aria-label={title}
          tabIndex={0}
        >
          {children}
        </div>
      ) : null}
      <ol className="diagram-mobile-summary" aria-label={title}>
        {mobileItems.map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ol>
      <figcaption id={`${id}-caption`}>
        <span>{caption}</span>
        <small>{source}</small>
      </figcaption>
      {mode === "summary" ? (
        <a className="diagram-full-link" href={fullHref}>
          Open full diagram / 打开宽版图解
        </a>
      ) : null}
      <details className="diagram-transcript">
        <summary>{title} — text alternative</summary>
        <p>{longText}</p>
      </details>
    </figure>
  );
}

export function OverallArchitectureDiagram({
  locale,
  mode = "full",
}: {
  locale: Locale;
  mode?: DiagramMode;
}) {
  const zh = locale === "zh";
  const layers = zh
    ? [
        "7 · Agent 与应用",
        "6 · Harness 与认知服务",
        "5 · Context / 状态 / 知识 / 记忆 / 目录",
        "4 · 认知微内核与 AKP",
        "3 · 操作 / 技能 / 运行时",
        "2 · 资源织构与异构计算",
        "1 · 宿主 / 网络 / 设备 / 物理世界",
      ]
    : [
        "7 · Agents & applications",
        "6 · Harness & cognitive services",
        "5 · Context / state / knowledge / memory / catalogue",
        "4 · Cognitive microkernel & AKP",
        "3 · Operations / skills / runtime",
        "2 · Resource fabric & heterogeneous compute",
        "1 · Host / network / device / physical world",
      ];

  return (
    <DiagramFrame
      id={`overall-${locale}`}
      title={zh ? "总体架构责任视图" : "Overall responsibility view"}
      caption={
        zh
          ? "Informative：三平面与七层解释责任；Context 横切但不成为 authority 平面。"
          : "Informative: three planes and seven layers explain responsibility; Context cuts across them without becoming an authority plane."
      }
      source={
        zh
          ? "来源：CognitiveOS-Architecture.md §4.1–§4.5；研究快照 b626e88。"
          : "Source: CognitiveOS-Architecture.md §4.1–§4.5; research snapshot b626e88."
      }
      longText={
        zh
          ? "七层从物理世界向上连接 Agent。第七至第六层属于体验平面，第五至第四层属于控制平面，第三至第一层属于执行与数据平面。认知微内核位于第四层，实时安全内核独立连接运行时与物理执行器。Context Engineering 横穿所有平面，但没有状态提交 authority。"
          : "Seven layers connect the physical world to agents. Layers seven to six form the experience plane, five to four the control plane, and three to one the execution and data plane. The cognitive microkernel sits at layer four. A separate real-time safety kernel connects runtime and physical actuators. Context Engineering crosses every plane without state-commit authority."
      }
      mobileItems={[
        ...(zh
          ? ["体验平面：7–6", "控制平面：5–4", "执行与数据平面：3–1"]
          : ["Experience plane: 7–6", "Control plane: 5–4", "Execution & data plane: 3–1"]),
        ...(zh
          ? ["横切：Context Engineering", "独立：实时安全内核"]
          : ["Cross-cutting: Context Engineering", "Separate: real-time safety kernel"]),
      ]}
      mode={mode}
      fullHref={`${pagePath(locale, "cognitiveos")}#overall-${locale}`}
    >
      <svg
        className="diagram-svg"
        viewBox="0 0 1120 730"
        role="img"
        aria-labelledby={`overall-${locale}-title overall-${locale}-desc`}
      >
        <title id={`overall-${locale}-title`}>
          {zh ? "双内核、三平面、七层总体图" : "Dual-kernel, three-plane, seven-layer overview"}
        </title>
        <desc id={`overall-${locale}-desc`}>
          {zh
            ? "七个水平层按三个平面分组，Context 纵向横切，实时安全内核位于右侧。"
            : "Seven horizontal layers are grouped into three planes, Context crosses vertically, and the safety kernel sits to the right."}
        </desc>
        <g className="diagram-plane">
          <rect x="40" y="42" width="760" height="172" rx="2" />
          <rect x="40" y="222" width="760" height="204" rx="2" />
          <rect x="40" y="434" width="760" height="252" rx="2" />
        </g>
        <g className="diagram-plane-label">
          <text x="64" y="72">{zh ? "体验平面" : "EXPERIENCE PLANE"}</text>
          <text x="64" y="252">{zh ? "控制平面" : "CONTROL PLANE"}</text>
          <text x="64" y="464">{zh ? "执行与数据平面" : "EXECUTION & DATA PLANE"}</text>
        </g>
        {layers.map((layer, index) => {
          const y = 84 + index * 84;
          return (
            <g key={layer} className={index === 3 ? "diagram-node authority" : "diagram-node"}>
              <rect x="188" y={y} width="540" height="56" rx="2" />
              <text x="214" y={y + 35}>
                {layer}
              </text>
            </g>
          );
        })}
        <g className="diagram-crosscut">
          <rect x="760" y="72" width="94" height="590" rx="2" />
          <text x="807" y="364" textAnchor="middle" transform="rotate(-90 807 364)">
            CONTEXT ENGINEERING
          </text>
        </g>
        <g className="diagram-safety">
          <rect x="886" y="430" width="190" height="166" rx="2" />
          <text x="981" y="486" textAnchor="middle">
            {zh ? "实时安全内核" : "REAL-TIME"}
          </text>
          <text x="981" y="518" textAnchor="middle">
            {zh ? "最终执行器仲裁" : "SAFETY KERNEL"}
          </text>
          <path d="M886 514H728M981 596V652H728" />
        </g>
      </svg>
    </DiagramFrame>
  );
}

export function AuthorityBoundaryDiagram({
  locale,
  mode = "full",
}: {
  locale: Locale;
  mode?: DiagramMode;
}) {
  const zh = locale === "zh";
  const proposal = zh
    ? ["LLM 候选", "检索集合", "排序 / 摘要", "操作匹配提议"]
    : ["LLM candidate", "retrieval set", "ranking / summary", "operation match proposal"];
  const authority = zh
    ? ["schema", "authorization", "CAS / transition", "budget / idempotency", "fencing / commit"]
    : ["schema", "authorization", "CAS / transition", "budget / idempotency", "fencing / commit"];

  return (
    <DiagramFrame
      id={`boundary-${locale}`}
      title={zh ? "提议与 authority 边界" : "Proposal and authority boundary"}
      caption={
        zh
          ? "概率组件只改变候选集合；确定性路径才可改变共享事实。"
          : "Probabilistic components may change the candidate set; only the deterministic path may change shared facts."
      }
      source={
        zh
          ? "来源：Core §2、§4、§15.5。"
          : "Source: Core §2, §4, and §15.5."
      }
      longText={
        zh
          ? "左侧 LLM、检索器与 ranker 产生 candidate 或 proposal。输出跨过 authority 边界后，由 schema、授权、CAS、状态迁移、硬预算、幂等、fencing 与 commit gate 检查。左侧不能产出授权决定或提交。"
          : "On the left, LLMs, retrievers, and rankers produce candidates or proposals. Across the authority boundary, schema, authorization, CAS, transitions, hard budgets, idempotency, fencing, and commit gates check the input. The left side cannot emit authorization decisions or commits."
      }
      mobileItems={[...proposal, "↓ authority boundary", ...authority]}
      mode={mode}
      fullHref={`${pagePath(locale, "cognitiveos")}#boundary-${locale}`}
    >
      <svg
        className="diagram-svg"
        viewBox="0 0 1120 560"
        role="img"
        aria-labelledby={`boundary-${locale}-title boundary-${locale}-desc`}
      >
        <title id={`boundary-${locale}-title`}>
          {zh ? "概率提议止于 authority 边界" : "Probabilistic proposals stop at the authority boundary"}
        </title>
        <desc id={`boundary-${locale}-desc`}>
          {zh
            ? "候选组件位于左侧，确定性门禁位于右侧，中间有清晰边界。"
            : "Candidate components sit left, deterministic gates right, with a clear boundary between them."}
        </desc>
        <text className="diagram-kicker" x="78" y="62">
          {zh ? "概率 / 可替换策略" : "PROBABILISTIC / REPLACEABLE POLICY"}
        </text>
        <text className="diagram-kicker" x="666" y="62">
          {zh ? "确定性 / AUTHORITY PATH" : "DETERMINISTIC / AUTHORITY PATH"}
        </text>
        {proposal.map((item, index) => (
          <g className="diagram-node proposal" key={item}>
            <rect x="78" y={96 + index * 94} width="338" height="62" rx="2" />
            <text x="100" y={134 + index * 94}>
              {item}
            </text>
          </g>
        ))}
        <path className="diagram-arrow" d="M416 285H632" />
        <path className="diagram-arrow-head" d="M614 270l20 15-20 15" />
        <g className="diagram-boundary">
          <line x1="554" x2="554" y1="82" y2="492" />
          <text x="542" y="488" textAnchor="end">
            AUTHORITY BOUNDARY
          </text>
        </g>
        {authority.map((item, index) => (
          <g className="diagram-node authority" key={item}>
            <rect x="674" y={86 + index * 80} width="364" height="54" rx="2" />
            <text x="696" y={120 + index * 80}>
              {item}
            </text>
          </g>
        ))}
      </svg>
    </DiagramFrame>
  );
}

export function GovernedFlowDiagram({
  locale,
  mode = "full",
}: {
  locale: Locale;
  mode?: DiagramMode;
}) {
  const zh = locale === "zh";
  const labels = zh
    ? ["Context", "提议", "持久 Intent", "授权", "Effect", "对账", "验证", "验收"]
    : [...governedFlowStages];

  return (
    <DiagramFrame
      id={`flow-${locale}`}
      title={zh ? "受治理改变主链" : "Governed change thread"}
      caption={
        zh
          ? "静态语义：OUTCOME_UNKNOWN 只能先对账；still unknown 进入独立授权补偿或隔离。"
          : "Static semantics: OUTCOME_UNKNOWN reconciles first; still unknown moves to independently authorized compensation or quarantine."
      }
      source={
        zh
          ? "来源：Intent/Effect standard §2–§5；Effect、Task、Verification transitions。"
          : "Source: Intent/Effect standard §2–§5; Effect, Task, and Verification transitions."
      }
      longText={
        zh
          ? "Context 支持提议，但提议必须先持久化 Intent、获得授权，再形成 Effect。Effect 结果未知时不能直接验证或验收，只能对账。仍未知的分支进入单独授权的补偿或隔离。Verification 通过后仍需 Acceptance authority 决定。"
          : "Context supports a proposal, which persists Intent and gains authorization before an Effect. An unknown Effect cannot move directly to Verification or Acceptance and must reconcile. A still-unknown branch enters separately authorized compensation or quarantine. Passed Verification still needs an Acceptance-authority decision."
      }
      mobileItems={[
        ...labels,
        zh
          ? "未知分支：OUTCOME_UNKNOWN → 对账 → 补偿 / 隔离"
          : "Unknown branch: OUTCOME_UNKNOWN → Reconcile → compensation / quarantine",
      ]}
      mode={mode}
      fullHref={`${pagePath(locale, "cognitiveos")}#flow-${locale}`}
    >
      <svg
        className="diagram-svg"
        viewBox="0 0 1180 580"
        role="img"
        aria-labelledby={`flow-${locale}-title flow-${locale}-desc`}
      >
        <title id={`flow-${locale}-title`}>
          {zh ? "从 Context 到验收的静态语义链" : "Static semantic chain from Context to Acceptance"}
        </title>
        <desc id={`flow-${locale}-desc`}>
          {zh
            ? "八个阶段从左到右连接，Effect 下方有未知结果分支，通向对账、补偿或隔离。"
            : "Eight stages connect left to right. An unknown-outcome branch below Effect leads to reconciliation, compensation, or quarantine."}
        </desc>
        <path className="diagram-thread" d="M64 176H1114" />
        {labels.map((label, index) => {
          const x = 64 + index * 150;
          return (
            <g className={index >= 5 ? "diagram-node authority" : "diagram-node"} key={label}>
              <rect x={x} y="134" width="126" height="84" rx="2" />
              <text x={x + 63} y="184" textAnchor="middle">
                {label}
              </text>
            </g>
          );
        })}
        <path className="diagram-unknown" d="M664 218V320H810" />
        <g className="diagram-node unresolved">
          <rect x="574" y="298" width="250" height="68" rx="2" />
          <text x="699" y="340" textAnchor="middle">
            OUTCOME_UNKNOWN
          </text>
        </g>
        <path className="diagram-unknown" d="M824 332H964V420" />
        <g className="diagram-node unresolved">
          <rect x="836" y="408" width="256" height="60" rx="2" />
          <text x="964" y="446" textAnchor="middle">
            {zh ? "独立授权补偿 / 隔离" : "SEPARATE COMPENSATION / QUARANTINE"}
          </text>
        </g>
        <path className="diagram-return" d="M810 298V258H814" />
      </svg>
    </DiagramFrame>
  );
}

export function LifecycleDomainsDiagram({
  locale,
  mode = "full",
}: {
  locale: Locale;
  mode?: DiagramMode;
}) {
  const zh = locale === "zh";
  return (
    <DiagramFrame
      id={`lifecycles-${locale}`}
      title={zh ? "五个正交生命周期域" : "Five orthogonal lifecycle domains"}
      caption={
        zh
          ? "每条泳道拥有自己的 authority 与迁移；细线表示证据引用，不是合并状态机。"
          : "Each lane owns its authority and transitions; thin links are evidence references, not a merged machine."
      }
      source={
        zh
          ? "来源：state-domains registry 与五份 transition JSON；状态名逐字复制。"
          : "Source: state-domain registry and five transition JSON files; state names copied exactly."
      }
      longText={
        zh
          ? lifecycleDomains
              .map((domain) => `${domain.id}：${domain.states.join("、")}`)
              .join("。")
          : lifecycleDomains
              .map((domain) => `${domain.id}: ${domain.states.join(", ")}`)
              .join(". ")
      }
      mobileItems={lifecycleDomains.map(
        (domain) => `${domain.id}: ${domain.states.join(" → ")}`,
      )}
      mode={mode}
      fullHref={`${pagePath(locale, "cognitiveos")}#lifecycles-${locale}`}
    >
      <svg
        className="diagram-svg diagram-svg--dense"
        viewBox="0 0 1260 720"
        role="img"
        aria-labelledby={`lifecycles-${locale}-title lifecycles-${locale}-desc`}
      >
        <title id={`lifecycles-${locale}-title`}>
          {zh ? "五个独立状态域" : "Five independent state domains"}
        </title>
        <desc id={`lifecycles-${locale}-desc`}>
          {zh
            ? "五条水平泳道列出各自完整状态集合。"
            : "Five horizontal lanes list their complete state sets."}
        </desc>
        {lifecycleDomains.map((domain, index) => {
          const y = 48 + index * 132;
          const columns = Math.min(domain.states.length, 8);
          return (
            <g className="diagram-lane" key={domain.id}>
              <rect x="30" y={y} width="1200" height="108" rx="2" />
              <text className="diagram-kicker" x="54" y={y + 28}>
                {domain.id.toUpperCase()} · {domain.authority}
              </text>
              {domain.states.map((state, stateIndex) => {
                const row = Math.floor(stateIndex / columns);
                const column = stateIndex % columns;
                const x = 54 + column * 142;
                const stateY = y + 48 + row * 34;
                return (
                  <g className={state.includes("UNKNOWN") ? "unresolved" : ""} key={state}>
                    <circle cx={x + 5} cy={stateY - 5} r="4" />
                    <text x={x + 16} y={stateY}>
                      {state}
                    </text>
                  </g>
                );
              })}
            </g>
          );
        })}
      </svg>
    </DiagramFrame>
  );
}

export function ContextPipelineDiagram({
  locale,
  mode = "full",
}: {
  locale: Locale;
  mode?: DiagramMode;
}) {
  const zh = locale === "zh";
  const standardLabels = zh
    ? [
        "请求准入",
        "治理预过滤",
        "候选检索",
        "逐对象授权重验",
        "语义排序 / 选择",
        "预算适配",
        "损失声明",
        "确定性渲染",
        "ContextView 输出",
      ]
    : [...standardContextStages];

  return (
    <DiagramFrame
      id={`context-pipeline-${locale}`}
      title={zh ? "Context 九阶段：两套词汇" : "Context nine stages: two vocabularies"}
      caption={
        zh
          ? "主轴采用 Context standard 顺序；下轴保留 Core 词汇差异，不静默一一合并。"
          : "The main axis uses Context-standard order; the lower axis preserves Core vocabulary differences instead of silently merging them."
      }
      source={
        zh
          ? "来源：Context Resolution and Cache Standard §2；Core §6.3。"
          : "Source: Context Resolution and Cache Standard §2; Core §6.3."
      }
      longText={
        zh
          ? `标准顺序：${standardLabels.join("，")}。Core 词汇：${coreContextStages.join("，")}。治理预过滤先于候选检索，逐对象授权重验先于排序器看到正文。`
          : `Standard order: ${standardLabels.join(", ")}. Core vocabulary: ${coreContextStages.join(", ")}. Governance pre-filter precedes candidate retrieval, and per-object authorization precedes body access by the ranker.`
      }
      mobileItems={[
        ...standardLabels.map(
          (stage, index) => `${index + 1}. ${stage} · Core: ${coreContextStages[index]}`,
        ),
      ]}
      mode={mode}
      fullHref={`${pagePath(locale, "cognitiveos")}#context-pipeline-${locale}`}
    >
      <svg
        className="diagram-svg diagram-svg--dense"
        viewBox="0 0 1260 640"
        role="img"
        aria-labelledby={`context-pipeline-${locale}-title context-pipeline-${locale}-desc`}
      >
        <title id={`context-pipeline-${locale}-title`}>
          {zh ? "Context standard 与 Core 的九阶段对照" : "Nine-stage comparison between the Context standard and Core"}
        </title>
        <desc id={`context-pipeline-${locale}-desc`}>
          {zh
            ? "上方是标准实现顺序，下方是 Core 概念词汇，中间以非一一映射连线连接。"
            : "The standard implementation order appears above the Core conceptual vocabulary with non-equivalent mapping lines between them."}
        </desc>
        <text className="diagram-kicker" x="48" y="52">
          {zh ? "CONTEXT STANDARD · 实现顺序" : "CONTEXT STANDARD · IMPLEMENTATION ORDER"}
        </text>
        <text className="diagram-kicker" x="48" y="398">
          {zh ? "CORE · 概念词汇（不等价别名）" : "CORE · CONCEPTUAL VOCABULARY (NOT ALIASES)"}
        </text>
        {standardLabels.map((stage, index) => {
          const column = index % 5;
          const row = Math.floor(index / 5);
          const x = 48 + column * 238;
          const y = 78 + row * 138;
          const coreX = 48 + index * 130;
          return (
            <g key={stage}>
              <g className={index === 1 || index === 3 ? "diagram-node authority" : "diagram-node"}>
                <rect x={x} y={y} width="208" height="82" rx="2" />
                <text x={x + 104} y={y + 34} textAnchor="middle">
                  {String(index + 1).padStart(2, "0")}
                </text>
                <text x={x + 104} y={y + 60} textAnchor="middle">
                  {stage}
                </text>
              </g>
              <g className="diagram-core-stage">
                <rect x={coreX} y="438" width="112" height="58" rx="2" />
                <text x={coreX + 56} y="474" textAnchor="middle">
                  {coreContextStages[index]}
                </text>
              </g>
            </g>
          );
        })}
        <path className="diagram-discrepancy" d="M512 512H1150" />
        <text className="diagram-kicker unresolved-text" x="1150" y="544" textAnchor="end">
          {zh ? "词汇存在差异" : "VOCABULARY DIFFERS"}
        </text>
      </svg>
    </DiagramFrame>
  );
}

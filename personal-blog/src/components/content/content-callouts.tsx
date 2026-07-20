import { cognitiveOsSnapshot } from "@/data/cognitiveos";

export function SampleNotice() {
  return (
    <aside className="sample-notice" aria-label="示例内容 / Sample content">
      <strong>
        <span className="locale-copy locale-copy--zh">示例内容</span>
        <span className="locale-copy locale-copy--en">Sample content</span>
      </strong>
      <p>
        <span className="locale-copy locale-copy--zh">
          仅用于展示信息结构，不代表真实经历、客户或结果。
        </span>
        <span className="locale-copy locale-copy--en">
          For information-architecture demonstration only; not a real role,
          client, or outcome.
        </span>
      </p>
    </aside>
  );
}

export function ArticleSnapshot() {
  const snapshot = cognitiveOsSnapshot;
  return (
    <aside className="article-snapshot" aria-label="CognitiveOS research snapshot">
      <div>
        <span>RESEARCH SNAPSHOT</span>
        <strong>{snapshot.commit.slice(0, 7)} · M1 in-progress</strong>
      </div>
      <dl>
        <div>
          <dt>specified</dt>
          <dd>{snapshot.requirementsSpecified}</dd>
        </div>
        <div>
          <dt>implementation-provided</dt>
          <dd>{snapshot.implementationProvidedRequirements}</dd>
        </div>
        <div>
          <dt>behavior-executed</dt>
          <dd>{snapshot.behaviorExecuted}</dd>
        </div>
        <div>
          <dt>conformant profiles</dt>
          <dd>{snapshot.conformantProfiles}</dd>
        </div>
        <div>
          <dt>vectors not-run</dt>
          <dd>{snapshot.vectorsNotRun}</dd>
        </div>
      </dl>
      <p>
        <span className="locale-copy locale-copy--zh">
          Lane-CTR 契约批已交付并新增两份 F-003 负例，但 REQ
          级实现声明与行为向量执行仍均为 0；快照计数不构成符合性证据。
        </span>
        <span className="locale-copy locale-copy--en">
          The Lane-CTR contract batch is delivered, including two new F-003
          negative vectors, but there are still zero REQ-level implementation
          claims and zero behavior-executed vectors. Snapshot counts are not
          conformance evidence.
        </span>
      </p>
    </aside>
  );
}

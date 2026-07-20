import { useId, type ComponentPropsWithoutRef } from "react";

export function CodeBlock({
  children,
  ...props
}: ComponentPropsWithoutRef<"pre">) {
  const labelId = useId();
  return (
    <div className="code-block">
      <button
        type="button"
        className="code-block__copy"
        data-copy-code
        disabled
      >
        <span aria-live="polite">
          <span className="locale-copy locale-copy--zh" data-copy-label-zh>
            复制
          </span>
          <span className="locale-copy locale-copy--en" data-copy-label-en>
            Copy
          </span>
        </span>
      </button>
      <span id={labelId} className="sr-only">
        <span className="locale-copy locale-copy--zh">可横向滚动的代码块</span>
        <span className="locale-copy locale-copy--en">Scrollable code block</span>
      </span>
      <pre {...props} tabIndex={0} aria-labelledby={labelId}>
        {children}
      </pre>
    </div>
  );
}

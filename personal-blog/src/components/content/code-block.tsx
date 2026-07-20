"use client";

import {
  useEffect,
  useRef,
  useState,
  type ComponentPropsWithoutRef,
} from "react";

type CopyState = "idle" | "copied" | "error";

export function CodeBlock({
  children,
  ...props
}: ComponentPropsWithoutRef<"pre">) {
  const [copyState, setCopyState] = useState<CopyState>("idle");
  const preRef = useRef<HTMLPreElement>(null);
  const resetTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(
    () => () => {
      if (resetTimer.current) {
        clearTimeout(resetTimer.current);
      }
    },
    [],
  );

  const copy = async () => {
    const source = preRef.current?.textContent || "";
    try {
      await navigator.clipboard.writeText(source);
      setCopyState("copied");
    } catch {
      setCopyState("error");
    }

    if (resetTimer.current) {
      clearTimeout(resetTimer.current);
    }
    resetTimer.current = setTimeout(() => setCopyState("idle"), 2200);
  };

  return (
    <div className="code-block">
      <button
        type="button"
        className="code-block__copy"
        onClick={copy}
        aria-label="复制代码 / Copy code"
      >
        <span aria-live="polite">
          <span className="locale-copy locale-copy--zh">
            {copyState === "copied"
              ? "已复制"
              : copyState === "error"
                ? "复制失败"
                : "复制"}
          </span>
          <span className="locale-copy locale-copy--en">
            {copyState === "copied"
              ? "Copied"
              : copyState === "error"
                ? "Copy failed"
                : "Copy"}
          </span>
        </span>
      </button>
      <pre
        {...props}
        ref={preRef}
        tabIndex={0}
        aria-label="Scrollable code block / 可滚动代码块"
      >
        {children}
      </pre>
    </div>
  );
}

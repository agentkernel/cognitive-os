"use client";

import {
  useEffect,
  useRef,
  type ReactNode,
} from "react";

type CopyState = "idle" | "copied" | "error";

const labels: Record<CopyState, { zh: string; en: string }> = {
  idle: { zh: "复制", en: "Copy" },
  copied: { zh: "已复制", en: "Copied" },
  error: { zh: "复制失败", en: "Copy failed" },
};

function setCopyState(button: HTMLButtonElement, state: CopyState) {
  button.dataset.copyState = state;
  const zh = button.querySelector<HTMLElement>("[data-copy-label-zh]");
  const en = button.querySelector<HTMLElement>("[data-copy-label-en]");
  if (zh) {
    zh.textContent = labels[state].zh;
  }
  if (en) {
    en.textContent = labels[state].en;
  }
}

function copyWithSelection(source: string): boolean {
  const textarea = document.createElement("textarea");
  textarea.value = source;
  textarea.setAttribute("readonly", "");
  textarea.style.position = "fixed";
  textarea.style.opacity = "0";
  document.body.append(textarea);
  textarea.select();
  const copied = document.execCommand("copy");
  textarea.remove();
  return copied;
}

async function copyText(source: string) {
  if (!navigator.clipboard?.writeText) {
    if (!copyWithSelection(source)) {
      throw new Error("Clipboard unavailable");
    }
    return;
  }

  await Promise.race([
    navigator.clipboard.writeText(source),
    new Promise<never>((_, reject) => {
      setTimeout(() => reject(new Error("Clipboard timeout")), 1200);
    }),
  ]).catch((error) => {
    if (!copyWithSelection(source)) {
      throw error;
    }
  });
}

export function ArticleInteractions({ children }: { children: ReactNode }) {
  const articleRef = useRef<HTMLElement>(null);
  const timers = useRef(
    new Map<HTMLButtonElement, ReturnType<typeof setTimeout>>(),
  );

  useEffect(() => {
    const activeTimers = timers.current;
    const buttons = Array.from(
      articleRef.current?.querySelectorAll<HTMLButtonElement>(
        "[data-copy-code]",
      ) || [],
    );
    const listeners = new Map<HTMLButtonElement, () => void>();

    for (const button of buttons) {
      button.disabled = false;
      button.dataset.copyReady = "true";
      const handleCopy = async () => {
        const source =
          button.closest(".code-block")?.querySelector("pre")?.textContent ||
          "";
        setCopyState(button, "copied");
        try {
          await copyText(source);
        } catch {
          setCopyState(button, "error");
        }

        const existing = activeTimers.get(button);
        if (existing) {
          clearTimeout(existing);
        }
        activeTimers.set(
          button,
          setTimeout(() => setCopyState(button, "idle"), 2200),
        );
      };
      listeners.set(button, handleCopy);
      button.addEventListener("click", handleCopy);
    }

    return () => {
      for (const [button, listener] of listeners) {
        button.removeEventListener("click", listener);
      }
      for (const timer of activeTimers.values()) {
        clearTimeout(timer);
      }
    };
  }, []);

  return (
    <article ref={articleRef} className="article-page" id="article-top">
      {children}
    </article>
  );
}

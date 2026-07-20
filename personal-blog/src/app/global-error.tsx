"use client";

import Link from "next/link";
import "@/styles/globals.css";

export default function GlobalError({ reset }: { error: Error; reset: () => void }) {
  return (
    <html lang="en">
      <body>
        <main className="error-panel" role="alert">
          <div>
            <p className="eyebrow">
              <span lang="en">GLOBAL RENDER ERROR</span>
              <span aria-hidden="true"> / </span>
              <span lang="zh-CN">全局渲染错误</span>
            </p>
            <h1>
              <span lang="en">The page could not render</span>
              <span aria-hidden="true"> / </span>
              <span lang="zh-CN">页面无法渲染</span>
            </h1>
            <p>
              <span lang="en">
                Retry the render. If it fails again, return to a localized home
                page.
              </span>
              <span aria-hidden="true"> / </span>
              <span lang="zh-CN">请重试；若仍失败，请返回对应语言首页。</span>
            </p>
            <div className="error-panel__actions">
              <button type="button" onClick={reset}>
                <span lang="en">Try again</span>
                <span aria-hidden="true"> / </span>
                <span lang="zh-CN">重试</span>
              </button>
              <Link href="/zh">中文首页</Link>
              <Link href="/en">English home</Link>
            </div>
          </div>
        </main>
      </body>
    </html>
  );
}

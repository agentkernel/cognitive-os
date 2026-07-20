"use client";

import Link from "next/link";
import "@/styles/globals.css";

export default function GlobalError({ reset }: { error: Error; reset: () => void }) {
  return (
    <html lang="en">
      <body>
        <main className="error-panel" role="alert">
          <div>
            <p className="eyebrow">GLOBAL RENDER ERROR / 全局渲染错误</p>
            <h1>The page could not render / 页面无法渲染</h1>
            <p>
              Retry the render. If it fails again, return to a localized home page. /
              请重试；若仍失败，请返回对应语言首页。
            </p>
            <div className="error-panel__actions">
              <button type="button" onClick={reset}>
                Try again / 重试
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

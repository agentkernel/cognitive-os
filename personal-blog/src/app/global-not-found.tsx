import Link from "next/link";
import "@/styles/globals.css";

export default function GlobalNotFound() {
  return (
    <html lang="en">
      <body>
        <main className="not-found">
          <p className="eyebrow">
            404 / <span lang="en">ROUTE NOT FOUND</span>
            <span aria-hidden="true"> / </span>
            <span lang="zh-CN">路由未找到</span>
          </p>
          <h1>
            <span lang="en">Page not found</span>
            <span aria-hidden="true"> / </span>
            <span lang="zh-CN">没有找到这页</span>
          </h1>
          <p>
            <span lang="en">Choose a localized home page.</span>
            <span aria-hidden="true"> / </span>
            <span lang="zh-CN">请选择对应语言首页。</span>
          </p>
          <div className="not-found__links">
            <Link href="/zh">中文首页</Link>
            <Link href="/en">English home</Link>
          </div>
        </main>
      </body>
    </html>
  );
}

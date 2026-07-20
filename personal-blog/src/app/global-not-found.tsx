import Link from "next/link";
import "@/styles/globals.css";

export default function GlobalNotFound() {
  return (
    <html lang="en">
      <body>
        <main className="not-found">
          <p className="eyebrow">404 / ROUTE NOT FOUND / 路由未找到</p>
          <h1>Page not found / 没有找到这页</h1>
          <p>
            Choose a localized home page. / 请选择对应语言首页。
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

import Link from "next/link";

export default function LocalizedNotFound() {
  return (
    <main id="main-content" className="not-found">
      <section className="not-found__zh" lang="zh-CN">
        <p className="eyebrow">404 / 路由未登记</p>
        <h1>没有找到这页</h1>
        <p>地址可能无效，或对应语言的内容尚未发布。</p>
        <div className="not-found__links">
          <Link href="/zh">返回中文首页</Link>
          <Link href="/en">English home</Link>
        </div>
      </section>
      <section className="not-found__en" lang="en">
        <p className="eyebrow">404 / UNREGISTERED ROUTE</p>
        <h1>This page was not found</h1>
        <p>The address may be invalid, or its translation has not been published.</p>
        <div className="not-found__links">
          <Link href="/en">Return to the English home</Link>
          <Link href="/zh">中文首页</Link>
        </div>
      </section>
    </main>
  );
}

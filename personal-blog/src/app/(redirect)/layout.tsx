import type { ReactNode } from "react";
import "@/styles/globals.css";

export default function RedirectLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="zh-CN">
      <body>{children}</body>
    </html>
  );
}

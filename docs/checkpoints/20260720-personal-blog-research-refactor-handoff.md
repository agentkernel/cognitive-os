# 20260720 Personal Blog Research Refactor Handoff

## 1. 本次会话完成

- 将 `personal-blog/` 从未署名作品集原型重构为以 CognitiveOS 为核心、作者低调存在的双语独立研究刊物。
- 主旅程收敛为 Home → Research → 完整设计说明 / Sources；Essays 只展示真实研究，7 组双语结构样例全部降级到 noindex Lab；旧 Projects 索引永久跳转 Lab。
- 新增 `/{locale}/cognitiveos/sources`，公开两份 18-fact sourcebook、来源层级、文件 hash、开放差异和公开措辞护栏。
- 将内容层拆为 metadata-only manifest、静态 per-entry MDX loader 和统一 publication/path helper；RSS、sitemap、metadata、hreflang、Lab 隔离与跨语言路由共用一套规则。
- 将 2,286 行全局样式拆为 tokens/base/shell/content/diagrams 五层；落地 Asymmetric Evidence Notebook、唯一 Governed Flow Thread signature、流式字阶、可读图表摘要与宽版视觉地图。
- 修复正文列表/链接 affordance、移动 drawer containing-block/focus/inert/短屏滚动、代码复制 hydration、深色表面 focus、多语言 accessible name、中文字体负载与静态安全头。
- 本批是隔离研究发布/展示层结构型重构，不修改 registry/schema/transition/vector、核心实现、F/IMP 或 Profile；无适用 CognitiveOS REQ-ID，未读取 `History/`。
- 博客代码提交哈希：`7fd473b`。

## 2. 未完成 / 进行中

- 未配置真实 HTTPS `NEXT_PUBLIC_SITE_URL`，因此全站继续 `noindex` 且 robots disallow。
- 未选择公开 code/content license；当前仍 `UNLICENSED`。
- 未执行 NVDA/Firefox、200–400% 缩放、中文/英文人审与真实辅助技术验证。
- 根 CI 尚未纳入隔离博客；`.github/workflows/` 属于 Lane-CFR，本批未跨车道修改。
- 未部署。

## 3. 测试与证据状态

- `pnpm verify`：通过（完整 check + production Playwright）。
- ESLint：通过，无 warning；strict `tsc --noEmit`：通过。
- Vitest：2 files，14/14 tests 通过。
- 内容/边界：4 对文章、4 对项目、每份 sourcebook 18 facts；16 个静态 MDX loader；4 个 intentional Client Components。
- Next.js production build：通过，38 个 static/SSG 页面。
- Playwright Chromium：22/22 通过；覆盖 375/768/1024/1440、键盘/焦点、复制、RSS/sitemap/robots、安全头、reduced-motion、forced-colors 和无溢出。
- axe：Home、Essays、Research、Sources、Method、Lab、flagship、sample detail 全模板 WCAG 2.0/2.1/2.2 A/AA 自动扫描通过。
- 截图和 Playwright 产物位于 `personal-blog/artifacts/evidence/`（gitignored）；它们是本地执行证据，不是 Profile 符合证据。
- 根 `pnpm run check:consistency`：通过（273 requirements、55 error codes、56 schemas、76 vectors，链接与追踪一致性通过）。
- conformance vectors：仍为 76 个 `not-run`，无状态变化。

## 4. 未决风险与漂移

- 未发现或登记新的 normative 漂移；研究快照继续固定在 `b626e88`。
- 公开发布仍受域名、许可、图像服务条款和人工无障碍验证阻断。
- Next 16.2.10 的间接 PostCSS moderate advisory 在当前无不可信 CSS 摄入路径下可利用性低，仍应随安全补丁升级。
- `globalNotFound` 仍为实验性 Next 约定，框架升级时需复核。
- `.cursor/skills/*` 与 `.cursor/rules/use-skills.mdc` 是会话开始前已有并行改动，不得混入博客提交。

## 5. 下一步入口

- 产品入口：`personal-blog/src/app/[locale]/page.tsx`
- 研究入口：`personal-blog/src/app/[locale]/cognitiveos/page.tsx`
- 来源入口：`personal-blog/src/app/[locale]/cognitiveos/sources/page.tsx`
- 发布合同：`personal-blog/src/lib/content/publication.ts`、`manifest.ts`、`loaders.ts`
- 完整验证：在 `personal-blog/` 执行 `pnpm verify`
- 第一个发布动作：确定 HTTPS 域名和许可证，然后执行 `tests/MANUAL-ACCESSIBILITY.md`。
- 工作分支：`main`（本地；不得直接 push，按仓库 PR 流程交付）。

## 6. 快照

- PROGRESS 已更新：是。
- 规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合四类状态：核心计数无变化。
- 本次提交列表：`7fd473b`（博客实现、测试与子工程文档）；本 handoff 与 PROGRESS 由随后的文档收尾提交承载。

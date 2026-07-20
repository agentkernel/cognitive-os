import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";

async function expectNoPageOverflow(page: Page) {
  const overflow = await page.evaluate(() => ({
    scrollWidth: document.documentElement.scrollWidth,
    clientWidth: document.documentElement.clientWidth,
  }));
  expect(overflow.scrollWidth).toBeLessThanOrEqual(overflow.clientWidth + 1);
}

test("root uses a permanent explicit redirect", async ({ request }) => {
  const response = await request.get("/", { maxRedirects: 0 });
  expect(response.status()).toBe(308);
  expect(response.headers().location).toBe("/zh");
});

test("primary journey presents CognitiveOS research without sample material", async ({
  page,
}) => {
  await page.goto("/zh");
  await expect(
    page.getByRole("heading", {
      level: 1,
      name: "Agent 可以提出完成，但不能自行决定完成。",
    }),
  ).toBeVisible();
  await expect(page.getByText("示例内容", { exact: true })).toHaveCount(0);
  await expect(
    page.getByRole("navigation", { name: "主导航" }).getByRole("link", {
      name: "项目",
    }),
  ).toHaveCount(0);
  await expect(page.getByRole("contentinfo").getByRole("link", { name: "Lab" })).toHaveAttribute(
    "href",
    "/zh/lab",
  );
});

test("lab contains samples but stays outside publication surfaces", async ({
  page,
  request,
}) => {
  await page.goto("/en/lab");
  await expect(page.getByRole("heading", { level: 1, name: "Lab" })).toBeVisible();
  await expect(page.getByText("Sample content", { exact: true }).first()).toBeVisible();
  await expect(page.locator('meta[name="robots"]')).toHaveAttribute(
    "content",
    /noindex.*follow/,
  );

  const sitemap = await (await request.get("/sitemap.xml")).text();
  const rss = await (await request.get("/en/rss.xml")).text();
  expect(sitemap).not.toContain("/lab");
  expect(sitemap).not.toContain("/projects/");
  expect(rss).not.toContain("designing-failure-semantics");
});

test("keyboard navigation reaches the skip link and primary route", async ({ page }) => {
  await page.goto("/zh");
  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: "跳到正文" })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.locator("#main-content")).toBeFocused();

  await page
    .getByRole("navigation", { name: "主导航" })
    .getByRole("link", { name: "Essays" })
    .focus();
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\/zh\/articles$/);
});

test("homepage carries the compact governed-flow signature", async ({ page }) => {
  await page.goto("/en");
  const compactFlow = page.locator(".governed-thread--compact");
  await expect(compactFlow).toBeVisible();
  await expect(compactFlow.locator("ol > li")).toHaveCount(8);
  await expect(compactFlow).toContainText("Never live progress");
  await expect(compactFlow).toContainText("OUTCOME_UNKNOWN");
  await expectNoPageOverflow(page);
});

test("language switching uses an equivalent registry route and shared anchors", async ({
  page,
}) => {
  await page.goto("/zh/cognitiveos/verifiable-agent-actions#intent-effect");
  const languageSwitch = page.getByRole("link", { name: "Read in English" }).first();
  await expect(languageSwitch).toHaveAttribute(
    "href",
    "/en/cognitiveos/verifiable-agent-actions",
  );
  await languageSwitch.click();
  await expect(page).toHaveURL(/\/en\/cognitiveos\/verifiable-agent-actions$/);
  await expect(
    page.getByRole("link", {
      name: "Open Intent, Effect, and reconciliation in the Chinese article",
      exact: true,
    }),
  ).toHaveAttribute(
    "href",
    "/zh/cognitiveos/verifiable-agent-actions#intent-effect",
  );
});

test("flagship renders complete governance semantics and five diagrams", async ({ page }) => {
  await page.goto("/en/cognitiveos/verifiable-agent-actions");
  await expect(
    page.getByRole("heading", {
      level: 1,
      name: "Making Agent Actions Verifiable: The Deterministic Kernel Design of CognitiveOS",
    }),
  ).toBeVisible();
  await expect(page.getByText("CANDIDATE_COMPLETE", { exact: false }).first()).toBeVisible();
  await expect(page.getByText("OUTCOME_UNKNOWN", { exact: false }).first()).toBeVisible();
  await expect(page.locator("figure.semantic-diagram")).toHaveCount(5);
  await expect(
    page.getByText("Never live progress", { exact: true }),
  ).toBeVisible();
  await expect(page.locator(".article-snapshot")).toContainText("b626e88");
  await expect(page.locator(".article-snapshot")).toContainText("76");
  await expect(page.locator(".article-snapshot")).toContainText(
    "zero REQ-level implementation claims",
  );
  await expectNoPageOverflow(page);
});

test("CognitiveOS uses a responsive manual-style sidebar", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/zh/cognitiveos");
  const manual = page.getByRole("navigation", { name: "CognitiveOS 说明书目录" });
  await expect(manual).toBeVisible();
  await expect(manual.getByRole("link", { name: "专题总览" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(
    manual.getByRole("link", { name: "Intent、Effect 与对账" }),
  ).toHaveAttribute(
    "href",
    "/zh/cognitiveos/verifiable-agent-actions#intent-effect",
  );

  await page.goto("/zh/cognitiveos/verifiable-agent-actions");
  await expect(
    page
      .getByRole("navigation", { name: "CognitiveOS 说明书目录" })
      .getByRole("link", { name: "完整设计说明" }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("navigation", { name: "文章目录" }),
  ).toBeVisible();

  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("/zh/cognitiveos");
  const compactManual = page.locator("details.manual-sidebar-mobile");
  await expect(compactManual).toBeVisible();
  await compactManual.locator("summary").click();
  await expect(
    compactManual.getByRole("link", { name: "完整设计说明" }),
  ).toBeVisible();
  await expectNoPageOverflow(page);

  await page.goto("/zh/cognitiveos/verifiable-agent-actions");
  const mobileColumns = await page.locator(".article-grid > *").evaluateAll(
    (elements) =>
      elements.map((element) => {
        const rect = element.getBoundingClientRect();
        return { left: rect.left, right: rect.right, width: rect.width };
      }),
  );
  for (const column of mobileColumns) {
    expect(column.left).toBeGreaterThanOrEqual(0);
    expect(column.right).toBeLessThanOrEqual(376);
    expect(column.width).toBeGreaterThan(300);
  }
});

test("research visual and licensed reading fonts are served locally", async ({
  page,
}) => {
  await page.goto("/en/cognitiveos/verifiable-agent-actions");
  await expect(
    page.locator('img[src*="governed-trace-hero"]').first(),
  ).toBeVisible();

  const localAssets = await page.evaluate(async () => {
    await document.fonts.ready;
    const resources = performance
      .getEntriesByType("resource")
      .map((entry) => entry.name);
    return {
      bodyFont: getComputedStyle(document.body).fontFamily,
      uiFont: getComputedStyle(document.querySelector(".site-mark")!).fontFamily,
      fontRequests: resources.filter((url) => /\.woff2(?:\?|$)/.test(url)),
      externalFontRequests: resources.filter((url) =>
        /fonts\.(?:googleapis|gstatic)\.com/.test(url),
      ),
    };
  });
  expect(localAssets.bodyFont).toContain("Source Serif 4 Variable");
  expect(localAssets.uiFont).toContain("Recursive Variable");
  expect(localAssets.fontRequests.length).toBeGreaterThan(0);
  expect(localAssets.externalFontRequests).toEqual([]);
});

test("mobile navigation traps focus and closes with Escape", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("/en");
  const trigger = page.getByRole("button", { name: "Open navigation" });
  await trigger.click();
  await expect(page.getByRole("dialog", { name: "Mobile navigation" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Close navigation" }).last()).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog", { name: "Mobile navigation" })).toHaveCount(0);
  await expect(trigger).toBeFocused();
  await expectNoPageOverflow(page);
});

test("every mobile navigation close path returns focus and keeps content reachable", async ({
  page,
}) => {
  await page.setViewportSize({ width: 375, height: 520 });
  await page.goto("/en");
  const trigger = page.getByRole("button", { name: "Open navigation" });

  await trigger.click();
  await page.getByRole("button", { name: "Close navigation" }).last().click();
  await expect(trigger).toBeFocused();

  await trigger.click();
  const hitTarget = await page.evaluate(() => {
    const element = document.elementFromPoint(340, 260) as HTMLElement | null;
    return {
      tag: element?.tagName,
      className: element?.className,
      parentClassName: element?.parentElement?.className,
    };
  });
  expect(hitTarget).toEqual({
    tag: "BUTTON",
    className: "mobile-navigation__backdrop",
    parentClassName: "mobile-navigation__layer",
  });
  await page.mouse.click(340, 260);
  await expect(trigger).toBeFocused();

  await trigger.click();
  const panel = page.getByRole("dialog", { name: "Mobile navigation" });
  await expect(panel).toHaveCSS("overflow-y", "auto");
});

test("prose restores list markers, persistent links, and readable diagram fallbacks", async ({
  page,
}) => {
  await page.setViewportSize({ width: 768, height: 900 });
  await page.goto("/en/cognitiveos/verifiable-agent-actions");

  const orderedList = page.locator(".prose > ol").first();
  await expect(orderedList).toHaveCSS("list-style-type", "decimal");

  const proseLink = page.locator(".prose a").first();
  if ((await proseLink.count()) > 0) {
    await expect(proseLink).not.toHaveCSS("text-decoration-line", "none");
  }

  await expect(page.locator(".semantic-diagram .diagram-canvas").first()).toBeHidden();
  await expect(page.locator(".semantic-diagram .diagram-mobile-summary").first()).toBeVisible();
});

test("responsive screenshots stay within the page viewport", async ({ page }) => {
  for (const width of [375, 768, 1024, 1440]) {
    await page.setViewportSize({ width, height: width === 375 ? 812 : 900 });
    await page.goto("/zh");
    await expectNoPageOverflow(page);
    await page.screenshot({
      path: `artifacts/evidence/screenshots/home-${width}.png`,
      fullPage: true,
    });
  }
  for (const width of [375, 1440]) {
    await page.setViewportSize({ width, height: width === 375 ? 812 : 900 });
    await page.goto("/zh/cognitiveos/verifiable-agent-actions");
    await expectNoPageOverflow(page);
    await page.screenshot({
      path: `artifacts/evidence/screenshots/article-${width}.png`,
      fullPage: true,
    });
    await page.goto("/zh/cognitiveos");
    await expectNoPageOverflow(page);
    await page.screenshot({
      path: `artifacts/evidence/screenshots/cognitiveos-overview-${width}.png`,
      fullPage: true,
    });
  }
});

test("long-form typography keeps a comfortable measure and rhythm", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/en/cognitiveos/verifiable-agent-actions");
  const readingMetrics = await page.locator(".prose").evaluate((element) => {
    const paragraph = element.querySelector(":scope > p")!;
    const style = getComputedStyle(paragraph);
    const fontSize = Number.parseFloat(style.fontSize);
    return {
      width: element.getBoundingClientRect().width,
      fontSize,
      lineHeightRatio: Number.parseFloat(style.lineHeight) / fontSize,
    };
  });
  expect(readingMetrics.width).toBeLessThanOrEqual(740);
  expect(readingMetrics.fontSize).toBeGreaterThanOrEqual(18);
  expect(readingMetrics.lineHeightRatio).toBeGreaterThanOrEqual(1.65);
  await expect(
    page.getByRole("link", {
      name: "Intent, Effect, and reconciliation",
      exact: true,
    }),
  ).toBeVisible();
  const articleNavigation = page.getByRole("navigation", {
    name: "Article navigation",
  });
  await expect(
    articleNavigation.getByRole("link", { name: "Back to Research" }),
  ).toBeVisible();
  await expect(
    articleNavigation.getByRole("link", { name: "Back to top" }),
  ).toHaveAttribute("href", "#article-top");
});

test("public routes have no console or page errors", async ({ page }) => {
  const errors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") {
      errors.push(`console: ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => errors.push(`page: ${error.message}`));

  for (const path of [
    "/zh",
    "/en/articles",
    "/zh/projects",
    "/en/cognitiveos",
    "/zh/about",
    "/en/cognitiveos/verifiable-agent-actions",
  ]) {
    await page.goto(path);
    await expectNoPageOverflow(page);
  }
  expect(errors).toEqual([]);
});

test("reduced motion removes meaningful transition duration", async ({ browser }) => {
  const context = await browser.newContext({ reducedMotion: "reduce" });
  const page = await context.newPage();
  await page.goto("/en");
  const duration = await page.locator(".primary-action").evaluate((element) => {
    const style = getComputedStyle(element);
    const toMilliseconds = (value: string) => {
      const first = value.split(",")[0].trim();
      return first.endsWith("ms") ? Number.parseFloat(first) : Number.parseFloat(first) * 1000;
    };
    return {
      animationMilliseconds: toMilliseconds(style.animationDuration),
      transitionMilliseconds: toMilliseconds(style.transitionDuration),
      scrollBehavior: getComputedStyle(document.documentElement).scrollBehavior,
    };
  });
  expect(duration.animationMilliseconds).toBeLessThanOrEqual(0.1);
  expect(duration.transitionMilliseconds).toBeLessThanOrEqual(0.1);
  expect(duration.scrollBehavior).toBe("auto");
  await context.close();
});

test("forced-colors mode preserves a visible focus indicator", async ({ page }) => {
  await page.emulateMedia({ forcedColors: "active" });
  await page.goto("/en");
  const primaryLink = page.getByRole("link", {
    name: "Read the full design guide",
  });
  await primaryLink.focus();
  const focusStyle = await primaryLink.evaluate((element) => {
    const style = getComputedStyle(element);
    return {
      outlineStyle: style.outlineStyle,
      outlineWidth: style.outlineWidth,
    };
  });
  expect(focusStyle.outlineStyle).not.toBe("none");
  expect(Number.parseFloat(focusStyle.outlineWidth)).toBeGreaterThan(0);
});

test("RSS, sitemap, and robots expose only safe publication surfaces", async ({
  request,
}) => {
  const rss = await request.get("/en/rss.xml");
  expect(rss.ok()).toBe(true);
  expect(rss.headers()["content-type"]).toContain("application/rss+xml");
  const rssBody = await rss.text();
  expect(rssBody).toContain("Making Agent Actions Verifiable");
  expect(rssBody).not.toContain("sample");

  const sitemap = await request.get("/sitemap.xml");
  expect(sitemap.ok()).toBe(true);
  const sitemapBody = await sitemap.text();
  expect(sitemapBody).toContain("/en/cognitiveos/verifiable-agent-actions");
  expect(sitemapBody).not.toContain("designing-failure-semantics");
  expect(sitemapBody).not.toContain("/projects/");

  const robots = await request.get("/robots.txt");
  expect(robots.ok()).toBe(true);
  expect(await robots.text()).toContain("Disallow: /");
});

test("production responses include the static security baseline", async ({
  request,
}) => {
  const response = await request.get("/en");
  expect(response.headers()["x-content-type-options"]).toBe("nosniff");
  expect(response.headers()["x-frame-options"]).toBe("DENY");
  expect(response.headers()["referrer-policy"]).toBe(
    "strict-origin-when-cross-origin",
  );
  expect(response.headers()["permissions-policy"]).toContain("camera=()");
  expect(response.headers()["content-security-policy"]).toContain(
    "frame-ancestors 'none'",
  );
});

test("sample pages are noindex and invalid routes are 404", async ({ page, request }) => {
  await page.goto("/en/articles/designing-failure-semantics");
  await expect(page.locator('meta[name="robots"]')).toHaveAttribute(
    "content",
    /noindex.*follow/,
  );
  expect((await request.get("/en/articles/not-registered")).status()).toBe(404);
  expect((await request.get("/fr")).status()).toBe(404);
});

test("tables, code, and footnotes stay contained on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.context().grantPermissions(["clipboard-read", "clipboard-write"], {
    origin: "http://127.0.0.1:3101",
  });
  await page.goto("/en/articles/testing-bilingual-content");
  const mobileToc = page.locator("details.mobile-article-toc");
  await expect(mobileToc).toBeVisible();
  await mobileToc.locator("summary").click();
  await expect(
    mobileToc.getByRole("link", { name: "Translation identity", exact: true }),
  ).toBeVisible();
  await expect(page.locator(".table-scroll")).toBeVisible();
  await expect(page.locator("pre[tabindex='0']")).toBeVisible();
  const copyButton = page.locator("[data-copy-code]").first();
  await expect(copyButton).toHaveAccessibleName("Copy");
  await expect(copyButton).toHaveAttribute("data-copy-ready", "true");
  await copyButton.click();
  await expect(copyButton).toHaveAttribute("data-copy-state", "copied");
  await expect(copyButton).toContainText("Copied");
  await expect(page.locator("[data-footnotes='true']")).toBeVisible();
  await expect(page.locator("[data-footnote-backref]")).toBeVisible();
  await expectNoPageOverflow(page);
});

test("all publication templates pass automated WCAG A and AA checks", async ({
  page,
}) => {
  for (const path of [
    "/en",
    "/en/articles",
    "/en/cognitiveos",
    "/en/cognitiveos/sources",
    "/en/about",
    "/en/lab",
    "/en/cognitiveos/verifiable-agent-actions",
    "/en/articles/testing-bilingual-content",
  ]) {
    await page.goto(path);
    const results = await new AxeBuilder({ page })
      .withTags([
        "wcag2a",
        "wcag2aa",
        "wcag21a",
        "wcag21aa",
        "wcag22aa",
      ])
      .analyze();
    expect(results.violations, path).toEqual([]);
  }
});

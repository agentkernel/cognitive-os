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

test("keyboard navigation reaches the skip link and primary route", async ({ page }) => {
  await page.goto("/zh");
  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: "跳到正文" })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.locator("#main-content")).toBeFocused();

  await page.getByRole("navigation", { name: "主导航" }).getByRole("link", { name: "文章" }).focus();
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\/zh\/articles$/);
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
    page.getByRole("link", { name: "Open intent-effect in the Chinese article" }),
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
  await expect(page.getByText("Never live progress")).toBeVisible();
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

test("generated visuals and licensed fonts are served locally", async ({ page }) => {
  await page.goto("/zh");
  await expect(page.locator('img[src*="governed-trace-hero"]').first()).toBeVisible();
  await page.goto("/en/cognitiveos");
  await expect(
    page.locator('img[src*="orthogonal-lifecycles-hero"]').first(),
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

test("responsive screenshots stay within the page viewport", async ({ page }) => {
  for (const width of [375, 768, 1440]) {
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
    page.getByRole("link", { name: "Intent, Effect, and reconciliation" }),
  ).toBeVisible();
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
  const duration = await page.locator(".primary-link").evaluate((element) => {
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
    name: "Read the CognitiveOS flagship",
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
  await page.goto("/en/articles/testing-bilingual-content");
  await expect(page.locator(".table-scroll")).toBeVisible();
  await expect(page.locator("pre[tabindex='0']")).toBeVisible();
  await expect(page.locator("[data-footnotes='true']")).toBeVisible();
  await expect(page.locator("[data-footnote-backref]")).toBeVisible();
  await expectNoPageOverflow(page);
});

test("home and flagship pass automated WCAG A and AA checks", async ({ page }) => {
  for (const path of ["/en", "/zh/cognitiveos/verifiable-agent-actions"]) {
    await page.goto(path);
    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
      .analyze();
    expect(results.violations, path).toEqual([]);
  }
});

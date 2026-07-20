import { afterEach, describe, expect, it } from "vitest";
import { localizedPath } from "@/i18n/routes";
import { articleFrontmatterSchema } from "@/lib/content/schema";
import { createLocalizedMetadata } from "@/lib/seo/metadata";

const originalOrigin = process.env.NEXT_PUBLIC_SITE_URL;

afterEach(() => {
  if (originalOrigin === undefined) {
    delete process.env.NEXT_PUBLIC_SITE_URL;
  } else {
    process.env.NEXT_PUBLIC_SITE_URL = originalOrigin;
  }
});

describe("localized route identity", () => {
  it("builds explicit equivalent paths and shared anchors", () => {
    expect(
      localizedPath(
        "en",
        { kind: "article", slug: "testing-bilingual-content" },
        "build-gate",
      ),
    ).toBe("/en/articles/testing-bilingual-content#build-gate");
    expect(
      localizedPath("zh", { kind: "flagship" }, "verification-acceptance"),
    ).toBe("/zh/cognitiveos/verifiable-agent-actions#verification-acceptance");
  });
});

describe("metadata indexing policy", () => {
  it("emits exact canonical and hreflang values", () => {
    process.env.NEXT_PUBLIC_SITE_URL = "https://notes.example.dev";
    const metadata = createLocalizedMetadata({
      locale: "zh",
      title: "测试页面标题",
      description: "用于测试 canonical 与 hreflang 输出的完整描述文本。",
      path: "/zh/articles/example",
      alternatePath: "/en/articles/example",
    });

    expect(metadata.alternates?.canonical).toBe("/zh/articles/example");
    expect(metadata.alternates?.languages).toEqual({
      "zh-CN": "/zh/articles/example",
      en: "/en/articles/example",
      "x-default": "/zh/articles/example",
    });
    expect(metadata.robots).toMatchObject({ index: true, follow: true });
  });

  it("keeps samples and placeholder origins noindex", () => {
    process.env.NEXT_PUBLIC_SITE_URL = "https://notes.example.dev";
    const sample = createLocalizedMetadata({
      locale: "en",
      title: "Sample article title",
      description: "A complete description for a sample article metadata policy test.",
      path: "/en/articles/sample",
      alternatePath: "/zh/articles/sample",
      noIndex: true,
    });
    expect(sample.robots).toMatchObject({ index: false, follow: true });

    delete process.env.NEXT_PUBLIC_SITE_URL;
    const local = createLocalizedMetadata({
      locale: "en",
      title: "Local article title",
      description: "A complete description for a local canonical-host policy test.",
      path: "/en/cognitiveos/verifiable-agent-actions",
      alternatePath: "/zh/cognitiveos/verifiable-agent-actions",
    });
    expect(local.robots).toMatchObject({ index: false, follow: true });
  });
});

describe("article schema policy", () => {
  it("rejects sample status without placeholder", () => {
    const result = articleFrontmatterSchema.safeParse({
      schemaVersion: 1,
      kind: "article",
      locale: "en",
      translationKey: "invalid-sample",
      slug: "invalid-sample",
      title: "Invalid sample article",
      description: "This object intentionally violates the placeholder policy for testing.",
      publishedAt: "2026-07-20",
      status: "sample",
      placeholder: false,
      featured: false,
      pairingSnapshot: "invalid-v1",
      primaryTopic: "testing",
      tags: ["testing"],
      hero: {
        src: "/images/projects/constraint-map.svg",
        alt: "Abstract image",
        license: "UNLICENSED",
        provenance: "Local",
      },
      anchors: ["example"],
      claimLevel: "sample",
    });
    expect(result.success).toBe(false);
  });
});

import { afterEach, describe, expect, it } from "vitest";
import {
  articlePath,
  contentPath,
  flagshipPath,
  localizedPath,
  pagePath,
  projectPath,
} from "@/i18n/routes";
import { hasPublishableOrigin } from "@content/data/site";
import { articleFrontmatterSchema } from "@/lib/content/schema";
import { isPublishableFrontmatter } from "@/lib/content/publication";
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

  it("keeps research, lab, and content paths behind one route contract", () => {
    expect(pagePath("zh", "lab")).toBe("/zh/lab");
    expect(
      contentPath("en", {
        kind: "cognitiveos",
        slug: "verifiable-agent-actions",
      }),
    ).toBe(flagshipPath("en"));
    expect(
      contentPath("zh", {
        kind: "article",
        slug: "testing-bilingual-content",
      }),
    ).toBe(articlePath("zh", "testing-bilingual-content"));
    expect(
      contentPath("en", {
        kind: "project",
        slug: "evidence-first-cli",
      }),
    ).toBe(projectPath("en", "evidence-first-cli"));
  });
});

describe("metadata indexing policy", () => {
  it("accepts only a clean HTTPS origin for publication", () => {
    process.env.NEXT_PUBLIC_SITE_URL = "https://research.example.dev";
    expect(hasPublishableOrigin()).toBe(true);

    process.env.NEXT_PUBLIC_SITE_URL = "https://research.example.dev/path";
    expect(hasPublishableOrigin()).toBe(false);

    process.env.NEXT_PUBLIC_SITE_URL = "https://localhost";
    expect(hasPublishableOrigin()).toBe(false);
  });

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
  it("publishes real research while keeping samples out of publication feeds", () => {
    const published = articleFrontmatterSchema.parse({
      schemaVersion: 1,
      kind: "article",
      locale: "en",
      translationKey: "real-research",
      slug: "real-research",
      title: "A real systems research article",
      description:
        "A complete description for a real systems research article publication test.",
      publishedAt: "2026-07-20",
      status: "published",
      placeholder: false,
      featured: false,
      pairingSnapshot: "real-research-v1",
      primaryTopic: "system-design",
      tags: ["system-design"],
      hero: {
        src: "/images/projects/constraint-map.svg",
        alt: "Abstract constraint map",
        license: "UNLICENSED",
        provenance: "Local",
      },
      anchors: ["example"],
    });
    const sample = articleFrontmatterSchema.parse({
      ...published,
      status: "sample",
      placeholder: true,
      claimLevel: "sample",
    });

    expect(isPublishableFrontmatter(published)).toBe(true);
    expect(isPublishableFrontmatter(sample)).toBe(false);
  });

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

  it("rejects impossible dates and backwards revisions", () => {
    const base = {
      schemaVersion: 1,
      kind: "article",
      locale: "en",
      translationKey: "invalid-date",
      slug: "invalid-date",
      title: "Invalid date article",
      description:
        "This object intentionally violates the publication date contract.",
      publishedAt: "2026-99-99",
      updatedAt: "2025-01-01",
      status: "published",
      placeholder: false,
      featured: false,
      pairingSnapshot: "invalid-date-v1",
      primaryTopic: "testing",
      tags: ["testing"],
      hero: {
        src: "/images/projects/constraint-map.svg",
        alt: "Abstract image",
        license: "UNLICENSED",
        provenance: "Local",
      },
      anchors: ["example"],
    };

    expect(articleFrontmatterSchema.safeParse(base).success).toBe(false);
    expect(
      articleFrontmatterSchema.safeParse({
        ...base,
        publishedAt: "2026-07-20",
      }).success,
    ).toBe(false);
  });
});

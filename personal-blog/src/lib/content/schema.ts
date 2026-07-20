import { z } from "zod";

const localeSchema = z.enum(["zh-CN", "en"]);
const isoDateSchema = z
  .string()
  .regex(/^\d{4}-\d{2}-\d{2}$/)
  .refine((value) => {
    const date = new Date(`${value}T00:00:00Z`);
    return (
      Number.isFinite(date.valueOf()) &&
      date.toISOString().slice(0, 10) === value
    );
  }, "Expected a real ISO calendar date.");
const anchorSchema = z.string().regex(/^[a-z][a-z0-9-]*$/);

const heroSchema = z
  .object({
    src: z.string().startsWith("/images/"),
    alt: z.string().min(1),
    license: z.string().min(1),
    provenance: z.string().min(1),
  })
  .strict();

const commonSchema = z
  .object({
    schemaVersion: z.literal(1),
    locale: localeSchema,
    translationKey: z.string().regex(/^[a-z0-9-]+$/),
    slug: z.string().regex(/^[a-z0-9-]+$/),
    title: z.string().min(8),
    description: z.string().min(30),
    publishedAt: isoDateSchema,
    updatedAt: isoDateSchema.optional(),
    placeholder: z.boolean(),
    pairingSnapshot: z.string().min(1),
    hero: heroSchema,
    anchors: z.array(anchorSchema).min(1),
  })
  .strict();

export const articleFrontmatterSchema = commonSchema
  .extend({
    kind: z.enum(["article", "cognitiveos"]),
    status: z.enum(["published", "sample"]),
    featured: z.boolean(),
    primaryTopic: z.enum([
      "agent-systems",
      "system-design",
      "reliability",
      "developer-tools",
      "testing",
      "accessibility",
    ]),
    tags: z
      .array(
        z.enum([
          "agent-systems",
          "system-design",
          "reliability",
          "governance",
          "developer-tools",
          "testing",
          "nextjs",
          "accessibility",
        ]),
      )
      .min(1)
      .max(3),
    factSnapshot: z.string().optional(),
    sourcebookRef: z.string().optional(),
    claimLevel: z.enum(["sample", "normative-synthesis"]).optional(),
  })
  .superRefine((value, context) => {
    if (value.updatedAt && value.updatedAt < value.publishedAt) {
      context.addIssue({
        code: "custom",
        path: ["updatedAt"],
        message: "updatedAt cannot be earlier than publishedAt.",
      });
    }

    if (value.status === "sample" && !value.placeholder) {
      context.addIssue({
        code: "custom",
        path: ["placeholder"],
        message: "Sample articles must be placeholders.",
      });
    }

    if (value.kind === "cognitiveos") {
      if (value.placeholder || !value.factSnapshot || !value.sourcebookRef) {
        context.addIssue({
          code: "custom",
          message: "CognitiveOS articles require a real fact snapshot and sourcebook.",
        });
      }
    }
  });

export const projectFrontmatterSchema = commonSchema
  .extend({
    kind: z.literal("project"),
    status: z.literal("sample"),
    placeholder: z.literal(true),
    role: z.string().min(1),
    constraints: z.array(z.string().min(1)).min(2),
    stack: z.array(z.string().min(1)).min(1),
    evidenceStatus: z.literal("sample-only"),
    structure: z.tuple([
      z.literal("problem"),
      z.literal("constraints"),
      z.literal("approach"),
      z.literal("outcome"),
      z.literal("reflection"),
    ]),
  })
  .strict()
  .superRefine((value, context) => {
    if (value.updatedAt && value.updatedAt < value.publishedAt) {
      context.addIssue({
        code: "custom",
        path: ["updatedAt"],
        message: "updatedAt cannot be earlier than publishedAt.",
      });
    }
  });

export type ArticleFrontmatter = z.infer<typeof articleFrontmatterSchema>;
export type ProjectFrontmatter = z.infer<typeof projectFrontmatterSchema>;
export type ContentFrontmatter = ArticleFrontmatter | ProjectFrontmatter;

export function parseArticleFrontmatter(input: unknown): ArticleFrontmatter {
  return articleFrontmatterSchema.parse(input);
}

export function parseProjectFrontmatter(input: unknown): ProjectFrontmatter {
  return projectFrontmatterSchema.parse(input);
}

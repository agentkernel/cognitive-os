import type { MetadataRoute } from "next";
import { hasPublishableOrigin } from "@content/data/site";
import { absoluteUrl } from "@/lib/seo/metadata";

export default function robots(): MetadataRoute.Robots {
  const publishable = hasPublishableOrigin();
  return {
    rules: publishable
      ? [{ userAgent: "*", allow: "/" }]
      : [{ userAgent: "*", disallow: "/" }],
    sitemap: absoluteUrl("/sitemap.xml"),
    host: publishable ? absoluteUrl("/") : undefined,
  };
}

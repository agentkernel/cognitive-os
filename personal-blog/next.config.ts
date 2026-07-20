import createMDX from "@next/mdx";

const production = process.env.NODE_ENV === "production";
const configuredOrigin = process.env.NEXT_PUBLIC_SITE_URL;
const publishableHttpsOrigin = (() => {
  if (!configuredOrigin) {
    return false;
  }
  try {
    const origin = new URL(configuredOrigin);
    return (
      origin.protocol === "https:" &&
      origin.username === "" &&
      origin.password === "" &&
      origin.pathname === "/" &&
      origin.search === "" &&
      origin.hash === "" &&
      origin.hostname !== "localhost" &&
      origin.hostname !== "example.com" &&
      !origin.hostname.endsWith(".invalid")
    );
  } catch {
    return false;
  }
})();

const contentSecurityPolicy = [
  "default-src 'self'",
  `script-src 'self' 'unsafe-inline'${production ? "" : " 'unsafe-eval'"}`,
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: blob:",
  "font-src 'self' data:",
  "connect-src 'self'",
  "object-src 'none'",
  "base-uri 'self'",
  "form-action 'self'",
  "frame-ancestors 'none'",
  ...(publishableHttpsOrigin ? ["upgrade-insecure-requests"] : []),
].join("; ");

const withMDX = createMDX({
  extension: /\.mdx?$/,
  options: {
    remarkPlugins: [
      ["remark-frontmatter", ["yaml"]],
      ["remark-mdx-frontmatter", { name: "frontmatter" }],
      "remark-gfm",
    ],
  },
});

export default withMDX({
  pageExtensions: ["ts", "tsx", "md", "mdx"],
  poweredByHeader: false,
  reactStrictMode: true,
  turbopack: {
    root: process.cwd(),
  },
  experimental: {
    cpus: 2,
    globalNotFound: true,
  },
  async headers() {
    return [
      {
        source: "/(.*)",
        headers: [
          {
            key: "Content-Security-Policy",
            value: contentSecurityPolicy,
          },
          { key: "X-Content-Type-Options", value: "nosniff" },
          { key: "X-Frame-Options", value: "DENY" },
          {
            key: "Referrer-Policy",
            value: "strict-origin-when-cross-origin",
          },
          {
            key: "Permissions-Policy",
            value:
              "camera=(), microphone=(), geolocation=(), payment=(), usb=()",
          },
          { key: "Cross-Origin-Opener-Policy", value: "same-origin" },
          { key: "Cross-Origin-Resource-Policy", value: "same-origin" },
          ...(publishableHttpsOrigin
            ? [
                {
                  key: "Strict-Transport-Security",
                  value: "max-age=63072000; includeSubDomains; preload",
                },
              ]
            : []),
        ],
      },
    ];
  },
});

import createMDX from "@next/mdx";

const withMDX = createMDX({
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
    globalNotFound: true,
  },
});

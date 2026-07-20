import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    include: ["tests/unit/**/*.test.ts", "tests/content/**/*.test.ts"],
    coverage: {
      enabled: false,
    },
  },
  resolve: {
    alias: {
      "@": new URL("./src", import.meta.url).pathname,
      "@content": new URL("./content", import.meta.url).pathname,
      "@scripts": new URL("./scripts", import.meta.url).pathname,
    },
  },
});

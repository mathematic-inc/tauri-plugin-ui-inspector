import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    fileParallelism: false,
    include: ["e2e/**/*.e2e.ts"],
    testTimeout: 120_000,
  },
});

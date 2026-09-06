import { describe, expect, it } from "vitest";

import { createElementSourceAdapter } from "../src/index.js";

describe("element source paths", () => {
  it.each([
    ["file:///Users/dev/My%20App/Button.svelte", "/Users/dev/My App/Button.svelte"],
    ["/@fs/Users/dev/My%20App/Button.svelte", "/Users/dev/My App/Button.svelte"],
    ["src/%E6%8C%89%E9%92%AE.svelte", "src/按钮.svelte"],
    ["src/Button.svelte", "src/Button.svelte"],
  ])("normalizes %s", async (filePath, file) => {
    const adapter = createElementSourceAdapter("fixture", {
      resolveStack: () => [{ filePath, lineNumber: 12, columnNumber: 3, componentName: "Button" }],
    });
    const source = await adapter.inspect(document.createElement("button"));
    expect(source?.location).toEqual({ file, line: 12, column: 3 });
  });
});

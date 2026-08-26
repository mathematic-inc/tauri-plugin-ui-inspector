import { describe, expect, it } from "vitest";

import { vueAdapter } from "../src/index.js";

describe("Vue adapter", () => {
  it("returns undefined when Vue development metadata is absent", async () => {
    const adapter = vueAdapter();
    expect(adapter.name).toBe("vue");
    expect(await adapter.inspect(document.createElement("button"))).toBeUndefined();
  });
});

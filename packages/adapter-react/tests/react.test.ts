import { describe, expect, it } from "vitest";
import { reactAdapter } from "../src/index.js";

describe("React adapter", () => {
  it("returns undefined when React development metadata is absent", async () => {
    const adapter = reactAdapter();
    expect(adapter.name).toBe("react");
    expect(
      await adapter.inspect(document.createElement("button")),
    ).toBeUndefined();
  });
});

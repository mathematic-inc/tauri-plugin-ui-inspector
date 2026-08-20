import { describe, expect, it } from "vitest";
import { svelteAdapter } from "../src/index.js";

describe("Svelte adapter", () => {
  it("reads Svelte 5 development metadata and ancestry", async () => {
    const button = document.createElement("button") as HTMLButtonElement & {
      __svelte_meta?: unknown;
    };
    button.__svelte_meta = {
      loc: {
        file: "src/lib/CreateWorkspaceButton.svelte",
        line: 47,
        column: 2,
      },
      parent: {
        componentTag: "CreateWorkspaceButton",
        file: "src/lib/CreateWorkspaceButton.svelte",
        line: 1,
        column: 0,
        parent: {
          componentTag: "WorkspaceToolbar",
          file: "src/lib/WorkspaceToolbar.svelte",
          line: 10,
          column: 0,
          parent: null,
        },
      },
    };
    const source = await svelteAdapter().inspect(button);
    expect(source).toMatchObject({
      framework: "svelte",
      component: "CreateWorkspaceButton",
      location: {
        file: "src/lib/CreateWorkspaceButton.svelte",
        line: 47,
        column: 3,
      },
    });
    expect(source?.ancestry[0]?.component).toBe("WorkspaceToolbar");
  });

  it("returns undefined when production metadata is absent", async () => {
    expect(
      await svelteAdapter().inspect(document.createElement("button")),
    ).toBeUndefined();
  });
});

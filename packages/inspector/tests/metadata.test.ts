import { beforeEach, describe, expect, it } from "vitest";

import {
  collectAccessibility,
  collectDomContext,
  collectElementInfo,
  collectSelection,
} from "../src/index.js";

describe("DOM metadata", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("extracts semantic button metadata and a test id locator", () => {
    document.body.innerHTML = `
      <button data-testid="create-workspace"><span>Create workspace</span></button>
    `;
    const button = document.querySelector("button")!;
    const info = collectElementInfo(button);
    expect(info.role).toBe("button");
    expect(info.accessibleName).toBe("Create workspace");
    expect(info.selectors.testId).toBe("create-workspace");
    expect(info.locators[0]).toMatchObject({
      strategy: "testId",
      confidence: 1,
      unique: true,
    });
  });

  it("never persists password values", () => {
    document.body.innerHTML = `
      <label for="secret">Password</label>
      <input id="secret" type="password" value="hunter2" data-token="abc">
    `;
    const input = document.querySelector("input")!;
    const accessibility = collectAccessibility(input, {
      captureFormValues: true,
    });
    const dom = collectDomContext(input);
    expect(accessibility.value).toBeNull();
    expect(dom.html).not.toContain("hunter2");
    expect(dom.html).not.toContain("abc");
  });

  it("does not capture values from token-like text controls", () => {
    document.body.innerHTML = `
      <label for="api-token">API token</label>
      <input id="api-token" name="api-token" value="sk-secret">
    `;
    const input = document.querySelector("input")!;
    expect(collectAccessibility(input, { captureFormValues: true }).value).toBeNull();
  });

  it("formats a unique id as a usable preferred selector", () => {
    document.body.innerHTML = '<div id="workspace-toolbar"></div>';
    const info = collectElementInfo(document.querySelector("div")!);
    expect(info.selectors.preferred).toBe("#workspace-toolbar");
  });

  it("bounds DOM ancestry and HTML size", () => {
    document.body.innerHTML = `<div>${"<section>".repeat(12)}<button>${"x".repeat(5000)}</button>${"</section>".repeat(12)}</div>`;
    const button = document.querySelector("button")!;
    const dom = collectDomContext(button);
    expect(dom.ancestry).toHaveLength(8);
    expect(dom.html.length).toBeLessThanOrEqual(4000);
  });

  it("records native selected and expanded states", () => {
    document.body.innerHTML =
      "<details open><summary>More</summary></details><select><option selected>One</option></select>";
    expect(collectAccessibility(document.querySelector("details")!).expanded).toBe(true);
    expect(collectAccessibility(document.querySelector("option")!).selected).toBe(true);
  });

  it("ranks source metadata before generated selectors", async () => {
    document.body.innerHTML = '<button data-testid="create-workspace">Create workspace</button>';
    const payload = await collectSelection(document.querySelector("button")!, {
      adapters: [
        {
          name: "fixture",
          inspect: () => ({
            framework: "fixture",
            component: "CreateWorkspaceButton",
            location: { file: "src/CreateWorkspaceButton.svelte" },
            ancestry: [],
          }),
        },
      ],
    });
    expect(payload.element.locators.map(({ strategy }) => strategy)).toEqual([
      "testId",
      "role",
      "source",
      "css",
      "domPath",
      "text",
    ]);
  });

  it("includes open shadow hosts in ancestry", () => {
    const host = document.createElement("div");
    host.id = "shadow-host";
    const root = host.attachShadow({ mode: "open" });
    const button = document.createElement("button");
    button.dataset.testid = "shadow-action";
    root.append(button);
    document.body.append(host);
    const info = collectElementInfo(button);
    expect(info.locators[0]).toMatchObject({
      strategy: "testId",
      unique: true,
    });
    expect(collectDomContext(button).ancestry[0]).toMatchObject({
      tagName: "div",
      id: "shadow-host",
    });
  });
});

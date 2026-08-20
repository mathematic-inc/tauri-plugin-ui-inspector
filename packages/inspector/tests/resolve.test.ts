import { beforeEach, describe, expect, it } from "vitest";
import { collectSelection, resolveReference } from "../src/index.js";
import type { SelectionPayload } from "@tauri-ui-inspector/shared";

function reference(payload: SelectionPayload) {
  return {
    schemaVersion: 1,
    kind: "element" as const,
    id: "ui_01ARZ3NDEKTSV4RRFFQ69G5FAV",
    createdAt: "2026-08-20T00:00:00Z",
    summary: "button",
    project: { root: null },
    window: {
      label: "main",
      title: null,
      scaleFactor: 1,
      outerPosition: { x: 0, y: 0 },
      innerPosition: { x: 0, y: 0 },
      outerSize: { width: 1, height: 1 },
      innerSize: { width: 1, height: 1 },
      viewport: payload.viewport,
    },
    element: payload.element,
    source: null,
    dom: payload.dom,
    screenshots: { window: null, element: null },
    capture: { padding: 0, pixelCrop: null, screenshotSize: null },
  };
}

describe("reference resolution", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("reacquires one exact test id match", async () => {
    document.body.innerHTML = '<button data-testid="save">Save</button>';
    const button = document.querySelector("button")!;
    const payload = await collectSelection(button);
    expect(resolveReference(reference(payload))).toMatchObject({
      status: "resolved",
    });
  });

  it("fails instead of choosing one of several matches", async () => {
    document.body.innerHTML = '<button data-testid="save">Save</button>';
    const button = document.querySelector("button")!;
    const payload = await collectSelection(button);
    document.body.insertAdjacentHTML(
      "beforeend",
      '<button data-testid="save">Save</button>',
    );
    expect(resolveReference(reference(payload))).toMatchObject({
      status: "notFound",
    });
  });

  it("reacquires a test id inside an open shadow root", async () => {
    const host = document.createElement("div");
    const root = host.attachShadow({ mode: "open" });
    const button = document.createElement("button");
    button.dataset.testid = "shadow-save";
    button.textContent = "Save";
    root.append(button);
    document.body.append(host);
    const payload = await collectSelection(button);
    expect(resolveReference(reference(payload))).toMatchObject({
      status: "resolved",
    });
  });
});

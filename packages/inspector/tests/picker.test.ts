import { beforeEach, describe, expect, it, vi } from "vitest";
import { ElementPicker } from "../src/picker.js";

function picker(onSelect = vi.fn(async () => undefined)) {
  const onCancel = vi.fn();
  const instance = new ElementPicker({
    describe: (element) => element.getAttribute("data-ui-component") ?? "div",
    onStarted: vi.fn(),
    onHovered: vi.fn(),
    onSelect,
    onCancel,
    onError: (error) => {
      throw error;
    },
  });
  return { instance, onCancel, onSelect };
}

describe("element picker", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("highlights an interactive ancestor and suppresses its click", async () => {
    document.body.innerHTML =
      '<button data-ui-component="CreateWorkspaceButton"><span>Create workspace</span></button>';
    const button = document.querySelector("button")!;
    const span = document.querySelector("span")!;
    button.getBoundingClientRect = () =>
      ({
        x: 10,
        y: 40,
        left: 10,
        top: 40,
        right: 194,
        bottom: 80,
        width: 184,
        height: 40,
        toJSON: () => ({}),
      }) as DOMRect;
    const applicationClick = vi.fn();
    button.addEventListener("click", applicationClick);
    const { instance, onSelect } = picker();
    instance.start();
    span.dispatchEvent(new MouseEvent("pointermove", { bubbles: true }));
    const overlay = document.querySelector("ui-inspector-overlay")!;
    const shadowRoot = overlay.shadowRoot!;
    expect(shadowRoot.textContent).toContain("CreateWorkspaceButton");
    expect(shadowRoot.textContent).toContain("184 × 40");
    const box = shadowRoot.querySelector<HTMLElement>("#box")!;
    const label = shadowRoot.querySelector<HTMLElement>("#label")!;
    expect(box.style.top).toBe("40px");
    expect(label.style.top).toBe("2px");
    expect(label.style.minWidth).toBe("184px");
    span.dispatchEvent(
      new MouseEvent("click", { bubbles: true, cancelable: true }),
    );
    await vi.waitFor(() => expect(instance.state).toBe("idle"));
    expect(onSelect).toHaveBeenCalledWith(button);
    expect(applicationClick).not.toHaveBeenCalled();
    expect(document.querySelector("ui-inspector-overlay")).toBeNull();
  });

  it("cancels on Escape and removes all temporary DOM", () => {
    const { instance, onCancel } = picker();
    instance.start();
    window.dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "Escape",
        bubbles: true,
        cancelable: true,
      }),
    );
    expect(instance.state).toBe("idle");
    expect(onCancel).toHaveBeenCalledOnce();
    expect(document.querySelector("ui-inspector-overlay")).toBeNull();
    expect(document.querySelector("[data-ui-inspector-cursor]")).toBeNull();
  });

  it("holds the frozen state until asynchronous capture completes", async () => {
    let finish!: () => void;
    const { instance } = picker(
      vi.fn(
        () =>
          new Promise<void>((resolve) => {
            finish = resolve;
          }),
      ),
    );
    document.body.innerHTML = "<button>Save</button>";
    const button = document.querySelector("button")!;
    instance.start();
    button.dispatchEvent(
      new MouseEvent("click", { bubbles: true, cancelable: true }),
    );
    expect(instance.state).toBe("capturing");
    expect(document.querySelector("ui-inspector-overlay")).not.toBeNull();
    finish();
    await vi.waitFor(() => expect(instance.state).toBe("idle"));
  });

  it("uses composed paths for elements inside open shadow roots", async () => {
    const host = document.createElement("div");
    const root = host.attachShadow({ mode: "open" });
    const button = document.createElement("button");
    button.textContent = "Shadow action";
    root.append(button);
    document.body.append(host);
    const { instance, onSelect } = picker();
    instance.start();
    button.dispatchEvent(
      new MouseEvent("click", {
        bubbles: true,
        composed: true,
        cancelable: true,
      }),
    );
    await vi.waitFor(() => expect(instance.state).toBe("idle"));
    expect(onSelect).toHaveBeenCalledWith(button);
  });

  it("selects an interactive shadow host from its internal content", async () => {
    const button = document.createElement("div");
    button.setAttribute("role", "button");
    const root = button.attachShadow({ mode: "open" });
    const span = document.createElement("span");
    span.textContent = "Shadow label";
    root.append(span);
    document.body.append(button);
    const { instance, onSelect } = picker();
    instance.start();
    span.dispatchEvent(
      new MouseEvent("click", {
        bubbles: true,
        composed: true,
        cancelable: true,
      }),
    );
    await vi.waitFor(() => expect(instance.state).toBe("idle"));
    expect(onSelect).toHaveBeenCalledWith(button);
  });
});

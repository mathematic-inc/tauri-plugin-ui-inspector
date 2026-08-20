import type { InspectorState } from "./types.js";

interface PickerOptions {
  describe(element: Element): Promise<string> | string;
  onStarted(): void;
  onHovered(element: Element): void;
  onSelect(element: Element): Promise<void>;
  onCancel(): void;
  onError(error: unknown): void;
}

const blockedEvents = [
  "pointerdown",
  "pointerup",
  "mousedown",
  "mouseup",
  "auxclick",
  "dblclick",
  "contextmenu",
] as const;

export class ElementPicker {
  #state: InspectorState = "idle";
  #hovered: Element | null = null;
  #description = 0;
  #overlay: PickerOverlay | null = null;
  #animations = new Map<Animation, boolean>();

  constructor(private readonly options: PickerOptions) {}

  get state(): InspectorState {
    return this.#state;
  }

  start(): void {
    if (this.#state !== "idle") {
      throw new Error("UI inspection is already active");
    }
    this.#state = "inspecting";
    this.#overlay = new PickerOverlay(document);
    this.#listen(true);
    this.#freezeAnimations();
    this.options.onStarted();
  }

  stop(): void {
    if (this.#state !== "inspecting") return;
    this.#cleanup();
    this.options.onCancel();
  }

  dispose(): void {
    if (this.#state === "inspecting") this.stop();
  }

  #listen(add: boolean): void {
    const method = add ? "addEventListener" : "removeEventListener";
    window[method]("pointermove", this.#pointerMove, true);
    window[method]("mousemove", this.#pointerMove, true);
    window[method]("click", this.#click, true);
    window[method]("keydown", this.#keyDown, true);
    window[method]("resize", this.#reposition, true);
    window[method]("scroll", this.#reposition, true);
    for (const event of blockedEvents) {
      window[method](event, this.#block, true);
    }
  }

  #pointerMove = (event: Event): void => {
    if (this.#state !== "inspecting") return;
    const element = targetElement(event);
    if (!element || element === this.#hovered) return;
    this.#hovered = element;
    this.#freezeAnimations();
    this.options.onHovered(element);
    const token = ++this.#description;
    this.#overlay?.show(element, fallbackName(element));
    void Promise.resolve(this.options.describe(element))
      .then((name) => {
        if (
          token === this.#description &&
          element === this.#hovered &&
          this.#state === "inspecting"
        ) {
          this.#overlay?.show(element, name || fallbackName(element));
        }
      })
      .catch(this.options.onError);
  };

  #click = (event: Event): void => {
    this.#block(event);
    if (this.#state !== "inspecting") return;
    const element = targetElement(event) ?? this.#hovered;
    if (!element) return;
    this.#state = "capturing";
    this.#hovered = element;
    this.#overlay?.hide();
    void this.options
      .onSelect(element)
      .catch(this.options.onError)
      .finally(() => this.#cleanup());
  };

  #keyDown = (event: Event): void => {
    const keyboard = event as KeyboardEvent;
    if (keyboard.key !== "Escape" || this.#state !== "inspecting") return;
    this.#block(event);
    this.#cleanup();
    this.options.onCancel();
  };

  #reposition = (): void => {
    if (this.#state === "inspecting" && this.#hovered) {
      this.#overlay?.position(this.#hovered);
    }
  };

  #block = (event: Event): void => {
    if (this.#state === "idle") return;
    event.preventDefault();
    event.stopImmediatePropagation();
  };

  #freezeAnimations(): void {
    for (const animation of document.getAnimations?.() ?? []) {
      if (this.#animations.has(animation)) continue;
      const resume = animation.playState === "running" || animation.pending;
      this.#animations.set(animation, resume);
      if (resume) {
        animation.pause();
      }
    }
  }

  #cleanup(): void {
    if (this.#state === "idle") return;
    this.#listen(false);
    this.#overlay?.destroy();
    this.#overlay = null;
    this.#hovered = null;
    this.#description += 1;
    for (const [animation, resume] of this.#animations) {
      if (resume) {
        try {
          animation.play();
        } catch {
          // Detached animation effects cannot be resumed and need no cleanup.
        }
      }
    }
    this.#animations.clear();
    this.#state = "idle";
  }
}

class PickerOverlay {
  readonly #host: HTMLElement;
  readonly #box: HTMLElement;
  readonly #label: HTMLElement;
  readonly #cursorStyle: HTMLStyleElement;

  constructor(document: Document) {
    this.#host = document.createElement("ui-inspector-overlay");
    this.#host.setAttribute("aria-hidden", "true");
    Object.assign(this.#host.style, {
      position: "fixed",
      inset: "0",
      pointerEvents: "none",
      zIndex: "2147483647",
    });
    const root = this.#host.attachShadow({ mode: "open" });
    const style = document.createElement("style");
    style.textContent = `
      :host { all: initial; }
      #box { position: fixed; box-sizing: border-box; outline: 2px solid #00d9ff; box-shadow: 0 0 0 3px rgba(0, 0, 0, .8); }
      #label { position: fixed; box-sizing: border-box; max-width: min(320px, 100vw); padding: 3px 6px; border: 1px solid #4b5563; background: #101418; color: #f8fafc; font: 11px/1.2 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; white-space: pre; }
      #label strong { color: #22d3ee; font-weight: 600; }
    `;
    this.#box = document.createElement("div");
    this.#box.id = "box";
    this.#label = document.createElement("div");
    this.#label.id = "label";
    root.append(style, this.#box, this.#label);
    document.documentElement.append(this.#host);

    this.#cursorStyle = document.createElement("style");
    this.#cursorStyle.dataset.uiInspectorCursor = "";
    this.#cursorStyle.textContent =
      "html, html * { cursor: crosshair !important; }";
    document.head.append(this.#cursorStyle);
  }

  show(element: Element, name: string): void {
    const rect = element.getBoundingClientRect();
    this.#host.style.display = "block";
    this.#label.style.minWidth = `${Math.min(rect.width, 320)}px`;
    this.#label.replaceChildren();
    const title = document.createElement("strong");
    title.textContent = name;
    this.#label.append(
      title,
      document.createElement("br"),
      `${Math.round(rect.width)} × ${Math.round(rect.height)}`,
    );
    this.position(element);
  }

  position(element: Element): void {
    const rect = element.getBoundingClientRect();
    Object.assign(this.#box.style, {
      left: `${rect.left}px`,
      top: `${rect.top}px`,
      width: `${rect.width}px`,
      height: `${rect.height}px`,
    });
    const labelHeight = this.#label.getBoundingClientRect().height || 34;
    Object.assign(this.#label.style, {
      left: `${Math.max(0, Math.min(rect.left, window.innerWidth - this.#label.offsetWidth))}px`,
      top: `${rect.top >= labelHeight + 4 ? rect.top - labelHeight - 4 : Math.min(window.innerHeight - labelHeight, rect.bottom + 4)}px`,
    });
  }

  hide(): void {
    this.#host.style.display = "none";
  }

  destroy(): void {
    this.#host.remove();
    this.#cursorStyle.remove();
  }
}

function targetElement(event: Event): Element | null {
  const elements = event
    .composedPath()
    .filter((node): node is Element => node instanceof Element);
  return (
    elements.find((element) =>
      element.matches(
        "button, a[href], input, textarea, select, summary, [role=button], [role=link]",
      ),
    ) ??
    elements[0] ??
    null
  );
}

function fallbackName(element: Element): string {
  return (
    element.getAttribute("data-ui-component") || element.localName || "element"
  );
}

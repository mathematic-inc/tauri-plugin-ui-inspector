import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ElementReference,
  PickRequestEvent,
  ResolveRequestEvent,
  ResolveResult,
} from "@tauri-ui-inspector/shared";
import { collectSelection } from "./metadata.js";
import { ElementPicker } from "./picker.js";
import { resolveReference } from "./resolve.js";
import type {
  InspectOptions,
  InspectorBridgeOptions,
  InspectorController,
  InspectorOptions,
  InspectorState,
} from "./types.js";

export async function inspectElement(
  element: Element,
  options: InspectOptions = {},
): Promise<ElementReference> {
  return captureElement(element, options);
}

async function captureElement(
  element: Element,
  options: InspectOptions,
  requestId: string | null = null,
): Promise<ElementReference> {
  const payload = await collectSelection(element, options);
  return invoke<ElementReference>("plugin:ui-inspector|capture_selection", {
    requestId,
    payload,
  });
}

export async function inspectSelector(
  selector: string,
  options: InspectOptions = {},
): Promise<ElementReference> {
  const matches = document.querySelectorAll(selector);
  if (matches.length !== 1) {
    throw new Error(
      `expected selector to match exactly one element, matched ${matches.length}`,
    );
  }
  const element = matches[0];
  if (!element) throw new Error("selector matched no element");
  return inspectElement(element, options);
}

export async function getLastReference(): Promise<ElementReference | null> {
  return invoke<ElementReference | null>(
    "plugin:ui-inspector|get_last_reference",
  );
}

export function createInspector(
  options: InspectorOptions = {},
): InspectorController {
  return new InspectorControllerImplementation(options);
}

let defaultInspector: InspectorController | undefined;

export function startInspecting(
  options: InspectorOptions = {},
): InspectorController {
  defaultInspector?.dispose();
  defaultInspector = createInspector(options);
  defaultInspector.start();
  return defaultInspector;
}

export function stopInspecting(): void {
  defaultInspector?.stop();
  defaultInspector = undefined;
}

export async function installInspectorBridge(
  options: InspectorBridgeOptions = {},
): Promise<UnlistenFn> {
  const inspector = new InspectorControllerImplementation(
    options,
    async (requestId) => {
      await invoke("plugin:ui-inspector|cancel_selection", { requestId });
    },
  );
  const unlistenPick = await listen<PickRequestEvent>(
    "ui-inspector://pick",
    ({ payload }) => {
      try {
        inspector.startRequest(payload.requestId);
      } catch (error) {
        void invoke("plugin:ui-inspector|cancel_selection", {
          requestId: payload.requestId,
        });
        options.onError?.(error);
      }
    },
  );
  try {
    const unlistenResolve = await listen<ResolveRequestEvent>(
      "ui-inspector://resolve",
      async ({ payload }) => {
        try {
          const result: ResolveResult = resolveReference(payload.reference);
          await invoke("plugin:ui-inspector|complete_resolution", {
            requestId: payload.requestId,
            result,
          });
        } catch (error) {
          options.onError?.(error);
        }
      },
    );
    return () => {
      inspector.dispose();
      unlistenPick();
      unlistenResolve();
    };
  } catch (error) {
    inspector.dispose();
    unlistenPick();
    throw error;
  }
}

class InspectorControllerImplementation implements InspectorController {
  readonly #picker: ElementPicker;
  #requestId: string | null = null;
  #shortcut: ((event: KeyboardEvent) => void) | null = null;

  constructor(
    private readonly options: InspectorOptions,
    private readonly cancelRequest?: (requestId: string) => Promise<void>,
  ) {
    this.#picker = new ElementPicker({
      describe: async (element) => {
        for (const adapter of options.adapters ?? []) {
          const source = await adapter.inspect(element);
          if (source?.component) return source.component;
        }
        return element.getAttribute("data-ui-component") ?? element.localName;
      },
      onStarted: () => options.onStarted?.(),
      onHovered: (element) => options.onHovered?.(element),
      onSelect: (element) => this.#capture(element),
      onCancel: () => this.#cancel(),
      onError: (error) => options.onError?.(error),
    });
    if (options.keyboardShortcut !== false) {
      this.#shortcut = (event) => {
        if (
          event.key.toLowerCase() === "c" &&
          event.shiftKey &&
          (event.metaKey || event.ctrlKey) &&
          !event.altKey
        ) {
          event.preventDefault();
          if (this.state === "idle") this.start();
          else this.stop();
        }
      };
      window.addEventListener("keydown", this.#shortcut, true);
    }
  }

  get state(): InspectorState {
    return this.#picker.state;
  }

  start(): void {
    this.startRequest(null);
  }

  startRequest(requestId: string | null): void {
    this.#requestId = requestId;
    this.#picker.start();
  }

  stop(): void {
    this.#picker.stop();
  }

  dispose(): void {
    this.#picker.dispose();
    if (this.#shortcut) {
      window.removeEventListener("keydown", this.#shortcut, true);
      this.#shortcut = null;
    }
  }

  async #capture(element: Element): Promise<void> {
    const requestId = this.#requestId;
    try {
      const reference = await captureElement(element, this.options, requestId);
      this.options.onSelect?.(reference);
    } catch (error) {
      if (requestId && this.cancelRequest) {
        await this.cancelRequest(requestId).catch(() => undefined);
      }
      throw error;
    } finally {
      this.#requestId = null;
    }
  }

  #cancel(): void {
    const requestId = this.#requestId;
    this.#requestId = null;
    if (requestId && this.cancelRequest) {
      void this.cancelRequest(requestId).catch((error) =>
        this.options.onError?.(error),
      );
    }
    this.options.onCancel?.();
  }
}

import type { ElementReference, SelectionPayload, SourceInfo } from "@tauri-ui-inspector/shared";

export interface FrameworkInspectorAdapter {
  readonly name: string;
  inspect(element: Element): Promise<SourceInfo | undefined> | SourceInfo | undefined;
}

export interface MetadataOptions {
  adapters?: readonly FrameworkInspectorAdapter[];
  captureFormValues?: boolean;
  redactText?: boolean;
  sensitiveAttributeFragments?: readonly string[];
}

export type InspectOptions = MetadataOptions;

export interface InspectorBridgeOptions extends MetadataOptions {
  keyboardShortcut?: boolean;
  onStarted?(): void;
  onHovered?(element: Element): void;
  onSelect?(reference: ElementReference): void;
  onCancel?(): void;
  onError?(error: unknown): void;
}

export type InspectorOptions = InspectorBridgeOptions;

export type InspectorState = "idle" | "inspecting" | "capturing";

export interface InspectorController {
  readonly state: InspectorState;
  start(): void;
  stop(): void;
  dispose(): void;
}

export type { ElementReference, SelectionPayload, SourceInfo };

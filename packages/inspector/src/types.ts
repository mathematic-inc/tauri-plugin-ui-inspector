import type { ElementReference, SelectionPayload, SourceInfo } from "@tauri-ui-inspector/shared";

export interface FrameworkInspectorAdapter {
  readonly name: string;
  readonly inspect: (element: Element) => Promise<SourceInfo | undefined> | SourceInfo | undefined;
}

export interface MetadataOptions {
  readonly adapters?: readonly FrameworkInspectorAdapter[];
  readonly captureFormValues?: boolean;
  readonly redactText?: boolean;
  readonly sensitiveAttributeFragments?: readonly string[];
}

export type InspectOptions = MetadataOptions;

export interface InspectorBridgeOptions extends MetadataOptions {
  readonly keyboardShortcut?: boolean;
  readonly onStarted?: () => void;
  readonly onHovered?: (element: Element) => void;
  readonly onSelect?: (reference: ElementReference) => void;
  readonly onCancel?: () => void;
  readonly onError?: (error: unknown) => void;
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

export {
  collectAccessibility,
  collectDomContext,
  collectElementInfo,
  collectSelection,
} from "./metadata.js";
export { buildLocators } from "./locators.js";
export { createElementSourceAdapter } from "./element-source.js";
export type { ElementSourceFrame, ElementSourceResolver } from "./element-source.js";
export { resolveReference } from "./resolve.js";
export {
  createInspector,
  getLastReference,
  inspectElement,
  inspectSelector,
  installInspectorBridge,
  startInspecting,
  stopInspecting,
} from "./tauri.js";
export type {
  FrameworkInspectorAdapter,
  InspectOptions,
  InspectorBridgeOptions,
  InspectorController,
  InspectorOptions,
  InspectorState,
  MetadataOptions,
} from "./types.js";
export type * from "@tauri-ui-inspector/shared";

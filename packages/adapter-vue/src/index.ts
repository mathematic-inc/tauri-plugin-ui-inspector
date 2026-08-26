import {
  createElementSourceAdapter,
  type FrameworkInspectorAdapter,
} from "@tauri-ui-inspector/inspector";
import { createSourceResolver, vueResolver } from "element-source";

const resolver = createSourceResolver({ resolvers: [vueResolver] });

export function vueAdapter(): FrameworkInspectorAdapter {
  return createElementSourceAdapter("vue", resolver);
}

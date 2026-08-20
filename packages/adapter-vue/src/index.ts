import { createSourceResolver, vueResolver } from "element-source";
import {
  createElementSourceAdapter,
  type FrameworkInspectorAdapter,
} from "@tauri-ui-inspector/inspector";

const resolver = createSourceResolver({ resolvers: [vueResolver] });

export function vueAdapter(): FrameworkInspectorAdapter {
  return createElementSourceAdapter("vue", resolver);
}

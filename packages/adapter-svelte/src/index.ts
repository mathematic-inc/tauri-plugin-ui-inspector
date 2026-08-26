import {
  createElementSourceAdapter,
  type FrameworkInspectorAdapter,
} from "@tauri-ui-inspector/inspector";
import { createSourceResolver, svelteResolver } from "element-source";

const resolver = createSourceResolver({ resolvers: [svelteResolver] });

export function svelteAdapter(): FrameworkInspectorAdapter {
  return createElementSourceAdapter("svelte", resolver);
}

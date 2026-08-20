import { createSourceResolver, svelteResolver } from "element-source";
import {
  createElementSourceAdapter,
  type FrameworkInspectorAdapter,
} from "@tauri-ui-inspector/inspector";

const resolver = createSourceResolver({ resolvers: [svelteResolver] });

export function svelteAdapter(): FrameworkInspectorAdapter {
  return createElementSourceAdapter("svelte", resolver);
}

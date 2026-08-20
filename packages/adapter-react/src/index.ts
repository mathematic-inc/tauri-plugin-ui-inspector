import { createSourceResolver, reactResolver } from "element-source";
import {
  createElementSourceAdapter,
  type FrameworkInspectorAdapter,
} from "@tauri-ui-inspector/inspector";

const resolver = createSourceResolver({ resolvers: [reactResolver] });

export function reactAdapter(): FrameworkInspectorAdapter {
  return createElementSourceAdapter("react", resolver);
}

import {
  createElementSourceAdapter,
  type FrameworkInspectorAdapter,
} from "@tauri-ui-inspector/inspector";
import { createSourceResolver, reactResolver } from "element-source";

const resolver = createSourceResolver({ resolvers: [reactResolver] });

export function reactAdapter(): FrameworkInspectorAdapter {
  return createElementSourceAdapter("react", resolver);
}

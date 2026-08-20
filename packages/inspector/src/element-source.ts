import type { FrameworkInspectorAdapter } from "./types.js";
import type {
  SourceComponent,
  SourceLocation,
} from "@tauri-ui-inspector/shared";

export interface ElementSourceFrame {
  filePath: string;
  lineNumber: number | null;
  columnNumber: number | null;
  componentName: string | null;
}

export interface ElementSourceResolver {
  resolveStack(
    element: Element,
  ): ElementSourceFrame[] | Promise<ElementSourceFrame[]>;
}

/** Adapts an `element-source` framework resolver to inspector metadata. */
export function createElementSourceAdapter(
  framework: string,
  resolver: ElementSourceResolver,
): FrameworkInspectorAdapter {
  return {
    name: framework,
    async inspect(element) {
      const stack = await resolver.resolveStack(element);
      const first = stack[0];
      if (!first) return undefined;

      const componentName =
        stack.find((frame) => frame.componentName)?.componentName ?? null;
      const ancestry = stack.slice(1);
      while (ancestry[0]?.componentName === componentName) ancestry.shift();

      return {
        framework,
        component: componentName,
        location: location(first),
        ancestry: ancestry.map(component),
      };
    },
  };
}

function component(frame: ElementSourceFrame): SourceComponent {
  return {
    component: frame.componentName,
    location: location(frame),
  };
}

function location(frame: ElementSourceFrame): SourceLocation {
  return {
    file: decodeURIComponent(
      frame.filePath.replace(/^file:\/\//, "").replace(/^\/@fs\//, "/"),
    ),
    line: frame.lineNumber,
    column: frame.columnNumber,
  };
}

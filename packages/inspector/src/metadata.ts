import {
  computeAccessibleDescription,
  computeAccessibleName,
  getRole,
} from "dom-accessibility-api";
import type {
  AccessibilityInfo,
  DomAncestor,
  DomContext,
  ElementInfo,
  SelectionPayload,
  ViewportInfo,
} from "@tauri-ui-inspector/shared";
import { composedParent } from "./dom.js";
import { buildLocators } from "./locators.js";
import type { MetadataOptions } from "./types.js";

const defaultSensitiveFragments = [
  "authorization",
  "cookie",
  "password",
  "secret",
  "token",
  "value",
];
const maxText = 200;
const maxHtml = 4_000;
const maxParentHtml = 6_000;
const maxAncestors = 8;

export async function collectSelection(
  element: Element,
  options: MetadataOptions = {},
): Promise<SelectionPayload> {
  const elementInfo = collectElementInfo(element, options);
  const source = await firstSource(element, options);
  if (source) {
    const sourceValue = `${source.location.file}:${source.location.line ?? ""}:${source.location.column ?? ""}`;
    const sourceIndex = elementInfo.locators.findIndex(
      (locator) => locator.confidence < 0.7,
    );
    elementInfo.locators.splice(
      sourceIndex < 0 ? elementInfo.locators.length : sourceIndex,
      0,
      {
        strategy: "source",
        value: sourceValue,
        attribute: null,
        name: source.component ?? null,
        confidence: 0.7,
        unique: false,
      },
    );
  }
  return {
    viewport: viewportInfo(),
    element: elementInfo,
    source: source ?? null,
    dom: collectDomContext(element, options),
  };
}

export function collectElementInfo(
  element: Element,
  options: MetadataOptions = {},
): ElementInfo {
  const accessibility = collectAccessibility(element, options);
  const text = options.redactText ? null : visibleText(element);
  const rect = element.getBoundingClientRect();
  const attributes = collectAttributes(element, options);
  const { locators, selectors } = buildLocators(element, accessibility, text);
  return {
    tagName: element.localName,
    namespace: element.namespaceURI,
    text,
    role: accessibility.role ?? null,
    accessibleName: accessibility.name ?? null,
    attributes,
    rect: {
      x: rect.x,
      y: rect.y,
      width: rect.width,
      height: rect.height,
      top: rect.top,
      right: rect.right,
      bottom: rect.bottom,
      left: rect.left,
    },
    locators,
    selectors,
    accessibility,
  };
}

export function collectAccessibility(
  element: Element,
  options: MetadataOptions = {},
): AccessibilityInfo {
  const html = element instanceof HTMLElement ? element : undefined;
  const input = element instanceof HTMLInputElement ? element : undefined;
  const textarea = element instanceof HTMLTextAreaElement ? element : undefined;
  const select = element instanceof HTMLSelectElement ? element : undefined;
  const option = element instanceof HTMLOptionElement ? element : undefined;
  const details = element instanceof HTMLDetailsElement ? element : undefined;
  const inputType = input?.type.toLowerCase() ?? null;
  const allowValue =
    options.captureFormValues === true &&
    !isSensitiveControl(element, inputType, options);
  const value = allowValue
    ? (input?.value ?? textarea?.value ?? select?.value ?? null)
    : null;
  return {
    role: getRole(element),
    name: options.redactText ? null : nullable(computeAccessibleName(element)),
    description: options.redactText
      ? null
      : nullable(computeAccessibleDescription(element)),
    ariaLabel: element.getAttribute("aria-label"),
    ariaLabelledBy: element.getAttribute("aria-labelledby"),
    ariaDescribedBy: element.getAttribute("aria-describedby"),
    disabled: html ? html.matches(':disabled,[aria-disabled="true"]') : null,
    checked: booleanState(element, "aria-checked", input?.checked),
    selected: booleanState(element, "aria-selected", option?.selected),
    expanded: booleanState(element, "aria-expanded", details?.open),
    pressed: booleanState(element, "aria-pressed", null),
    placeholder: input?.placeholder ?? textarea?.placeholder ?? null,
    formLabel: options.redactText
      ? null
      : formLabel(input ?? textarea ?? select),
    inputType,
    value,
  };
}

export function collectDomContext(
  element: Element,
  options: MetadataOptions = {},
): DomContext {
  const ancestry: DomAncestor[] = [];
  for (
    let current = composedParent(element);
    current && ancestry.length < maxAncestors;
    current = composedParent(current)
  ) {
    ancestry.push({
      tagName: current.localName,
      id: nullable(current.id),
      classes: [...current.classList].slice(0, 8),
      role: getRole(current),
      accessibleName: options.redactText
        ? null
        : nullable(computeAccessibleName(current)),
    });
  }
  const parent = composedParent(element);
  return {
    html: sanitizeHtml(element, maxHtml, options),
    parentHtml: parent ? sanitizeHtml(parent, maxParentHtml, options) : null,
    ancestry,
  };
}

function collectAttributes(
  element: Element,
  options: MetadataOptions,
): Record<string, string> {
  const sensitive = sensitiveFragments(options);
  return Object.fromEntries(
    [...element.attributes]
      .slice(0, 50)
      .map(({ name, value }) => [
        name,
        isSensitive(name, sensitive) ? "[redacted]" : truncate(value, 500),
      ]),
  );
}

function sanitizeHtml(
  element: Element,
  limit: number,
  options: MetadataOptions,
): string {
  if (options.redactText) return "[redacted]";
  const clone = element.cloneNode(true) as Element;
  const sensitive = sensitiveFragments(options);
  const sanitize = (node: Element): void => {
    for (const attribute of node.attributes) {
      if (isSensitive(attribute.name, sensitive)) {
        node.setAttribute(attribute.name, "[redacted]");
      }
    }
    if (
      node instanceof HTMLInputElement ||
      node instanceof HTMLTextAreaElement
    ) {
      node.removeAttribute("value");
    }
  };
  sanitize(clone);
  for (const node of clone.querySelectorAll("*")) {
    sanitize(node);
  }
  return truncate(clone.outerHTML, limit);
}

function visibleText(element: Element): string | null {
  if (
    element instanceof HTMLInputElement ||
    element instanceof HTMLTextAreaElement
  ) {
    return null;
  }
  return nullable(truncate(normalize(element.textContent ?? ""), maxText));
}

function viewportInfo(): ViewportInfo {
  const visual = window.visualViewport;
  return {
    size: { width: window.innerWidth, height: window.innerHeight },
    devicePixelRatio: window.devicePixelRatio,
    visualViewport: visual
      ? {
          offsetLeft: visual.offsetLeft,
          offsetTop: visual.offsetTop,
          scale: visual.scale,
          width: visual.width,
          height: visual.height,
        }
      : null,
  };
}

async function firstSource(element: Element, options: MetadataOptions) {
  for (const adapter of options.adapters ?? []) {
    const source = await adapter.inspect(element);
    if (source) return source;
  }
  return undefined;
}

function booleanState(
  element: Element,
  attribute: string,
  fallback: boolean | null | undefined,
): boolean | null {
  const value = element.getAttribute(attribute);
  if (value === "true") return true;
  if (value === "false") return false;
  return fallback ?? null;
}

function formLabel(
  element:
    HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement | undefined,
): string | null {
  return nullable(normalize(element?.labels?.[0]?.textContent ?? ""));
}

function sensitiveFragments(options: MetadataOptions): readonly string[] {
  return options.sensitiveAttributeFragments ?? defaultSensitiveFragments;
}

function isSensitiveControl(
  element: Element,
  inputType: string | null,
  options: MetadataOptions,
): boolean {
  if (inputType === "password" || inputType === "hidden") return true;
  const sensitiveAutocomplete = [
    "current-password",
    "new-password",
    "one-time-code",
    "cc-number",
    "cc-csc",
  ];
  const fragments = sensitiveFragments(options).filter(
    (fragment) => fragment.toLowerCase() !== "value",
  );
  return ["name", "id", "autocomplete", "aria-label", "placeholder"].some(
    (name) => {
      const value = element.getAttribute(name)?.toLowerCase();
      return (
        value !== undefined &&
        (sensitiveAutocomplete.some((token) => value.includes(token)) ||
          fragments.some((fragment) => value.includes(fragment.toLowerCase())))
      );
    },
  );
}

function isSensitive(name: string, fragments: readonly string[]): boolean {
  const lower = name.toLowerCase();
  return fragments.some((fragment) => lower.includes(fragment.toLowerCase()));
}

function normalize(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

function truncate(value: string, limit: number): string {
  return value.length <= limit ? value : `${value.slice(0, limit - 1)}…`;
}

function nullable(value: string | null | undefined): string | null {
  const normalized = normalize(value ?? "");
  return normalized.length > 0 ? normalized : null;
}

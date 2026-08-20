import { computeAccessibleName, getRole } from "dom-accessibility-api";
import type {
  ElementReference,
  Locator,
  ResolveResult,
} from "@tauri-ui-inspector/shared";
import { allElements, querySelectorAllDeep } from "./dom.js";

export function resolveReference(reference: ElementReference): ResolveResult {
  for (const [index, locator] of reference.element.locators.entries()) {
    if (!locator.unique || locator.confidence < 0.5) continue;
    const matches = locate(locator);
    if (matches.length !== 1) continue;
    const element = matches[0];
    if (!element || !sameSignature(element, reference)) continue;
    const rect = element.getBoundingClientRect();
    return {
      status: "resolved",
      locatorIndex: index,
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
    };
  }
  return {
    status: "notFound",
    reason: "no stored locator uniquely matched the original element signature",
  };
}

function locate(locator: Locator): Element[] {
  switch (locator.strategy) {
    case "testId":
    case "attribute":
      return query(
        `[${CSS.escape(locator.attribute ?? "")}="${escapeString(locator.value)}"]`,
      );
    case "id":
      return query(`#${CSS.escape(locator.value)}`);
    case "css":
    case "domPath":
      return query(locator.value);
    case "role":
      return allElements().filter(
        (candidate) =>
          getRole(candidate) === locator.value &&
          computeAccessibleName(candidate) === locator.name,
      );
    case "text":
      return allElements(document.body).filter(
        (candidate) => normalize(candidate.textContent ?? "") === locator.value,
      );
    case "source":
      return [];
  }
  return [];
}

function sameSignature(element: Element, reference: ElementReference): boolean {
  if (element.localName !== reference.element.tagName) return false;
  if (reference.element.role && getRole(element) !== reference.element.role)
    return false;
  if (
    reference.element.accessibleName &&
    computeAccessibleName(element) !== reference.element.accessibleName
  ) {
    return false;
  }
  return true;
}

function query(selector: string): Element[] {
  if (!selector) return [];
  return querySelectorAllDeep(selector);
}

function escapeString(value: string): string {
  return value.replaceAll("\\", "\\\\").replaceAll('"', '\\"');
}

function normalize(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

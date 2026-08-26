import { finder } from "@medv/finder";
import type { AccessibilityInfo, Locator, SelectorSummary } from "@tauri-ui-inspector/shared";
import { computeAccessibleName, getRole } from "dom-accessibility-api";

import { allElements, querySelectorAllDeep } from "./dom.js";

const testIdAttributes = ["data-testid", "data-test", "data-cy"];
const stableAttributes = ["name", "aria-label", "title", "placeholder", "href"];

export function buildLocators(
  element: Element,
  accessibility: AccessibilityInfo,
  text: string | null,
): { locators: Locator[]; selectors: SelectorSummary } {
  const locators: Locator[] = [];
  let testId: string | null = null;
  for (const attribute of testIdAttributes) {
    const value = element.getAttribute(attribute);
    if (!value) {
      continue;
    }
    const selector = `[${escapeIdentifier(attribute)}="${escapeString(value)}"]`;
    const unique = uniqueSelector(selector);
    locators.push(locator("testId", value, 1, unique, attribute));
    testId ??= value;
  }

  if (accessibility.role && accessibility.name) {
    locators.push({
      strategy: "role",
      value: accessibility.role,
      attribute: null,
      name: accessibility.name,
      confidence: 0.95,
      unique: uniqueRole(accessibility.role, accessibility.name),
    });
  }

  const id = element.id || null;
  if (id) {
    locators.push(locator("id", id, 0.9, uniqueSelector(`#${CSS.escape(id)}`)));
  }

  for (const attribute of stableAttributes) {
    const value = element.getAttribute(attribute);
    if (!value) {
      continue;
    }
    const selector = `[${escapeIdentifier(attribute)}="${escapeString(value)}"]`;
    locators.push(locator("attribute", value, 0.8, uniqueSelector(selector), attribute));
  }

  let css: string | null = null;
  try {
    css = finder(element);
    locators.push(locator("css", css, 0.55, uniqueSelector(css)));
  } catch {
    css = null;
  }

  const path = domPath(element);
  if (path !== css) {
    locators.push(locator("domPath", path, 0.35, uniqueSelector(path)));
  }

  if (text && uniqueText(text)) {
    locators.push(locator("text", text, 0.25, true));
  }

  locators.sort((left, right) => right.confidence - left.confidence);
  const preferred = locators.find((candidate) => candidate.unique) ?? locators[0];
  return {
    locators,
    selectors: {
      preferred: preferred ? displayLocator(preferred) : null,
      css,
      testId,
      id,
      role: accessibility.role ?? null,
      text,
    },
  };
}

function locator(
  strategy: Locator["strategy"],
  value: string,
  confidence: number,
  unique: boolean,
  attribute: string | null = null,
): Locator {
  return { strategy, value, attribute, name: null, confidence, unique };
}

function uniqueSelector(selector: string): boolean {
  return querySelectorAllDeep(selector).length === 1;
}

function uniqueRole(role: string, name: string): boolean {
  let matches = 0;
  for (const candidate of allElements()) {
    if (getRole(candidate) === role && computeAccessibleName(candidate) === name) {
      matches += 1;
      if (matches > 1) {
        return false;
      }
    }
  }
  return matches === 1;
}

function uniqueText(text: string): boolean {
  let matches = 0;
  for (const candidate of allElements(document.body)) {
    if (normalize(candidate.textContent ?? "") === text) {
      matches += 1;
      if (matches > 1) {
        return false;
      }
    }
  }
  return matches === 1;
}

function domPath(element: Element): string {
  const parts: string[] = [];
  for (
    let current: Element | null = element;
    current && current !== document.body;
    current = current.parentElement
  ) {
    const siblings = current.parentElement
      ? [...current.parentElement.children].filter(
          (sibling) => sibling.localName === current.localName,
        )
      : [];
    const suffix = siblings.length > 1 ? `:nth-of-type(${siblings.indexOf(current) + 1})` : "";
    parts.unshift(`${current.localName}${suffix}`);
  }
  return `body > ${parts.join(" > ")}`;
}

function displayLocator(locator: Locator): string {
  if (locator.strategy === "role") {
    return `${locator.value}[name="${locator.name ?? ""}"]`;
  }
  if (locator.strategy === "id") {
    return `#${CSS.escape(locator.value)}`;
  }
  if (locator.attribute) {
    return `[${locator.attribute}="${locator.value}"]`;
  }
  return locator.value;
}

function escapeString(value: string): string {
  return value.replaceAll("\\", "\\\\").replaceAll('"', '\\"');
}

function escapeIdentifier(value: string): string {
  return CSS.escape(value);
}

function normalize(value: string): string {
  return value.replaceAll(/\s+/gv, " ").trim();
}

export function allElements(root: ParentNode = document): Element[] {
  const elements = [...root.querySelectorAll("*")];
  for (const element of elements) {
    if (element.shadowRoot) elements.push(...allElements(element.shadowRoot));
  }
  return elements;
}

export function querySelectorAllDeep(selector: string): Element[] {
  try {
    return allElements().filter((element) => element.matches(selector));
  } catch {
    return [];
  }
}

export function composedParent(element: Element): Element | null {
  if (element.parentElement) return element.parentElement;
  const root = element.getRootNode();
  return root instanceof ShadowRoot ? root.host : null;
}

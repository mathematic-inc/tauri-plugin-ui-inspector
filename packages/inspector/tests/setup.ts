if (!("CSS" in globalThis)) {
  Object.defineProperty(globalThis, "CSS", { value: {} });
}

if (!("escape" in globalThis.CSS)) {
  globalThis.CSS.escape = (value: string) =>
    value.replaceAll(/[^a-zA-Z0-9_\-]/gv, (character) => `\\${character}`);
}

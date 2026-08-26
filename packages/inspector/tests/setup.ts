if (!globalThis.CSS) {
  Object.defineProperty(globalThis, "CSS", { value: {} });
}

if (!globalThis.CSS.escape) {
  globalThis.CSS.escape = (value: string) =>
    value.replaceAll(/[^a-zA-Z0-9_\-]/gv, (character) => `\\${character}`);
}

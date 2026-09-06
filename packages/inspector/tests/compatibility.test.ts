// @vitest-environment node
import { readdirSync, readFileSync } from "node:fs";

import { parse } from "acorn";
import { describe, expect, it } from "vitest";

const directory = new URL("../dist/", import.meta.url);
const modules = readdirSync(directory).filter((name) => name.endsWith(".js"));

describe("published WebView syntax", () => {
  it("includes the inspector entry point", () => {
    expect(modules).toContain("index.js");
  });

  it.each(modules)("parses %s as ES2021", (name) => {
    const source = readFileSync(new URL(name, directory), "utf8");
    expect(() => parse(source, { ecmaVersion: 2021, sourceType: "module" })).not.toThrow();
  });
});

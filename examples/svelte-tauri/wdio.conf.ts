import path from "node:path";

import type { TauriCapabilities } from "@wdio/tauri-service";

const binary = path.resolve(
  import.meta.dirname,
  "../../target/debug",
  process.platform === "win32" ? "ui-inspector-svelte-example.exe" : "ui-inspector-svelte-example",
);
const capabilities: TauriCapabilities = {
  browserName: "tauri",
  "tauri:options": { application: binary },
};

export const config: WebdriverIO.Config = {
  runner: "local",
  specs: ["./e2e/**/*.e2e.ts"],
  maxInstances: 1,
  capabilities: [capabilities],
  services: [
    [
      "@wdio/tauri-service",
      {
        appBinaryPath: binary,
        driverProvider: "embedded",
        embeddedPort: 4445,
        startTimeout: 60_000,
      },
    ],
  ],
  framework: "mocha",
  reporters: ["spec"],
  logLevel: "warn",
  waitforTimeout: 15_000,
  connectionRetryTimeout: 90_000,
  connectionRetryCount: 2,
  mochaOpts: { ui: "bdd", timeout: 120_000 },
};

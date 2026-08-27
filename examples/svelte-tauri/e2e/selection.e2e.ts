import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import { cleanupWdioSession, createTauriCapabilities, startWdioSession } from "@wdio/tauri-service";
import { imageSize } from "image-size";
import { PNG } from "pngjs";
import { afterAll, beforeAll, describe, it } from "vitest";

const repository = path.resolve(import.meta.dirname, "../../..");
const executable = process.platform === "win32" ? "ui-inspector.exe" : "ui-inspector";
const cli = path.join(repository, "target", "debug", executable);
const screenshots = path.join(repository, "docs", "screenshots");
const binary = path.join(
  repository,
  "target",
  "debug",
  process.platform === "win32" ? "ui-inspector-svelte-example.exe" : "ui-inspector-svelte-example",
);
const startTimeout = 60_000;
const waitForTimeout = 15_000;

type TauriBrowser = Awaited<ReturnType<typeof startWdioSession>>;

let driver: TauriBrowser | undefined;

beforeAll(async () => {
  driver = await startWdioSession(
    createTauriCapabilities(binary, {
      driverProvider: "embedded",
      logLevel: "warn",
      startTimeout,
    }),
  );
}, 120_000);

afterAll(async () => {
  if (driver) {
    await cleanupWdioSession(driver);
  }
}, 120_000);

describe("native UI selection", () => {
  it("captures, persists, reads, and resolves a Svelte element", async () => {
    assert.ok(driver);
    const browser = driver;
    mkdirSync(screenshots, { recursive: true });
    runCli(["clear"]);
    await browser.waitUntil(
      async () => {
        const button = await browser.$('[data-testid="create-workspace"]');
        const ready = await browser.execute(() => document.documentElement.dataset.inspectorReady);
        return (await button.isExisting()) && ready === "true";
      },
      { interval: 500, timeout: startTimeout },
    );
    const viewportSize = await browser.execute(() => ({
      width: window.innerWidth,
      height: window.innerHeight,
    }));
    await saveViewportScreenshot(browser, path.join(screenshots, "fixture-idle.actual.png"));

    const pick = spawn(cli, ["--project", repository, "pick"], {
      cwd: repository,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    pick.stdout.setEncoding("utf8").on("data", (chunk) => (stdout += chunk));
    pick.stderr.setEncoding("utf8").on("data", (chunk) => (stderr += chunk));
    let pickExit: number | null | undefined;
    const pickFinished = new Promise<number | null>((resolve, reject) => {
      pick.once("error", reject);
      pick.once("exit", (code) => {
        pickExit = code;
        resolve(code);
      });
    });

    await browser.waitUntil(
      async () => {
        if (pickExit !== undefined) {
          return true;
        }
        return browser.execute(() => Boolean(document.querySelector("ui-inspector-overlay")));
      },
      { timeout: waitForTimeout },
    );
    assert.equal(pickExit, undefined, `${stderr}\n${stdout}`);
    const button = await browser.$('[data-testid="create-workspace"]');
    await button.moveTo();
    await browser.waitUntil(
      () =>
        browser.execute(() =>
          document
            .querySelector("ui-inspector-overlay")
            ?.shadowRoot?.textContent?.includes("CreateWorkspaceButton"),
        ),
      { timeout: waitForTimeout },
    );
    await saveViewportScreenshot(browser, path.join(screenshots, "fixture-inspecting.actual.png"));
    await button.click();

    const code = await pickFinished;
    assert.equal(code, 0, stderr);
    const id = stdout.match(/ui_[0-9A-HJKMNP-TV-Z]{26}/v)?.[0];
    assert.ok(id, `missing reference id in CLI output: ${stdout}`);

    const reference = JSON.parse(runCli(["get", `@${id}`, "--json"])) as {
      id: string;
      window: { viewport: { size: { width: number; height: number } } };
      element: {
        rect: { top: number; width: number; height: number };
      };
      source: { component: string | null; location: { file: string } } | null;
      screenshots: { window: string; element: string };
      capture: {
        pixelCrop: { x: number; y: number; width: number; height: number };
      };
    };
    assert.equal(reference.id, id);
    assert.equal(reference.element.rect.width, 184);
    assert.equal(reference.element.rect.height, 40);
    assert.equal(reference.element.rect.top, 39);
    assert.deepEqual(reference.window.viewport.size, viewportSize);
    assert.equal(reference.source?.component, "CreateWorkspaceButton");
    assert.match(reference.source?.location.file ?? "", /CreateWorkspaceButton\.svelte$/v);

    const directory = path.join(repository, ".ui-inspector", "refs", id);
    const windowPng = path.join(directory, reference.screenshots.window);
    const elementPng = path.join(directory, reference.screenshots.element);
    assert.ok(existsSync(windowPng), "window.png was not created");
    assert.ok(existsSync(elementPng), "element.png was not created");
    const dimensions = imageSize(readFileSync(elementPng));
    assert.equal(dimensions.width, reference.capture.pixelCrop.width);
    assert.equal(dimensions.height, reference.capture.pixelCrop.height);
    assert.ok(dimensions.width > reference.element.rect.width);
    assert.ok(dimensions.height > reference.element.rect.height);
    assertCropEquals(windowPng, elementPng, reference.capture.pixelCrop);
    copyFileSync(elementPng, path.join(screenshots, "create-workspace-element.actual.png"));

    const resolution = JSON.parse(runCli(["resolve", id, "--json"])) as {
      status: string;
    };
    assert.equal(resolution.status, "resolved");
  });
});

function runCli(args: string[]): string {
  const result = spawnSync(cli, ["--project", repository, ...args], {
    cwd: repository,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}

async function saveViewportScreenshot(driver: TauriBrowser, destination: string): Promise<void> {
  const metrics = await driver.execute(() => ({
    width: window.innerWidth,
    height: window.innerHeight,
    devicePixelRatio: window.devicePixelRatio,
  }));
  const screenshot = PNG.sync.read(Buffer.from(await driver.takeScreenshot(), "base64"));
  const width = Math.round(metrics.width * metrics.devicePixelRatio);
  const height = Math.round(metrics.height * metrics.devicePixelRatio);
  assert.ok(screenshot.width >= width && screenshot.height >= height);
  const viewport = new PNG({ width, height });
  PNG.bitblt(screenshot, viewport, 0, 0, width, height, 0, 0);
  writeFileSync(destination, PNG.sync.write(viewport));
}

function assertCropEquals(
  windowPath: string,
  elementPath: string,
  crop: { x: number; y: number; width: number; height: number },
): void {
  const window = PNG.sync.read(readFileSync(windowPath));
  const element = PNG.sync.read(readFileSync(elementPath));
  for (let y = 0; y < crop.height; y += 1) {
    const windowStart = ((crop.y + y) * window.width + crop.x) * 4;
    const elementStart = y * element.width * 4;
    assert.deepEqual(
      window.data.subarray(windowStart, windowStart + crop.width * 4),
      element.data.subarray(elementStart, elementStart + crop.width * 4),
    );
  }
}

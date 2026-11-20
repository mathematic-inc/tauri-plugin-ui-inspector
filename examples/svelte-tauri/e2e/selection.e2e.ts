import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import { cleanupWdioSession, createTauriCapabilities, startWdioSession } from "@wdio/tauri-service";
import { imageSize } from "image-size";
import { PNG } from "pngjs";
import { afterAll, beforeAll, describe, it } from "vitest";

const repository = path.resolve(import.meta.dirname, "../../..");
const cargoTargetDirectory = process.env.CARGO_TARGET_DIR;
const target =
  cargoTargetDirectory === undefined || cargoTargetDirectory === ""
    ? path.join(repository, "target")
    : path.resolve(repository, cargoTargetDirectory);
const executable = process.platform === "win32" ? "ui-inspector.exe" : "ui-inspector";
const cli = path.join(target, "debug", executable);
const screenshots = path.join(repository, "docs", "screenshots");
const binary = path.join(
  target,
  "debug",
  process.platform === "win32" ? "ui-inspector-svelte-example.exe" : "ui-inspector-svelte-example",
);
const startTimeout = 60_000;
const waitForTimeout = 15_000;

type TauriBrowser = Awaited<ReturnType<typeof startWdioSession>>;

interface PixelCrop {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

interface StoredReference {
  readonly id: string;
  readonly window: {
    readonly viewport: { readonly size: { readonly width: number; readonly height: number } };
  };
  readonly element: {
    readonly rect: { readonly top: number; readonly width: number; readonly height: number };
  };
  readonly source: {
    readonly component: string | null;
    readonly location: { readonly file: string };
  } | null;
  readonly screenshots: { readonly window: string; readonly element: string };
  readonly capture: { readonly pixelCrop: PixelCrop };
}

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
        const button = browser.$('[data-testid="create-workspace"]');
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
    pick.stdout.setEncoding("utf8").on("data", (chunk: string) => {
      stdout += chunk;
    });
    pick.stderr.setEncoding("utf8").on("data", (chunk: string) => {
      stderr += chunk;
    });
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
    const button = browser.$('[data-testid="create-workspace"]');
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
    if (id === undefined) {
      assert.fail(`missing reference id in CLI output: ${stdout}`);
    }

    const reference = parseJson(runCli(["get", `@${id}`, "--json"]), isStoredReference);
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

    const resolution = parseJson(runCli(["resolve", id, "--json"]), isResolution);
    assert.equal(resolution.status, "resolved");
  });
});

function runCli(args: readonly string[]): string {
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

function assertCropEquals(windowPath: string, elementPath: string, crop: PixelCrop): void {
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

function parseJson<Value>(text: string, isValue: (value: unknown) => value is Value): Value {
  const value: unknown = JSON.parse(text);
  assert.ok(isValue(value), `unexpected CLI output: ${text}`);
  return value;
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return typeof value === "object" && value !== null;
}

function isNumberRecord(value: unknown, keys: readonly string[]): boolean {
  return isRecord(value) && keys.every((key) => typeof value[key] === "number");
}

function isResolution(value: unknown): value is { readonly status: string } {
  return isRecord(value) && typeof value.status === "string";
}

function isStoredReference(value: unknown): value is StoredReference {
  if (!isRecord(value) || !isRecord(value.window) || !isRecord(value.window.viewport)) {
    return false;
  }
  if (!isRecord(value.element) || !isRecord(value.screenshots) || !isRecord(value.capture)) {
    return false;
  }
  const { source } = value;
  const validSource =
    source === null ||
    (isRecord(source) &&
      (source.component === null || typeof source.component === "string") &&
      isRecord(source.location) &&
      typeof source.location.file === "string");
  return (
    typeof value.id === "string" &&
    isNumberRecord(value.window.viewport.size, ["width", "height"]) &&
    isNumberRecord(value.element.rect, ["top", "width", "height"]) &&
    validSource &&
    typeof value.screenshots.window === "string" &&
    typeof value.screenshots.element === "string" &&
    isNumberRecord(value.capture.pixelCrop, ["x", "y", "width", "height"])
  );
}

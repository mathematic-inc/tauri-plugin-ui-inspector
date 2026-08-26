<script lang="ts">
  import { svelteAdapter } from "@tauri-ui-inspector/adapter-svelte";
  import {
    installInspectorBridge,
    startInspecting,
    type ElementReference,
  } from "@tauri-ui-inspector/inspector";
  import { onMount } from "svelte";

  import CreateWorkspaceButton from "./lib/CreateWorkspaceButton.svelte";

  const adapter = svelteAdapter();
  let dialog: HTMLDialogElement;
  let popover: HTMLDivElement;
  // oxlint-disable-next-line no-unassigned-vars -- assigned by Svelte bind:this
  let canvas: HTMLCanvasElement;
  // oxlint-disable-next-line no-unassigned-vars -- assigned by Svelte bind:this
  let webgl: HTMLCanvasElement;
  // oxlint-disable-next-line no-unassigned-vars -- assigned by Svelte bind:this
  let shadowHost: HTMLDivElement;
  let lastReference: ElementReference | undefined;

  function inspect(): void {
    startInspecting({
      adapters: [adapter],
      keyboardShortcut: false,
      onSelect: (reference) => {
        lastReference = reference;
      },
    });
  }

  onMount(() => {
    const context = canvas.getContext("2d");
    if (context) {
      for (let y = 0; y < canvas.height; y += 16) {
        for (let x = 0; x < canvas.width; x += 16) {
          context.fillStyle = (x + y) % 32 === 0 ? "#30343a" : "#24282d";
          context.fillRect(x, y, 16, 16);
        }
      }
    }

    const gl = webgl.getContext("webgl");
    if (gl) {
      gl.clearColor(0.03, 0.48, 0.92, 1);
      gl.clear(gl.COLOR_BUFFER_BIT);
    }

    const shadow = shadowHost.attachShadow({ mode: "open" });
    shadow.innerHTML = `<style>button{border:1px solid #626970;background:#171a1e;color:#e5e7eb;padding:6px 10px}</style><button type="button">Shadow DOM</button>`;

    let unlisten: (() => void) | undefined;
    void installInspectorBridge({
      adapters: [adapter],
      onSelect: (reference) => {
        lastReference = reference;
      },
    }).then((stop) => {
      unlisten = stop;
      document.documentElement.dataset.inspectorReady = "true";
    });
    return () => {
      delete document.documentElement.dataset.inspectorReady;
      unlisten?.();
    };
  });
</script>

<svelte:head>
  <meta name="color-scheme" content="dark" />
</svelte:head>

<header class="toolbar">
  <CreateWorkspaceButton onclick={() => undefined} />
  <button class="secondary nested" type="button">Nested <span>span</span></button>
  <button class="secondary inspect" type="button" onclick={inspect} aria-label="Inspect">
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="m5 3 13 9-6 1-3 6z" />
    </svg>
    Inspect
  </button>
  {#if lastReference}
    <output class="reference" aria-live="polite">@{lastReference.id}</output>
  {/if}
</header>

<main>
  <section class="forms" aria-label="Form fixtures">
    <label for="fixture-input">Input</label>
    <input id="fixture-input" value="Sample value" />

    <label for="fixture-textarea">Textarea</label>
    <textarea id="fixture-textarea">Multiple lines</textarea>

    <label class="check"><input type="checkbox" checked /> Enabled</label>

    <button class="fixture-button" type="button" onclick={() => dialog.showModal()}>
      Open dialog
    </button>
    <button class="fixture-button" type="button" popovertarget="fixture-popover">
      Open popover
    </button>
    <button class="fixture-button tooltip" type="button" data-tooltip="Tooltip content">
      Show tooltip
    </button>

    <details class="dropdown">
      <summary>Dropdown</summary>
      <button type="button">Dropdown action</button>
    </details>
  </section>

  <section class="workbench" aria-label="Visual fixtures">
    <div class="top-fixtures">
      <div>
        <h2>Scroll container</h2>
        <div class="scroll">
          {#each Array(8) as _, index}
            <div class="scroll-row" aria-label={`Row ${index + 1}`}></div>
          {/each}
          <button class="scroll-boundary" type="button">Scroll boundary</button>
        </div>
      </div>
      <div>
        <h2>Canvas</h2>
        <canvas bind:this={canvas} width="320" height="264"></canvas>
      </div>
    </div>

    <div class="bottom-fixtures">
      <div>
        <h2>Small</h2>
        <button class="small" type="button" aria-label="Very small element"></button>
        <canvas class="webgl" bind:this={webgl} width="32" height="32" aria-label="WebGL"></canvas>
        <div class="zoom" aria-label="CSS zoom">Zoom</div>
        <div bind:this={shadowHost} class="shadow-host"></div>
      </div>
      <div>
        <h2>Absolute</h2>
        <div class="relative-boundary">
          <button type="button">Absolute</button>
        </div>
      </div>
      <div>
        <h2>Transformed</h2>
        <button class="transformed" type="button">Transformed</button>
      </div>
    </div>
  </section>
</main>

<button class="fixed" type="button">Fixed</button>
<button class="offscreen" type="button">Off-screen</button>

<dialog bind:this={dialog}>
  <form method="dialog">
    <strong>Dialog</strong>
    <button type="submit">Close</button>
  </form>
</dialog>

<div id="fixture-popover" class="popover" popover bind:this={popover}>
  Popover
  <button type="button" onclick={() => popover.hidePopover()}>Close</button>
</div>

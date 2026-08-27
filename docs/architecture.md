# Architecture

## Design target

The inspector has to join two worlds without confusing them. JavaScript owns the DOM node, accessibility tree, and framework development metadata. Rust owns the native window, physical pixels, local persistence, and agent IPC.

The split matters because a DOM renderer cannot reproduce every pixel the webview displayed, while a native screenshot API cannot identify a Svelte component or accessible name.

The verified single-element flow is:

```text
CLI pick
  -> authenticated project-local socket or named pipe
  -> Tauri window selection
  -> ui-inspector://pick
  -> frontend picker
  -> DOM, accessibility, locators, framework metadata
  -> capture_selection command
  -> xcap native window bitmap
  -> CSS/native/bitmap coordinate transform
  -> image crop
  -> backend redaction
  -> atomic reference directory
  -> CLI response with @ui_<ULID>
```

Live resolution loads the stored reference, asks the chosen webview to try its strong locators, checks the original signature, and returns `notFound` if exactly one element cannot be proven.

## Responsibility map

| Owner | Responsibility | Explicit exclusions | Main dependencies | Exposed surface |
| --- | --- | --- | --- | --- |
| `tauri-ui-inspector-core` | schema, IDs, coordinates, cropping, redaction, storage, wire types | Tauri, DOM, framework runtimes | `serde`, `ts-rs`, `ulid`, `image`, `fs4` | Rust types used by plugin and CLI |
| `tauri-plugin-ui-inspector` | commands, window matching, native capture, lifecycle, local IPC | DOM traversal, Svelte, agent policy | Tauri 2, `xcap`, `interprocess` | Tauri plugin builder and events |
| `@tauri-ui-inspector/inspector` | picker, overlay, DOM/ARIA collection, locators, exact resolution, Tauri bridge | native pixels, framework-specific probes | Tauri JS API, `dom-accessibility-api`, `@medv/finder` | framework-neutral TypeScript API |
| `@tauri-ui-inspector/adapter-svelte` | Svelte 5 development source recovery | capture, persistence, picker UI | `element-source` | `svelteAdapter()` |
| `@tauri-ui-inspector/adapter-react` | React development source recovery | capture, persistence, picker UI | `element-source` | `reactAdapter()` |
| `@tauri-ui-inspector/adapter-vue` | Vue 3 development source recovery | capture, persistence, picker UI | `element-source` | `vueAdapter()` |
| `tauri-ui-inspector` | command grammar, project discovery, JSON/human output, exit codes | capture implementation, DOM logic | `clap`, `interprocess` | `ui-inspector` binary |
| `skills/ui-inspector` | agent procedure for consuming a reference | capture logic, Codex coupling in core | CLI and local files | Codex skill |
| `examples/svelte-tauri` | visual fixtures and native E2E | reusable plugin behavior | Svelte 5, Vite, Tauri, WebdriverIO | development example |

The Tauri plugin composes capture, storage, callbacks, and IPC in `capture_selection`. The frontend bridge composes picker selection with metadata collection. No sibling reaches through another sibling's private representation.

## Physical layout

The repository uses a layer-first workspace because the Rust process boundary, browser package boundary, and framework adapter boundary each have different dependencies and release artifacts.

```text
crates/
  ui-inspector-core/
  ui-inspector-plugin/
  ui-inspector/
packages/
  shared/
  inspector/
  adapter-svelte/
  adapter-react/
  adapter-vue/
examples/
  svelte-tauri/
skills/
  ui-inspector/
docs/
  mockups/
  screenshots/
```

The current files stay split by present ownership:

- `coordinate.rs`, `storage.rs`, `redaction.rs`, and `protocol.rs` protect different invariants and have separate tests.
- `capture.rs` isolates native screenshot dependencies from command orchestration.
- `ipc.rs` owns authentication, framing, timeouts, and process crossing.
- `picker.ts`, `metadata.ts`, `locators.ts`, and `resolve.ts` each own a browser behavior with direct tests.
- `dom.ts` contains the open-shadow traversal shared by collection and resolution.
- generated TypeScript stays in `packages/shared/src/generated.ts`; handwritten exports stay outside it.

No platform directories exist yet because `xcap` owns the platform implementations. Add private `capture/<platform>.rs` files only if a platform needs substantial policy beyond `xcap`.

## Reference schema

Rust owns schema version 1. `ts-rs` exports the TypeScript declarations, so the two languages do not carry handwritten copies.

The top-level record uses `kind` plus `schemaVersion`. `element` is implemented. `group` and `region` reserve the discriminator for later schema versions, but no API creates them yet.

Collection limits keep the record useful without dumping a page:

- 200 normalized text characters;
- 4,000 characters of selected HTML;
- 6,000 characters of parent HTML;
- 8 DOM ancestors;
- 50 selected attributes with 500 characters per value.

Serde ignores unknown object fields by default. Readers should branch on `schemaVersion`; a breaking field or variant change requires a new version.

## Picker state machine

The picker has three states:

```text
idle -> inspecting -> capturing -> idle
  ^          |
  +----------+ Escape or stop
```

`inspecting` installs capture-phase listeners, a pointer-transparent shadow-DOM overlay, and a crosshair cursor. Pointer movement chooses the first interactive element in the composed path, which handles nested text, SVG descendants, and interactive shadow hosts.

The click listener prevents default behavior and stops propagation before entering `capturing`. The overlay hides before native capture; listeners and frozen animations stay in place until capture finishes. Cleanup removes every listener and temporary node, then resumes only animations that were running.

The overlay uses `getBoundingClientRect()`, so scroll containers and CSS transforms share the browser's own geometry. Pinch-zoom offsets travel with the selection payload.

Open shadow roots are traversable. Closed roots remain private by browser design and resolve only at their exposed host.

## Accessibility and privacy boundary

`dom-accessibility-api` computes role, accessible name, and accessible description. Handwritten collection is limited to direct ARIA relationships and native control states that the reference schema needs.

Form values are opt-in. Password, hidden, password-autocomplete, one-time-code, credit-card, and token-like fields always lose their value. Sensitive attribute names are redacted in cloned HTML and structured attributes.

The Rust backend repeats redaction before callbacks and persistence. This is a trust-boundary check: the webview payload is untrusted input even when the frontend package normally created it.

Screenshot capture has its own switch because DOM rules cannot remove secrets already rendered as pixels.

## Locator and resolution policy

The ranking is deterministic:

1. test ID;
2. unique role plus accessible name;
3. unique DOM ID;
4. stable attribute;
5. source component/location;
6. CSS selector;
7. DOM path;
8. exact text.

`@medv/finder` generates CSS. Custom code assigns confidence, tests uniqueness, records semantic locators, and supplies a diagnostic structural fallback. Explicit selectors and semantic matching traverse open shadow roots.

The resolver tries only locators that were unique at capture time and have confidence at least `0.5`. It then compares tag, role, and accessible name. A mismatch returns a structured failure. DOM paths and text stay in the record for investigation but cannot silently reacquire an element.

## Framework adapters

The Svelte, React, and Vue packages call the corresponding resolver from `element-source`. A shared, framework-neutral conversion maps source frames into the inspector schema and removes a duplicate child-component call-site frame.

The adapter returns `undefined` when metadata is absent. Production builds therefore keep inspection, locators, and screenshots without claiming a source location.

No framework internal name leaks into the core package or Tauri plugin.

## Native capture and coordinates

`xcap` captures the window. The plugin filters candidates by the current PID, then ranks exact title matches before outer-size distance. The protocol selects a Tauri webview by label before capture, so multiple windows remain deterministic.

The transform joins four measured spaces:

```text
DOMRect in CSS viewport pixels
  -> webview content in native screen units
  -> captured window bounds
  -> PNG pixels
```

For each axis:

```text
content_scale = content_native_size / css_viewport_size
capture_scale = png_size / native_window_size

png_edge = (
  content_origin
  + (css_edge + visual_viewport_offset) * content_scale
  - native_window_origin
) * capture_scale
```

Left and top round down; right and bottom round up. Padding is applied in CSS pixels before scaling. The result is clamped to the PNG, and a disjoint rectangle fails.

Tauri can report identical inner and outer geometry on macOS even when the captured bitmap contains a titlebar. `calibrate_content_from_viewport` detects this from `innerWidth × devicePixelRatio`, `innerHeight × devicePixelRatio`, the native bounds, and the bitmap density. Platforms that report a real inner origin keep it.

Pure tests cover negative monitor coordinates, decorations, 2× captures, different viewport scales, page zoom, padding, partial clipping, and disjoint elements. Native E2E on macOS exercised an external 1× monitor with negative Y coordinates and the built-in Retina display at 2×.

## Persistence

`Storage::save` acquires a filesystem lock, writes JSON and PNGs into a temporary directory under `refs`, and renames that directory into place. It never overwrites an existing ID. ULID lexical order supplies newest-first listing and bounded cleanup.

`fs4` owns cross-platform file locking. `tempfile` owns temporary directory cleanup. A max history of zero means unlimited retention; the default is 100.

The running-instance discovery file lives under `run/` and does not count toward history. Normal Tauri exit removes it only if its PID still belongs to the current process. A stale file produces CLI exit code 3 when the endpoint cannot connect.

## Local IPC

`interprocess` supplies Unix local sockets and Windows named pipes. Each run creates a namespaced endpoint and a secret with 160 random ULID bits. The project-local instance file stores the endpoint, token, PID, roots, protocol version, and start time. Unix permissions are set to `0600`.

Messages are newline-delimited JSON capped at 64 KiB. The server checks protocol version and token before dispatch. One pick or resolve operation may wait at a time. A two-minute default timeout clears abandoned operations.

No TCP socket, HTTP server, public bind address, telemetry, or upload client exists.

## Dependency audit

Versions were checked against crates.io and npm on 2026-08-20.

| Concern | Library | Why custom code remains |
| --- | --- | --- |
| Tauri plugin | Tauri 2.11.5, `tauri-plugin` 2.6.3 | commands, events, product policy |
| Window pixels | `xcap` 0.9.8 | match one Tauri window and add context |
| PNG crop | `image` 0.25.10 | browser-to-bitmap rectangle |
| IDs | `ulid` 3.0.0 | `ui_` and optional `@` wrapper |
| Storage locking | `fs4` 1.1.0 | reference directory transaction and retention |
| Temporary writes | `tempfile` 3.27.0 | final rename and schema layout |
| Schema | `serde` 1.0.229, `ts-rs` 12.0.1 | versioned domain fields |
| Local IPC | `interprocess` 2.4.3 | authentication and request semantics |
| CLI | `clap` 4.6.6 | command behavior and exit mapping |
| Time | `jiff` 0.2.35 | schema timestamp placement |
| Accessibility | `dom-accessibility-api` 0.7.1 | choose compact persisted fields |
| CSS selectors | `@medv/finder` 4.0.2 | non-CSS locator ranking and confidence |
| Framework source | `element-source` 0.0.5 | common schema conversion |
| E2E | Vitest 4.1.11, WebdriverIO 9.31.3, and Tauri service 1.3.0 | fixture assertions and CLI orchestration |

The Rust workspace uses edition 2024, resolver 3, and Rust 1.97 as both the pinned toolchain and MSRV. Node 26.5.1 and pnpm 11.18.0 are pinned through mise and `packageManager`.

The example keeps TypeScript 6.0.3 for `svelte-check`, whose current peer range ends at TypeScript 6. It also installs the TypeScript 7.0.2 native compiler as `@typescript/native` and runs `svelte-check --tsgo`. This is an upstream compatibility boundary, not a stale runtime.

`@wdio/tauri-service` 1.3.0 pins internal WebdriverIO 9.29/9.30 packages while the standalone Vitest suite uses WebdriverIO 9.31.3. The service accepts WebdriverIO 9. Its peer exception covers only `expect-webdriverio`, which the standalone session never imports. The E2E job exercises the embedded driver.

## Release architecture

Release Please treats the three crates and five npm packages as one linked version. The Tauri plugin omits its component name from the tag, so `v<version>` is the only tag that starts the publish workflow; component tags still provide package-specific GitHub releases.

The core crate publishes before the plugin and CLI. The shared npm package publishes before the inspector, followed by the Svelte, React, and Vue adapters. pnpm creates tarballs first so `workspace:*` dependencies become concrete release versions before npm sees them. crates.io and npm trusted publishing use short-lived GitHub OIDC credentials after the bootstrap release.

The initial release is manual because trusted publisher records cannot target packages that do not exist yet. CI verifies the core crate tarball and lists the plugin and CLI package contents; Cargo cannot assemble those two tarballs until the same-version core crate has reached the crates.io index.

## Primary research

- [Tauri 2 plugin development](https://v2.tauri.app/develop/plugins/)
- [Tauri WebviewWindow API](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindow.html)
- [Tauri plugin workspace](https://github.com/tauri-apps/plugins-workspace)
- [WebdriverIO Tauri configuration](https://webdriver.io/docs/desktop-testing/tauri/configuration/)
- [WebdriverIO Tauri plugin setup](https://webdriver.io/docs/desktop-testing/tauri/plugin-setup/)
- [`xcap` source](https://github.com/nashaofu/xcap)
- [`interprocess` local sockets](https://docs.rs/interprocess/latest/interprocess/local_socket/)
- [`image` crate](https://docs.rs/image/latest/image/)
- [`ulid` crate](https://docs.rs/ulid/latest/ulid/)
- [`ts-rs`](https://docs.rs/ts-rs/latest/ts_rs/)
- [`dom-accessibility-api`](https://github.com/eps1lon/dom-accessibility-api)
- [`@medv/finder`](https://github.com/antonmedv/finder)
- [`element-source`](https://github.com/mattpocock/element-source)
- [`svelte-grab`](https://github.com/mattpocock/svelte-grab)
- [Svelte 5 documentation](https://svelte.dev/docs/svelte/overview)
- [Codex skills](https://developers.openai.com/codex/skills/)

## Alternatives declined

DOM-to-image libraries cannot be the primary screenshot path because they reconstruct DOM-owned rendering and may miss WebGL, canvas, fonts, compositor effects, or native webview behavior.

SQLite would add migrations and another persistence layer while the real payload remains PNG files. Atomic directories match the read/write pattern.

An unauthenticated localhost service would add a network surface, port management, and origin policy without helping the local CLI.

A browser automation locator engine would add a larger runtime than the in-process exact resolver needs. The current design reuses selector and accessibility libraries, then keeps only product-specific ranking and refusal logic.

Platform-specific screenshot code would duplicate `xcap`. Add it only if a supported platform demonstrates a failure that `xcap` cannot address.

## Extensions

Multi-selection should collect several existing `ElementInfo` values and capture one window bitmap after the final Shift-click. Each crop should come from that shared bitmap. The single-element state machine and locator policy should remain unchanged.

Region selection should reuse `CoordinateTransform` with a CSS rectangle and a region-specific schema body. It should not create a fake DOM element.

Solid and future adapters should return the current `SourceInfo`. Framework runtime probes belong in their adapter packages.

## Verification boundary

The repository has native macOS evidence, including 1×, Retina 2×, titlebar calibration, and negative monitor coordinates. The E2E test persists both PNGs, checks source metadata, compares every crop pixel, parses CLI JSON, and performs live resolution.

Windows and Linux use the same public `xcap` and `interprocess` backends. Pure and compile-time checks cover shared code, but native screenshot permissions and compositor behavior require real platform sessions. Wayland may deny capture or require portal consent; the plugin surfaces that failure instead of writing a misleading image.

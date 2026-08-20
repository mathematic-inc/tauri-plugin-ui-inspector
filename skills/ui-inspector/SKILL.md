---
name: ui-inspector
description: Resolve and use durable @ui_ references created by tauri-plugin-ui-inspector. Use when a coding request names @ui_..., ui_..., a selected Tauri UI element, an inspector screenshot, or asks to fix the element the developer picked.
---

# UI inspector references

Use the recorded reference. Do not infer the element from the user's description when an `@ui_` identifier is available.

## Resolve the reference

1. Work from the repository that owns `.ui-inspector/`.
2. Normalize an optional leading `@`; the CLI accepts either form.
3. Run:

   ```sh
   ui-inspector get @ui_01... --json
   ```

4. Treat stdout as JSON. Do not mix diagnostics into stdout or scrape the human format.
5. If exit code 2 reports a missing reference, stop and ask for the correct project or ID. Do not choose a similar reference.

Read these fields first:

- `summary`
- `element.role` and `element.accessibleName`
- `source.location` and `source.component`
- `element.locators`
- `screenshots.element` and `screenshots.window`
- `dom.ancestry`

## Inspect the pixels

Resolve absolute screenshot paths with:

```sh
ui-inspector screenshot @ui_01... --json
```

Open `element.png` before editing. Open `window.png` when spacing, alignment, layering, or consistency depends on surrounding controls. If screenshot fields are null, rely on structured metadata and say that pixels were unavailable.

Never upload the screenshots or reference JSON. They may contain private interface data.

## Locate source

Open `source.location.file` at the recorded line and column when present. Confirm that the source component, semantic role, accessible name, and local markup agree with the reference before editing.

Source metadata is optional. If it is absent:

1. Search the project for the strongest locator, starting with a test ID.
2. Use role and accessible name next.
3. Use stable ID or attributes next.
4. Use the generated CSS selector only as supporting evidence.
5. Do not use a weak DOM path or text match as proof that you found the right source.

## Edit and verify

Make the smallest source change that addresses the request. Run the component's normal checks.

If the Tauri app is running, verify that the original element still resolves:

```sh
ui-inspector resolve @ui_01... --json
```

Exit code 5 means the element no longer resolves exactly. Inspect the change instead of accepting a nearby match.

For visual changes, ask the developer to capture a fresh reference after reload, or use the application's programmatic inspection hook. Compare the new element crop with the old crop and inspect the full window when context matters.

## CLI failures

| Exit | Action |
| ---: | --- |
| 2 | Reference is absent. Check the project and ID. |
| 3 | App is not running or the inspector is disabled. Start the debug app. |
| 4 | The developer cancelled selection. Stop without guessing. |
| 5 | Exact live resolution failed. Inspect locators and source changes. |

Honor redaction markers. Never reconstruct a password, token, form value, or hidden text from nearby data.

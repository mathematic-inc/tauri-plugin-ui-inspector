# Picker mockup prompts

These are the normalized prompts used to generate the approved idle and inspecting states in `docs/mockups/`. `manifest.json` records the chosen outputs and exact geometry. Independent review approved the mockups before implementation; a second reviewer approved the coded 1× and Retina 2× captures in `docs/screenshots/`.

## `fixture-idle`

```text
Use case: ui-mockup
Asset type: comparison-grade desktop Tauri webview fixture page, stable label fixture-idle
Primary request: Create a polished developer fixture page whose only purpose is testing UI element capture. Exact intended CSS viewport: 1280 × 800, 16:10 landscape. Show the page straight-on, edge-to-edge.

Scene/backdrop: one neutral near-black developer-tooling canvas with a slim top toolbar and a practical sparse test area below. No sidebar, browser chrome, device frame, outer padding, marketing shell, or presentation frame.
Style/medium: realistic production UI screenshot, neutral dark developer tooling, flat surfaces, high contrast, restrained cool-gray palette, one accessible cyan-blue primary action, crisp sans-serif UI type, monospace only where useful. No gradients, glass, shadows, ornamental cards, illustrations, or decoration.

Composition/framing:
- Top toolbar around 64 px high. At its left, a primary button exactly 184 × 40 labeled verbatim "Create workspace". For the selected 1586 × 992 raster representing the 1280 × 800 CSS viewport, its exact raster bounds are x=18, y=40, width=228, height=50 pixels. The button is unchanged idle state with clear accessible contrast.
- In the same toolbar, include a secondary button whose visible text is exactly "Nested span"; its text should visibly include a small nested inline span treatment without adding words. Include a compact square SVG icon button with a simple outline inspect/cursor icon and a visible concise accessible label "Inspect".
- Main area uses a simple two-column workbench separated by spacing and fine dividers, not cards.
- Left column: literal form fixtures in a tidy vertical stack: label "Input" above a single-line input containing "Sample value"; label "Textarea" above a textarea containing "Multiple lines"; checkbox labeled "Enabled"; buttons "Open dialog", "Open popover", and "Show tooltip", with the tooltip target visually focusable but tooltip closed.
- Right column: a bordered scroll container labeled "Scroll container" with a few plain rows and an internal scrollbar; a canvas fixture shown as a dark checker/grid rectangle labeled "Canvas"; a visibly tiny interactive element labeled "Small"; a fixed-position fixture anchored near the lower-right labeled "Fixed"; an absolute-position fixture inside a simple relative boundary labeled "Absolute"; and a subtly rotated transformed fixture labeled "Transformed".
- Include a partially off-screen fixture at the far right edge so only part of its rectangle and the beginning of the literal label "Off-screen" are visible, clearly clipped by the viewport.
- Every required fixture must be visible and visually distinct at once without crowding.

Text (verbatim, render exactly):
"Create workspace"
"Nested span"
"Inspect"
"Input"
"Sample value"
"Textarea"
"Multiple lines"
"Enabled"
"Scroll container"
"Open dialog"
"Open popover"
"Show tooltip"
"Canvas"
"Small"
"Fixed"
"Absolute"
"Transformed"
"Off-screen"

Constraints:
- Practical fixture UI, not a dashboard and not a component catalog poster.
- No page title, hero heading, subtitle, description, instructional paragraph, badge, status metric, navigation, breadcrumbs, footer, source code, invented brand, logo, watermark, or extra explanatory copy.
- Do not open the dialog, popover, dropdown, or tooltip in this idle state.
- The fixed, absolute, transformed, canvas, small, scroll, checkbox, input, textarea, buttons, icon button, nested-span button, and clipped off-screen fixtures each need one unique test purpose.
- Keep labels concise and literal. No duplicate containers. Use spacing, alignment, typography, and fine rules before borders.
- Straight-on edge-to-edge 16:10 screenshot with square canvas corners; no perspective, drop-shadowed frame, exterior background, or device chrome.
Output intent: high-fidelity raster UI mockup for exact CSS viewport 1280 × 800, stable state fixture-idle.
```

## `fixture-inspecting`

```text
Use case: ui-mockup
Asset type: precise edit of a comparison-grade desktop Tauri webview fixture page, stable label fixture-inspecting
Input images: Image 1 is the selected fixture-idle mockup. Edit Image 1; it defines the binding page layout, controls, typography, colors, spacing, fixture positions, clipping, and exact 1280 × 800 CSS-viewport intent.
Primary request: Change only the inspection state. Show a pointer conceptually hovering the existing primary button labeled exactly "Create workspace" and draw an inspector hover overlay above the page without changing, moving, tinting, pressing, or obscuring the button.

Overlay specification:
- Add a crisp bright cyan inspection outline exactly around the full 184 × 40 CSS bounds, or 228 × 50 raster bounds, of the existing "Create workspace" button. The outline sits just outside the button edge and does not cover its fill, border, or label.
- Add one compact dark overlay label immediately above and left-aligned with the outlined button, visually attached to the target, fully inside the viewport, and without overlapping the button. For the selected raster, use x=18, y=2, width=228, height=34 pixels. The label contains exactly two readable lines: "CreateWorkspaceButton" and "184 × 40". Use high-contrast compact monospace text large enough to remain clearly legible when normalized to 1280 × 800, with the semantic component identity first and dimensions second.
- Show a precise crosshair cursor near the button's center-right area, conceptually hovering it, while keeping all button text readable.
- Overlay layer must read as inspector tooling drawn above the unchanged page. No dimmer, mask, selection wash, tooltip arrow, handles, rulers, guides, DOM tree, side panel, or measurement lines.

Text added verbatim, render exactly once:
"CreateWorkspaceButton"
"184 × 40"

Invariants:
- Preserve Image 1 pixel-for-pixel except for the inspection outline, compact label, and crosshair cursor.
- Preserve the unchanged primary button text "Create workspace" and its exact 184 × 40 size, position, color, and idle visual styling.
- Preserve all existing fixtures and exact visible strings: "Nested span", "Inspect", "Input", "Sample value", "Textarea", "Multiple lines", "Enabled", "Scroll container", "Open dialog", "Open popover", "Show tooltip", "Canvas", "Small", "Fixed", "Absolute", "Transformed", and the partially clipped "Off-screen" fixture.
- Keep dialog, popover, dropdown, and tooltip closed.
- Keep the exact straight-on, edge-to-edge 16:10 composition and exact 1280 × 800 CSS-viewport intent.

Constraints:
- This is a surgical state edit, not a redesign.
- Add only the bright outline, compact two-line overlay label, and crosshair cursor.
- Overlay must not obscure the selected button or alter document layout.
- No page title, heading, description, badge, status copy, inspector panel, extra labels, extra controls, watermark, device chrome, exterior padding, perspective, or presentation frame.
Output intent: high-fidelity raster UI mockup for exact CSS viewport 1280 × 800, stable state fixture-inspecting hover.
```

The selected outputs are stored under `docs/mockups/`. Regeneration must use new stable labels and pass the same independent visual review before replacing them.

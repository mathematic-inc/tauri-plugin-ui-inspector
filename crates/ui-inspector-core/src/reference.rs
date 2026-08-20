//! Versioned, compact UI reference schema.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CssRect, CssSize, PhysicalPoint, PhysicalSize, PixelRect};

/// Current serialized reference schema version.
pub const REFERENCE_SCHEMA_VERSION: u32 = 1;

/// The captured subject represented by a reference.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub enum ReferenceKind {
    /// A single DOM element.
    Element,
    /// Reserved for future multi-element capture.
    Group,
    /// Reserved for future arbitrary region capture.
    Region,
}

/// Project metadata attached to a reference.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct ProjectInfo {
    /// Project root recorded by the plugin, when configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
}

/// Browser viewport metrics recorded at selection time.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct ViewportInfo {
    /// Layout viewport size in CSS pixels.
    pub size: CssSize,
    /// Browser-reported device pixel ratio.
    pub device_pixel_ratio: f64,
    /// Visual viewport data when supported by the webview.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visual_viewport: Option<VisualViewportInfo>,
}

/// Browser visual viewport metrics used during pinch zoom.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VisualViewportInfo {
    /// Horizontal layout-viewport offset in CSS pixels.
    pub offset_left: f64,
    /// Vertical layout-viewport offset in CSS pixels.
    pub offset_top: f64,
    /// Visual viewport scale.
    pub scale: f64,
    /// Visual viewport width in CSS pixels.
    pub width: f64,
    /// Visual viewport height in CSS pixels.
    pub height: f64,
}

/// Native and browser geometry for the selected Tauri window.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct WindowInfo {
    /// Tauri window label.
    pub label: String,
    /// Native window title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Tauri scale factor at capture time.
    pub scale_factor: f64,
    /// Native outer-window position.
    pub outer_position: PhysicalPoint,
    /// Native content-area position.
    pub inner_position: PhysicalPoint,
    /// Native outer-window size.
    pub outer_size: PhysicalSize,
    /// Native content-area size.
    pub inner_size: PhysicalSize,
    /// Browser viewport metrics.
    pub viewport: ViewportInfo,
}

/// A ranked stable locator for reacquiring the element.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct Locator {
    /// Locator strategy.
    pub strategy: LocatorStrategy,
    /// Primary locator value.
    pub value: String,
    /// Attribute name for attribute-based locators.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<String>,
    /// Accessible name for role locators.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Deterministic confidence score between 0 and 1.
    pub confidence: f32,
    /// Whether the locator was unique at selection time.
    pub unique: bool,
}

/// Strategy used by a stable locator.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub enum LocatorStrategy {
    /// Explicit test identifier such as `data-testid`.
    TestId,
    /// Unique semantic role and accessible name.
    Role,
    /// Unique DOM id.
    Id,
    /// Stable attribute selector.
    Attribute,
    /// Development source component and location.
    Source,
    /// Generated CSS selector.
    Css,
    /// Structural DOM path.
    DomPath,
    /// Exact normalized text match.
    Text,
}

/// Convenient top-ranked selector fields for simple consumers.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct SelectorSummary {
    /// Highest-confidence selector-like value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred: Option<String>,
    /// Generated CSS selector.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css: Option<String>,
    /// Explicit test identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_id: Option<String>,
    /// Unique element id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Semantic role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Short normalized text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Accessibility state useful to coding agents.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct AccessibilityInfo {
    /// Computed semantic role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Computed accessible name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Computed accessible description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// `aria-label` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aria_label: Option<String>,
    /// `aria-labelledby` token list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aria_labelled_by: Option<String>,
    /// `aria-describedby` token list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aria_described_by: Option<String>,
    /// Disabled state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// Checked state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    /// Selected state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    /// Expanded state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expanded: Option<bool>,
    /// Pressed state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed: Option<bool>,
    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// Associated form label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form_label: Option<String>,
    /// Form control input type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
    /// Form value, present only when explicitly allowed and not sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Compact metadata for the selected element.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct ElementInfo {
    /// Lowercase DOM tag name.
    pub tag_name: String,
    /// Namespace URI for SVG and other non-HTML elements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Short normalized visible text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Computed semantic role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Computed accessible name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessible_name: Option<String>,
    /// Selected attributes after redaction.
    pub attributes: BTreeMap<String, String>,
    /// Element rectangle in CSS viewport coordinates.
    pub rect: CssRect,
    /// Stable locators ordered from strongest to weakest.
    pub locators: Vec<Locator>,
    /// Convenient top-ranked selectors.
    pub selectors: SelectorSummary,
    /// Accessibility metadata.
    pub accessibility: AccessibilityInfo,
}

/// Source location recovered from framework development metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct SourceLocation {
    /// Source file path as emitted by the framework toolchain.
    pub file: String,
    /// One-based source line when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// One-based source column when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

/// A component frame in the source ancestry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct SourceComponent {
    /// Framework component name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    /// Component source location.
    pub location: SourceLocation,
}

/// Optional framework and source-code metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct SourceInfo {
    /// Framework adapter name.
    pub framework: String,
    /// Closest component name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    /// Closest source location.
    pub location: SourceLocation,
    /// Parent component frames, nearest first.
    pub ancestry: Vec<SourceComponent>,
}

/// Compact DOM ancestor metadata.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct DomAncestor {
    /// Lowercase tag name.
    pub tag_name: String,
    /// Unique id when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Short class list.
    pub classes: Vec<String>,
    /// Semantic role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Accessible name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessible_name: Option<String>,
}

/// Bounded HTML and ancestry around the selected element.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct DomContext {
    /// Sanitized, truncated selected-element HTML.
    pub html: String,
    /// Sanitized, truncated parent HTML.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_html: Option<String>,
    /// Ancestors from parent to root, capped by the frontend.
    pub ancestry: Vec<DomAncestor>,
}

/// Screenshot filenames relative to the reference directory.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct ScreenshotInfo {
    /// Full native window screenshot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<String>,
    /// Selected element crop.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
}

/// Capture geometry retained for troubleshooting and validation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct CaptureInfo {
    /// Padding applied around the element in CSS pixels.
    pub padding: u32,
    /// Pixel crop within `window.png`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_crop: Option<PixelRect>,
    /// Actual screenshot bitmap size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot_size: Option<PhysicalSize>,
}

/// Frontend payload sent to the Tauri backend for native capture.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct SelectionPayload {
    /// Browser viewport metrics.
    pub viewport: ViewportInfo,
    /// Selected element metadata.
    pub element: ElementInfo,
    /// Optional source metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceInfo>,
    /// Bounded DOM context.
    pub dom: DomContext,
}

/// A durable reference to a selected frontend element.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields = nullable)]
pub struct ElementReference {
    /// Schema version used to serialize this reference.
    pub schema_version: u32,
    /// Captured subject type.
    pub kind: ReferenceKind,
    /// Opaque `ui_<ULID>` identifier.
    pub id: String,
    /// RFC 3339 timestamp.
    pub created_at: String,
    /// Deterministic, agent-oriented summary.
    pub summary: String,
    /// Project metadata.
    pub project: ProjectInfo,
    /// Window and viewport metadata.
    pub window: WindowInfo,
    /// Selected element metadata.
    pub element: ElementInfo,
    /// Optional framework source mapping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceInfo>,
    /// Bounded DOM context.
    pub dom: DomContext,
    /// Screenshot filenames.
    pub screenshots: ScreenshotInfo,
    /// Native capture geometry.
    pub capture: CaptureInfo,
}

/// Builds a compact summary from structured metadata.
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
/// use tauri_ui_inspector_core::{
///     AccessibilityInfo, CssRect, ElementInfo, SelectorSummary, SourceInfo,
///     SourceLocation, deterministic_summary,
/// };
///
/// let element = ElementInfo {
///     tag_name: "button".into(), namespace: None,
///     text: Some("Create workspace".into()), role: Some("button".into()),
///     accessible_name: Some("Create workspace".into()),
///     attributes: BTreeMap::new(), rect: CssRect::default(), locators: vec![],
///     selectors: SelectorSummary::default(), accessibility: AccessibilityInfo::default(),
/// };
/// let source = SourceInfo {
///     framework: "svelte".into(), component: Some("CreateWorkspaceButton".into()),
///     location: SourceLocation { file: "src/CreateWorkspaceButton.svelte".into(), line: Some(47), column: Some(3) },
///     ancestry: vec![],
/// };
/// assert_eq!(deterministic_summary(&element, Some(&source)),
///     "CreateWorkspaceButton: button 'Create workspace' at src/CreateWorkspaceButton.svelte:47:3");
/// ```
#[must_use]
pub fn deterministic_summary(element: &ElementInfo, source: Option<&SourceInfo>) -> String {
    let subject = source
        .and_then(|source| source.component.as_deref())
        .unwrap_or(&element.tag_name);
    let role = element.role.as_deref().unwrap_or(&element.tag_name);
    let name = element
        .accessible_name
        .as_deref()
        .or(element.text.as_deref())
        .map(|name| format!(" '{name}'"))
        .unwrap_or_default();
    let location = source.map(|source| {
        let mut value = format!(" at {}", source.location.file);
        if let Some(line) = source.location.line {
            value.push(':');
            value.push_str(&line.to_string());
            if let Some(column) = source.location.column {
                value.push(':');
                value.push_str(&column.to_string());
            }
        }
        value
    });
    format!("{subject}: {role}{name}{}", location.unwrap_or_default())
}

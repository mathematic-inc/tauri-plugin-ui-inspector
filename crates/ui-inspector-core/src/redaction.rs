//! Defense-in-depth redaction for data arriving from the webview.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{ElementReference, LocatorStrategy, deterministic_summary};

const REDACTED: &str = "[redacted]";
const DEFAULT_SENSITIVE_NAMES: &[&str] = &[
    "authorization",
    "cookie",
    "password",
    "secret",
    "token",
    "value",
];
const IDENTIFYING_ATTRIBUTES: &[&str] =
    &["name", "id", "autocomplete", "aria-label", "placeholder"];
const SENSITIVE_AUTOCOMPLETE: &[&str] = &[
    "current-password",
    "new-password",
    "one-time-code",
    "cc-number",
    "cc-csc",
];

/// Backend redaction settings applied before persistence or callbacks.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RedactionConfig {
    /// Lowercase attribute-name fragments whose values are redacted.
    pub sensitive_attribute_fragments: Vec<String>,
    /// Whether visible text and accessible names are removed.
    pub redact_text: bool,
}

impl RedactionConfig {
    /// Creates the default privacy-preserving policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_ui_inspector_core::RedactionConfig;
    ///
    /// let config = RedactionConfig::new();
    /// assert!(config.sensitive_attribute_fragments.iter().any(|name| name == "password"));
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            sensitive_attribute_fragments: DEFAULT_SENSITIVE_NAMES
                .iter()
                .map(ToString::to_string)
                .collect(),
            redact_text: false,
        }
    }
}

impl Default for RedactionConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Applies redaction in place before the reference leaves the backend.
///
/// Password controls are always scrubbed regardless of configuration.
///
/// # Examples
///
/// ```
/// # use std::collections::BTreeMap;
/// # use tauri_ui_inspector_core::*;
/// # let mut reference = ElementReference {
/// # schema_version: 1, kind: ReferenceKind::Element, id: ReferenceId::new().to_string(), created_at: String::new(), summary: String::new(),
/// # project: ProjectInfo::default(), window: WindowInfo { label: "main".into(), title: None, scale_factor: 1.0, outer_position: PhysicalPoint::default(), inner_position: PhysicalPoint::default(), outer_size: PhysicalSize::default(), inner_size: PhysicalSize::default(), viewport: ViewportInfo { size: CssSize { width: 1.0, height: 1.0 }, device_pixel_ratio: 1.0, visual_viewport: None } },
/// # element: ElementInfo { tag_name: "input".into(), namespace: None, text: None, role: None, accessible_name: None, attributes: BTreeMap::from([("type".into(), "password".into()), ("value".into(), "hunter2".into())]), rect: CssRect::default(), locators: vec![], selectors: SelectorSummary::default(), accessibility: AccessibilityInfo { input_type: Some("password".into()), value: Some("hunter2".into()), ..AccessibilityInfo::default() } },
/// # source: None, dom: DomContext { html: "<input type=\"password\" value=\"hunter2\">".into(), parent_html: None, ancestry: vec![] }, screenshots: ScreenshotInfo::default(), capture: CaptureInfo { padding: 0, pixel_crop: None, screenshot_size: None } };
/// redact_reference(&mut reference, &RedactionConfig::new());
/// assert_eq!(reference.element.accessibility.value, None);
/// assert_eq!(reference.element.attributes["value"], "[redacted]");
/// ```
pub fn redact_reference(reference: &mut ElementReference, config: &RedactionConfig) {
    let sensitive_control = is_sensitive_control(reference, config);
    let mut redacted_attribute = false;

    for (name, value) in &mut reference.element.attributes {
        let lowercase = name.to_ascii_lowercase();
        if (sensitive_control && lowercase == "value")
            || config
                .sensitive_attribute_fragments
                .iter()
                .any(|fragment| lowercase.contains(&fragment.to_ascii_lowercase()))
        {
            redact_string(value);
            redacted_attribute = true;
        }
    }

    if sensitive_control {
        reference.element.accessibility.value = None;
        reference.element.text = None;
    }

    if config.redact_text {
        reference.element.text = None;
        reference.element.accessible_name = None;
        reference.element.accessibility.name = None;
        reference.element.accessibility.description = None;
        reference.element.accessibility.aria_label = None;
        reference.element.accessibility.placeholder = None;
        reference.element.accessibility.form_label = None;
        reference.element.selectors.preferred = None;
        reference.element.selectors.text = None;
        reference
            .element
            .locators
            .retain(|locator| locator.strategy != LocatorStrategy::Text);
        for locator in &mut reference.element.locators {
            if locator.strategy == LocatorStrategy::Role {
                locator.name = None;
            }
        }
        for name in ["alt", "aria-label", "placeholder", "title"] {
            if let Some(value) = reference.element.attributes.get_mut(name) {
                redact_string(value);
            }
        }
        for ancestor in &mut reference.dom.ancestry {
            ancestor.accessible_name = None;
        }
    }

    if sensitive_control || redacted_attribute || config.redact_text {
        redact_string(&mut reference.dom.html);
        if let Some(parent_html) = &mut reference.dom.parent_html {
            redact_string(parent_html);
        }
    }

    reference.summary = deterministic_summary(&reference.element, reference.source.as_ref());
}

fn is_sensitive_control(reference: &ElementReference, config: &RedactionConfig) -> bool {
    let input_type = reference
        .element
        .accessibility
        .input_type
        .as_deref()
        .or_else(|| reference.element.attributes.get("type").map(String::as_str));
    if input_type.is_some_and(|value| {
        value.eq_ignore_ascii_case("password") || value.eq_ignore_ascii_case("hidden")
    }) {
        return true;
    }

    IDENTIFYING_ATTRIBUTES.iter().any(|name| {
        reference
            .element
            .attributes
            .get(*name)
            .is_some_and(|value| {
                let value = value.to_ascii_lowercase();
                SENSITIVE_AUTOCOMPLETE
                    .iter()
                    .any(|token| value.contains(token))
                    || config.sensitive_attribute_fragments.iter().any(|fragment| {
                        !fragment.eq_ignore_ascii_case("value")
                            && value.contains(&fragment.to_ascii_lowercase())
                    })
            })
    })
}

fn redact_string(value: &mut String) {
    value.clear();
    value.push_str(REDACTED);
}

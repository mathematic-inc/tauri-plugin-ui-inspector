use std::collections::BTreeMap;

use tauri_ui_inspector_core::{
    AccessibilityInfo, CaptureInfo, CssRect, CssSize, DomContext, ElementInfo, ElementReference,
    PhysicalPoint, PhysicalSize, ProjectInfo, ReferenceKind, ScreenshotInfo, SelectorSummary,
    ViewportInfo, WindowInfo,
};

pub(crate) fn reference(id: &str) -> ElementReference {
    ElementReference {
        schema_version: 1,
        kind: ReferenceKind::Element,
        id: id.to_owned(),
        created_at: "2026-08-20T00:00:00Z".to_owned(),
        summary: "button: button 'Save'".to_owned(),
        project: ProjectInfo::default(),
        window: WindowInfo {
            label: "main".to_owned(),
            title: Some("Fixture".to_owned()),
            scale_factor: 2.0,
            outer_position: PhysicalPoint::default(),
            inner_position: PhysicalPoint::default(),
            outer_size: PhysicalSize {
                width: 100.0,
                height: 100.0,
            },
            inner_size: PhysicalSize {
                width: 100.0,
                height: 100.0,
            },
            viewport: ViewportInfo {
                size: CssSize {
                    width: 100.0,
                    height: 100.0,
                },
                device_pixel_ratio: 2.0,
                visual_viewport: None,
            },
        },
        element: ElementInfo {
            tag_name: "button".to_owned(),
            namespace: Some("http://www.w3.org/1999/xhtml".to_owned()),
            text: Some("Save".to_owned()),
            role: Some("button".to_owned()),
            accessible_name: Some("Save".to_owned()),
            attributes: BTreeMap::new(),
            rect: CssRect::default(),
            locators: Vec::new(),
            selectors: SelectorSummary::default(),
            accessibility: AccessibilityInfo::default(),
        },
        source: None,
        dom: DomContext {
            html: "<button>Save</button>".to_owned(),
            parent_html: None,
            ancestry: Vec::new(),
        },
        screenshots: ScreenshotInfo {
            window: Some("window.png".to_owned()),
            element: Some("element.png".to_owned()),
        },
        capture: CaptureInfo {
            padding: 8,
            pixel_crop: None,
            screenshot_size: None,
        },
    }
}

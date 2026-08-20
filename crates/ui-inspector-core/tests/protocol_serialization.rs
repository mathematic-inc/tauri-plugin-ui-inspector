//! IPC protocol serialization tests.

#![warn(rust_2018_idioms)]

use tauri_ui_inspector_core::{IpcRequest, ResolveResult};

#[test]
fn request_fields_are_camel_case() {
    let value = serde_json::to_value(IpcRequest::Pick {
        window_label: Some("main".to_owned()),
    })
    .unwrap();
    assert_eq!(value["type"], "pick");
    assert_eq!(value["windowLabel"], "main");
    assert!(value.get("window_label").is_none());
}

#[test]
fn resolve_result_fields_are_camel_case() {
    let value = serde_json::to_value(ResolveResult::Resolved {
        locator_index: 2,
        rect: Box::new(tauri_ui_inspector_core::CssRect::default()),
    })
    .unwrap();
    assert_eq!(value["status"], "resolved");
    assert_eq!(value["locatorIndex"], 2);
    assert!(value.get("locator_index").is_none());
}

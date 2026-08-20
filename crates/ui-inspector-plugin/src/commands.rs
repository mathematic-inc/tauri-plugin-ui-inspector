//! Tauri command boundary between the framework-neutral frontend and backend.

use std::panic::{AssertUnwindSafe, catch_unwind};

use tauri::{Emitter, Runtime, State, WebviewWindow, command};
use tauri_ui_inspector_core::{
    CaptureInfo, ElementReference, PhysicalPoint, PhysicalSize, PickResult, ProjectInfo,
    ReferenceEvent, ReferenceId, ReferenceKind, ResolveResult, ScreenshotInfo, SelectionPayload,
    WindowInfo, deterministic_summary, redact_reference,
};

use crate::{
    EVENT_CANCELLED, EVENT_SELECTED, capture,
    state::{PendingCompletion, PluginState},
};

#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    reason = "Tauri command extractors are owned and this function is one atomic capture transaction"
)]
#[command]
pub(crate) async fn capture_selection<R: Runtime>(
    window: WebviewWindow<R>,
    state: State<'_, PluginState>,
    request_id: Option<String>,
    payload: SelectionPayload,
) -> Result<ElementReference, String> {
    if !state.enabled() {
        return Err("UI inspector is disabled outside debug builds".to_owned());
    }
    let config = state.config().clone();
    let screenshot = if config.capture_screenshots && config.persist_references {
        let capture_window = window.clone();
        let rect = payload.element.rect;
        let viewport = payload.viewport;
        Some(
            tauri::async_runtime::spawn_blocking(move || {
                capture::capture(&capture_window, rect, viewport, config.crop_padding)
            })
            .await
            .map_err(|error| error.to_string())?
            .map_err(|error| error.to_string())?,
        )
    } else {
        None
    };

    let outer_position = window.outer_position().map_err(|error| error.to_string())?;
    let inner_position = window.inner_position().map_err(|error| error.to_string())?;
    let outer_size = window.outer_size().map_err(|error| error.to_string())?;
    let inner_size = window.inner_size().map_err(|error| error.to_string())?;
    let id = ReferenceId::new();
    let summary = deterministic_summary(&payload.element, payload.source.as_ref());
    let screenshots = if screenshot.is_some() {
        ScreenshotInfo {
            window: Some("window.png".to_owned()),
            element: Some("element.png".to_owned()),
        }
    } else {
        ScreenshotInfo::default()
    };
    let mut reference = ElementReference {
        schema_version: tauri_ui_inspector_core::REFERENCE_SCHEMA_VERSION,
        kind: ReferenceKind::Element,
        id: id.to_string(),
        created_at: jiff::Timestamp::now().to_string(),
        summary,
        project: ProjectInfo {
            root: Some(state.project_root().to_string_lossy().into_owned()),
        },
        window: WindowInfo {
            label: window.label().to_owned(),
            title: window.title().ok(),
            scale_factor: window.scale_factor().map_err(|error| error.to_string())?,
            outer_position: PhysicalPoint {
                x: f64::from(outer_position.x),
                y: f64::from(outer_position.y),
            },
            inner_position: PhysicalPoint {
                x: f64::from(inner_position.x),
                y: f64::from(inner_position.y),
            },
            outer_size: PhysicalSize {
                width: f64::from(outer_size.width),
                height: f64::from(outer_size.height),
            },
            inner_size: PhysicalSize {
                width: f64::from(inner_size.width),
                height: f64::from(inner_size.height),
            },
            viewport: payload.viewport,
        },
        element: payload.element,
        source: payload.source,
        dom: payload.dom,
        screenshots,
        capture: CaptureInfo {
            padding: config.crop_padding,
            pixel_crop: screenshot.as_ref().map(|capture| capture.pixel_crop),
            screenshot_size: screenshot.as_ref().map(|capture| PhysicalSize {
                width: f64::from(capture.window.width()),
                height: f64::from(capture.window.height()),
            }),
        },
    };
    redact_reference(&mut reference, &config.redaction);

    let directory = if config.persist_references {
        state
            .storage()
            .save(
                &reference,
                screenshot.as_ref().map(|capture| &capture.window),
                screenshot.as_ref().map(|capture| &capture.element),
            )
            .map_err(|error| error.to_string())?
    } else {
        state.storage().root().join("refs").join(&reference.id)
    };

    if let Some(callback) = state.callback() {
        let callback = callback.clone();
        let callback_reference = reference.clone();
        let _ = catch_unwind(AssertUnwindSafe(move || callback(&callback_reference)));
    }

    let _ = window.emit(
        EVENT_SELECTED,
        ReferenceEvent {
            reference: reference.clone(),
        },
    );
    if let Some(request_id) = request_id {
        let relative = directory
            .strip_prefix(state.project_root())
            .unwrap_or(&directory)
            .to_string_lossy()
            .into_owned();
        state.complete_operation(
            &request_id,
            PendingCompletion::Pick(PickResult::Selected {
                reference: Box::new(reference.clone()),
                reference_dir: relative,
            }),
        );
    }
    Ok(reference)
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri command extractors are passed by value"
)]
#[command]
pub(crate) fn cancel_selection<R: Runtime>(
    window: WebviewWindow<R>,
    state: State<'_, PluginState>,
    request_id: Option<String>,
) {
    let _ = window.emit(EVENT_CANCELLED, ());
    if let Some(request_id) = request_id {
        state.complete_operation(&request_id, PendingCompletion::Pick(PickResult::Cancelled));
    }
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri command extractors are passed by value"
)]
#[command]
pub(crate) fn complete_resolution(
    state: State<'_, PluginState>,
    request_id: String,
    result: ResolveResult,
) {
    state.complete_operation(&request_id, PendingCompletion::Resolve(result));
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri command extractors are passed by value"
)]
#[command]
pub(crate) fn get_last_reference(
    state: State<'_, PluginState>,
) -> Result<Option<ElementReference>, String> {
    if !state.enabled() {
        return Err("UI inspector is disabled outside debug builds".to_owned());
    }
    state
        .storage()
        .last()
        .map(|entry| entry.map(tauri_ui_inspector_core::ReferenceEntry::into_reference))
        .map_err(|error| error.to_string())
}

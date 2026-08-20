#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

//! Shared primitives for UI inspector frontends, Tauri plugins, and CLIs.
//!
//! This crate owns the versioned reference schema and the pure operations that
//! must behave identically in every consumer: identifier parsing, coordinate
//! conversion, cropping, redaction, persistence, and local IPC messages.
//!
//! # Examples
//!
//! ```
//! use tauri_ui_inspector_core::ReferenceId;
//!
//! let id = ReferenceId::new();
//! assert!(id.as_str().starts_with("ui_"));
//! assert_eq!(ReferenceId::parse(format!("@{id}")).unwrap(), id);
//! ```

mod coordinate;
mod error;
mod id;
mod protocol;
mod redaction;
mod reference;
mod storage;

pub use coordinate::{
    CoordinateTransform, CssRect, CssSize, PhysicalPoint, PhysicalRect, PhysicalSize, PixelRect,
    crop_image,
};
pub use error::{CropError, ReferenceIdError, StorageError};
pub use id::ReferenceId;
pub use protocol::{
    ClientMessage, IPC_PROTOCOL_VERSION, InstanceInfo, IpcErrorCode, IpcRequest, IpcResponse,
    PickRequestEvent, PickResult, ReferenceEvent, ResolveRequestEvent, ResolveResult,
    ServerMessage,
};
pub use redaction::{RedactionConfig, redact_reference};
pub use reference::{
    AccessibilityInfo, CaptureInfo, DomAncestor, DomContext, ElementInfo, ElementReference,
    Locator, LocatorStrategy, ProjectInfo, REFERENCE_SCHEMA_VERSION, ReferenceKind, ScreenshotInfo,
    SelectionPayload, SelectorSummary, SourceComponent, SourceInfo, SourceLocation, ViewportInfo,
    VisualViewportInfo, WindowInfo, deterministic_summary,
};
pub use storage::{ReferenceEntry, Storage, StorageConfig};

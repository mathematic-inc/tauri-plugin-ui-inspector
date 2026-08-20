//! Versioned, newline-delimited JSON messages used by the local CLI socket.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ElementReference;

/// Current local IPC protocol version.
pub const IPC_PROTOCOL_VERSION: u32 = 1;

/// Frontend event payload requesting one interactive selection.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PickRequestEvent {
    /// Pending backend operation identifier.
    pub request_id: String,
}

/// Frontend event payload requesting exact live resolution.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ResolveRequestEvent {
    /// Pending backend operation identifier.
    pub request_id: String,
    /// Stored reference to reacquire.
    pub reference: ElementReference,
}

/// Frontend event payload emitted after capture succeeds.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceEvent {
    /// Created reference.
    pub reference: ElementReference,
}

/// Project-local discovery record for one running Tauri application.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InstanceInfo {
    /// Protocol version spoken by the instance.
    pub version: u32,
    /// Native process identifier.
    pub pid: u32,
    /// Project root used to disambiguate applications.
    pub project_root: String,
    /// Absolute storage root.
    pub storage_root: String,
    /// Cross-platform local-socket name.
    pub endpoint: String,
    /// Per-process authentication secret.
    pub token: String,
    /// RFC 3339 startup timestamp.
    pub started_at: String,
}

/// Authenticated client envelope.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ClientMessage {
    /// Protocol version.
    pub version: u32,
    /// Per-running-app secret read from the project-local instance file.
    pub token: String,
    /// Requested operation.
    pub request: IpcRequest,
}

/// Operations accepted by a running inspector plugin.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
#[non_exhaustive]
pub enum IpcRequest {
    /// Verifies that the instance is alive.
    Ping,
    /// Starts inspect mode and waits for a selection.
    Pick {
        /// Optional Tauri window label.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        window_label: Option<String>,
    },
    /// Attempts to reacquire a stored reference in the live DOM.
    Resolve {
        /// Normalized reference identifier.
        id: String,
        /// Optional Tauri window label.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        window_label: Option<String>,
    },
}

/// Server response envelope.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ServerMessage {
    /// Protocol version.
    pub version: u32,
    /// Operation result.
    pub response: IpcResponse,
}

/// Response sent by the running plugin.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
#[non_exhaustive]
pub enum IpcResponse {
    /// The instance is alive.
    Pong,
    /// Inspect mode completed or was cancelled.
    Pick(PickResult),
    /// Live element resolution completed.
    Resolve(ResolveResult),
    /// The operation failed before a typed result was available.
    Error {
        /// Stable machine-readable error code.
        code: IpcErrorCode,
        /// Human-readable diagnostic.
        message: String,
    },
}

/// Result of a CLI-driven pick request.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "status"
)]
#[non_exhaustive]
pub enum PickResult {
    /// A reference was created.
    Selected {
        /// Complete created reference.
        reference: Box<ElementReference>,
        /// Project-relative reference directory.
        reference_dir: String,
    },
    /// The user cancelled inspect mode.
    Cancelled,
}

/// Result of trying to reacquire an element in the running webview.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "status"
)]
#[non_exhaustive]
pub enum ResolveResult {
    /// Exactly one element matched a stored locator and signature.
    Resolved {
        /// Locator strategy that matched.
        locator_index: usize,
        /// Fresh element rectangle.
        rect: Box<crate::CssRect>,
    },
    /// No stored locator uniquely matched the original element.
    NotFound {
        /// Deterministic failure explanation.
        reason: String,
    },
}

/// Stable machine-readable IPC failures.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum IpcErrorCode {
    /// Authentication token did not match the running instance.
    Unauthorized,
    /// Requested window label does not exist.
    WindowNotFound,
    /// Another pick or resolve request is already pending.
    Busy,
    /// Stored reference does not exist.
    ReferenceNotFound,
    /// Frontend did not answer within the configured timeout.
    TimedOut,
    /// Plugin is disabled in the current build.
    Disabled,
    /// Unexpected internal failure.
    Internal,
}

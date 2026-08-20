//! Authenticated local-socket server used by the CLI.

use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
    sync::mpsc::RecvTimeoutError,
    thread,
};

use interprocess::local_socket::{GenericNamespaced, ListenerOptions, prelude::*};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};
use tauri_ui_inspector_core::{
    ClientMessage, IPC_PROTOCOL_VERSION, InstanceInfo, IpcErrorCode, IpcRequest, IpcResponse,
    PickRequestEvent, ResolveRequestEvent, ServerMessage,
};

use crate::{
    EVENT_PICK_REQUESTED, EVENT_RESOLVE_REQUESTED,
    state::{PendingCompletion, PluginState},
};

const INSTANCE_FILE: &str = "instance.json";
/// Limits accidental or malicious same-user messages while keeping the wire
/// format comfortably above the compact reference request size.
const MAX_MESSAGE_BYTES: usize = 64 * 1024;

pub(crate) fn start<R: Runtime>(app: AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<PluginState>();
    let endpoint = format!("tauri-ui-inspector-{}", ulid::Ulid::generate());
    let token = format!("{}{}", ulid::Ulid::generate(), ulid::Ulid::generate());
    let name = endpoint.clone().to_ns_name::<GenericNamespaced>()?;
    let listener = ListenerOptions::new().name(name).create_sync()?;
    let run_dir = state.storage().root().join("run");
    fs::create_dir_all(&run_dir)?;
    let instance_path = run_dir.join(INSTANCE_FILE);
    let instance = InstanceInfo {
        version: IPC_PROTOCOL_VERSION,
        pid: std::process::id(),
        project_root: state.project_root().to_string_lossy().into_owned(),
        storage_root: state.storage().root().to_string_lossy().into_owned(),
        endpoint,
        token: token.clone(),
        started_at: jiff::Timestamp::now().to_string(),
    };
    write_private_json(&instance_path, &instance)?;
    state.set_instance_path(instance_path);

    thread::Builder::new()
        .name("ui-inspector-ipc".to_owned())
        .spawn(move || {
            for connection in listener.incoming() {
                let Ok(connection) = connection else {
                    continue;
                };
                let app = app.clone();
                let token = token.clone();
                let _ = thread::Builder::new()
                    .name("ui-inspector-ipc-client".to_owned())
                    .spawn(move || handle_connection(&app, &token, connection));
            }
        })?;
    Ok(())
}

fn handle_connection<R: Runtime>(
    app: &AppHandle<R>,
    token: &str,
    connection: interprocess::local_socket::Stream,
) {
    let mut reader = BufReader::new(connection);
    let mut line = String::new();
    let read_result = {
        let mut limited = std::io::Read::take(&mut reader, (MAX_MESSAGE_BYTES + 1) as u64);
        limited.read_line(&mut line)
    };
    let response = match read_result {
        Ok(bytes) if bytes <= MAX_MESSAGE_BYTES => {
            match serde_json::from_str::<ClientMessage>(&line) {
                Ok(message)
                    if message.version == IPC_PROTOCOL_VERSION && message.token == token =>
                {
                    handle_request(app, message.request)
                }
                Ok(_) => error(IpcErrorCode::Unauthorized, "invalid IPC version or token"),
                Err(parse_error) => error(IpcErrorCode::Internal, parse_error.to_string()),
            }
        }
        Ok(_) => error(IpcErrorCode::Internal, "IPC message exceeded 64 KiB"),
        Err(read_error) => error(IpcErrorCode::Internal, read_error.to_string()),
    };
    let mut writer = BufWriter::new(reader.get_mut());
    if serde_json::to_writer(&mut writer, &response).is_ok() {
        let _ = writer.write_all(b"\n");
        let _ = writer.flush();
    }
}

fn handle_request<R: Runtime>(app: &AppHandle<R>, request: IpcRequest) -> ServerMessage {
    let state = app.state::<PluginState>();
    match request {
        IpcRequest::Ping => success(IpcResponse::Pong),
        IpcRequest::Pick { window_label } => {
            let window = match select_window(app, window_label.as_deref()) {
                Ok(window) => window,
                Err(response) => return response,
            };
            wait_for_operation(
                &state,
                |request_id| {
                    window.emit(
                        EVENT_PICK_REQUESTED,
                        PickRequestEvent {
                            request_id: request_id.to_owned(),
                        },
                    )
                },
                |completion| match completion {
                    PendingCompletion::Pick(result) => IpcResponse::Pick(result),
                    PendingCompletion::Resolve(_) => IpcResponse::Error {
                        code: IpcErrorCode::Internal,
                        message: "received resolve result for pick request".to_owned(),
                    },
                },
            )
        }
        IpcRequest::Resolve { id, window_label } => {
            let reference = match state.storage().get(&id) {
                Ok(entry) => entry.into_reference(),
                Err(tauri_ui_inspector_core::StorageError::NotFound { .. }) => {
                    return error(
                        IpcErrorCode::ReferenceNotFound,
                        format!("reference `{id}` was not found"),
                    );
                }
                Err(storage_error) => {
                    return error(IpcErrorCode::Internal, storage_error.to_string());
                }
            };
            let window = match select_window(app, window_label.as_deref()) {
                Ok(window) => window,
                Err(response) => return response,
            };
            wait_for_operation(
                &state,
                |request_id| {
                    window.emit(
                        EVENT_RESOLVE_REQUESTED,
                        ResolveRequestEvent {
                            request_id: request_id.to_owned(),
                            reference: reference.clone(),
                        },
                    )
                },
                |completion| match completion {
                    PendingCompletion::Resolve(result) => IpcResponse::Resolve(result),
                    PendingCompletion::Pick(_) => IpcResponse::Error {
                        code: IpcErrorCode::Internal,
                        message: "received pick result for resolve request".to_owned(),
                    },
                },
            )
        }
        _ => error(IpcErrorCode::Internal, "unsupported IPC request"),
    }
}

fn wait_for_operation(
    state: &PluginState,
    emit: impl FnOnce(&str) -> tauri::Result<()>,
    map: impl FnOnce(PendingCompletion) -> IpcResponse,
) -> ServerMessage {
    let (request_id, receiver) = match state.begin_operation() {
        Ok(pending) => pending,
        Err(code) => return error(code, "another inspector request is already pending"),
    };
    if let Err(emit_error) = emit(&request_id) {
        state.clear_operation(&request_id);
        return error(IpcErrorCode::Internal, emit_error.to_string());
    }
    match receiver.recv_timeout(state.config().request_timeout) {
        Ok(completion) => success(map(completion)),
        Err(RecvTimeoutError::Timeout) => {
            state.clear_operation(&request_id);
            error(
                IpcErrorCode::TimedOut,
                "the frontend did not answer before the request timeout",
            )
        }
        Err(RecvTimeoutError::Disconnected) => {
            state.clear_operation(&request_id);
            error(
                IpcErrorCode::Internal,
                "the frontend response channel disconnected",
            )
        }
    }
}

fn select_window<R: Runtime>(
    app: &AppHandle<R>,
    requested: Option<&str>,
) -> Result<WebviewWindow<R>, ServerMessage> {
    if let Some(label) = requested {
        return app.get_webview_window(label).ok_or_else(|| {
            error(
                IpcErrorCode::WindowNotFound,
                format!("window `{label}` was not found"),
            )
        });
    }
    let mut windows = app.webview_windows().into_values().collect::<Vec<_>>();
    windows.sort_by(|left, right| left.label().cmp(right.label()));
    if let Some(focused) = windows
        .iter()
        .find(|window| window.is_focused().unwrap_or(false))
    {
        return Ok(focused.clone());
    }
    windows.into_iter().next().ok_or_else(|| {
        error(
            IpcErrorCode::WindowNotFound,
            "the application has no webview windows",
        )
    })
}

fn success(response: IpcResponse) -> ServerMessage {
    ServerMessage {
        version: IPC_PROTOCOL_VERSION,
        response,
    }
}

fn error(code: IpcErrorCode, message: impl Into<String>) -> ServerMessage {
    success(IpcResponse::Error {
        code,
        message: message.into(),
    })
}

fn write_private_json(
    path: &Path,
    value: &impl Serialize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

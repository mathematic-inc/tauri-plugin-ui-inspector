//! Synchronous newline-delimited JSON client for the running plugin.

use std::io::{BufRead, BufReader, BufWriter, Write};

use interprocess::local_socket::{GenericNamespaced, Stream, prelude::*};
use tauri_ui_inspector_core::{
    ClientMessage, IPC_PROTOCOL_VERSION, InstanceInfo, IpcRequest, ServerMessage,
};

#[derive(Debug, thiserror::Error)]
pub(crate) enum ClientError {
    #[error("could not connect to the running Tauri application: {0}")]
    Connect(std::io::Error),
    #[error("local IPC operation failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("local IPC JSON operation failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("running application uses unsupported IPC version {0}")]
    Version(u32),
}

pub(crate) fn request(
    instance: &InstanceInfo,
    request: IpcRequest,
) -> Result<ServerMessage, ClientError> {
    let name = instance
        .endpoint
        .clone()
        .to_ns_name::<GenericNamespaced>()
        .map_err(ClientError::Connect)?;
    let stream = Stream::connect(name).map_err(ClientError::Connect)?;
    let mut reader = BufReader::new(stream);
    {
        let mut writer = BufWriter::new(reader.get_mut());
        serde_json::to_writer(
            &mut writer,
            &ClientMessage {
                version: IPC_PROTOCOL_VERSION,
                token: instance.token.clone(),
                request,
            },
        )?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }
    let mut response = String::new();
    reader.read_line(&mut response)?;
    let message: ServerMessage = serde_json::from_str(&response)?;
    if message.version != IPC_PROTOCOL_VERSION {
        return Err(ClientError::Version(message.version));
    }
    Ok(message)
}

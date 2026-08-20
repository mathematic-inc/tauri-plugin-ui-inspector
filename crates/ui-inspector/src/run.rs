//! Command orchestration and output policy.

use std::{
    fmt::Display,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use tauri_ui_inspector_core::{
    ElementReference, InstanceInfo, IpcErrorCode, IpcRequest, IpcResponse, PickResult,
    ReferenceEntry, ResolveResult, Storage, StorageConfig, StorageError,
};

use crate::{
    args::{Cli, Command},
    client,
};

const NOT_FOUND: i32 = 2;
const APP_NOT_RUNNING: i32 = 3;
const CANCELLED: i32 = 4;
const NOT_RESOLVABLE: i32 = 5;

#[derive(Debug)]
pub(crate) struct CliError {
    code: i32,
    message: Option<String>,
}

impl CliError {
    pub(crate) fn code(&self) -> i32 {
        self.code
    }

    pub(crate) fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    fn new(code: i32, message: impl Display) -> Self {
        Self {
            code,
            message: Some(message.to_string()),
        }
    }
}

pub(crate) fn run(cli: Cli) -> Result<(), CliError> {
    let project = resolve_project_root(cli.project.as_deref(), &cli.storage_dir)
        .map_err(|error| CliError::new(1, error.to_string()))?;
    let storage_root = if cli.storage_dir.is_absolute() {
        cli.storage_dir.clone()
    } else {
        project.join(&cli.storage_dir)
    };
    let storage = Storage::new(StorageConfig {
        root: storage_root,
        max_history: 0,
    });

    match cli.command {
        Command::Pick { window } => pick(&storage, cli.json, window),
        Command::Last => {
            let entry = storage
                .last()
                .map_err(storage_error)?
                .ok_or_else(|| CliError::new(NOT_FOUND, "no UI references have been captured"))?;
            print_reference(entry.reference(), cli.json)
        }
        Command::Get { id } => {
            let entry = storage.get(id).map_err(storage_error)?;
            print_reference(entry.reference(), cli.json)
        }
        Command::List => {
            let references = storage
                .list()
                .map_err(storage_error)?
                .into_iter()
                .map(ReferenceEntry::into_reference)
                .collect::<Vec<_>>();
            if cli.json {
                print_json(&references)
            } else {
                for reference in references {
                    println!("@{}\t{}", reference.id, reference.summary);
                }
                Ok(())
            }
        }
        Command::Screenshot { id } => {
            let entry = storage.get(id).map_err(storage_error)?;
            print_screenshots(&entry, cli.json)
        }
        Command::Resolve { id, window } => resolve(&storage, id, window, cli.json),
        Command::Delete { id } => {
            storage.delete(id).map_err(storage_error)?;
            if cli.json {
                print_json(&serde_json::json!({ "deleted": true }))
            } else {
                println!("Deleted reference");
                Ok(())
            }
        }
        Command::Clear => {
            storage.clear().map_err(storage_error)?;
            if cli.json {
                print_json(&serde_json::json!({ "cleared": true }))
            } else {
                println!("Cleared references");
                Ok(())
            }
        }
    }
}

fn pick(storage: &Storage, json: bool, window: Option<String>) -> Result<(), CliError> {
    if !json {
        eprintln!("Waiting for UI selection...");
    }
    let response = request_running(
        storage,
        IpcRequest::Pick {
            window_label: window,
        },
    )?;
    match response.response {
        IpcResponse::Pick(PickResult::Selected {
            reference,
            reference_dir,
        }) => {
            if json {
                print_json(&reference)
            } else {
                println!("Selected @{}", reference.id);
                println!("{}", reference.summary);
                if let Some(source) = reference.source.as_ref() {
                    print!("{}", source.location.file);
                    if let Some(line) = source.location.line {
                        print!(":{line}");
                        if let Some(column) = source.location.column {
                            print!(":{column}");
                        }
                    }
                    println!();
                }
                if let Some(element) = reference.screenshots.element.as_deref() {
                    println!("{reference_dir}/{element}");
                }
                Ok(())
            }
        }
        IpcResponse::Pick(PickResult::Cancelled) => {
            Err(CliError::new(CANCELLED, "inspection cancelled"))
        }
        other => map_ipc_error(other),
    }
}

fn resolve(
    storage: &Storage,
    id: String,
    window: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    let normalized = storage
        .get(&id)
        .map_err(storage_error)?
        .reference()
        .id
        .clone();
    let response = request_running(
        storage,
        IpcRequest::Resolve {
            id: normalized,
            window_label: window,
        },
    )?;
    match response.response {
        IpcResponse::Resolve(result @ ResolveResult::Resolved { .. }) => {
            if json {
                print_json(&result)
            } else {
                println!(
                    "Resolved @{}",
                    storage.get(id).map_err(storage_error)?.reference().id
                );
                Ok(())
            }
        }
        IpcResponse::Resolve(ResolveResult::NotFound { reason }) => {
            Err(CliError::new(NOT_RESOLVABLE, reason))
        }
        other => map_ipc_error(other),
    }
}

fn request_running(
    storage: &Storage,
    request: IpcRequest,
) -> Result<tauri_ui_inspector_core::ServerMessage, CliError> {
    let instance = read_instance(storage)?;
    client::request(&instance, request).map_err(|error| match error {
        client::ClientError::Connect(_) => CliError::new(
            APP_NOT_RUNNING,
            "the Tauri application is not running or its inspector socket is stale",
        ),
        other => CliError::new(1, other.to_string()),
    })
}

fn read_instance(storage: &Storage) -> Result<InstanceInfo, CliError> {
    let path = storage.root().join("run/instance.json");
    let file = File::open(&path).map_err(|_| {
        CliError::new(
            APP_NOT_RUNNING,
            format!("running inspector instance not found at {}", path.display()),
        )
    })?;
    let instance: InstanceInfo = serde_json::from_reader(BufReader::new(file))
        .map_err(|error| CliError::new(1, error.to_string()))?;
    if instance.version != tauri_ui_inspector_core::IPC_PROTOCOL_VERSION {
        return Err(CliError::new(
            1,
            format!("unsupported inspector IPC version {}", instance.version),
        ));
    }
    Ok(instance)
}

fn print_reference(reference: &ElementReference, json: bool) -> Result<(), CliError> {
    if json {
        print_json(reference)
    } else {
        println!("@{}", reference.id);
        println!("{}", reference.summary);
        Ok(())
    }
}

fn print_screenshots(entry: &ReferenceEntry, json: bool) -> Result<(), CliError> {
    let reference = entry.reference();
    let window = reference
        .screenshots
        .window
        .as_deref()
        .map(|name| entry.directory().join(name));
    let element = reference
        .screenshots
        .element
        .as_deref()
        .map(|name| entry.directory().join(name));
    if json {
        print_json(&serde_json::json!({
            "window": window.as_ref().map(|path| path.to_string_lossy()),
            "element": element.as_ref().map(|path| path.to_string_lossy()),
        }))
    } else {
        if let Some(window) = window {
            println!("window\t{}", window.display());
        }
        if let Some(element) = element {
            println!("element\t{}", element.display());
        }
        Ok(())
    }
}

fn print_json(value: &impl serde::Serialize) -> Result<(), CliError> {
    serde_json::to_writer(std::io::stdout().lock(), value)
        .map_err(|error| CliError::new(1, error.to_string()))?;
    println!();
    Ok(())
}

fn map_ipc_error(response: IpcResponse) -> Result<(), CliError> {
    match response {
        IpcResponse::Error { code, message } => {
            let exit = match code {
                IpcErrorCode::ReferenceNotFound => NOT_FOUND,
                IpcErrorCode::Disabled => APP_NOT_RUNNING,
                _ => 1,
            };
            Err(CliError::new(exit, message))
        }
        _ => Err(CliError::new(
            1,
            "unexpected response from the running application",
        )),
    }
}

fn storage_error(error: StorageError) -> CliError {
    let code = if matches!(&error, StorageError::NotFound { .. }) {
        NOT_FOUND
    } else {
        1
    };
    CliError::new(code, error)
}

fn resolve_project_root(requested: Option<&Path>, storage_dir: &Path) -> std::io::Result<PathBuf> {
    if let Some(path) = requested {
        return absolute(path);
    }
    let current = std::env::current_dir()?;
    if storage_dir.is_absolute() {
        return Ok(current);
    }
    for ancestor in current.ancestors() {
        if ancestor.join(storage_dir).exists() {
            return Ok(ancestor.to_owned());
        }
    }
    Ok(current)
}

fn absolute(path: &Path) -> std::io::Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    path.canonicalize()
}

#[cfg(test)]
mod tests {
    use super::resolve_project_root;

    #[test]
    fn explicit_project_root_is_canonicalized() {
        let root = std::env::current_dir().unwrap();
        assert_eq!(
            resolve_project_root(Some(&root), ".ui-inspector".as_ref()).unwrap(),
            root.canonicalize().unwrap()
        );
    }
}

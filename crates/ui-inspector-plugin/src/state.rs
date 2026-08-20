//! Activated plugin state and pending CLI operation coordination.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, mpsc},
    time::Duration,
};

use tauri_ui_inspector_core::{
    InstanceInfo, IpcErrorCode, PickResult, RedactionConfig, ResolveResult, Storage, StorageConfig,
};

use crate::ReferenceCallback;

/// The default 8 CSS-pixel crop includes borders and nearby context without
/// overwhelming small controls. It can be calibrated per application.
const DEFAULT_CROP_PADDING: u32 = 8;
/// Two minutes lets a user navigate menus before selecting without leaving a
/// CLI process blocked indefinitely.
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_mins(2);

#[derive(Clone, Debug)]
pub(crate) struct PluginConfig {
    pub(crate) storage_dir: PathBuf,
    pub(crate) project_root: Option<PathBuf>,
    pub(crate) max_history: usize,
    pub(crate) crop_padding: u32,
    pub(crate) capture_screenshots: bool,
    pub(crate) persist_references: bool,
    pub(crate) enable_in_production: bool,
    pub(crate) request_timeout: Duration,
    pub(crate) redaction: RedactionConfig,
}

impl PluginConfig {
    pub(crate) fn new() -> Self {
        Self {
            storage_dir: PathBuf::from(".ui-inspector"),
            project_root: None,
            max_history: 100,
            crop_padding: DEFAULT_CROP_PADDING,
            capture_screenshots: true,
            persist_references: true,
            enable_in_production: false,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            redaction: RedactionConfig::new(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum PendingCompletion {
    Pick(PickResult),
    Resolve(ResolveResult),
}

#[derive(Debug)]
struct PendingOperation {
    id: String,
    sender: mpsc::SyncSender<PendingCompletion>,
}

#[derive(Debug, Default)]
struct PendingOperations(Mutex<Option<PendingOperation>>);

impl PendingOperations {
    fn begin(&self) -> Result<(String, mpsc::Receiver<PendingCompletion>), IpcErrorCode> {
        let mut pending = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if pending.is_some() {
            return Err(IpcErrorCode::Busy);
        }
        let id = ulid::Ulid::generate().to_string();
        let (sender, receiver) = mpsc::sync_channel(1);
        *pending = Some(PendingOperation {
            id: id.clone(),
            sender,
        });
        Ok((id, receiver))
    }

    fn complete(&self, id: &str, completion: PendingCompletion) -> bool {
        let operation = {
            let mut pending = self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match pending.as_ref() {
                Some(operation) if operation.id == id => pending.take(),
                _ => None,
            }
        };
        operation.is_some_and(|operation| operation.sender.send(completion).is_ok())
    }

    fn clear(&self, id: &str) {
        let mut pending = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if pending.as_ref().is_some_and(|operation| operation.id == id) {
            *pending = None;
        }
    }
}

pub(crate) struct PluginState {
    config: PluginConfig,
    project_root: PathBuf,
    storage: Storage,
    callback: Option<ReferenceCallback>,
    pending: PendingOperations,
    instance_path: Mutex<Option<PathBuf>>,
}

impl std::fmt::Debug for PluginState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PluginState")
            .field("config", &self.config)
            .field("project_root", &self.project_root)
            .field("storage", &self.storage)
            .field("callback", &self.callback.as_ref().map(|_| "configured"))
            .finish_non_exhaustive()
    }
}

impl PluginState {
    pub(crate) fn activate(
        config: PluginConfig,
        callback: Option<ReferenceCallback>,
    ) -> Result<Self, std::io::Error> {
        let project_root = match config.project_root.as_ref() {
            Some(root) => absolute(root)?,
            None => std::env::current_dir()?,
        };
        let storage_root = if config.storage_dir.is_absolute() {
            config.storage_dir.clone()
        } else {
            project_root.join(&config.storage_dir)
        };
        let storage = Storage::new(StorageConfig {
            root: storage_root,
            max_history: config.max_history,
        });
        Ok(Self {
            config,
            project_root,
            storage,
            callback,
            pending: PendingOperations::default(),
            instance_path: Mutex::new(None),
        })
    }

    #[inline]
    pub(crate) fn enabled(&self) -> bool {
        cfg!(debug_assertions) || self.config.enable_in_production
    }

    #[inline]
    pub(crate) fn config(&self) -> &PluginConfig {
        &self.config
    }

    #[inline]
    pub(crate) fn project_root(&self) -> &Path {
        &self.project_root
    }

    #[inline]
    pub(crate) fn storage(&self) -> &Storage {
        &self.storage
    }

    #[inline]
    pub(crate) fn callback(&self) -> Option<&ReferenceCallback> {
        self.callback.as_ref()
    }

    pub(crate) fn begin_operation(
        &self,
    ) -> Result<(String, mpsc::Receiver<PendingCompletion>), IpcErrorCode> {
        self.pending.begin()
    }

    pub(crate) fn complete_operation(&self, id: &str, completion: PendingCompletion) -> bool {
        self.pending.complete(id, completion)
    }

    pub(crate) fn clear_operation(&self, id: &str) {
        self.pending.clear(id);
    }

    pub(crate) fn set_instance_path(&self, path: PathBuf) {
        *self
            .instance_path
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(path);
    }

    pub(crate) fn remove_instance_file(&self) {
        let path = self
            .instance_path
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(path) = path {
            let belongs_to_this_process = fs::read(&path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<InstanceInfo>(&bytes).ok())
                .is_some_and(|instance| instance.pid == std::process::id());
            if belongs_to_this_process {
                let _ = fs::remove_file(path);
            }
        }
    }
}

fn absolute(path: &Path) -> Result<PathBuf, std::io::Error> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    path.canonicalize()
}

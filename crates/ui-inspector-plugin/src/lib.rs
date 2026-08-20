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
#![cfg(not(any(target_os = "android", target_os = "ios")))]

//! Native capture and local-agent bridge for UI inspection in Tauri 2 apps.
//!
//! The JavaScript inspector owns DOM interaction and semantic extraction. This
//! plugin owns native window capture, coordinate conversion, durable local
//! references, and authenticated project-local CLI communication.
//!
//! Enable the plugin in development builds and install the matching frontend
//! package in each window that should be inspectable.
//!
//! # Examples
//!
//! ```no_run
//! let mut inspector = tauri_plugin_ui_inspector::Builder::new();
//! inspector.storage_dir(".ui-inspector").max_history(100);
//!
//! let _app = tauri::Builder::default().plugin(inspector.build());
//! ```

use std::{path::PathBuf, sync::Arc, time::Duration};

use tauri::{Manager, RunEvent, Runtime, plugin::TauriPlugin};
use tauri_ui_inspector_core::{ElementReference, RedactionConfig};

mod capture;
mod commands;
mod ipc;
mod state;

use state::{PluginConfig, PluginState};

/// Event emitted when a CLI requests inspect mode.
pub const EVENT_PICK_REQUESTED: &str = "ui-inspector://pick";
/// Event emitted when a CLI requests live element resolution.
pub const EVENT_RESOLVE_REQUESTED: &str = "ui-inspector://resolve";
/// Event emitted after a reference is created.
pub const EVENT_SELECTED: &str = "ui-inspector://selected";
/// Event emitted when inspect mode is cancelled.
pub const EVENT_CANCELLED: &str = "ui-inspector://cancelled";

/// Callback invoked after a reference has been created and redacted.
pub type ReferenceCallback = Arc<dyn Fn(&ElementReference) + Send + Sync + 'static>;

/// Configures and constructs the UI inspector plugin.
#[derive(Clone)]
pub struct Builder {
    config: PluginConfig,
    callback: Option<ReferenceCallback>,
}

impl std::fmt::Debug for Builder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Builder")
            .field("config", &self.config)
            .field("callback", &self.callback.as_ref().map(|_| "configured"))
            .finish()
    }
}

impl Builder {
    /// Creates a development-only plugin builder with bounded local storage.
    ///
    /// # Examples
    ///
    /// ```
    /// let _builder = tauri_plugin_ui_inspector::Builder::new();
    /// ```
    #[must_use = "a plugin builder must be configured or built"]
    pub fn new() -> Self {
        Self {
            config: PluginConfig::new(),
            callback: None,
        }
    }

    /// Sets the project-local storage directory.
    ///
    /// Relative paths are resolved against [`project_root`](Self::project_root)
    /// or the process working directory.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut builder = tauri_plugin_ui_inspector::Builder::new();
    /// builder.storage_dir(".ui-inspector");
    /// ```
    pub fn storage_dir(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.config.storage_dir = path.into();
        self
    }

    /// Sets the project root used for discovery and relative paths.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut builder = tauri_plugin_ui_inspector::Builder::new();
    /// builder.project_root(env!("CARGO_MANIFEST_DIR"));
    /// ```
    pub fn project_root(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.config.project_root = Some(path.into());
        self
    }

    /// Sets the maximum persisted history. A value of zero disables cleanup.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut builder = tauri_plugin_ui_inspector::Builder::new();
    /// builder.max_history(50);
    /// ```
    pub fn max_history(&mut self, max_history: usize) -> &mut Self {
        self.config.max_history = max_history;
        self
    }

    /// Sets crop padding in CSS pixels.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut builder = tauri_plugin_ui_inspector::Builder::new();
    /// builder.crop_padding(16);
    /// ```
    pub fn crop_padding(&mut self, padding: u32) -> &mut Self {
        self.config.crop_padding = padding;
        self
    }

    /// Enables or disables native screenshots.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut builder = tauri_plugin_ui_inspector::Builder::new();
    /// builder.capture_screenshots(false);
    /// ```
    pub fn capture_screenshots(&mut self, enabled: bool) -> &mut Self {
        self.config.capture_screenshots = enabled;
        self
    }

    /// Enables or disables persistence.
    ///
    /// When disabled, references are returned to frontend callbacks but are
    /// not discoverable by the CLI.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut builder = tauri_plugin_ui_inspector::Builder::new();
    /// builder.persist_references(false);
    /// ```
    pub fn persist_references(&mut self, enabled: bool) -> &mut Self {
        self.config.persist_references = enabled;
        self
    }

    /// Allows the inspector to run in non-debug builds.
    ///
    /// Production enablement must be explicit because screenshots and UI
    /// metadata may contain sensitive information.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut builder = tauri_plugin_ui_inspector::Builder::new();
    /// builder.enable_in_production(true);
    /// ```
    pub fn enable_in_production(&mut self, enabled: bool) -> &mut Self {
        self.config.enable_in_production = enabled;
        self
    }

    /// Sets the maximum time a CLI request waits for the frontend.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// let mut builder = tauri_plugin_ui_inspector::Builder::new();
    /// builder.request_timeout(Duration::from_secs(30));
    /// ```
    pub fn request_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.config.request_timeout = timeout;
        self
    }

    /// Replaces the backend redaction policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_ui_inspector_core::RedactionConfig;
    /// let mut builder = tauri_plugin_ui_inspector::Builder::new();
    /// builder.redaction(RedactionConfig::new());
    /// ```
    pub fn redaction(&mut self, config: RedactionConfig) -> &mut Self {
        self.config.redaction = config;
        self
    }

    /// Registers an optional handoff callback independent of any coding agent.
    ///
    /// Callback panics are caught so they cannot corrupt plugin state.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut builder = tauri_plugin_ui_inspector::Builder::new();
    /// builder.on_reference_created(|reference| {
    ///     assert!(reference.id.starts_with("ui_"));
    /// });
    /// ```
    pub fn on_reference_created<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(&ElementReference) + Send + Sync + 'static,
    {
        self.callback = Some(Arc::new(callback));
        self
    }

    /// Builds the Tauri plugin.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _app = tauri::Builder::default().plugin(
    ///     tauri_plugin_ui_inspector::Builder::new().build(),
    /// );
    /// ```
    #[must_use = "the returned Tauri plugin must be installed"]
    pub fn build<R: Runtime>(&self) -> TauriPlugin<R> {
        let config = self.config.clone();
        let callback = self.callback.clone();

        tauri::plugin::Builder::new("ui-inspector")
            .invoke_handler(tauri::generate_handler![
                commands::capture_selection,
                commands::cancel_selection,
                commands::complete_resolution,
                commands::get_last_reference,
            ])
            .setup(move |app, _api| {
                let state = PluginState::activate(config.clone(), callback.clone())?;
                let enabled = state.enabled();
                app.manage(state);
                if enabled {
                    ipc::start(app.clone())?;
                }
                Ok(())
            })
            .on_event(|app, event| {
                if let RunEvent::Exit = event
                    && let Some(state) = app.try_state::<PluginState>()
                {
                    state.remove_instance_file();
                }
            })
            .build()
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

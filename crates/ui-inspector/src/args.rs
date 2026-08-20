//! CLI grammar.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "ui-inspector",
    version,
    about = "Inspect and resolve durable Tauri UI element references"
)]
pub(crate) struct Cli {
    /// Emit only machine-readable JSON to stdout.
    #[arg(long, global = true)]
    pub(crate) json: bool,

    /// Project root. Defaults to the nearest ancestor containing `.ui-inspector`.
    #[arg(long, global = true, value_name = "PATH")]
    pub(crate) project: Option<PathBuf>,

    /// Project-relative or absolute storage directory.
    #[arg(long, global = true, default_value = ".ui-inspector")]
    pub(crate) storage_dir: PathBuf,

    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Ask the running Tauri app to enter inspect mode and wait for a selection.
    Pick {
        /// Target Tauri window label.
        #[arg(long)]
        window: Option<String>,
    },
    /// Print the newest stored reference.
    Last,
    /// Print one stored reference.
    Get {
        /// Reference in `ui_<ULID>` or `@ui_<ULID>` form.
        id: String,
    },
    /// List stored references newest first.
    List,
    /// Print screenshot paths for one reference.
    Screenshot {
        /// Reference in `ui_<ULID>` or `@ui_<ULID>` form.
        id: String,
    },
    /// Reacquire a stored reference in the live app without fuzzy matching.
    Resolve {
        /// Reference in `ui_<ULID>` or `@ui_<ULID>` form.
        id: String,
        /// Target Tauri window label.
        #[arg(long)]
        window: Option<String>,
    },
    /// Delete one stored reference.
    Delete {
        /// Reference in `ui_<ULID>` or `@ui_<ULID>` form.
        id: String,
    },
    /// Delete all stored references.
    Clear,
}

//! Generates Tauri command permissions for the plugin.

const COMMANDS: &[&str] = &[
    "capture_selection",
    "cancel_selection",
    "complete_resolution",
    "get_last_reference",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}

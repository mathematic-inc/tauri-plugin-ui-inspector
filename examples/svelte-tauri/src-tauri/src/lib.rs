#![warn(rust_2018_idioms)]

//! Native shell for the Svelte UI inspector fixture.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Runs the fixture application.
///
/// # Panics
///
/// Panics if Tauri cannot initialize or run the fixture.
pub fn run() {
    let builder = tauri::Builder::default();
    #[cfg(feature = "e2e")]
    let builder = builder
        .plugin(tauri_plugin_wdio::init())
        .plugin(tauri_plugin_wdio_webdriver::init());

    let mut inspector = tauri_plugin_ui_inspector::Builder::new();
    inspector.project_root(concat!(env!("CARGO_MANIFEST_DIR"), "/../../.."));
    #[cfg(feature = "e2e")]
    inspector.enable_in_production(true);
    builder
        .plugin(inspector.build())
        .run(tauri::generate_context!())
        .expect("example Tauri application failed");
}

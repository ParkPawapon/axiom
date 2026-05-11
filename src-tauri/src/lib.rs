pub mod application;
pub mod bootstrap;
pub mod commands;
pub mod domain;
pub mod infrastructure;
pub mod platform;
pub mod ports;
pub mod shared;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt().with_target(false).try_init();

    let builder = tauri::Builder::default().setup(|app| {
        app.manage(bootstrap::app_state::AppState::new());
        Ok(())
    });

    if let Err(error) = builder.run(tauri::generate_context!()) {
        tracing::error!(?error, "failed to run AxiomPHP");
    }
}

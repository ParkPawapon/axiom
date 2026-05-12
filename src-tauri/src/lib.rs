pub mod application;
pub mod bootstrap;
pub mod commands;
pub mod domain;
pub mod infrastructure;
pub mod platform;
pub mod ports;
pub mod shared;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt().with_target(false).try_init();

    let builder = tauri::Builder::default().setup(|app| {
        app.manage(bootstrap::app_state::AppState::new()?);
        Ok(())
    });

    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::project_commands::get_project_php_version,
        commands::project_commands::install_project_php_runtime,
        commands::project_commands::request_project_php_install,
        commands::project_commands::select_project_php_version,
        commands::service_commands::list_services,
        commands::service_commands::get_service_status,
        commands::service_commands::start_service,
        commands::service_commands::stop_service,
        commands::service_commands::restart_service,
    ]);

    if let Err(error) = builder.run(tauri::generate_context!()) {
        tracing::error!(?error, "failed to run AxiomPHP");
    }
}

pub mod application;
pub mod bootstrap;
pub mod commands;
pub mod domain;
pub mod infrastructure;
pub mod platform;
pub mod ports;
pub mod shared;

pub fn run() {
    let _ = tracing_subscriber::fmt().with_target(false).try_init();

    let app_state = match bootstrap::app_state::AppState::new() {
        Ok(state) => state,
        Err(error) => {
            tracing::error!(?error, "failed to initialize AxiomPHP application state");
            return;
        }
    };

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::project_commands::list_projects,
            commands::project_commands::get_project,
            commands::project_commands::create_project,
            commands::project_commands::update_project,
            commands::project_commands::delete_project,
            commands::project_commands::validate_project_path,
            commands::project_commands::get_project_php_version,
            commands::project_commands::select_project_php_version,
            commands::project_commands::request_project_php_install,
            commands::project_commands::install_project_php_runtime,
            commands::project_commands::get_project_php_process_status,
            commands::project_commands::start_project_php_process,
            commands::project_commands::start_project_php_processes,
            commands::project_commands::stop_project_php_process,
            commands::project_commands::stop_project_php_processes,
            commands::project_commands::restart_project_php_process,
            commands::project_commands::restart_project_php_processes,
            commands::database_commands::list_project_database_profiles,
            commands::database_commands::provision_project_database,
            commands::database_commands::backup_project_database,
            commands::database_commands::list_database_backup_policies,
            commands::database_commands::update_database_backup_policy,
            commands::database_commands::run_due_database_backups,
            commands::database_commands::restore_project_database,
            commands::database_commands::create_project_database_migration,
            commands::database_commands::run_project_database_migrations,
            commands::log_commands::read_project_logs,
            commands::security_commands::get_security_status,
            commands::security_commands::update_hosts_file,
            commands::security_commands::generate_local_certificate,
            commands::security_commands::inspect_certificate_trust,
            commands::security_commands::trust_certificate_authority,
            commands::security_commands::read_audit_log,
            commands::security_commands::prune_audit_log,
            commands::service_commands::list_services,
            commands::service_commands::get_service_status,
            commands::service_commands::start_service,
            commands::service_commands::stop_service,
            commands::service_commands::restart_service,
        ]);

    if let Err(error) = builder.run(tauri::generate_context!()) {
        tracing::error!(?error, "failed to run AxiomPHP desktop application");
    }
}

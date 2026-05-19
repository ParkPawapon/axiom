pub mod application;
pub mod bootstrap;
pub mod commands;
pub mod domain;
pub mod infrastructure;
pub mod platform;
pub mod ports;
pub mod shared;

use crate::application::databases::run_due_database_backups_use_case;
use crate::domain::database::database_config::ScheduledDatabaseBackupRunResult;
use crate::shared::result::app_result::AppResult;

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
            commands::database_commands::list_database_backup_destinations,
            commands::database_commands::list_database_backup_policies,
            commands::database_commands::update_database_backup_destination,
            commands::database_commands::update_database_backup_policy,
            commands::database_commands::run_due_database_backups,
            commands::database_commands::get_database_backup_scheduler_status,
            commands::database_commands::get_database_backup_key_management_status,
            commands::database_commands::export_database_backup_trust_bundle,
            commands::database_commands::import_database_backup_trust_bundle,
            commands::database_commands::install_database_backup_scheduler,
            commands::database_commands::uninstall_database_backup_scheduler,
            commands::database_commands::restore_project_database,
            commands::database_commands::restore_project_database_to_point_in_time,
            commands::database_commands::restore_project_database_with_replay,
            commands::database_commands::create_project_database_migration,
            commands::database_commands::generate_project_database_migration_rollback,
            commands::database_commands::rollback_project_database_migrations,
            commands::database_commands::run_project_database_migrations,
            commands::docker_commands::get_docker_diagnostics,
            commands::docker_commands::resolve_project_docker_image_pins,
            commands::docker_commands::generate_project_docker_compose,
            commands::docker_commands::get_project_docker_status,
            commands::docker_commands::start_project_docker_services,
            commands::docker_commands::stop_project_docker_services,
            commands::docker_commands::restart_project_docker_services,
            commands::docker_commands::ensure_project_docker_volumes,
            commands::docker_commands::remove_project_docker_volumes,
            commands::docker_commands::read_project_docker_logs,
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

pub fn run_due_database_backups_once() -> AppResult<ScheduledDatabaseBackupRunResult> {
    let _ = tracing_subscriber::fmt().with_target(false).try_init();
    let state = bootstrap::app_state::AppState::new()?;

    let result = run_due_database_backups_use_case::run_due_database_backups(
        state.database_backup_policy_repository(),
        state.database_backup_destination_repository(),
        state.database_provisioning_repository(),
        state.database_provisioner(),
    )?;

    tracing::info!(
        checked_policies = result.checked_policies,
        completed_backups = result.completed_backups,
        skipped_backups = result.skipped_backups,
        errors = result.errors.len(),
        "scheduled database backup CLI sweep completed"
    );

    Ok(result)
}

use chrono::Utc;

use crate::domain::control_center::control_center_summary::{
    ControlCenterLogPreview, ControlCenterPortSummary, ControlCenterProjectSummary,
    ControlCenterServiceSummary, ControlCenterSummary,
};
use crate::domain::control_center::quick_action::{QuickActionKind, QuickActionSummary};
use crate::domain::control_center::setup_diagnostic::{
    ControlCenterSeverity, ControlCenterStatus, SetupDiagnostic, SetupDiagnosticStep,
    SetupDiagnosticsReport,
};
use crate::domain::database::database_config::{
    DatabaseProvisioningStatus, ProjectDatabaseProfile,
};
use crate::domain::database::database_type::DatabaseType;
use crate::domain::project::project::Project;
use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_process::{ProjectPhpProcessState, ProjectPhpProcessStatus};
use crate::domain::service::service::Service;
use crate::domain::service::service_status::ServiceStatus;
use crate::domain::system::system_status::{PortCheckResult, PortCheckState};
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::ports::docker_project_orchestrator::DockerProjectOrchestrator;
use crate::ports::log_reader::LogReader;
use crate::ports::port_scanner::PortScanner;
use crate::ports::project_php_process_manager::ProjectPhpProcessManager;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::ports::service_manager::ServiceManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

const LOG_PREVIEW_LINES: usize = 6;

pub struct ControlCenterDependencies<'a> {
    pub project_repository: &'a dyn ProjectRepository,
    pub project_runtime_repository: &'a dyn ProjectRuntimeRepository,
    pub project_php_process_manager: &'a dyn ProjectPhpProcessManager,
    pub database_repository: &'a dyn DatabaseProvisioningRepository,
    pub docker_project_orchestrator: &'a dyn DockerProjectOrchestrator,
    pub log_reader: &'a dyn LogReader,
    pub port_scanner: &'a dyn PortScanner,
    pub service_manager: &'a dyn ServiceManager,
}

pub fn get_control_center_summary(
    dependencies: ControlCenterDependencies<'_>,
    selected_project_id: Option<String>,
) -> AppResult<ControlCenterSummary> {
    let projects = dependencies.project_repository.list_projects()?;
    let selected_project = selected_project(&projects, selected_project_id.as_deref())?;
    let selected_project_id = selected_project.map(|project| project.id.clone());
    let php_status = selected_project
        .map(|project| {
            dependencies
                .project_php_process_manager
                .get_php_process_status(&project.id)
        })
        .transpose()?;
    let php_selection = selected_project_id
        .as_ref()
        .map(|project_id| {
            dependencies
                .project_runtime_repository
                .get_php_selection(project_id)
        })
        .transpose()?
        .flatten();
    let database_profiles = selected_project_id
        .as_ref()
        .map(|project_id| dependencies.database_repository.list_profiles(project_id))
        .transpose()?
        .unwrap_or_default();
    let services = dependencies
        .service_manager
        .list_services()
        .unwrap_or_default();
    let docker_diagnostics = dependencies.docker_project_orchestrator.diagnostics().ok();
    let docker_runtime = selected_project.and_then(|project| {
        dependencies
            .docker_project_orchestrator
            .get_runtime_status(project)
            .ok()
    });

    let mut selected_project_summary = selected_project.map(ControlCenterProjectSummary::from);
    if let (Some(summary), Some(status)) = (&mut selected_project_summary, &php_status) {
        summary.php_url = status.url.clone();
        summary.php_port = status.port;
        summary.php_version = status
            .php_version
            .as_ref()
            .map(|version| version.as_str().to_string());
    }
    if let (Some(summary), Some(selection)) =
        (&mut selected_project_summary, php_selection.as_ref())
    {
        summary.php_version = Some(selection.php_version.as_str().to_string());
    }

    let diagnostics = diagnostics(
        selected_project,
        php_status.as_ref(),
        php_selection.as_ref().map(|_| ()),
        &database_profiles,
        &services,
        docker_diagnostics.as_ref(),
    );
    let service_summaries = service_summaries(
        php_status.as_ref(),
        php_selection.as_ref().map(|_| ()),
        &database_profiles,
        &services,
        docker_diagnostics.as_ref(),
        docker_runtime
            .as_ref()
            .map(|runtime| runtime.engine_running),
    );
    let quick_actions = quick_actions(
        selected_project,
        php_status.as_ref(),
        php_selection.as_ref().map(|_| ()),
        &database_profiles,
    );
    let ports = port_summaries(
        dependencies.port_scanner,
        php_status.as_ref(),
        &database_profiles,
    );
    let log_preview = log_preview(
        dependencies.log_reader,
        selected_project_id.as_ref(),
        php_status.as_ref(),
    );
    let setup = setup_report_from_diagnostics(&diagnostics);
    let status = ControlCenterSummary::readiness_status(&diagnostics);
    let status_message = match status {
        ControlCenterStatus::Ready | ControlCenterStatus::Running => {
            "Control Center is ready for daily project work.".to_string()
        }
        ControlCenterStatus::NeedsSetup => {
            "AxiomPHP needs setup before every service can run cleanly.".to_string()
        }
        ControlCenterStatus::BlockedSafely => {
            "One or more actions are blocked safely until setup is complete.".to_string()
        }
        _ => "Control Center readiness was checked.".to_string(),
    };

    Ok(ControlCenterSummary {
        projects: projects
            .iter()
            .map(ControlCenterProjectSummary::from)
            .collect(),
        selected_project: selected_project_summary,
        services: service_summaries,
        diagnostics,
        quick_actions,
        ports,
        log_preview,
        setup,
        generated_at: Utc::now(),
        status,
        status_message,
    })
}

fn selected_project<'a>(
    projects: &'a [Project],
    selected_project_id: Option<&str>,
) -> AppResult<Option<&'a Project>> {
    let Some(project_id) = selected_project_id else {
        return Ok(projects.first());
    };
    let project_id = validate_project_id(project_id)?;

    projects
        .iter()
        .find(|project| project.id.0 == project_id)
        .map(Some)
        .ok_or_else(|| AppError::NotFound(format!("project `{project_id}` was not found")))
}

fn diagnostics(
    selected_project: Option<&Project>,
    php_status: Option<&ProjectPhpProcessStatus>,
    php_selection: Option<()>,
    database_profiles: &[ProjectDatabaseProfile],
    services: &[Service],
    docker_diagnostics: Option<&crate::domain::docker::docker_project::DockerDiagnosticsReport>,
) -> Vec<SetupDiagnostic> {
    let mut diagnostics = Vec::new();

    if selected_project.is_none() {
        diagnostics.push(SetupDiagnostic::new(
            "project.missing",
            "No project selected",
            ControlCenterStatus::NeedsSetup,
            ControlCenterSeverity::Warning,
            "Create or select a PHP project before starting services.",
            "Open Projects and add a document root.",
        ));
        return diagnostics;
    }

    if php_selection.is_none() {
        diagnostics.push(SetupDiagnostic::new(
            "php.binary.missing",
            "PHP binary is not selected",
            ControlCenterStatus::NeedsSetup,
            ControlCenterSeverity::Warning,
            "This project needs a selected PHP version before Start can run.",
            "Choose a PHP version in the setup wizard or Projects screen.",
        ));
    }

    if php_status.is_some_and(|status| status.state == ProjectPhpProcessState::Failed) {
        diagnostics.push(SetupDiagnostic::new(
            "php.process.failed",
            "PHP process failed",
            ControlCenterStatus::Error,
            ControlCenterSeverity::Error,
            "The last PHP process status is failed.",
            "Open Logs and inspect the backend-managed PHP process log.",
        ));
    }

    if !has_ready_database_profile(database_profiles, DatabaseType::Mysql) {
        diagnostics.push(SetupDiagnostic::new(
            "mysql.profile.missing",
            "MySQL needs setup",
            ControlCenterStatus::NeedsSetup,
            ControlCenterSeverity::Warning,
            "No ready MySQL profile is provisioned for this project.",
            "Use Databases or the setup wizard to provision MySQL.",
        ));
    }

    if !has_ready_database_profile(database_profiles, DatabaseType::Postgresql) {
        diagnostics.push(SetupDiagnostic::new(
            "postgresql.profile.missing",
            "PostgreSQL needs setup",
            ControlCenterStatus::NeedsSetup,
            ControlCenterSeverity::Info,
            "No ready PostgreSQL profile is provisioned for this project.",
            "Provision PostgreSQL only if this project needs it.",
        ));
    }

    let docker_ready = docker_diagnostics
        .map(|diagnostics| diagnostics.cli_found && diagnostics.engine_running)
        .unwrap_or(false);
    if !docker_ready {
        diagnostics.push(SetupDiagnostic::new(
            "docker.not.ready",
            "Docker Desktop is not ready",
            ControlCenterStatus::Missing,
            ControlCenterSeverity::Info,
            "Docker is optional, but Docker-backed project services need the engine running.",
            "Open Docker Desktop, then run diagnostics again.",
        ));
    }

    if service_by_id(services, "reverse-proxy")
        .or_else(|| service_by_id(services, "reverse_proxy"))
        .is_none()
    {
        diagnostics.push(SetupDiagnostic::new(
            "proxy.not.configured",
            "Reverse proxy is not configured",
            ControlCenterStatus::NeedsSetup,
            ControlCenterSeverity::Info,
            "Local domains and shared HTTP routing need a reverse proxy profile.",
            "Use Services or Docker advanced settings when local domains are required.",
        ));
    }

    diagnostics
}

fn service_summaries(
    php_status: Option<&ProjectPhpProcessStatus>,
    php_selection: Option<()>,
    database_profiles: &[ProjectDatabaseProfile],
    services: &[Service],
    docker_diagnostics: Option<&crate::domain::docker::docker_project::DockerDiagnosticsReport>,
    docker_engine_running: Option<bool>,
) -> Vec<ControlCenterServiceSummary> {
    vec![
        php_service_summary(php_status, php_selection),
        database_service_summary(
            "mysql",
            "MySQL",
            DatabaseType::Mysql,
            database_profiles,
            services,
        ),
        database_service_summary(
            "postgresql",
            "PostgreSQL",
            DatabaseType::Postgresql,
            database_profiles,
            services,
        ),
        docker_service_summary(docker_diagnostics, docker_engine_running),
        service_summary_from_manager("reverse-proxy", "Reverse Proxy", services),
        logs_service_summary(php_status),
    ]
}

fn php_service_summary(
    status: Option<&ProjectPhpProcessStatus>,
    php_selection: Option<()>,
) -> ControlCenterServiceSummary {
    match status.map(|status| status.state) {
        Some(ProjectPhpProcessState::Running) => ControlCenterServiceSummary {
            id: "php".to_string(),
            label: "PHP".to_string(),
            status: ControlCenterStatus::Running,
            primary_detail: status
                .and_then(|status| status.url.clone())
                .unwrap_or_else(|| "PHP process is running.".to_string()),
            secondary_detail: status.and_then(|status| status.port.map(|port| format!(":{port}"))),
            can_start: false,
            can_stop: true,
            can_restart: true,
        },
        Some(ProjectPhpProcessState::Failed) => ControlCenterServiceSummary {
            id: "php".to_string(),
            label: "PHP".to_string(),
            status: ControlCenterStatus::Error,
            primary_detail: "PHP failed safely.".to_string(),
            secondary_detail: status.map(|status| status.status_message.clone()),
            can_start: php_selection.is_some(),
            can_stop: false,
            can_restart: php_selection.is_some(),
        },
        Some(ProjectPhpProcessState::Stopped) if php_selection.is_some() => {
            ControlCenterServiceSummary {
                id: "php".to_string(),
                label: "PHP".to_string(),
                status: ControlCenterStatus::Stopped,
                primary_detail: "Ready to start.".to_string(),
                secondary_detail: status.map(|status| status.status_message.clone()),
                can_start: true,
                can_stop: false,
                can_restart: true,
            }
        }
        _ => ControlCenterServiceSummary {
            id: "php".to_string(),
            label: "PHP".to_string(),
            status: ControlCenterStatus::NeedsSetup,
            primary_detail: "Select a PHP binary first.".to_string(),
            secondary_detail: None,
            can_start: false,
            can_stop: false,
            can_restart: false,
        },
    }
}

fn database_service_summary(
    id: &str,
    label: &str,
    database_type: DatabaseType,
    profiles: &[ProjectDatabaseProfile],
    services: &[Service],
) -> ControlCenterServiceSummary {
    let profile = profiles
        .iter()
        .find(|profile| profile.database_type == database_type);
    let service = service_by_id(services, id);
    let status = match (
        profile.map(|profile| profile.status),
        service.map(|service| service.status),
    ) {
        (Some(DatabaseProvisioningStatus::Ready), Some(ServiceStatus::Running)) => {
            ControlCenterStatus::Running
        }
        (Some(DatabaseProvisioningStatus::Ready), _) => ControlCenterStatus::Ready,
        (Some(DatabaseProvisioningStatus::Failed), _) => ControlCenterStatus::Error,
        _ => ControlCenterStatus::NeedsSetup,
    };

    ControlCenterServiceSummary {
        id: id.to_string(),
        label: label.to_string(),
        status,
        primary_detail: profile
            .map(|profile| {
                format!(
                    "{}@{}:{}",
                    profile.database_name, profile.host, profile.port
                )
            })
            .unwrap_or_else(|| "No project profile.".to_string()),
        secondary_detail: service.map(|service| service.status_message.clone()),
        can_start: service.is_some_and(|service| service.can_start),
        can_stop: service.is_some_and(|service| service.can_stop),
        can_restart: service.is_some_and(|service| service.can_restart),
    }
}

fn docker_service_summary(
    diagnostics: Option<&crate::domain::docker::docker_project::DockerDiagnosticsReport>,
    engine_running: Option<bool>,
) -> ControlCenterServiceSummary {
    let Some(diagnostics) = diagnostics else {
        return ControlCenterServiceSummary {
            id: "docker".to_string(),
            label: "Docker".to_string(),
            status: ControlCenterStatus::Missing,
            primary_detail: "Docker diagnostics unavailable.".to_string(),
            secondary_detail: None,
            can_start: false,
            can_stop: false,
            can_restart: false,
        };
    };
    let running = engine_running.unwrap_or(diagnostics.engine_running);

    ControlCenterServiceSummary {
        id: "docker".to_string(),
        label: "Docker".to_string(),
        status: if running {
            ControlCenterStatus::Running
        } else if diagnostics.cli_found {
            ControlCenterStatus::Stopped
        } else {
            ControlCenterStatus::Missing
        },
        primary_detail: diagnostics.status_message.clone(),
        secondary_detail: diagnostics.docker_context.clone(),
        can_start: false,
        can_stop: false,
        can_restart: false,
    }
}

fn service_summary_from_manager(
    id: &str,
    label: &str,
    services: &[Service],
) -> ControlCenterServiceSummary {
    let Some(service) = service_by_id(services, id) else {
        return ControlCenterServiceSummary {
            id: id.to_string(),
            label: label.to_string(),
            status: ControlCenterStatus::NeedsSetup,
            primary_detail: "Service adapter is not configured.".to_string(),
            secondary_detail: None,
            can_start: false,
            can_stop: false,
            can_restart: false,
        };
    };

    ControlCenterServiceSummary {
        id: id.to_string(),
        label: label.to_string(),
        status: simple_service_status(service.status),
        primary_detail: service.status_message.clone(),
        secondary_detail: Some(service.description.clone()),
        can_start: service.can_start,
        can_stop: service.can_stop,
        can_restart: service.can_restart,
    }
}

fn logs_service_summary(status: Option<&ProjectPhpProcessStatus>) -> ControlCenterServiceSummary {
    let log_file = status.and_then(|status| status.log_file.clone());

    ControlCenterServiceSummary {
        id: "logs".to_string(),
        label: "Logs".to_string(),
        status: if log_file.is_some() {
            ControlCenterStatus::Ready
        } else {
            ControlCenterStatus::NeedsSetup
        },
        primary_detail: log_file
            .unwrap_or_else(|| "Log file is created after PHP starts.".to_string()),
        secondary_detail: None,
        can_start: false,
        can_stop: false,
        can_restart: false,
    }
}

fn quick_actions(
    selected_project: Option<&Project>,
    php_status: Option<&ProjectPhpProcessStatus>,
    php_selection: Option<()>,
    database_profiles: &[ProjectDatabaseProfile],
) -> Vec<QuickActionSummary> {
    let Some(project) = selected_project else {
        return vec![
            QuickActionSummary::disabled(
                QuickActionKind::StartProject,
                "start",
                "Start",
                "Add a project first.",
            ),
            QuickActionSummary::disabled(
                QuickActionKind::OpenLogs,
                "open-logs",
                "Open Logs",
                "Add a project first.",
            ),
        ];
    };
    let running = php_status.is_some_and(|status| status.state == ProjectPhpProcessState::Running);
    let php_ready = php_selection.is_some();
    let mysql_admin_url = database_profiles
        .iter()
        .find(|profile| profile.database_type == DatabaseType::Mysql)
        .and_then(|profile| profile.admin_url.clone());

    vec![
        if php_ready && !running {
            QuickActionSummary::enabled(
                QuickActionKind::StartProject,
                "start",
                "Start",
                Some(project.id.0.clone()),
            )
        } else {
            QuickActionSummary::disabled(
                QuickActionKind::StartProject,
                "start",
                "Start",
                if running {
                    "Project is already running."
                } else {
                    "Select a PHP binary before starting."
                },
            )
        },
        if running {
            QuickActionSummary::enabled(
                QuickActionKind::StopProject,
                "stop",
                "Stop",
                Some(project.id.0.clone()),
            )
        } else {
            QuickActionSummary::disabled(
                QuickActionKind::StopProject,
                "stop",
                "Stop",
                "Project is not running.",
            )
        },
        if php_ready {
            QuickActionSummary::enabled(
                QuickActionKind::RestartProject,
                "restart",
                "Restart",
                Some(project.id.0.clone()),
            )
        } else {
            QuickActionSummary::disabled(
                QuickActionKind::RestartProject,
                "restart",
                "Restart",
                "Select a PHP binary before restarting.",
            )
        },
        php_status
            .and_then(|status| status.url.clone())
            .map(|url| {
                QuickActionSummary::enabled(
                    QuickActionKind::OpenBrowser,
                    "open-browser",
                    "Open Browser",
                    Some(url),
                )
            })
            .unwrap_or_else(|| {
                QuickActionSummary::disabled(
                    QuickActionKind::OpenBrowser,
                    "open-browser",
                    "Open Browser",
                    "Start the project to create a local URL.",
                )
            }),
        QuickActionSummary::enabled(
            QuickActionKind::OpenFolder,
            "open-folder",
            "Open Folder",
            Some(project.document_root.0.clone()),
        ),
        QuickActionSummary::enabled(
            QuickActionKind::OpenLogs,
            "open-logs",
            "Open Logs",
            Some(project.id.0.clone()),
        ),
        mysql_admin_url
            .map(|url| {
                QuickActionSummary::enabled(
                    QuickActionKind::OpenPhpMyAdmin,
                    "open-phpmyadmin",
                    "Open phpMyAdmin",
                    Some(url),
                )
            })
            .unwrap_or_else(|| {
                QuickActionSummary::disabled(
                    QuickActionKind::OpenPhpMyAdmin,
                    "open-phpmyadmin",
                    "Open phpMyAdmin",
                    "Provision MySQL with phpMyAdmin access first.",
                )
            }),
    ]
}

fn port_summaries(
    port_scanner: &dyn PortScanner,
    php_status: Option<&ProjectPhpProcessStatus>,
    database_profiles: &[ProjectDatabaseProfile],
) -> Vec<ControlCenterPortSummary> {
    let mut ports = Vec::new();

    if let Some(port) = php_status.and_then(|status| status.port).or(Some(8500)) {
        ports.push(port_summary(port_scanner, "php", "PHP", port));
    }
    ports.push(port_summary(
        port_scanner,
        "mysql",
        "MySQL",
        database_profiles
            .iter()
            .find(|profile| profile.database_type == DatabaseType::Mysql)
            .map(|profile| profile.port)
            .unwrap_or_else(|| DatabaseType::Mysql.default_port()),
    ));
    ports.push(port_summary(
        port_scanner,
        "postgresql",
        "PostgreSQL",
        database_profiles
            .iter()
            .find(|profile| profile.database_type == DatabaseType::Postgresql)
            .map(|profile| profile.port)
            .unwrap_or_else(|| DatabaseType::Postgresql.default_port()),
    ));
    ports.push(port_summary(port_scanner, "http", "HTTP", 80));

    ports
}

fn port_summary(
    port_scanner: &dyn PortScanner,
    id: &str,
    label: &str,
    port: u16,
) -> ControlCenterPortSummary {
    match port_scanner.check_loopback_port(port) {
        Ok(result) => port_summary_from_result(id, label, result),
        Err(error) => ControlCenterPortSummary {
            id: id.to_string(),
            label: label.to_string(),
            port,
            status: ControlCenterStatus::Error,
            status_message: format!("Port check failed safely: {error}"),
        },
    }
}

fn port_summary_from_result(
    id: &str,
    label: &str,
    result: PortCheckResult,
) -> ControlCenterPortSummary {
    let status = match result.state {
        PortCheckState::Available => ControlCenterStatus::Ready,
        PortCheckState::InUse => ControlCenterStatus::Running,
        PortCheckState::Unavailable => ControlCenterStatus::BlockedSafely,
    };

    ControlCenterPortSummary {
        id: id.to_string(),
        label: label.to_string(),
        port: result.port,
        status,
        status_message: result.status_message,
    }
}

fn log_preview(
    log_reader: &dyn LogReader,
    project_id: Option<&ProjectId>,
    php_status: Option<&ProjectPhpProcessStatus>,
) -> ControlCenterLogPreview {
    let Some(project_id) = project_id else {
        return ControlCenterLogPreview {
            source: "project".to_string(),
            entries: Vec::new(),
            status: ControlCenterStatus::NeedsSetup,
            status_message: "Add a project to read logs.".to_string(),
        };
    };

    match log_reader.read_project_process_log(project_id, LOG_PREVIEW_LINES, None) {
        Ok(result) => ControlCenterLogPreview {
            source: "php".to_string(),
            entries: result.entries,
            status: ControlCenterStatus::Ready,
            status_message: result.status_message,
        },
        Err(_error)
            if php_status
                .and_then(|status| status.log_file.as_ref())
                .is_none() =>
        {
            ControlCenterLogPreview {
                source: "php".to_string(),
                entries: Vec::new(),
                status: ControlCenterStatus::NeedsSetup,
                status_message: "Start the project to create a PHP log file.".to_string(),
            }
        }
        Err(error) => ControlCenterLogPreview {
            source: "php".to_string(),
            entries: Vec::new(),
            status: ControlCenterStatus::Error,
            status_message: format!("Log preview failed safely: {error}"),
        },
    }
}

pub fn setup_report_from_diagnostics(diagnostics: &[SetupDiagnostic]) -> SetupDiagnosticsReport {
    let steps = vec![
        step("system", "System Check", diagnostics, &["docker", "proxy"]),
        step("php", "PHP Version", diagnostics, &["php"]),
        step(
            "database",
            "Database",
            diagnostics,
            &["mysql", "postgresql"],
        ),
        step("domain", "Local Domain", diagnostics, &["proxy"]),
        step("permission", "Permissions", diagnostics, &["permission"]),
        step("summary", "Summary", diagnostics, &[]),
    ];
    let ready = !diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == ControlCenterSeverity::Error);
    let status_message = if diagnostics.is_empty() {
        "Setup checks are ready.".to_string()
    } else {
        format!("{} setup item(s) need attention.", diagnostics.len())
    };

    SetupDiagnosticsReport {
        steps,
        ready,
        status_message,
    }
}

fn step(
    id: &str,
    title: &str,
    diagnostics: &[SetupDiagnostic],
    prefixes: &[&str],
) -> SetupDiagnosticStep {
    let step_diagnostics = diagnostics
        .iter()
        .filter(|diagnostic| {
            prefixes.is_empty()
                || prefixes
                    .iter()
                    .any(|prefix| diagnostic.id.starts_with(prefix))
        })
        .cloned()
        .collect::<Vec<_>>();
    let status = if step_diagnostics.is_empty() {
        ControlCenterStatus::Ready
    } else {
        ControlCenterSummary::readiness_status(&step_diagnostics)
    };

    SetupDiagnosticStep {
        id: id.to_string(),
        title: title.to_string(),
        status,
        diagnostics: step_diagnostics,
    }
}

fn has_ready_database_profile(
    profiles: &[ProjectDatabaseProfile],
    database_type: DatabaseType,
) -> bool {
    profiles.iter().any(|profile| {
        profile.database_type == database_type
            && profile.status == DatabaseProvisioningStatus::Ready
    })
}

fn service_by_id<'a>(services: &'a [Service], id: &str) -> Option<&'a Service> {
    services.iter().find(|service| service.id == id)
}

fn simple_service_status(status: ServiceStatus) -> ControlCenterStatus {
    match status {
        ServiceStatus::Detected => ControlCenterStatus::Ready,
        ServiceStatus::Failed => ControlCenterStatus::Error,
        ServiceStatus::NotConfigured => ControlCenterStatus::NeedsSetup,
        ServiceStatus::Running => ControlCenterStatus::Running,
        ServiceStatus::Stopped => ControlCenterStatus::Stopped,
        ServiceStatus::Unknown => ControlCenterStatus::Checking,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_failed_service_to_simple_error_status() {
        assert_eq!(
            simple_service_status(ServiceStatus::Failed),
            ControlCenterStatus::Error
        );
    }

    #[test]
    fn setup_report_marks_empty_diagnostics_ready() {
        let report = setup_report_from_diagnostics(&[]);

        assert!(report.ready);
        assert!(report
            .steps
            .iter()
            .all(|step| step.status == ControlCenterStatus::Ready));
    }
}

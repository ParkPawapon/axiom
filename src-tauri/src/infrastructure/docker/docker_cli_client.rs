use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

use directories::ProjectDirs;
use serde_json::Value;

use crate::domain::project::project::Project;
use crate::domain::project::project_docker::{
    ProjectDockerAction, ProjectDockerActionResult, ProjectDockerState, ProjectDockerStatus,
};
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::docker::docker_compose_generator::DockerComposeGenerator;
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::infrastructure::services::adapters::executable_resolver::ExecutableResolver;
use crate::ports::docker_client::{DockerClient, DockerEngineProbe};
use crate::ports::process_manager::ProcessManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const DOCKER_TIMEOUT: Duration = Duration::from_secs(5);
const COMPOSE_TIMEOUT: Duration = Duration::from_secs(60);
const DOCKER_OUTPUT_LIMIT_BYTES: usize = 64 * 1024;
const MANAGED_COMPOSE_FILE_ENV: &str = "AXIOM_DOCKER_COMPOSE_FILE";
const MANAGED_COMPOSE_PROJECT_NAME: &str = "axiomphp";
const PROJECT_DOCKER_SERVICE_NAME: &str = "php";
const PROJECT_DOCKER_CONTAINER_PORT: u16 = 8080;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DockerCliClient {
    compose_root: PathBuf,
}

impl DockerCliClient {
    pub fn new() -> Self {
        Self {
            compose_root: default_project_compose_root(),
        }
    }

    pub fn with_compose_root(compose_root: PathBuf) -> Self {
        Self { compose_root }
    }

    fn compose_generator(&self) -> DockerComposeGenerator {
        DockerComposeGenerator::new(self.compose_root.clone())
    }

    fn resolve_docker(&self) -> AppResult<PathBuf> {
        ExecutableResolver::from_env()
            .resolve("docker")
            .ok_or_else(|| {
                AppError::NotFound("Docker CLI executable was not found on PATH".to_string())
            })
    }

    fn run_docker(
        &self,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> AppResult<ProcessOutput> {
        self.run_docker_with_timeout(args, DOCKER_TIMEOUT)
    }

    fn run_docker_with_timeout(
        &self,
        args: impl IntoIterator<Item = impl Into<String>>,
        timeout: Duration,
    ) -> AppResult<ProcessOutput> {
        let docker_path = self.resolve_docker()?;
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([docker_path.clone()])
                .with_default_timeout(timeout)
                .with_max_output_bytes(DOCKER_OUTPUT_LIMIT_BYTES),
        );

        runner.execute(
            ProcessCommand::new(docker_path.to_string_lossy().into_owned())
                .args(args)
                .timeout(timeout),
        )
    }

    fn compose_project_count(&self) -> Option<usize> {
        let output = self
            .run_docker(["compose", "ls", "--format", "json"])
            .ok()?;

        if output.timed_out || output.exit_code != Some(0) {
            return None;
        }

        parse_compose_project_count(&output.stdout)
    }

    fn configured_compose_file(&self) -> AppResult<Option<PathBuf>> {
        let Some(raw_path) = env::var_os(MANAGED_COMPOSE_FILE_ENV) else {
            return Ok(None);
        };
        let path = PathBuf::from(raw_path);

        validate_compose_file_path(&path)?;

        Ok(Some(path))
    }

    fn project_compose_file(&self, project: &Project) -> AppResult<PathBuf> {
        self.compose_generator().project_compose_file(&project.id.0)
    }

    fn project_compose_name(&self, project: &Project) -> AppResult<String> {
        self.compose_generator().compose_project_name(&project.id.0)
    }

    fn compose_command_args(
        project_name: &str,
        compose_file: &Path,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Vec<String> {
        let mut command_args = vec![
            "compose".to_string(),
            "--project-name".to_string(),
            project_name.to_string(),
            "--file".to_string(),
            compose_file.to_string_lossy().into_owned(),
        ];
        command_args.extend(args.into_iter().map(Into::into));
        command_args
    }

    fn run_compose(
        &self,
        project_name: &str,
        compose_file: &Path,
        args: impl IntoIterator<Item = impl Into<String>>,
        timeout: Duration,
    ) -> AppResult<ProcessOutput> {
        self.run_docker_with_timeout(
            Self::compose_command_args(project_name, compose_file, args),
            timeout,
        )
    }

    fn project_port(&self, project_name: &str, compose_file: &Path) -> Option<u16> {
        let output = self
            .run_compose(
                project_name,
                compose_file,
                [
                    "port".to_string(),
                    PROJECT_DOCKER_SERVICE_NAME.to_string(),
                    PROJECT_DOCKER_CONTAINER_PORT.to_string(),
                ],
                DOCKER_TIMEOUT,
            )
            .ok()?;

        if output.timed_out || output.exit_code != Some(0) {
            return None;
        }

        parse_published_port(&output.stdout)
    }

    fn status_for_project(
        &self,
        project: &Project,
        state: ProjectDockerState,
        compose_file: Option<PathBuf>,
        container_id: Option<String>,
        published_port: Option<u16>,
        status_message: String,
    ) -> AppResult<ProjectDockerStatus> {
        let url = published_port.map(|port| format!("http://127.0.0.1:{port}"));

        Ok(ProjectDockerStatus {
            project_id: project.id.clone(),
            state,
            compose_project_name: self.project_compose_name(project)?,
            compose_file_path: compose_file.map(|path| path.to_string_lossy().into_owned()),
            service_name: PROJECT_DOCKER_SERVICE_NAME.to_string(),
            container_id,
            published_port,
            url,
            status_message,
        })
    }
}

impl DockerClient for DockerCliClient {
    fn probe_engine(&self) -> AppResult<DockerEngineProbe> {
        let docker_path = match self.resolve_docker() {
            Ok(path) => path,
            Err(error) => {
                return Ok(DockerEngineProbe {
                    cli_found: false,
                    engine_running: false,
                    compose_project_count: None,
                    status_message: format!("Docker CLI is not configured: {error}"),
                });
            }
        };

        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([docker_path.clone()])
                .with_default_timeout(DOCKER_TIMEOUT)
                .with_max_output_bytes(DOCKER_OUTPUT_LIMIT_BYTES),
        );
        let output = runner.execute(
            ProcessCommand::new(docker_path.to_string_lossy().into_owned())
                .args(["info", "--format", "{{json .ServerVersion}}"])
                .timeout(DOCKER_TIMEOUT),
        )?;

        if output.timed_out {
            return Ok(DockerEngineProbe {
                cli_found: true,
                engine_running: false,
                compose_project_count: None,
                status_message: "Docker engine probe timed out.".to_string(),
            });
        }

        if output.exit_code == Some(0) {
            let compose_project_count = self.compose_project_count();
            let compose_message = compose_project_count
                .map(|count| format!(" Compose projects visible: {count}."))
                .unwrap_or_else(|| " Compose diagnostics are unavailable.".to_string());

            return Ok(DockerEngineProbe {
                cli_found: true,
                engine_running: true,
                compose_project_count,
                status_message: format!(
                    "Docker engine is running.{compose_message}{}",
                    managed_compose_status_message(self.configured_compose_file())
                ),
            });
        }

        Ok(DockerEngineProbe {
            cli_found: true,
            engine_running: false,
            compose_project_count: None,
            status_message: format!(
                "Docker CLI is installed, but the engine is not ready. {}{}",
                summarize_output(&output),
                managed_compose_status_message(self.configured_compose_file())
            ),
        })
    }

    fn start_configured_compose_project(&self) -> AppResult<Option<String>> {
        let Some(compose_file) = self.configured_compose_file()? else {
            return Ok(None);
        };

        let output = self.run_docker_with_timeout(
            [
                "compose".to_string(),
                "--project-name".to_string(),
                MANAGED_COMPOSE_PROJECT_NAME.to_string(),
                "--file".to_string(),
                compose_file.to_string_lossy().into_owned(),
                "up".to_string(),
                "--detach".to_string(),
            ],
            COMPOSE_TIMEOUT,
        )?;

        ensure_successful_output("docker compose up", &output)?;

        Ok(Some(format!(
            "Managed Docker Compose project `{MANAGED_COMPOSE_PROJECT_NAME}` started from {}.",
            compose_file.to_string_lossy()
        )))
    }

    fn stop_configured_compose_project(&self) -> AppResult<Option<String>> {
        let Some(compose_file) = self.configured_compose_file()? else {
            return Ok(None);
        };

        let output = self.run_docker_with_timeout(
            [
                "compose".to_string(),
                "--project-name".to_string(),
                MANAGED_COMPOSE_PROJECT_NAME.to_string(),
                "--file".to_string(),
                compose_file.to_string_lossy().into_owned(),
                "down".to_string(),
                "--remove-orphans".to_string(),
            ],
            COMPOSE_TIMEOUT,
        )?;

        ensure_successful_output("docker compose down", &output)?;

        Ok(Some(format!(
            "Managed Docker Compose project `{MANAGED_COMPOSE_PROJECT_NAME}` stopped from {}.",
            compose_file.to_string_lossy()
        )))
    }

    fn generate_project_compose(
        &self,
        project: &Project,
        php_version: &RuntimeVersion,
    ) -> AppResult<crate::domain::project::project_docker::ProjectDockerComposeConfig> {
        self.compose_generator().generate(project, php_version)
    }

    fn get_project_status(&self, project: &Project) -> AppResult<ProjectDockerStatus> {
        let compose_file = self.project_compose_file(project)?;

        if !compose_file.exists() {
            return self.status_for_project(
                project,
                ProjectDockerState::NotGenerated,
                None,
                None,
                None,
                "Project Compose file has not been generated yet.".to_string(),
            );
        }

        let probe = self.probe_engine()?;
        if !probe.cli_found || !probe.engine_running {
            return self.status_for_project(
                project,
                ProjectDockerState::Unavailable,
                Some(compose_file),
                None,
                None,
                probe.status_message,
            );
        }

        let project_name = self.project_compose_name(project)?;
        let output = self.run_compose(
            &project_name,
            &compose_file,
            ["ps", "--format", "json"],
            DOCKER_TIMEOUT,
        )?;

        if output.timed_out {
            return self.status_for_project(
                project,
                ProjectDockerState::Failed,
                Some(compose_file),
                None,
                None,
                "Docker Compose status timed out.".to_string(),
            );
        }

        if output.exit_code != Some(0) {
            return self.status_for_project(
                project,
                ProjectDockerState::Failed,
                Some(compose_file),
                None,
                None,
                format!(
                    "Docker Compose status failed. {}",
                    summarize_output(&output)
                ),
            );
        }

        let summary = parse_compose_status(&output.stdout);
        let published_port = if summary.running {
            self.project_port(&project_name, &compose_file)
        } else {
            None
        };

        self.status_for_project(
            project,
            if summary.running {
                ProjectDockerState::Running
            } else {
                ProjectDockerState::Stopped
            },
            Some(compose_file),
            summary.container_id,
            published_port,
            if summary.running {
                "Project Docker Compose runtime is running.".to_string()
            } else {
                "Project Docker Compose runtime is stopped.".to_string()
            },
        )
    }

    fn start_project(
        &self,
        config: &crate::domain::project::project_docker::ProjectDockerComposeConfig,
    ) -> AppResult<ProjectDockerActionResult> {
        let compose_file = PathBuf::from(&config.compose_file_path);
        validate_compose_file_path(&compose_file)?;
        let output = self.run_compose(
            &config.compose_project_name,
            &compose_file,
            ["up", "--detach", "--remove-orphans"],
            COMPOSE_TIMEOUT,
        )?;

        ensure_successful_output("docker compose up", &output)?;

        let project = Project {
            id: config.project_id.clone(),
            name: config.compose_project_name.clone(),
            document_root: crate::domain::project::project_path::ProjectPath(
                config.document_root.clone(),
            ),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let status = self.get_project_status(&project)?;

        Ok(ProjectDockerActionResult {
            project_id: config.project_id.clone(),
            action: ProjectDockerAction::Start,
            status,
            message: "Project Docker Compose runtime started.".to_string(),
        })
    }

    fn stop_project(&self, project: &Project) -> AppResult<ProjectDockerActionResult> {
        let compose_file = self.project_compose_file(project)?;
        if !compose_file.exists() {
            let status = self.status_for_project(
                project,
                ProjectDockerState::NotGenerated,
                None,
                None,
                None,
                "Project Compose file has not been generated yet.".to_string(),
            )?;

            return Ok(ProjectDockerActionResult {
                project_id: project.id.clone(),
                action: ProjectDockerAction::Stop,
                status,
                message: "Project Docker runtime was not generated, so nothing was stopped."
                    .to_string(),
            });
        }

        let project_name = self.project_compose_name(project)?;
        let output = self.run_compose(
            &project_name,
            &compose_file,
            ["down", "--remove-orphans"],
            COMPOSE_TIMEOUT,
        )?;

        ensure_successful_output("docker compose down", &output)?;

        Ok(ProjectDockerActionResult {
            project_id: project.id.clone(),
            action: ProjectDockerAction::Stop,
            status: self.get_project_status(project)?,
            message: "Project Docker Compose runtime stopped.".to_string(),
        })
    }

    fn restart_project(
        &self,
        config: &crate::domain::project::project_docker::ProjectDockerComposeConfig,
    ) -> AppResult<ProjectDockerActionResult> {
        let project = Project {
            id: config.project_id.clone(),
            name: config.compose_project_name.clone(),
            document_root: crate::domain::project::project_path::ProjectPath(
                config.document_root.clone(),
            ),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let _ = self.stop_project(&project)?;
        let mut result = self.start_project(config)?;
        result.action = ProjectDockerAction::Restart;
        result.message = "Project Docker Compose runtime restarted.".to_string();

        Ok(result)
    }
}

impl Default for DockerCliClient {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_compose_file_path(path: &Path) -> AppResult<()> {
    if !path.is_absolute() {
        return Err(AppError::Validation(format!(
            "{MANAGED_COMPOSE_FILE_ENV} must be an absolute path"
        )));
    }

    if !path.is_file() {
        return Err(AppError::Validation(format!(
            "{MANAGED_COMPOSE_FILE_ENV} must point to an existing Compose file"
        )));
    }

    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if !matches!(extension.as_str(), "yaml" | "yml") {
        return Err(AppError::Validation(format!(
            "{MANAGED_COMPOSE_FILE_ENV} must point to a .yaml or .yml file"
        )));
    }

    Ok(())
}

fn managed_compose_status_message(result: AppResult<Option<PathBuf>>) -> String {
    match result {
        Ok(Some(path)) => format!(
            " Managed Compose file configured: {}.",
            path.to_string_lossy()
        ),
        Ok(None) => " No managed Compose file is configured.".to_string(),
        Err(error) => format!(" Managed Compose configuration is invalid: {error}."),
    }
}

fn parse_compose_project_count(contents: &str) -> Option<usize> {
    let trimmed = contents.trim();

    if trimmed.is_empty() {
        return Some(0);
    }

    let value = serde_json::from_str::<Value>(trimmed).ok()?;

    match value {
        Value::Array(projects) => Some(projects.len()),
        _ => None,
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct ComposeStatusSummary {
    container_id: Option<String>,
    running: bool,
}

fn parse_compose_status(contents: &str) -> ComposeStatusSummary {
    let trimmed = contents.trim();

    if trimmed.is_empty() {
        return ComposeStatusSummary::default();
    }

    if let Ok(Value::Array(containers)) = serde_json::from_str::<Value>(trimmed) {
        return summarize_compose_values(&containers);
    }

    let containers = trimmed
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect::<Vec<_>>();

    summarize_compose_values(&containers)
}

fn summarize_compose_values(containers: &[Value]) -> ComposeStatusSummary {
    let mut summary = ComposeStatusSummary::default();

    for container in containers {
        let service_matches = container
            .get("Service")
            .and_then(Value::as_str)
            .is_none_or(|service| service == PROJECT_DOCKER_SERVICE_NAME);

        if !service_matches {
            continue;
        }

        if summary.container_id.is_none() {
            summary.container_id = container
                .get("ID")
                .or_else(|| container.get("Id"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
        }

        let state = container
            .get("State")
            .or_else(|| container.get("Status"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_ascii_lowercase();

        summary.running |= state.contains("running") || state == "up";
    }

    summary
}

fn parse_published_port(contents: &str) -> Option<u16> {
    let endpoint = contents
        .lines()
        .find(|line| !line.trim().is_empty())?
        .trim();
    let port = endpoint.rsplit(':').next()?;

    port.parse::<u16>().ok()
}

fn default_project_compose_root() -> PathBuf {
    ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP")
        .map(|dirs| dirs.data_local_dir().join("docker").join("projects"))
        .unwrap_or_else(|| {
            std::env::temp_dir()
                .join("AxiomPHP")
                .join("docker")
                .join("projects")
        })
}

fn ensure_successful_output(label: &str, output: &ProcessOutput) -> AppResult<()> {
    if output.timed_out {
        return Err(AppError::Infrastructure(format!("{label} timed out")));
    }

    if output.exit_code == Some(0) {
        return Ok(());
    }

    Err(AppError::Infrastructure(format!(
        "{label} failed. {}",
        summarize_output(output)
    )))
}

fn summarize_output(output: &ProcessOutput) -> String {
    let text = if output.stderr.trim().is_empty() {
        output.stdout.trim()
    } else {
        output.stderr.trim()
    };

    if text.is_empty() {
        "No Docker diagnostic output was returned.".to_string()
    } else {
        text.lines().rev().take(4).collect::<Vec<_>>().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compose_project_count() {
        let contents = r#"[{"Name":"axiom"},{"Name":"payments"}]"#;

        assert_eq!(parse_compose_project_count(contents), Some(2));
    }

    #[test]
    fn treats_empty_compose_output_as_zero_projects() {
        assert_eq!(parse_compose_project_count(""), Some(0));
    }

    #[test]
    fn rejects_relative_compose_file_paths() {
        let result = validate_compose_file_path(Path::new("docker-compose.yml"));

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn parses_compose_status_from_array() {
        let summary = parse_compose_status(
            r#"[{"ID":"abc","Service":"php","State":"running"},{"ID":"def","Service":"db","State":"exited"}]"#,
        );

        assert!(summary.running);
        assert_eq!(summary.container_id.as_deref(), Some("abc"));
    }

    #[test]
    fn parses_compose_status_from_json_lines() {
        let summary =
            parse_compose_status("{\"ID\":\"abc\",\"Service\":\"php\",\"State\":\"exited\"}\n");

        assert!(!summary.running);
        assert_eq!(summary.container_id.as_deref(), Some("abc"));
    }

    #[test]
    fn parses_published_ports() {
        assert_eq!(parse_published_port("127.0.0.1:49153\n"), Some(49153));
    }
}

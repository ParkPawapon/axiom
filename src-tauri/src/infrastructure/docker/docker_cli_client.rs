use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
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

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct DockerCliClient;

impl DockerCliClient {
    pub fn new() -> Self {
        Self
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
}

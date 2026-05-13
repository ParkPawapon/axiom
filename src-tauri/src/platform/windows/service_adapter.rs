use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::domain::service::service_status::ServiceStatus;
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::infrastructure::services::adapters::executable_resolver::ExecutableResolver;
use crate::infrastructure::services::adapters::service_lifecycle_adapter::ServiceLifecycleActionResult;
use crate::infrastructure::services::adapters::service_status_adapter::ServiceProbeResult;
use crate::ports::process_manager::ProcessManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const LIFECYCLE_TIMEOUT: Duration = Duration::from_secs(20);
const OUTPUT_LIMIT_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct WindowsServiceDefinition {
    pub service_name: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WindowsServiceAdapter {
    display_name: &'static str,
    definitions: &'static [WindowsServiceDefinition],
}

impl WindowsServiceAdapter {
    pub const fn new(
        display_name: &'static str,
        definitions: &'static [WindowsServiceDefinition],
    ) -> Self {
        Self {
            display_name,
            definitions,
        }
    }

    pub fn probe(&self) -> ServiceProbeResult {
        match self.find_service_state() {
            Ok(Some((service_name, status))) => match status {
                ServiceStatus::Running => ServiceProbeResult::running(format!(
                    "{} is running as Windows service {}.",
                    self.display_name, service_name
                )),
                ServiceStatus::Stopped => ServiceProbeResult::stopped(format!(
                    "{} Windows service {} is installed but stopped.",
                    self.display_name, service_name
                )),
                _ => ServiceProbeResult::failed(format!(
                    "{} Windows service {} returned an unsupported state.",
                    self.display_name, service_name
                )),
            },
            Ok(None) => ServiceProbeResult::not_configured(format!(
                "{} Windows service was not found in the supported service-name allowlist.",
                self.display_name
            )),
            Err(error) => ServiceProbeResult::failed(format!(
                "{} Windows service probe failed safely: {error}",
                self.display_name
            )),
        }
    }

    pub fn start(&self) -> AppResult<ServiceLifecycleActionResult> {
        self.run_lifecycle_command("start", "start")
    }

    pub fn stop(&self) -> AppResult<ServiceLifecycleActionResult> {
        self.run_lifecycle_command("stop", "stop")
    }

    pub fn restart(&self) -> AppResult<ServiceLifecycleActionResult> {
        let Some((service_name, _status)) = self.find_service_state()? else {
            return Ok(ServiceLifecycleActionResult::blocked(
                format!(
                    "{} cannot restart because no supported Windows service is configured.",
                    self.display_name
                ),
                self.probe(),
            ));
        };

        let stop_output = self.run_sc(["stop".to_string(), service_name.to_string()])?;
        if stop_output.exit_code != Some(0) && !sc_output_contains_already_stopped(&stop_output) {
            ensure_successful_output("sc.exe stop", &stop_output)?;
        }

        let start_output = self.run_sc(["start".to_string(), service_name.to_string()])?;
        ensure_successful_output("sc.exe start", &start_output)?;

        Ok(ServiceLifecycleActionResult::completed(
            format!(
                "{} restart requested through Windows service {}.",
                self.display_name, service_name
            ),
            self.probe(),
        ))
    }

    fn run_lifecycle_command(
        &self,
        sc_action: &'static str,
        message_action: &'static str,
    ) -> AppResult<ServiceLifecycleActionResult> {
        let Some((service_name, _status)) = self.find_service_state()? else {
            return Ok(ServiceLifecycleActionResult::blocked(
                format!(
                    "{} cannot {message_action} because no supported Windows service is configured.",
                    self.display_name
                ),
                self.probe(),
            ));
        };

        let output = self.run_sc([sc_action.to_string(), service_name.to_string()])?;
        ensure_successful_output(&format!("sc.exe {sc_action}"), &output)?;

        Ok(ServiceLifecycleActionResult::completed(
            format!(
                "{} {message_action} requested through Windows service {}.",
                self.display_name, service_name
            ),
            self.probe(),
        ))
    }

    fn find_service_state(&self) -> AppResult<Option<(&'static str, ServiceStatus)>> {
        for definition in self.definitions {
            let output = self.run_sc(["query".to_string(), definition.service_name.to_string()])?;

            if output.exit_code == Some(0) && !output.timed_out {
                return Ok(Some((
                    definition.service_name,
                    windows_service_status_from_output(&output),
                )));
            }
        }

        Ok(None)
    }

    fn run_sc(&self, args: impl IntoIterator<Item = String>) -> AppResult<ProcessOutput> {
        let sc_path = resolve_program("sc.exe", &["C:\\Windows\\System32\\sc.exe"])?;
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([sc_path.clone()])
                .with_default_timeout(LIFECYCLE_TIMEOUT)
                .with_max_output_bytes(OUTPUT_LIMIT_BYTES),
        );

        runner.execute(
            ProcessCommand::new(sc_path.to_string_lossy().into_owned())
                .args(args)
                .timeout(LIFECYCLE_TIMEOUT),
        )
    }
}

fn windows_service_status_from_output(output: &ProcessOutput) -> ServiceStatus {
    let text = format!("{}\n{}", output.stdout, output.stderr).to_ascii_uppercase();

    if text.contains("RUNNING") {
        ServiceStatus::Running
    } else if text.contains("STOPPED") {
        ServiceStatus::Stopped
    } else {
        ServiceStatus::Unknown
    }
}

fn sc_output_contains_already_stopped(output: &ProcessOutput) -> bool {
    let text = format!("{}\n{}", output.stdout, output.stderr).to_ascii_lowercase();

    text.contains("service has not been started") || text.contains("already stopped")
}

fn resolve_program(program_name: &str, fallback_paths: &[&str]) -> AppResult<PathBuf> {
    if let Some(path) = ExecutableResolver::from_env().resolve(program_name) {
        return Ok(path);
    }

    fallback_paths
        .iter()
        .map(Path::new)
        .find(|path| path.is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "required executable `{program_name}` was not found"
            ))
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
        "No diagnostic output was returned.".to_string()
    } else {
        text.lines().rev().take(4).collect::<Vec<_>>().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_running_service_state() {
        let output = ProcessOutput {
            exit_code: Some(0),
            stdout: "STATE              : 4  RUNNING".to_string(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            duration_ms: 1,
        };

        assert_eq!(
            windows_service_status_from_output(&output),
            ServiceStatus::Running
        );
    }

    #[test]
    fn parses_stopped_service_state() {
        let output = ProcessOutput {
            exit_code: Some(0),
            stdout: "STATE              : 1  STOPPED".to_string(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            duration_ms: 1,
        };

        assert_eq!(
            windows_service_status_from_output(&output),
            ServiceStatus::Stopped
        );
    }
}

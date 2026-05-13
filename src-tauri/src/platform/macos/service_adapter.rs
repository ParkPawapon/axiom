use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

use directories::BaseDirs;

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
pub struct MacosLaunchdServiceDefinition {
    pub label: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MacosServiceAdapter {
    service_name: &'static str,
    definitions: &'static [MacosLaunchdServiceDefinition],
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct LaunchdCandidate {
    label: &'static str,
    target: String,
    bootstrap_domain: String,
    plist_path: Option<PathBuf>,
}

impl MacosServiceAdapter {
    pub const fn new(
        service_name: &'static str,
        definitions: &'static [MacosLaunchdServiceDefinition],
    ) -> Self {
        Self {
            service_name,
            definitions,
        }
    }

    pub fn probe(&self) -> ServiceProbeResult {
        match self.load_candidate_state() {
            Ok(Some((candidate, Some(output)))) => {
                let state = launchd_status_from_output(&output);
                match state {
                    ServiceStatus::Running => ServiceProbeResult::running(format!(
                        "{} is running through launchd label {}.",
                        self.service_name, candidate.label
                    )),
                    _ => ServiceProbeResult::stopped(format!(
                        "{} launchd label {} is loaded but not running.",
                        self.service_name, candidate.label
                    )),
                }
            }
            Ok(Some((candidate, None))) => ServiceProbeResult::stopped(format!(
                "{} launchd plist is installed for label {}.",
                self.service_name, candidate.label
            )),
            Ok(None) => ServiceProbeResult::not_configured(format!(
                "{} launchd service was not found in the supported label allowlist.",
                self.service_name
            )),
            Err(error) => ServiceProbeResult::failed(format!(
                "{} launchd probe failed safely: {error}",
                self.service_name
            )),
        }
    }

    pub fn start(&self) -> AppResult<ServiceLifecycleActionResult> {
        self.start_or_restart("start")
    }

    pub fn stop(&self) -> AppResult<ServiceLifecycleActionResult> {
        let Some((candidate, loaded_output)) = self.load_candidate_state()? else {
            return Ok(ServiceLifecycleActionResult::blocked(
                format!(
                    "{} cannot stop because no supported launchd label is configured.",
                    self.service_name
                ),
                self.probe(),
            ));
        };

        if loaded_output.is_none() {
            return Ok(ServiceLifecycleActionResult::blocked(
                format!(
                    "{} is already stopped because launchd label {} is not loaded.",
                    self.service_name, candidate.label
                ),
                self.probe(),
            ));
        }

        self.run_launchctl(["bootout".to_string(), candidate.target.clone()])?;

        Ok(ServiceLifecycleActionResult::completed(
            format!(
                "{} stop requested through launchd label {}.",
                self.service_name, candidate.label
            ),
            self.probe(),
        ))
    }

    pub fn restart(&self) -> AppResult<ServiceLifecycleActionResult> {
        self.start_or_restart("restart")
    }

    fn start_or_restart(&self, action: &str) -> AppResult<ServiceLifecycleActionResult> {
        let Some((candidate, loaded_output)) = self.load_candidate_state()? else {
            return Ok(ServiceLifecycleActionResult::blocked(
                format!(
                    "{} cannot {action} because no supported launchd label is configured.",
                    self.service_name
                ),
                self.probe(),
            ));
        };

        if loaded_output.is_none() {
            let Some(plist_path) = &candidate.plist_path else {
                return Ok(ServiceLifecycleActionResult::blocked(
                    format!(
                        "{} launchd label {} is not loaded and no plist path is available.",
                        self.service_name, candidate.label
                    ),
                    self.probe(),
                ));
            };

            self.run_launchctl([
                "bootstrap".to_string(),
                candidate.bootstrap_domain.clone(),
                plist_path.to_string_lossy().into_owned(),
            ])?;
        }

        self.run_launchctl([
            "kickstart".to_string(),
            "-k".to_string(),
            candidate.target.clone(),
        ])?;

        Ok(ServiceLifecycleActionResult::completed(
            format!(
                "{} {action} requested through launchd label {}.",
                self.service_name, candidate.label
            ),
            self.probe(),
        ))
    }

    fn load_candidate_state(&self) -> AppResult<Option<(LaunchdCandidate, Option<ProcessOutput>)>> {
        let candidates = self.launchd_candidates()?;

        for candidate in &candidates {
            let output = self.run_launchctl_without_success_check([
                "print".to_string(),
                candidate.target.clone(),
            ])?;

            if output.exit_code == Some(0) && !output.timed_out {
                return Ok(Some((candidate.clone(), Some(output))));
            }
        }

        Ok(candidates
            .into_iter()
            .find(|candidate| candidate.plist_path.is_some())
            .map(|candidate| (candidate, None)))
    }

    fn launchd_candidates(&self) -> AppResult<Vec<LaunchdCandidate>> {
        let uid = current_uid()?;
        let home_dir = BaseDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .ok_or_else(|| {
                AppError::Configuration("home directory could not be resolved".to_string())
            })?;
        let user_domain = format!("gui/{uid}");

        Ok(self
            .definitions
            .iter()
            .flat_map(|definition| {
                let launch_agent_path = home_dir
                    .join("Library")
                    .join("LaunchAgents")
                    .join(format!("{}.plist", definition.label));
                let system_daemon_path =
                    PathBuf::from(format!("/Library/LaunchDaemons/{}.plist", definition.label));

                [
                    LaunchdCandidate {
                        label: definition.label,
                        target: format!("{user_domain}/{}", definition.label),
                        bootstrap_domain: user_domain.clone(),
                        plist_path: launch_agent_path.exists().then_some(launch_agent_path),
                    },
                    LaunchdCandidate {
                        label: definition.label,
                        target: format!("system/{}", definition.label),
                        bootstrap_domain: "system".to_string(),
                        plist_path: system_daemon_path.exists().then_some(system_daemon_path),
                    },
                ]
            })
            .collect())
    }

    fn run_launchctl(&self, args: impl IntoIterator<Item = String>) -> AppResult<ProcessOutput> {
        let output = self.run_launchctl_without_success_check(args)?;

        ensure_successful_output("launchctl", &output)?;

        Ok(output)
    }

    fn run_launchctl_without_success_check(
        &self,
        args: impl IntoIterator<Item = String>,
    ) -> AppResult<ProcessOutput> {
        let launchctl_path = resolve_program("launchctl", &["/bin/launchctl"])?;
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([launchctl_path.clone()])
                .with_default_timeout(LIFECYCLE_TIMEOUT)
                .with_max_output_bytes(OUTPUT_LIMIT_BYTES),
        );

        runner.execute(
            ProcessCommand::new(launchctl_path.to_string_lossy().into_owned())
                .args(args)
                .timeout(LIFECYCLE_TIMEOUT),
        )
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct MacosDockerDesktopAdapter;

impl MacosDockerDesktopAdapter {
    pub fn start() -> AppResult<ServiceLifecycleActionResult> {
        let open_path = resolve_program("open", &["/usr/bin/open"])?;
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([open_path.clone()])
                .with_default_timeout(LIFECYCLE_TIMEOUT)
                .with_max_output_bytes(OUTPUT_LIMIT_BYTES),
        );

        let output = runner.execute(
            ProcessCommand::new(open_path.to_string_lossy().into_owned())
                .args(["-gja", "Docker"])
                .timeout(LIFECYCLE_TIMEOUT),
        )?;
        ensure_successful_output("open Docker", &output)?;

        Ok(ServiceLifecycleActionResult::completed(
            "Docker Desktop start requested through the macOS application launcher.",
            ServiceProbeResult::stopped(
                "Docker Desktop start was requested. The Docker engine may take a moment to become ready.",
            ),
        ))
    }

    pub fn stop() -> AppResult<ServiceLifecycleActionResult> {
        let shutdown_path = PathBuf::from("/Applications/Docker.app/Contents/MacOS/com.docker.cli");

        if !shutdown_path.is_file() {
            return Ok(ServiceLifecycleActionResult::blocked(
                "Docker Desktop shutdown helper was not found at the expected signed application path.",
                ServiceProbeResult::not_configured(
                    "Docker Desktop shutdown helper is not configured on this macOS host.",
                ),
            ));
        }

        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([shutdown_path.clone()])
                .with_default_timeout(LIFECYCLE_TIMEOUT)
                .with_max_output_bytes(OUTPUT_LIMIT_BYTES),
        );
        let output = runner.execute(
            ProcessCommand::new(shutdown_path.to_string_lossy().into_owned())
                .args(["-Shutdown"])
                .timeout(LIFECYCLE_TIMEOUT),
        )?;
        ensure_successful_output("Docker Desktop shutdown", &output)?;

        Ok(ServiceLifecycleActionResult::completed(
            "Docker Desktop shutdown requested through the Docker CLI helper.",
            ServiceProbeResult::stopped("Docker Desktop shutdown was requested."),
        ))
    }

    pub fn restart() -> AppResult<ServiceLifecycleActionResult> {
        let stop_result = Self::stop()?;

        if !stop_result.executed {
            return Ok(stop_result);
        }

        Self::start().map(|mut result| {
            result.message = format!("{} {}", stop_result.message, result.message);
            result
        })
    }
}

fn current_uid() -> AppResult<String> {
    if let Ok(uid) = env::var("UID") {
        if uid.chars().all(|char| char.is_ascii_digit()) && !uid.is_empty() {
            return Ok(uid);
        }
    }

    let id_path = resolve_program("id", &["/usr/bin/id"])?;
    let runner = CommandRunner::new(
        CommandPolicy::deny_all()
            .allow_program_paths([id_path.clone()])
            .with_default_timeout(Duration::from_secs(2))
            .with_max_output_bytes(256),
    );
    let output = runner.execute(
        ProcessCommand::new(id_path.to_string_lossy().into_owned())
            .args(["-u"])
            .timeout(Duration::from_secs(2)),
    )?;
    ensure_successful_output("id -u", &output)?;
    let uid = output.stdout.trim();

    if uid.chars().all(|char| char.is_ascii_digit()) && !uid.is_empty() {
        Ok(uid.to_string())
    } else {
        Err(AppError::Infrastructure(
            "current user id could not be parsed".to_string(),
        ))
    }
}

fn launchd_status_from_output(output: &ProcessOutput) -> ServiceStatus {
    let text = format!("{}\n{}", output.stdout, output.stderr).to_ascii_lowercase();

    if text.contains("state = running") || text.contains("\npid = ") {
        ServiceStatus::Running
    } else {
        ServiceStatus::Stopped
    }
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
    fn parses_running_launchd_state() {
        let output = ProcessOutput {
            exit_code: Some(0),
            stdout: "state = running\npid = 123".to_string(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            duration_ms: 1,
        };

        assert_eq!(launchd_status_from_output(&output), ServiceStatus::Running);
    }

    #[test]
    fn parses_stopped_launchd_state() {
        let output = ProcessOutput {
            exit_code: Some(0),
            stdout: "state = waiting".to_string(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            duration_ms: 1,
        };

        assert_eq!(launchd_status_from_output(&output), ServiceStatus::Stopped);
    }
}

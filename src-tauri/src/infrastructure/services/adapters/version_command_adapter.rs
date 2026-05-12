use std::time::Duration;

use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::ports::process_manager::ProcessManager;

use super::executable_resolver::ExecutableResolver;
use super::service_status_adapter::{ServiceProbeResult, ServiceStatusAdapter};

const PROBE_TIMEOUT: Duration = Duration::from_secs(2);
const PROBE_OUTPUT_LIMIT_BYTES: usize = 4 * 1024;
const MAX_STATUS_DETAIL_CHARS: usize = 160;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct VersionCommandCandidate {
    pub program_name: &'static str,
    pub args: &'static [&'static str],
    pub display_name: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VersionCommandAdapter {
    service_name: &'static str,
    candidates: &'static [VersionCommandCandidate],
}

impl VersionCommandAdapter {
    pub const fn new(
        service_name: &'static str,
        candidates: &'static [VersionCommandCandidate],
    ) -> Self {
        Self {
            service_name,
            candidates,
        }
    }
}

impl ServiceStatusAdapter for VersionCommandAdapter {
    fn probe(&self) -> ServiceProbeResult {
        let resolver = ExecutableResolver::from_env();

        for candidate in self.candidates {
            let Some(program_path) = resolver.resolve(candidate.program_name) else {
                continue;
            };

            let runner = CommandRunner::new(
                CommandPolicy::deny_all()
                    .allow_program_paths([program_path.clone()])
                    .with_default_timeout(PROBE_TIMEOUT)
                    .with_max_output_bytes(PROBE_OUTPUT_LIMIT_BYTES),
            );
            let command = ProcessCommand::new(program_path.to_string_lossy().into_owned())
                .args(candidate.args.iter().copied())
                .timeout(PROBE_TIMEOUT);

            return match runner.execute(command) {
                Ok(output) => probe_result_from_output(self.service_name, candidate, output),
                Err(_error) => ServiceProbeResult::failed(format!(
                    "{} executable was resolved, but the allowlisted version probe failed.",
                    self.service_name
                )),
            };
        }

        ServiceProbeResult::not_configured(format!(
            "{} executable was not found on PATH. Configure an explicit runtime path before lifecycle actions are enabled.",
            self.service_name
        ))
    }
}

fn probe_result_from_output(
    service_name: &str,
    candidate: &VersionCommandCandidate,
    output: ProcessOutput,
) -> ServiceProbeResult {
    if output.timed_out {
        return ServiceProbeResult::failed(format!(
            "{} version probe timed out before lifecycle actions were enabled.",
            service_name
        ));
    }

    if output.exit_code != Some(0) {
        return ServiceProbeResult::failed(format!(
            "{} version probe exited unsuccessfully. Lifecycle actions remain disabled.",
            service_name
        ));
    }

    let detail = first_output_line(&output)
        .map(truncate_status_detail)
        .unwrap_or_else(|| candidate.display_name.to_string());

    ServiceProbeResult::detected(format!(
        "{} detected through allowlisted version probe: {}. Lifecycle actions remain disabled.",
        service_name, detail
    ))
}

fn first_output_line(output: &ProcessOutput) -> Option<String> {
    output
        .stdout
        .lines()
        .chain(output.stderr.lines())
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn truncate_status_detail(detail: String) -> String {
    let mut chars = detail.chars();
    let truncated = chars
        .by_ref()
        .take(MAX_STATUS_DETAIL_CHARS)
        .collect::<String>();

    if chars.next().is_none() {
        truncated
    } else {
        format!("{truncated}...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUSTC_CANDIDATES: &[VersionCommandCandidate] = &[VersionCommandCandidate {
        program_name: "rustc",
        args: &["--version"],
        display_name: "Rust compiler",
    }];

    #[test]
    fn detects_allowlisted_version_command() {
        let adapter = VersionCommandAdapter::new("Rust compiler", RUSTC_CANDIDATES);

        let result = adapter.probe();

        assert_eq!(
            result.status,
            crate::domain::service::service_status::ServiceStatus::Detected
        );
        assert!(!result.can_start);
        assert!(result.status_message.contains("allowlisted version probe"));
    }
}

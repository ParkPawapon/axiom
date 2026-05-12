use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

use crate::domain::runtime::php_runtime::DetectedPhpBinary;
use crate::domain::runtime::runtime_path::RuntimePath;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::infrastructure::services::adapters::executable_resolver::ExecutableResolver;
use crate::ports::php_runtime_detector::PhpRuntimeDetector;
use crate::ports::process_manager::ProcessManager;
use crate::shared::result::app_result::AppResult;

const PHP_VERSION_TIMEOUT: Duration = Duration::from_secs(2);
const PHP_VERSION_OUTPUT_LIMIT_BYTES: usize = 4 * 1024;

#[derive(Debug, Default, Clone, Copy)]
pub struct PhpBinaryDetector;

impl PhpBinaryDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PhpRuntimeDetector for PhpBinaryDetector {
    fn detect_php_binary(&self, version: &RuntimeVersion) -> AppResult<Option<DetectedPhpBinary>> {
        let resolver = ExecutableResolver::from_env();
        let mut checked_paths = BTreeSet::new();

        for candidate in php_binary_candidates(version.as_str()) {
            let Some(path) = resolver.resolve(&candidate) else {
                continue;
            };

            if !checked_paths.insert(path.clone()) {
                continue;
            }

            let output = run_php_version_probe(&path)?;

            if output.exit_code != Some(0) || output.timed_out {
                continue;
            }

            let Some(detected_version) = parse_php_version(&output) else {
                continue;
            };

            if detected_version != *version {
                continue;
            }

            return Ok(Some(DetectedPhpBinary {
                version: detected_version,
                path: RuntimePath(path.to_string_lossy().into_owned()),
                display_name: binary_display_name(&path, &output),
            }));
        }

        Ok(None)
    }
}

fn run_php_version_probe(path: &Path) -> AppResult<ProcessOutput> {
    let runner = CommandRunner::new(
        CommandPolicy::deny_all()
            .allow_program_paths([path.to_path_buf()])
            .with_default_timeout(PHP_VERSION_TIMEOUT)
            .with_max_output_bytes(PHP_VERSION_OUTPUT_LIMIT_BYTES),
    );
    let command = ProcessCommand::new(path.to_string_lossy().into_owned())
        .args(["--version"])
        .timeout(PHP_VERSION_TIMEOUT);

    runner.execute(command)
}

fn php_binary_candidates(version: &str) -> Vec<String> {
    let compact_version = version.replace('.', "");

    vec![
        format!("php{version}"),
        format!("php{compact_version}"),
        "php".to_string(),
    ]
}

fn parse_php_version(output: &ProcessOutput) -> Option<RuntimeVersion> {
    output
        .stdout
        .lines()
        .chain(output.stderr.lines())
        .find_map(parse_php_version_line)
}

fn parse_php_version_line(line: &str) -> Option<RuntimeVersion> {
    let (_, version_text) = line.split_once("PHP ")?;
    let branch = version_text.split_whitespace().next()?;
    let mut parts = branch.split('.');
    let major = parts.next()?;
    let minor = parts.next()?;

    RuntimeVersion::new(&format!("{major}.{minor}")).ok()
}

fn binary_display_name(path: &Path, output: &ProcessOutput) -> String {
    let binary_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("php");
    let version_line = output
        .stdout
        .lines()
        .chain(output.stderr.lines())
        .map(str::trim)
        .find(|line| line.starts_with("PHP "))
        .unwrap_or("PHP binary");

    format!("{binary_name} ({version_line})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_php_version_branch_from_version_output() {
        let output = ProcessOutput {
            exit_code: Some(0),
            stdout: "PHP 8.4.12 (cli) (built: Oct 1 2026)\n".to_string(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            duration_ms: 1,
        };

        let version = parse_php_version(&output).expect("version should parse");

        assert_eq!(version.as_str(), "8.4");
    }
}

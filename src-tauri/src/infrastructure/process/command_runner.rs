use std::fs::File;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::process::process_guard::ProcessTimeoutGuard;
use crate::ports::process_manager::ProcessManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug, Clone)]
pub struct CommandRunner {
    policy: CommandPolicy,
}

#[derive(Debug)]
struct CapturedOutput {
    bytes: Vec<u8>,
    truncated: bool,
}

impl CommandRunner {
    pub fn new(policy: CommandPolicy) -> Self {
        Self { policy }
    }

    pub fn policy(&self) -> &CommandPolicy {
        &self.policy
    }
}

impl Default for CommandRunner {
    fn default() -> Self {
        Self::new(CommandPolicy::deny_all())
    }
}

impl ProcessManager for CommandRunner {
    fn execute(&self, process_command: ProcessCommand) -> AppResult<ProcessOutput> {
        self.policy.validate(&process_command)?;

        let timeout = process_command
            .timeout
            .unwrap_or_else(|| self.policy.default_timeout());
        let guard = ProcessTimeoutGuard::new(timeout);
        let program_name = safe_program_name(&process_command.program);

        tracing::info!(
            program = %program_name,
            args_count = process_command.args.len(),
            timeout_ms = timeout.as_millis(),
            "starting managed process"
        );

        let mut command = Command::new(&process_command.program);
        command.args(&process_command.args);
        if let Some(stdin_file) = &process_command.stdin_file {
            let file = File::open(stdin_file).map_err(|error| {
                AppError::Infrastructure(format!("failed to open process stdin file: {error}"))
            })?;
            command.stdin(Stdio::from(file));
        } else {
            command.stdin(Stdio::null());
        }
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.env_clear();
        apply_minimal_environment(&mut command);
        command.envs(&process_command.env);

        if let Some(current_dir) = &process_command.current_dir {
            command.current_dir(current_dir);
        }

        let mut child = command.spawn().map_err(|error| {
            AppError::Infrastructure(format!("failed to spawn managed process: {error}"))
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            AppError::Infrastructure("failed to capture process stdout".to_string())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            AppError::Infrastructure("failed to capture process stderr".to_string())
        })?;

        let max_output_bytes = self.policy.max_output_bytes();
        let stdout_reader = thread::spawn(move || read_limited(stdout, max_output_bytes));
        let stderr_reader = thread::spawn(move || read_limited(stderr, max_output_bytes));
        let mut timed_out = false;

        loop {
            match child.try_wait() {
                Ok(Some(_status)) => break,
                Ok(None) => {
                    if guard.is_expired() {
                        timed_out = true;
                        child.kill().map_err(|error| {
                            AppError::Infrastructure(format!(
                                "failed to kill timed out managed process: {error}"
                            ))
                        })?;
                        break;
                    }

                    thread::sleep(Duration::from_millis(25));
                }
                Err(error) => {
                    return Err(AppError::Infrastructure(format!(
                        "failed to poll managed process: {error}"
                    )));
                }
            }
        }

        let status = child.wait().map_err(|error| {
            AppError::Infrastructure(format!("failed to wait for managed process: {error}"))
        })?;

        let stdout = join_reader(stdout_reader, "stdout")?;
        let stderr = join_reader(stderr_reader, "stderr")?;
        let stdout_text = redact_sensitive_text(&String::from_utf8_lossy(&stdout.bytes));
        let stderr_text = redact_sensitive_text(&String::from_utf8_lossy(&stderr.bytes));

        tracing::info!(
            program = %program_name,
            exit_code = status.code(),
            timed_out,
            duration_ms = guard.elapsed_ms(),
            "managed process completed"
        );

        Ok(ProcessOutput {
            exit_code: status.code(),
            stdout: stdout_text,
            stderr: stderr_text,
            stdout_truncated: stdout.truncated,
            stderr_truncated: stderr.truncated,
            timed_out,
            duration_ms: guard.elapsed_ms(),
        })
    }
}

fn read_limited(mut reader: impl Read, max_output_bytes: usize) -> AppResult<CapturedOutput> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8192];
    let mut truncated = false;

    loop {
        let read_count = reader.read(&mut buffer).map_err(|error| {
            AppError::Infrastructure(format!("failed to read process output: {error}"))
        })?;

        if read_count == 0 {
            break;
        }

        let remaining = max_output_bytes.saturating_sub(bytes.len());

        if remaining == 0 {
            truncated = true;
            continue;
        }

        let writable_count = remaining.min(read_count);
        bytes.extend_from_slice(&buffer[..writable_count]);

        if writable_count < read_count {
            truncated = true;
        }
    }

    Ok(CapturedOutput { bytes, truncated })
}

fn join_reader(
    reader: thread::JoinHandle<AppResult<CapturedOutput>>,
    stream_name: &str,
) -> AppResult<CapturedOutput> {
    reader.join().map_err(|_error| {
        AppError::Infrastructure(format!("process {stream_name} reader panicked"))
    })?
}

fn apply_minimal_environment(command: &mut Command) {
    for key in minimal_environment_keys() {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
}

#[cfg(windows)]
fn minimal_environment_keys() -> &'static [&'static str] {
    &[
        "PATH",
        "Path",
        "PATHEXT",
        "SYSTEMROOT",
        "SystemRoot",
        "WINDIR",
    ]
}

#[cfg(not(windows))]
fn minimal_environment_keys() -> &'static [&'static str] {
    &["PATH"]
}

fn safe_program_name(program: &str) -> String {
    std::path::Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string()
}

pub(crate) fn redact_sensitive_text(text: &str) -> String {
    text.lines()
        .map(redact_sensitive_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn redact_sensitive_line(line: &str) -> String {
    let lowercase = line.to_ascii_lowercase();
    let sensitive_markers = [
        "access_key",
        "api_key",
        "apikey",
        "auth",
        "credential",
        "password",
        "private_key",
        "secret",
        "token",
    ];

    if sensitive_markers
        .iter()
        .any(|marker| lowercase.contains(marker))
    {
        return "[redacted sensitive process output]".to_string();
    }

    line.to_string()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn executes_allowed_program_without_shell() {
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_names(["rustc"])
                .with_default_timeout(Duration::from_secs(5)),
        );

        let output = runner
            .execute(ProcessCommand::new("rustc").args(["--version"]))
            .expect("rustc should be available in the Rust toolchain");

        assert_eq!(output.exit_code, Some(0));
        assert!(output.stdout.contains("rustc"));
        assert!(!output.timed_out);
    }

    #[test]
    fn rejects_commands_outside_policy() {
        let runner = CommandRunner::default();

        let result = runner.execute(ProcessCommand::new("rustc").args(["--version"]));

        assert!(matches!(result, Err(AppError::PermissionDenied(_))));
    }

    #[test]
    fn truncates_large_output() {
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_names(["rustc"])
                .with_max_output_bytes(8),
        );

        let output = runner
            .execute(ProcessCommand::new("rustc").args(["--version"]))
            .expect("rustc should be available in the Rust toolchain");

        assert!(output.stdout_truncated);
        assert!(output.stdout.len() <= 8);
    }

    #[test]
    fn redacts_sensitive_output_lines() {
        let redacted = redact_sensitive_text("ready\nPASSWORD=secret\nok");

        assert_eq!(redacted, "ready\n[redacted sensitive process output]\nok");
    }
}

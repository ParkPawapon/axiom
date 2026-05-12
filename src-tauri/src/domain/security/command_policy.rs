use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const DEFAULT_TIMEOUT_MS: u64 = 5_000;
const DEFAULT_MAX_ARGS: usize = 32;
const DEFAULT_MAX_ARG_BYTES: usize = 4_096;
const DEFAULT_MAX_ENV_VALUE_BYTES: usize = 8_192;
const DEFAULT_MAX_OUTPUT_BYTES: usize = 256 * 1024;

const BLOCKED_PROGRAM_NAMES: &[&str] = &[
    "bash",
    "cmd",
    "fish",
    "osascript",
    "powershell",
    "pwsh",
    "sh",
    "wscript",
    "zsh",
];

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProcessCommand {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
    pub timeout: Option<Duration>,
}

impl ProcessCommand {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
            env: BTreeMap::new(),
            timeout: None,
        }
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn current_dir(mut self, current_dir: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(current_dir.into());
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub timed_out: bool,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandPolicy {
    allowed_program_names: BTreeSet<String>,
    allowed_program_paths: BTreeSet<PathBuf>,
    blocked_program_names: BTreeSet<String>,
    default_timeout: Duration,
    max_args: usize,
    max_arg_bytes: usize,
    max_env_value_bytes: usize,
    max_output_bytes: usize,
}

impl CommandPolicy {
    pub fn deny_all() -> Self {
        Self {
            allowed_program_names: BTreeSet::new(),
            allowed_program_paths: BTreeSet::new(),
            blocked_program_names: BLOCKED_PROGRAM_NAMES
                .iter()
                .map(|program| (*program).to_string())
                .collect(),
            default_timeout: Duration::from_millis(DEFAULT_TIMEOUT_MS),
            max_args: DEFAULT_MAX_ARGS,
            max_arg_bytes: DEFAULT_MAX_ARG_BYTES,
            max_env_value_bytes: DEFAULT_MAX_ENV_VALUE_BYTES,
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
        }
    }

    pub fn allow_program_names(
        mut self,
        program_names: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        self.allowed_program_names.extend(
            program_names
                .into_iter()
                .map(|program| normalize_program_name(program.as_ref())),
        );
        self
    }

    pub fn allow_program_paths(
        mut self,
        program_paths: impl IntoIterator<Item = impl Into<PathBuf>>,
    ) -> Self {
        self.allowed_program_paths
            .extend(program_paths.into_iter().map(Into::into));
        self
    }

    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    pub fn with_max_output_bytes(mut self, max_output_bytes: usize) -> Self {
        self.max_output_bytes = max_output_bytes;
        self
    }

    pub fn default_timeout(&self) -> Duration {
        self.default_timeout
    }

    pub fn max_output_bytes(&self) -> usize {
        self.max_output_bytes
    }

    pub fn validate(&self, command: &ProcessCommand) -> AppResult<()> {
        validate_program_value(&command.program)?;
        self.validate_program_allowed(&command.program)?;
        self.validate_args(&command.args)?;
        self.validate_current_dir(command.current_dir.as_deref())?;
        self.validate_env(&command.env)?;

        if let Some(timeout) = command.timeout {
            if timeout.is_zero() {
                return Err(AppError::Validation(
                    "process timeout must be greater than zero".to_string(),
                ));
            }

            if timeout > self.default_timeout {
                return Err(AppError::Validation(
                    "process timeout exceeds the command policy limit".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn validate_program_allowed(&self, program: &str) -> AppResult<()> {
        let program_path = Path::new(program);
        let program_name = program_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(normalize_program_name)
            .ok_or_else(|| AppError::Validation("program name is invalid".to_string()))?;

        if self.blocked_program_names.contains(&program_name) {
            return Err(AppError::PermissionDenied(format!(
                "program `{program_name}` is blocked by command policy"
            )));
        }

        if program_path.components().count() > 1 {
            if self.allowed_program_paths.contains(program_path) {
                return Ok(());
            }

            return Err(AppError::PermissionDenied(
                "absolute or relative program paths must be explicitly allowed".to_string(),
            ));
        }

        if self.allowed_program_names.contains(&program_name) {
            return Ok(());
        }

        Err(AppError::PermissionDenied(format!(
            "program `{program_name}` is not allowed by command policy"
        )))
    }

    fn validate_args(&self, args: &[String]) -> AppResult<()> {
        if args.len() > self.max_args {
            return Err(AppError::Validation(
                "process argument count exceeds the command policy limit".to_string(),
            ));
        }

        for arg in args {
            validate_argument_value(arg, self.max_arg_bytes)?;
        }

        Ok(())
    }

    fn validate_current_dir(&self, current_dir: Option<&Path>) -> AppResult<()> {
        let Some(current_dir) = current_dir else {
            return Ok(());
        };

        if !current_dir.is_absolute() {
            return Err(AppError::Validation(
                "process working directory must be absolute".to_string(),
            ));
        }

        if !current_dir.is_dir() {
            return Err(AppError::Validation(
                "process working directory must exist and be a directory".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_env(&self, env: &BTreeMap<String, String>) -> AppResult<()> {
        for (key, value) in env {
            validate_environment_key(key)?;

            if value.as_bytes().contains(&0) {
                return Err(AppError::Validation(
                    "process environment values must not contain null bytes".to_string(),
                ));
            }

            if value.len() > self.max_env_value_bytes {
                return Err(AppError::Validation(
                    "process environment value exceeds the command policy limit".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Default for CommandPolicy {
    fn default() -> Self {
        Self::deny_all()
    }
}

fn validate_program_value(program: &str) -> AppResult<()> {
    let trimmed = program.trim();

    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "program must not be empty".to_string(),
        ));
    }

    if trimmed != program {
        return Err(AppError::Validation(
            "program must not include leading or trailing whitespace".to_string(),
        ));
    }

    if program.as_bytes().contains(&0) || program.contains('\n') || program.contains('\r') {
        return Err(AppError::Validation(
            "program must not contain control characters".to_string(),
        ));
    }

    Ok(())
}

fn validate_argument_value(arg: &str, max_arg_bytes: usize) -> AppResult<()> {
    if arg.as_bytes().contains(&0) {
        return Err(AppError::Validation(
            "process arguments must not contain null bytes".to_string(),
        ));
    }

    if arg.len() > max_arg_bytes {
        return Err(AppError::Validation(
            "process argument exceeds the command policy limit".to_string(),
        ));
    }

    Ok(())
}

fn validate_environment_key(key: &str) -> AppResult<()> {
    if key.is_empty() {
        return Err(AppError::Validation(
            "process environment key must not be empty".to_string(),
        ));
    }

    let mut chars = key.chars();
    let first = chars.next().expect("key is not empty");

    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(AppError::Validation(
            "process environment key must start with a letter or underscore".to_string(),
        ));
    }

    if !chars.all(|char| char.is_ascii_alphanumeric() || char == '_') {
        return Err(AppError::Validation(
            "process environment key may only contain letters, numbers, and underscores"
                .to_string(),
        ));
    }

    Ok(())
}

fn normalize_program_name(program_name: &str) -> String {
    program_name.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_shell_programs_even_when_allowed() {
        let policy = CommandPolicy::deny_all().allow_program_names(["sh"]);
        let command = ProcessCommand::new("sh");

        let result = policy.validate(&command);

        assert!(matches!(result, Err(AppError::PermissionDenied(_))));
    }

    #[test]
    fn rejects_unlisted_programs() {
        let policy = CommandPolicy::deny_all();
        let command = ProcessCommand::new("rustc");

        let result = policy.validate(&command);

        assert!(matches!(result, Err(AppError::PermissionDenied(_))));
    }

    #[test]
    fn accepts_allowed_program_names() {
        let policy = CommandPolicy::deny_all().allow_program_names(["rustc"]);
        let command = ProcessCommand::new("rustc").args(["--version"]);

        assert!(policy.validate(&command).is_ok());
    }

    #[test]
    fn rejects_invalid_environment_keys() {
        let policy = CommandPolicy::deny_all().allow_program_names(["rustc"]);
        let command = ProcessCommand::new("rustc").env("1SECRET", "value");

        let result = policy.validate(&command);

        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}

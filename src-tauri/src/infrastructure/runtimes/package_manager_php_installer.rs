use std::path::PathBuf;
use std::time::Duration;

use serde_json::Value;

use crate::domain::project::project_php_version::{
    PhpRuntimeInstallDiagnostic, PhpRuntimeInstallProvider, PhpRuntimeInstallRollback,
};
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::infrastructure::services::adapters::executable_resolver::ExecutableResolver;
use crate::ports::php_runtime_installer::{PhpRuntimeInstallReport, PhpRuntimeInstaller};
use crate::ports::process_manager::ProcessManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const INSTALL_TIMEOUT: Duration = Duration::from_secs(20 * 60);
const DIAGNOSTIC_TIMEOUT: Duration = Duration::from_secs(60);
const INSTALL_OUTPUT_LIMIT_BYTES: usize = 1024 * 1024;
const HOMEBREW_TRUSTED_TAP: &str = "shivammathur/php";
const HOMEBREW_TRUSTED_TAP_REMOTE: &str = "github.com/shivammathur/homebrew-php";
const HOMEBREW_TRUSTED_TAP_REPOSITORY: &str = "shivammathur/homebrew-php";
const SCOOP_TRUSTED_BUCKET: &str = "versions";
const SCOOP_TRUSTED_BUCKET_REMOTE: &str = "github.com/ScoopInstaller/Versions";
const SCOOP_TRUSTED_BUCKET_REPOSITORY: &str = "ScoopInstaller/Versions";
const SCOOP_TRUSTED_SOURCE_HOSTS: &[&str] = &["windows.php.net", "museum.php.net", "github.com"];
const SCOOP_EXECUTABLE_CANDIDATES: &[&str] = &["scoop.cmd", "scoop.exe", "scoop"];

#[derive(Debug, Default, Clone, Copy)]
pub struct PackageManagerPhpInstaller;

#[derive(Debug, Clone, Eq, PartialEq)]
struct InstallPlan {
    provider: PhpRuntimeInstallProvider,
    manager_path: PathBuf,
    package_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct InstallStep {
    args: Vec<String>,
    description: &'static str,
    timeout: Duration,
}

#[derive(Debug, Default)]
struct InstallContext {
    diagnostics: Vec<PhpRuntimeInstallDiagnostic>,
    rollback: Option<PhpRuntimeInstallRollback>,
}

impl InstallContext {
    fn info(&mut self, code: &str, message: impl Into<String>) {
        self.diagnostics
            .push(PhpRuntimeInstallDiagnostic::info(code, message));
    }

    fn warning(&mut self, code: &str, message: impl Into<String>) {
        self.diagnostics
            .push(PhpRuntimeInstallDiagnostic::warning(code, message));
    }
}

impl PackageManagerPhpInstaller {
    pub fn new() -> Self {
        Self
    }
}

impl PhpRuntimeInstaller for PackageManagerPhpInstaller {
    fn install_php_runtime(&self, version: &RuntimeVersion) -> AppResult<PhpRuntimeInstallReport> {
        let plan = build_install_plan(version)?;
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([plan.manager_path.clone()])
                .with_default_timeout(INSTALL_TIMEOUT)
                .with_max_output_bytes(INSTALL_OUTPUT_LIMIT_BYTES),
        );
        let mut context = InstallContext::default();

        context.info(
            "packageManagerResolved",
            format!(
                "{} executable resolved to {}.",
                plan.provider.label(),
                plan.manager_path.to_string_lossy()
            ),
        );

        let package_was_installed = package_is_installed(&plan, &runner)?;

        verify_package_manager(&plan, &runner, &mut context)?;
        let source_added = ensure_trusted_source(&plan, &runner, &mut context)?;
        verify_package_metadata(&plan, &runner, &mut context)?;

        let mut completed_steps = 0_usize;
        let install_step = InstallStep {
            args: install_args(&plan),
            description: "install PHP runtime",
            timeout: INSTALL_TIMEOUT,
        };

        if let Err(error) = run_step(&plan, &runner, &install_step) {
            let rollback =
                rollback_after_failure(&plan, &runner, package_was_installed, source_added);
            let rollback_message = rollback.message.clone();
            context.rollback = Some(rollback);

            return Err(AppError::Infrastructure(format!(
                "{error} Rollback: {rollback_message}"
            )));
        }

        completed_steps += 1;
        context.info(
            "installCompleted",
            format!(
                "{} completed installation for {}.",
                plan.provider.label(),
                plan.package_name
            ),
        );

        Ok(PhpRuntimeInstallReport {
            provider: plan.provider,
            package_name: plan.package_name,
            diagnostics: context.diagnostics,
            rollback: context.rollback,
            status_message: format!(
                "{} completed {} package-manager install step(s) after source and checksum verification.",
                plan.provider.label(),
                completed_steps
            ),
        })
    }
}

fn build_install_plan(version: &RuntimeVersion) -> AppResult<InstallPlan> {
    let resolver = ExecutableResolver::from_env();

    if cfg!(target_os = "macos") {
        let manager_path = resolver.resolve("brew").ok_or_else(|| {
            AppError::NotFound("Homebrew executable was not found on PATH".to_string())
        })?;

        return Ok(homebrew_install_plan(version, manager_path));
    }

    if cfg!(windows) {
        let manager_path = resolver
            .resolve_first(SCOOP_EXECUTABLE_CANDIDATES)
            .ok_or_else(|| {
                AppError::NotFound("Scoop executable was not found on PATH".to_string())
            })?;

        return Ok(scoop_install_plan(version, manager_path));
    }

    Err(AppError::Configuration(
        "automatic PHP installation is currently supported only on macOS with Homebrew and Windows with Scoop".to_string(),
    ))
}

fn homebrew_install_plan(version: &RuntimeVersion, manager_path: PathBuf) -> InstallPlan {
    InstallPlan {
        provider: PhpRuntimeInstallProvider::Homebrew,
        manager_path,
        package_name: homebrew_package_name(version.as_str()),
    }
}

fn scoop_install_plan(version: &RuntimeVersion, manager_path: PathBuf) -> InstallPlan {
    InstallPlan {
        provider: PhpRuntimeInstallProvider::Scoop,
        manager_path,
        package_name: scoop_package_name(version.as_str()),
    }
}

fn verify_package_manager(
    plan: &InstallPlan,
    runner: &CommandRunner,
    context: &mut InstallContext,
) -> AppResult<()> {
    let output = run_diagnostic_command(plan, runner, vec!["--version".to_string()])?;

    if output.exit_code != Some(0) || output.timed_out {
        return Err(AppError::Infrastructure(format!(
            "{} diagnostic failed before install. {}",
            plan.provider.label(),
            summarize_process_output(&output)
        )));
    }

    context.info(
        "packageManagerVersion",
        format!(
            "{} version diagnostic completed. {}",
            plan.provider.label(),
            first_non_empty_output_line(&output)
        ),
    );

    Ok(())
}

fn ensure_trusted_source(
    plan: &InstallPlan,
    runner: &CommandRunner,
    context: &mut InstallContext,
) -> AppResult<bool> {
    match plan.provider {
        PhpRuntimeInstallProvider::Homebrew => {
            if !uses_external_homebrew_tap(&plan.package_name) {
                context.info(
                    "sourceTrust",
                    "Homebrew core formula source is trusted by the installer policy.",
                );
                return Ok(false);
            }

            let tap_existed = homebrew_tap_exists(plan, runner)?;

            if !tap_existed {
                run_step(
                    plan,
                    runner,
                    &InstallStep {
                        args: vec!["tap".to_string(), HOMEBREW_TRUSTED_TAP.to_string()],
                        description: "tap trusted PHP formula repository",
                        timeout: INSTALL_TIMEOUT,
                    },
                )?;
                context.info(
                    "sourceAdded",
                    format!("Homebrew tap {HOMEBREW_TRUSTED_TAP} was added."),
                );
            }

            verify_homebrew_tap_source(plan, runner)?;
            context.info(
                "sourceTrust",
                format!(
                    "Homebrew tap {HOMEBREW_TRUSTED_TAP} is trusted and points to {HOMEBREW_TRUSTED_TAP_REMOTE}."
                ),
            );

            Ok(!tap_existed)
        }
        PhpRuntimeInstallProvider::Scoop => {
            let bucket_existed = scoop_versions_bucket_exists(plan, runner)?;

            if !bucket_existed {
                run_step(
                    plan,
                    runner,
                    &InstallStep {
                        args: vec![
                            "bucket".to_string(),
                            "add".to_string(),
                            SCOOP_TRUSTED_BUCKET.to_string(),
                        ],
                        description: "add trusted Scoop versions bucket",
                        timeout: INSTALL_TIMEOUT,
                    },
                )?;
                context.info(
                    "sourceAdded",
                    format!("Scoop bucket {SCOOP_TRUSTED_BUCKET} was added."),
                );
            }

            verify_scoop_versions_bucket(plan, runner)?;
            context.info(
                "sourceTrust",
                format!(
                    "Scoop bucket {SCOOP_TRUSTED_BUCKET} is trusted and points to {SCOOP_TRUSTED_BUCKET_REMOTE}."
                ),
            );

            Ok(!bucket_existed)
        }
    }
}

fn verify_package_metadata(
    plan: &InstallPlan,
    runner: &CommandRunner,
    context: &mut InstallContext,
) -> AppResult<()> {
    match plan.provider {
        PhpRuntimeInstallProvider::Homebrew => {
            let output = run_diagnostic_command(
                plan,
                runner,
                vec![
                    "info".to_string(),
                    "--json=v2".to_string(),
                    plan.package_name.clone(),
                ],
            )?;

            if output.exit_code != Some(0) || output.timed_out {
                return Err(AppError::Infrastructure(format!(
                    "Homebrew formula metadata check failed for {}. {}",
                    plan.package_name,
                    summarize_process_output(&output)
                )));
            }

            ensure_homebrew_formula_has_checksum(&output.stdout)?;
            context.info(
                "checksumVerified",
                format!(
                    "Homebrew formula metadata for {} includes checksum data.",
                    plan.package_name
                ),
            );
        }
        PhpRuntimeInstallProvider::Scoop => {
            let output = run_diagnostic_command(
                plan,
                runner,
                vec!["cat".to_string(), plan.package_name.clone()],
            )?;

            if output.exit_code != Some(0) || output.timed_out {
                return Err(AppError::Infrastructure(format!(
                    "Scoop manifest check failed for {}. {}",
                    plan.package_name,
                    summarize_process_output(&output)
                )));
            }

            ensure_scoop_manifest_is_trusted(&output.stdout)?;
            context.info(
                "checksumVerified",
                format!(
                    "Scoop manifest for {} has trusted source URLs and checksum data.",
                    plan.package_name
                ),
            );
        }
    }

    if package_is_installed(plan, runner)? {
        context.warning(
            "packageAlreadyInstalled",
            format!(
                "{} reports {} is already installed before this request.",
                plan.provider.label(),
                plan.package_name
            ),
        );
    }

    Ok(())
}

fn package_is_installed(plan: &InstallPlan, runner: &CommandRunner) -> AppResult<bool> {
    let args = match plan.provider {
        PhpRuntimeInstallProvider::Homebrew => {
            vec![
                "list".to_string(),
                "--versions".to_string(),
                plan.package_name.clone(),
            ]
        }
        PhpRuntimeInstallProvider::Scoop => vec![
            "list".to_string(),
            scoop_installed_package_name(&plan.package_name),
        ],
    };
    let output = run_diagnostic_command(plan, runner, args)?;

    Ok(output.exit_code == Some(0)
        && !output.timed_out
        && output_text_contains_package(&output, &plan.package_name))
}

fn homebrew_tap_exists(plan: &InstallPlan, runner: &CommandRunner) -> AppResult<bool> {
    let output = run_diagnostic_command(plan, runner, vec!["tap".to_string()])?;

    if output.exit_code != Some(0) || output.timed_out {
        return Ok(false);
    }

    Ok(output
        .stdout
        .lines()
        .any(|line| line.trim() == HOMEBREW_TRUSTED_TAP))
}

fn verify_homebrew_tap_source(plan: &InstallPlan, runner: &CommandRunner) -> AppResult<()> {
    let output = run_diagnostic_command(
        plan,
        runner,
        vec![
            "tap-info".to_string(),
            "--json".to_string(),
            HOMEBREW_TRUSTED_TAP.to_string(),
        ],
    )?;

    if output.exit_code != Some(0) || output.timed_out {
        return Err(AppError::Infrastructure(format!(
            "Homebrew tap source verification failed for {HOMEBREW_TRUSTED_TAP}. {}",
            summarize_process_output(&output)
        )));
    }

    if !homebrew_tap_info_contains_trusted_remote(&output.stdout)? {
        return Err(AppError::PermissionDenied(format!(
            "Homebrew tap {HOMEBREW_TRUSTED_TAP} does not point to trusted source {HOMEBREW_TRUSTED_TAP_REMOTE}"
        )));
    }

    Ok(())
}

fn scoop_versions_bucket_exists(plan: &InstallPlan, runner: &CommandRunner) -> AppResult<bool> {
    let output =
        run_diagnostic_command(plan, runner, vec!["bucket".to_string(), "list".to_string()])?;

    if output.timed_out || output.exit_code != Some(0) {
        return Ok(false);
    }

    Ok(output.stdout.lines().any(|line| {
        line.split_whitespace()
            .any(|part| part == SCOOP_TRUSTED_BUCKET)
    }))
}

fn verify_scoop_versions_bucket(plan: &InstallPlan, runner: &CommandRunner) -> AppResult<()> {
    let output =
        run_diagnostic_command(plan, runner, vec!["bucket".to_string(), "list".to_string()])?;

    if output.exit_code != Some(0) || output.timed_out {
        return Err(AppError::Infrastructure(format!(
            "Scoop bucket verification failed. {}",
            summarize_process_output(&output)
        )));
    }

    if !scoop_bucket_list_contains_trusted_versions_bucket(&output.stdout) {
        return Err(AppError::PermissionDenied(format!(
            "Scoop bucket `{SCOOP_TRUSTED_BUCKET}` is missing or does not point to trusted source {SCOOP_TRUSTED_BUCKET_REMOTE}"
        )));
    }

    Ok(())
}

fn rollback_after_failure(
    plan: &InstallPlan,
    runner: &CommandRunner,
    package_was_installed: bool,
    source_added: bool,
) -> PhpRuntimeInstallRollback {
    let mut attempted = false;
    let mut succeeded = true;
    let mut messages = Vec::new();

    if !package_was_installed {
        attempted = true;
        let uninstall_result = run_step(
            plan,
            runner,
            &InstallStep {
                args: uninstall_args(plan),
                description: "roll back partial PHP runtime installation",
                timeout: INSTALL_TIMEOUT,
            },
        );

        match uninstall_result {
            Ok(()) => messages.push(format!("Removed partial package {}.", plan.package_name)),
            Err(error) => {
                succeeded = false;
                messages.push(format!("Could not remove partial package: {error}"));
            }
        }
    }

    if source_added {
        attempted = true;
        let remove_source_result = run_step(
            plan,
            runner,
            &InstallStep {
                args: remove_source_args(plan),
                description: "roll back temporary package source",
                timeout: INSTALL_TIMEOUT,
            },
        );

        match remove_source_result {
            Ok(()) => messages.push("Removed package source added by this request.".to_string()),
            Err(error) => {
                succeeded = false;
                messages.push(format!("Could not remove package source: {error}"));
            }
        }
    }

    if !attempted {
        messages.push("No rollback action was required because no new package source or package install was confirmed.".to_string());
    }

    PhpRuntimeInstallRollback {
        attempted,
        succeeded,
        message: messages.join(" "),
    }
}

fn run_step(plan: &InstallPlan, runner: &CommandRunner, step: &InstallStep) -> AppResult<()> {
    let output = runner.execute(
        ProcessCommand::new(plan.manager_path.to_string_lossy().into_owned())
            .args(step.args.clone())
            .timeout(step.timeout),
    )?;

    ensure_successful_step(plan, step, &output)
}

fn run_diagnostic_command(
    plan: &InstallPlan,
    runner: &CommandRunner,
    args: Vec<String>,
) -> AppResult<ProcessOutput> {
    runner.execute(
        ProcessCommand::new(plan.manager_path.to_string_lossy().into_owned())
            .args(args)
            .timeout(DIAGNOSTIC_TIMEOUT),
    )
}

fn homebrew_package_name(version: &str) -> String {
    match version {
        "8.5" => "php".to_string(),
        "8.2" | "8.3" | "8.4" => format!("php@{version}"),
        _ => format!("{HOMEBREW_TRUSTED_TAP}/php@{version}"),
    }
}

fn scoop_package_name(version: &str) -> String {
    format!("{SCOOP_TRUSTED_BUCKET}/php{}", version.replace('.', ""))
}

fn install_args(plan: &InstallPlan) -> Vec<String> {
    match plan.provider {
        PhpRuntimeInstallProvider::Homebrew => {
            vec!["install".to_string(), plan.package_name.clone()]
        }
        PhpRuntimeInstallProvider::Scoop => {
            vec!["install".to_string(), plan.package_name.clone()]
        }
    }
}

fn uninstall_args(plan: &InstallPlan) -> Vec<String> {
    match plan.provider {
        PhpRuntimeInstallProvider::Homebrew => vec![
            "uninstall".to_string(),
            "--ignore-dependencies".to_string(),
            plan.package_name.clone(),
        ],
        PhpRuntimeInstallProvider::Scoop => {
            vec![
                "uninstall".to_string(),
                scoop_installed_package_name(&plan.package_name),
            ]
        }
    }
}

fn remove_source_args(plan: &InstallPlan) -> Vec<String> {
    match plan.provider {
        PhpRuntimeInstallProvider::Homebrew => {
            vec!["untap".to_string(), HOMEBREW_TRUSTED_TAP.to_string()]
        }
        PhpRuntimeInstallProvider::Scoop => vec![
            "bucket".to_string(),
            "rm".to_string(),
            SCOOP_TRUSTED_BUCKET.to_string(),
        ],
    }
}

fn ensure_successful_step(
    plan: &InstallPlan,
    step: &InstallStep,
    output: &ProcessOutput,
) -> AppResult<()> {
    if output.timed_out {
        return Err(AppError::Infrastructure(format!(
            "{} timed out while trying to {} for {}",
            plan.provider.label(),
            step.description,
            plan.package_name
        )));
    }

    if output.exit_code == Some(0) {
        return Ok(());
    }

    Err(AppError::Infrastructure(format!(
        "{} failed to {} for {}. {}",
        plan.provider.label(),
        step.description,
        plan.package_name,
        summarize_process_output(output)
    )))
}

fn ensure_homebrew_formula_has_checksum(contents: &str) -> AppResult<()> {
    let value = parse_json(contents, "Homebrew formula metadata")?;

    if json_has_non_skip_hash_key(&value) {
        return Ok(());
    }

    Err(AppError::PermissionDenied(
        "Homebrew formula metadata did not include checksum data".to_string(),
    ))
}

fn homebrew_tap_info_contains_trusted_remote(contents: &str) -> AppResult<bool> {
    let value = parse_json(contents, "Homebrew tap metadata")?;
    let mut remotes = Vec::new();

    collect_string_values_for_key(&value, "remote", &mut remotes);

    Ok(remotes
        .iter()
        .any(|remote| github_repository_url_matches(remote, HOMEBREW_TRUSTED_TAP_REPOSITORY)))
}

fn ensure_scoop_manifest_is_trusted(contents: &str) -> AppResult<()> {
    let value = parse_json(contents, "Scoop manifest")?;

    if !manifest_has_trusted_urls(&value) {
        return Err(AppError::PermissionDenied(
            "Scoop manifest does not reference a trusted PHP runtime source".to_string(),
        ));
    }

    if !json_has_non_skip_hash_key(&value) {
        return Err(AppError::PermissionDenied(
            "Scoop manifest does not include checksum data".to_string(),
        ));
    }

    Ok(())
}

fn parse_json(contents: &str, label: &str) -> AppResult<Value> {
    serde_json::from_str(contents)
        .map_err(|error| AppError::Infrastructure(format!("{label} was not valid JSON: {error}")))
}

fn json_has_non_skip_hash_key(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, value)| {
            (key == "hash" || key == "sha256") && hash_value_is_strong(value)
                || json_has_non_skip_hash_key(value)
        }),
        Value::Array(values) => values.iter().any(json_has_non_skip_hash_key),
        _ => false,
    }
}

fn hash_value_is_strong(value: &Value) -> bool {
    match value {
        Value::String(hash) => {
            let hash = hash.trim();
            !hash.is_empty() && !hash.eq_ignore_ascii_case("skip")
        }
        Value::Array(values) => !values.is_empty() && values.iter().all(hash_value_is_strong),
        Value::Object(map) => !map.is_empty() && map.values().all(hash_value_is_strong),
        _ => false,
    }
}

fn manifest_has_trusted_urls(value: &Value) -> bool {
    let mut urls = Vec::new();
    collect_manifest_urls(value, &mut urls);

    !urls.is_empty()
        && urls.iter().all(|url| {
            SCOOP_TRUSTED_SOURCE_HOSTS
                .iter()
                .any(|host| url_matches_host(url, host))
        })
}

fn url_matches_host(url: &str, trusted_host: &str) -> bool {
    let normalized_url = url.trim().to_ascii_lowercase();
    let Some((host, _path)) = split_https_host_and_path(&normalized_url) else {
        return false;
    };
    let trusted_host = trusted_host.to_ascii_lowercase();

    host == trusted_host || host.ends_with(&format!(".{trusted_host}"))
}

fn github_repository_url_matches(url: &str, trusted_repository: &str) -> bool {
    let normalized_url = url.trim().trim_end_matches(".git").to_ascii_lowercase();
    let Some((host, path)) = split_https_host_and_path(&normalized_url) else {
        return false;
    };
    let trusted_repository = trusted_repository.to_ascii_lowercase();

    host == "github.com" && path.trim_matches('/') == trusted_repository
}

fn split_https_host_and_path(normalized_url: &str) -> Option<(&str, &str)> {
    let without_scheme = normalized_url.strip_prefix("https://")?;
    let authority = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    let host_with_port = authority.rsplit('@').next().unwrap_or_default();
    let host = host_with_port.split(':').next().unwrap_or_default();
    let path = without_scheme
        .strip_prefix(authority)
        .unwrap_or_default()
        .split(['?', '#'])
        .next()
        .unwrap_or_default();

    Some((host, path))
}

fn collect_string_values_for_key<'a>(
    value: &'a Value,
    expected_key: &str,
    values: &mut Vec<&'a str>,
) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == expected_key {
                    if let Value::String(text) = value {
                        values.push(text);
                    }
                }

                collect_string_values_for_key(value, expected_key, values);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_string_values_for_key(item, expected_key, values);
            }
        }
        _ => {}
    }
}

fn collect_manifest_urls<'a>(value: &'a Value, urls: &mut Vec<&'a str>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "url" {
                    collect_url_values(value, urls);
                } else {
                    collect_manifest_urls(value, urls);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_manifest_urls(value, urls);
            }
        }
        _ => {}
    }
}

fn collect_url_values<'a>(value: &'a Value, urls: &mut Vec<&'a str>) {
    match value {
        Value::String(url) => urls.push(url),
        Value::Array(values) => {
            for value in values {
                collect_url_values(value, urls);
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                collect_url_values(value, urls);
            }
        }
        _ => {}
    }
}

fn scoop_bucket_list_contains_trusted_versions_bucket(output: &str) -> bool {
    output.lines().any(|line| {
        line.split_whitespace()
            .any(|part| part == SCOOP_TRUSTED_BUCKET)
            && line
                .split_whitespace()
                .any(|part| github_repository_url_matches(part, SCOOP_TRUSTED_BUCKET_REPOSITORY))
    })
}

fn output_text_contains_package(output: &ProcessOutput, package_name: &str) -> bool {
    let installed_name = scoop_installed_package_name(package_name);
    output.stdout.contains(package_name)
        || output.stderr.contains(package_name)
        || output.stdout.contains(&installed_name)
        || output.stderr.contains(&installed_name)
}

fn scoop_installed_package_name(package_name: &str) -> String {
    package_name
        .rsplit('/')
        .next()
        .unwrap_or(package_name)
        .to_string()
}

fn uses_external_homebrew_tap(package_name: &str) -> bool {
    package_name.starts_with(&format!("{HOMEBREW_TRUSTED_TAP}/"))
}

fn summarize_process_output(output: &ProcessOutput) -> String {
    let text = if output.stderr.trim().is_empty() {
        output.stdout.trim()
    } else {
        output.stderr.trim()
    };

    if text.is_empty() {
        return "The package manager returned no diagnostic output.".to_string();
    }

    let mut lines = text.lines().rev().take(8).collect::<Vec<_>>();
    lines.reverse();

    lines.join("\n")
}

fn first_non_empty_output_line(output: &ProcessOutput) -> String {
    output
        .stdout
        .lines()
        .chain(output.stderr.lines())
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("No version text returned.")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_active_homebrew_versions_to_core_formulae() {
        assert_eq!(homebrew_package_name("8.5"), "php");
        assert_eq!(homebrew_package_name("8.4"), "php@8.4");
        assert_eq!(homebrew_package_name("8.2"), "php@8.2");
    }

    #[test]
    fn maps_legacy_homebrew_versions_to_trusted_tap_formulae() {
        let version = RuntimeVersion::trusted("7.4");
        let plan = homebrew_install_plan(&version, PathBuf::from("/opt/homebrew/bin/brew"));

        assert_eq!(plan.package_name, "shivammathur/php/php@7.4");
        assert!(uses_external_homebrew_tap(&plan.package_name));
    }

    #[test]
    fn maps_scoop_versions_to_versions_bucket_packages() {
        assert_eq!(scoop_package_name("8.5"), "versions/php85");
        assert_eq!(scoop_package_name("7.4"), "versions/php74");
        assert_eq!(scoop_package_name("5.6"), "versions/php56");
    }

    #[test]
    fn avoids_powershell_scoop_entrypoints() {
        assert!(!SCOOP_EXECUTABLE_CANDIDATES
            .iter()
            .any(|candidate| candidate.ends_with(".ps1")));
    }

    #[test]
    fn verifies_scoop_manifest_source_and_checksum() {
        let manifest = r#"{
          "url": "https://windows.php.net/downloads/releases/php-8.4.0-Win32.zip",
          "hash": "abc123"
        }"#;

        ensure_scoop_manifest_is_trusted(manifest).expect("manifest should be trusted");
    }

    #[test]
    fn rejects_scoop_manifest_without_checksum() {
        let manifest = r#"{
          "url": "https://windows.php.net/downloads/releases/php-8.4.0-Win32.zip"
        }"#;

        let result = ensure_scoop_manifest_is_trusted(manifest);

        assert!(matches!(result, Err(AppError::PermissionDenied(_))));
    }

    #[test]
    fn rejects_scoop_manifest_from_untrusted_source() {
        let manifest = r#"{
          "url": "https://example.com/php.zip",
          "hash": "abc123"
        }"#;

        let result = ensure_scoop_manifest_is_trusted(manifest);

        assert!(matches!(result, Err(AppError::PermissionDenied(_))));
    }

    #[test]
    fn rejects_skip_hash_values() {
        let manifest = r#"{
          "url": "https://windows.php.net/downloads/releases/php-8.4.0-Win32.zip",
          "hash": "skip"
        }"#;

        let result = ensure_scoop_manifest_is_trusted(manifest);

        assert!(matches!(result, Err(AppError::PermissionDenied(_))));
    }

    #[test]
    fn rejects_empty_hash_values() {
        let manifest = r#"{
          "url": "https://windows.php.net/downloads/releases/php-8.4.0-Win32.zip",
          "hash": {}
        }"#;

        let result = ensure_scoop_manifest_is_trusted(manifest);

        assert!(matches!(result, Err(AppError::PermissionDenied(_))));
    }

    #[test]
    fn rejects_deceptive_scoop_manifest_hosts() {
        let manifest = r#"{
          "url": "https://example.com/downloads/php.zip?mirror=windows.php.net",
          "hash": "abc123"
        }"#;

        let result = ensure_scoop_manifest_is_trusted(manifest);

        assert!(matches!(result, Err(AppError::PermissionDenied(_))));
    }

    #[test]
    fn rejects_non_https_scoop_manifest_sources() {
        let manifest = r#"{
          "url": "http://windows.php.net/downloads/releases/php-8.4.0-Win32.zip",
          "hash": "abc123"
        }"#;

        let result = ensure_scoop_manifest_is_trusted(manifest);

        assert!(matches!(result, Err(AppError::PermissionDenied(_))));
    }

    #[test]
    fn verifies_homebrew_checksum_metadata() {
        let metadata = r#"{
          "formulae": [{
            "name": "php@8.4",
            "bottle": { "stable": { "files": { "arm64_sequoia": { "sha256": "abc123" } } } }
          }]
        }"#;

        ensure_homebrew_formula_has_checksum(metadata).expect("metadata should have checksum");
    }

    #[test]
    fn verifies_homebrew_tap_trusted_remote() {
        let metadata = r#"[{
          "name": "shivammathur/php",
          "remote": "https://github.com/shivammathur/homebrew-php.git"
        }]"#;

        assert!(homebrew_tap_info_contains_trusted_remote(metadata).expect("metadata should parse"));
    }

    #[test]
    fn rejects_homebrew_tap_deceptive_remote() {
        let metadata = r#"[{
          "name": "shivammathur/php",
          "remote": "https://example.com/github.com/shivammathur/homebrew-php.git"
        }]"#;

        let result = homebrew_tap_info_contains_trusted_remote(metadata);

        assert!(matches!(result, Ok(false)));
    }

    #[test]
    fn verifies_scoop_versions_bucket_source() {
        let output = "Name     Source\nversions https://github.com/ScoopInstaller/Versions.git\n";

        assert!(scoop_bucket_list_contains_trusted_versions_bucket(output));
    }

    #[test]
    fn rejects_deceptive_scoop_versions_bucket_source() {
        let output =
            "Name     Source\nversions https://example.com/github.com/ScoopInstaller/Versions.git\n";

        assert!(!scoop_bucket_list_contains_trusted_versions_bucket(output));
    }
}

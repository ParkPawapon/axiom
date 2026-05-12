use std::path::PathBuf;
use std::time::Duration;

use crate::domain::project::project_php_version::PhpRuntimeInstallProvider;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::infrastructure::services::adapters::executable_resolver::ExecutableResolver;
use crate::ports::php_runtime_installer::{PhpRuntimeInstallReport, PhpRuntimeInstaller};
use crate::ports::process_manager::ProcessManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const INSTALL_TIMEOUT: Duration = Duration::from_secs(20 * 60);
const INSTALL_OUTPUT_LIMIT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Default, Clone, Copy)]
pub struct PackageManagerPhpInstaller;

#[derive(Debug, Clone, Eq, PartialEq)]
struct InstallPlan {
    provider: PhpRuntimeInstallProvider,
    manager_path: PathBuf,
    package_name: String,
    steps: Vec<InstallStep>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct InstallStep {
    args: Vec<String>,
    description: &'static str,
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

        let steps = runnable_steps(&plan, &runner)?;
        let mut completed_steps = 0_usize;

        for step in &steps {
            let output = runner.execute(
                ProcessCommand::new(plan.manager_path.to_string_lossy().into_owned())
                    .args(step.args.clone())
                    .timeout(INSTALL_TIMEOUT),
            )?;

            ensure_successful_step(&plan, step, &output)?;
            completed_steps += 1;
        }

        Ok(PhpRuntimeInstallReport {
            provider: plan.provider,
            package_name: plan.package_name,
            status_message: format!(
                "{} completed {} package-manager step(s).",
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
            .resolve_first(&["scoop.cmd", "scoop.exe", "scoop.ps1", "scoop"])
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
    let package_name = homebrew_package_name(version.as_str());
    let steps = if package_name.starts_with("shivammathur/php/") {
        vec![
            InstallStep {
                args: vec!["tap".to_string(), "shivammathur/php".to_string()],
                description: "tap trusted PHP formula repository",
            },
            InstallStep {
                args: vec!["install".to_string(), package_name.clone()],
                description: "install PHP runtime",
            },
        ]
    } else {
        vec![InstallStep {
            args: vec!["install".to_string(), package_name.clone()],
            description: "install PHP runtime",
        }]
    };

    InstallPlan {
        provider: PhpRuntimeInstallProvider::Homebrew,
        manager_path,
        package_name,
        steps,
    }
}

fn scoop_install_plan(version: &RuntimeVersion, manager_path: PathBuf) -> InstallPlan {
    let package_name = scoop_package_name(version.as_str());

    InstallPlan {
        provider: PhpRuntimeInstallProvider::Scoop,
        manager_path,
        package_name: package_name.clone(),
        steps: vec![
            InstallStep {
                args: vec![
                    "bucket".to_string(),
                    "add".to_string(),
                    "versions".to_string(),
                ],
                description: "ensure Scoop versions bucket",
            },
            InstallStep {
                args: vec!["install".to_string(), package_name],
                description: "install PHP runtime",
            },
        ],
    }
}

fn homebrew_package_name(version: &str) -> String {
    match version {
        "8.5" => "php".to_string(),
        "8.2" | "8.3" | "8.4" => format!("php@{version}"),
        _ => format!("shivammathur/php/php@{version}"),
    }
}

fn scoop_package_name(version: &str) -> String {
    format!("versions/php{}", version.replace('.', ""))
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

fn runnable_steps(plan: &InstallPlan, runner: &CommandRunner) -> AppResult<Vec<InstallStep>> {
    if plan.provider != PhpRuntimeInstallProvider::Scoop {
        return Ok(plan.steps.clone());
    }

    if !scoop_versions_bucket_exists(plan, runner)? {
        return Ok(plan.steps.clone());
    }

    Ok(plan
        .steps
        .iter()
        .filter(|step| step.args != ["bucket", "add", "versions"])
        .cloned()
        .collect())
}

fn scoop_versions_bucket_exists(plan: &InstallPlan, runner: &CommandRunner) -> AppResult<bool> {
    let output = runner.execute(
        ProcessCommand::new(plan.manager_path.to_string_lossy().into_owned())
            .args(["bucket", "list"])
            .timeout(Duration::from_secs(30)),
    )?;

    if output.timed_out || output.exit_code != Some(0) {
        return Ok(false);
    }

    Ok(output
        .stdout
        .lines()
        .any(|line| line.split_whitespace().any(|part| part == "versions")))
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
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].args, ["tap", "shivammathur/php"]);
    }

    #[test]
    fn maps_scoop_versions_to_versions_bucket_packages() {
        assert_eq!(scoop_package_name("8.5"), "versions/php85");
        assert_eq!(scoop_package_name("7.4"), "versions/php74");
        assert_eq!(scoop_package_name("5.6"), "versions/php56");
    }
}

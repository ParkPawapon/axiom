use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use directories::ProjectDirs;
use uuid::Uuid;

use crate::domain::database::database_config::{
    ManagedDatabaseDependencyReport, ManagedDatabaseDependencyStatus, ManagedDatabasePackage,
    ManagedDatabaseServiceReport, PhpMyAdminAccess, ProjectDatabaseProfile,
};
use crate::domain::database::database_type::DatabaseType;
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::infrastructure::services::adapters::executable_resolver::ExecutableResolver;
use crate::ports::database_dependency_manager::DatabaseDependencyManager;
use crate::ports::process_manager::ProcessManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const INSTALL_TIMEOUT: Duration = Duration::from_secs(20 * 60);
const SERVICE_TIMEOUT: Duration = Duration::from_secs(60);
const OUTPUT_LIMIT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct ManagedDatabaseDependencyManager;

#[derive(Debug, Clone, Eq, PartialEq)]
struct PackageManagerPlan {
    provider: &'static str,
    manager_path: PathBuf,
    packages: Vec<&'static str>,
    service_package: &'static str,
}

impl ManagedDatabaseDependencyManager {
    pub fn new() -> Self {
        Self
    }
}

impl DatabaseDependencyManager for ManagedDatabaseDependencyManager {
    fn ensure_database_dependencies(
        &self,
        database_type: DatabaseType,
    ) -> AppResult<ManagedDatabaseDependencyReport> {
        let plan = match package_manager_plan(database_type) {
            Ok(plan) => plan,
            Err(error) => {
                return Ok(ManagedDatabaseDependencyReport {
                    database_type,
                    provider: "unavailable".to_string(),
                    status: ManagedDatabaseDependencyStatus::Pending,
                    packages: Vec::new(),
                    diagnostics: vec![error.to_string()],
                    status_message:
                        "Managed package installation is pending because no supported package manager was found."
                            .to_string(),
                });
            }
        };
        let runner = package_manager_runner(&plan);
        let mut packages = Vec::new();
        let mut diagnostics = Vec::new();

        for package_name in &plan.packages {
            let already_installed = package_is_installed(&plan, &runner, package_name)?;
            let mut installed_now = false;

            if already_installed {
                diagnostics.push(format!("{package_name} is already installed."));
            } else {
                install_package(&plan, &runner, package_name)?;
                installed_now = true;
                diagnostics.push(format!("{package_name} was installed."));
            }

            packages.push(ManagedDatabasePackage {
                package_name: (*package_name).to_string(),
                already_installed,
                installed_now,
            });
        }

        Ok(ManagedDatabaseDependencyReport {
            database_type,
            provider: plan.provider.to_string(),
            status: ManagedDatabaseDependencyStatus::Installed,
            packages,
            diagnostics,
            status_message: format!(
                "{} managed dependency check completed for {}.",
                plan.provider,
                database_type.as_key()
            ),
        })
    }

    fn start_database_service(
        &self,
        database_type: DatabaseType,
    ) -> AppResult<ManagedDatabaseServiceReport> {
        let plan = package_manager_plan(database_type)?;
        let runner = package_manager_runner(&plan);

        match plan.provider {
            "homebrew" => {
                run_package_manager_command(
                    &runner,
                    &plan,
                    vec![
                        "services".to_string(),
                        "start".to_string(),
                        plan.service_package.to_string(),
                    ],
                    SERVICE_TIMEOUT,
                )?;

                Ok(ManagedDatabaseServiceReport {
                    service_id: service_id(database_type).to_string(),
                    started: true,
                    status_message: format!(
                        "Homebrew service start requested for {}.",
                        plan.service_package
                    ),
                })
            }
            "scoop" => Ok(ManagedDatabaseServiceReport {
                service_id: service_id(database_type).to_string(),
                started: false,
                status_message:
                    "Scoop package installation completed; Windows service startup is delegated to the Windows service adapter when a supported service is registered."
                        .to_string(),
            }),
            _ => Err(AppError::Configuration(
                "unsupported database package manager provider".to_string(),
            )),
        }
    }

    fn configure_phpmyadmin(
        &self,
        profile: &ProjectDatabaseProfile,
    ) -> AppResult<Option<PhpMyAdminAccess>> {
        if profile.database_type != DatabaseType::Mysql {
            return Ok(None);
        }

        let document_root = phpmyadmin_document_root()?;
        let config_dir = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP")
            .map(|dirs| dirs.config_dir().join("phpmyadmin"))
            .ok_or_else(|| {
                AppError::Configuration("failed to resolve phpMyAdmin config directory".to_string())
            })?;
        fs::create_dir_all(&config_dir).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to create phpMyAdmin config directory: {error}"
            ))
        })?;

        let config_path = config_dir.join("config.inc.php");
        let reverse_proxy_config_path = config_dir.join("Caddyfile");
        let url = profile.admin_url.clone().unwrap_or_else(|| {
            format!(
                "http://127.0.0.1:8088/phpmyadmin?db={}",
                profile.database_name
            )
        });
        let base_url = phpmyadmin_base_url(&url);
        let site_address = phpmyadmin_site_address(&base_url)?;

        fs::write(&config_path, phpmyadmin_config(profile)).map_err(|error| {
            AppError::Infrastructure(format!("failed to write phpMyAdmin config: {error}"))
        })?;
        fs::write(
            &reverse_proxy_config_path,
            caddy_phpmyadmin_route(&site_address, &document_root),
        )
        .map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to write phpMyAdmin reverse-proxy config: {error}"
            ))
        })?;
        let reverse_proxy_started = start_phpmyadmin_reverse_proxy(&reverse_proxy_config_path)?;

        Ok(Some(PhpMyAdminAccess {
            url,
            document_root: document_root.to_string_lossy().into_owned(),
            config_path: config_path.to_string_lossy().into_owned(),
            reverse_proxy_config_path: reverse_proxy_config_path.to_string_lossy().into_owned(),
            reverse_proxy_started,
            status_message: "phpMyAdmin config was generated and Caddy reverse-proxy startup was requested for the managed MySQL profile."
                .to_string(),
        }))
    }
}

fn package_manager_plan(database_type: DatabaseType) -> AppResult<PackageManagerPlan> {
    let resolver = ExecutableResolver::from_env();

    if cfg!(target_os = "macos") {
        let manager_path = resolver.resolve("brew").ok_or_else(|| {
            AppError::NotFound("Homebrew executable was not found on PATH".to_string())
        })?;
        let (packages, service_package) = match database_type {
            DatabaseType::Mysql => (vec!["mysql", "phpmyadmin", "caddy"], "mysql"),
            DatabaseType::Postgresql => (vec!["postgresql@17"], "postgresql@17"),
        };

        return Ok(PackageManagerPlan {
            provider: "homebrew",
            manager_path,
            packages,
            service_package,
        });
    }

    if cfg!(windows) {
        let manager_path = resolver
            .resolve_first(&["scoop.cmd", "scoop.exe", "scoop"])
            .ok_or_else(|| {
                AppError::NotFound("Scoop executable was not found on PATH".to_string())
            })?;
        let (packages, service_package) = match database_type {
            DatabaseType::Mysql => (vec!["mysql", "phpmyadmin", "caddy"], "mysql"),
            DatabaseType::Postgresql => (vec!["postgresql"], "postgresql"),
        };

        return Ok(PackageManagerPlan {
            provider: "scoop",
            manager_path,
            packages,
            service_package,
        });
    }

    Err(AppError::Configuration(
        "managed database installation is supported on macOS/Homebrew and Windows/Scoop only"
            .to_string(),
    ))
}

fn package_manager_runner(plan: &PackageManagerPlan) -> CommandRunner {
    CommandRunner::new(
        CommandPolicy::deny_all()
            .allow_program_paths([plan.manager_path.clone()])
            .with_default_timeout(INSTALL_TIMEOUT)
            .with_max_output_bytes(OUTPUT_LIMIT_BYTES),
    )
}

fn package_is_installed(
    plan: &PackageManagerPlan,
    runner: &CommandRunner,
    package_name: &str,
) -> AppResult<bool> {
    let args = match plan.provider {
        "homebrew" => vec![
            "list".to_string(),
            "--formula".to_string(),
            package_name.to_string(),
        ],
        "scoop" => vec!["prefix".to_string(), package_name.to_string()],
        _ => {
            return Err(AppError::Configuration(
                "unsupported package manager provider".to_string(),
            ));
        }
    };
    let output = run_package_manager_command_without_success_check(
        runner,
        plan,
        args,
        Duration::from_secs(30),
    )?;

    Ok(output.exit_code == Some(0) && !output.timed_out)
}

fn install_package(
    plan: &PackageManagerPlan,
    runner: &CommandRunner,
    package_name: &str,
) -> AppResult<()> {
    let args = match plan.provider {
        "homebrew" => vec!["install".to_string(), package_name.to_string()],
        "scoop" => vec!["install".to_string(), package_name.to_string()],
        _ => {
            return Err(AppError::Configuration(
                "unsupported package manager provider".to_string(),
            ));
        }
    };

    run_package_manager_command(runner, plan, args, INSTALL_TIMEOUT).map(|_| ())
}

fn run_package_manager_command(
    runner: &CommandRunner,
    plan: &PackageManagerPlan,
    args: Vec<String>,
    timeout: Duration,
) -> AppResult<ProcessOutput> {
    let output = run_package_manager_command_without_success_check(runner, plan, args, timeout)?;

    if output.timed_out {
        return Err(AppError::Infrastructure(format!(
            "{} command timed out",
            plan.provider
        )));
    }

    if output.exit_code != Some(0) {
        return Err(AppError::Infrastructure(format!(
            "{} command failed with exit code {:?}: {}",
            plan.provider,
            output.exit_code,
            summarize_output(&output)
        )));
    }

    Ok(output)
}

fn run_package_manager_command_without_success_check(
    runner: &CommandRunner,
    plan: &PackageManagerPlan,
    args: Vec<String>,
    timeout: Duration,
) -> AppResult<ProcessOutput> {
    runner.execute(
        ProcessCommand::new(plan.manager_path.to_string_lossy().into_owned())
            .args(args)
            .timeout(timeout),
    )
}

fn phpmyadmin_document_root() -> AppResult<PathBuf> {
    if cfg!(target_os = "macos") {
        for candidate in [
            "/opt/homebrew/share/phpmyadmin",
            "/usr/local/share/phpmyadmin",
        ] {
            let path = Path::new(candidate);
            if path.is_dir() {
                return Ok(path.to_path_buf());
            }
        }
    }

    if cfg!(windows) {
        if let Some(path) = std::env::var_os("SCOOP") {
            let path = PathBuf::from(path)
                .join("apps")
                .join("phpmyadmin")
                .join("current");
            if path.is_dir() {
                return Ok(path);
            }
        }
    }

    Err(AppError::Configuration(
        "phpMyAdmin document root was not found after package installation".to_string(),
    ))
}

fn phpmyadmin_config(profile: &ProjectDatabaseProfile) -> String {
    let blowfish_secret = Uuid::new_v4().simple().to_string();

    format!(
        "<?php\n$cfg['blowfish_secret'] = '{blowfish_secret}';\n$i = 0;\n$i++;\n$cfg['Servers'][$i]['auth_type'] = 'cookie';\n$cfg['Servers'][$i]['host'] = '{}';\n$cfg['Servers'][$i]['port'] = '{}';\n$cfg['Servers'][$i]['compress'] = false;\n$cfg['Servers'][$i]['AllowNoPassword'] = false;\n",
        profile.host, profile.port
    )
}

fn caddy_phpmyadmin_route(site_address: &str, document_root: &Path) -> String {
    format!(
        "{{\n  auto_https off\n}}\n\n{} {{\n  handle_path /phpmyadmin* {{\n    root * {}\n    php_fastcgi 127.0.0.1:9000\n    file_server\n  }}\n}}\n",
        site_address,
        document_root.to_string_lossy()
    )
}

fn phpmyadmin_base_url(admin_url: &str) -> String {
    admin_url
        .split('?')
        .next()
        .unwrap_or(admin_url)
        .trim_end_matches('/')
        .to_string()
}

fn phpmyadmin_site_address(base_url: &str) -> AppResult<String> {
    let (scheme, rest) = base_url.split_once("://").ok_or_else(|| {
        AppError::Validation("phpMyAdmin base URL must include a scheme".to_string())
    })?;

    if scheme != "http" && scheme != "https" {
        return Err(AppError::Validation(
            "phpMyAdmin base URL must use http or https".to_string(),
        ));
    }

    let host_and_port = rest
        .split('/')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::Validation("phpMyAdmin base URL host is required".to_string()))?;

    if !is_loopback_host_and_port(host_and_port) {
        return Err(AppError::Validation(
            "phpMyAdmin managed reverse proxy must bind to localhost".to_string(),
        ));
    }

    Ok(format!("{scheme}://{host_and_port}"))
}

fn start_phpmyadmin_reverse_proxy(config_path: &Path) -> AppResult<bool> {
    let Some(caddy_path) = ExecutableResolver::from_env().resolve("caddy") else {
        return Err(AppError::Configuration(
            "Caddy executable was not found after package installation".to_string(),
        ));
    };
    let runner = CommandRunner::new(
        CommandPolicy::deny_all()
            .allow_program_paths([caddy_path.clone()])
            .with_default_timeout(SERVICE_TIMEOUT)
            .with_max_output_bytes(OUTPUT_LIMIT_BYTES),
    );
    let config_path = config_path.to_string_lossy().into_owned();

    run_caddy_command(
        &runner,
        &caddy_path,
        [
            "validate",
            "--config",
            &config_path,
            "--adapter",
            "caddyfile",
        ],
    )?;

    let reload_output = run_caddy_command_without_success_check(
        &runner,
        &caddy_path,
        ["reload", "--config", &config_path, "--adapter", "caddyfile"],
    )?;

    if reload_output.exit_code == Some(0) && !reload_output.timed_out {
        return Ok(true);
    }

    run_caddy_command(
        &runner,
        &caddy_path,
        ["start", "--config", &config_path, "--adapter", "caddyfile"],
    )?;

    Ok(true)
}

fn is_loopback_host_and_port(host_and_port: &str) -> bool {
    host_and_port == "127.0.0.1"
        || host_and_port.starts_with("127.0.0.1:")
        || host_and_port == "localhost"
        || host_and_port.starts_with("localhost:")
        || host_and_port == "[::1]"
        || host_and_port.starts_with("[::1]:")
}

fn run_caddy_command(
    runner: &CommandRunner,
    caddy_path: &Path,
    args: impl IntoIterator<Item = impl Into<String>>,
) -> AppResult<ProcessOutput> {
    let output = run_caddy_command_without_success_check(runner, caddy_path, args)?;

    if output.timed_out {
        return Err(AppError::Infrastructure(
            "Caddy command timed out while configuring phpMyAdmin".to_string(),
        ));
    }

    if output.exit_code != Some(0) {
        return Err(AppError::Infrastructure(format!(
            "Caddy command failed while configuring phpMyAdmin: {}",
            summarize_output(&output)
        )));
    }

    Ok(output)
}

fn run_caddy_command_without_success_check(
    runner: &CommandRunner,
    caddy_path: &Path,
    args: impl IntoIterator<Item = impl Into<String>>,
) -> AppResult<ProcessOutput> {
    runner.execute(
        ProcessCommand::new(caddy_path.to_string_lossy().into_owned())
            .args(args)
            .timeout(SERVICE_TIMEOUT),
    )
}

fn service_id(database_type: DatabaseType) -> &'static str {
    match database_type {
        DatabaseType::Mysql => "mysql",
        DatabaseType::Postgresql => "postgresql",
    }
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
        text.chars().take(500).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_mysql_packages_for_homebrew() {
        if !cfg!(target_os = "macos") {
            return;
        }

        let plan = package_manager_plan(DatabaseType::Mysql);

        if let Ok(plan) = plan {
            assert!(plan.packages.contains(&"mysql"));
            assert!(plan.packages.contains(&"phpmyadmin"));
            assert!(plan.packages.contains(&"caddy"));
        }
    }
}

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use directories::{BaseDirs, ProjectDirs};

use crate::domain::database::database_config::{
    DatabaseBackupSchedulerInstallResult, DatabaseBackupSchedulerStatus,
};
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::ports::database_backup_scheduler::DatabaseBackupScheduler;
use crate::ports::process_manager::ProcessManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const SCHEDULER_LABEL: &str = "dev.axiomphp.database-backups";
const CLI_ARG: &str = "--run-due-database-backups";
const SCHEDULE_INTERVAL_SECONDS: u32 = 300;

#[derive(Debug, Clone)]
pub struct OsDatabaseBackupScheduler {
    executable_path: PathBuf,
}

impl OsDatabaseBackupScheduler {
    pub fn new() -> AppResult<Self> {
        let executable_path = std::env::current_exe().map_err(|error| {
            AppError::Configuration(format!("failed to resolve scheduler executable: {error}"))
        })?;

        Ok(Self { executable_path })
    }

    fn status(
        &self,
        installed: bool,
        manifest_path: Option<PathBuf>,
        message: String,
    ) -> DatabaseBackupSchedulerStatus {
        DatabaseBackupSchedulerStatus {
            installed,
            platform: platform_name().to_string(),
            schedule_label: SCHEDULER_LABEL.to_string(),
            manifest_path: manifest_path.map(|path| path.to_string_lossy().into_owned()),
            last_checked_at: Utc::now(),
            status_message: message,
        }
    }
}

impl DatabaseBackupScheduler for OsDatabaseBackupScheduler {
    fn scheduler_status(&self) -> AppResult<DatabaseBackupSchedulerStatus> {
        platform_status(self)
    }

    fn install_scheduler(&self) -> AppResult<DatabaseBackupSchedulerInstallResult> {
        let status = platform_install(self)?;
        Ok(DatabaseBackupSchedulerInstallResult {
            status: status.clone(),
            status_message: status.status_message.clone(),
        })
    }

    fn uninstall_scheduler(&self) -> AppResult<DatabaseBackupSchedulerInstallResult> {
        let status = platform_uninstall(self)?;
        Ok(DatabaseBackupSchedulerInstallResult {
            status: status.clone(),
            status_message: status.status_message.clone(),
        })
    }
}

#[cfg(target_os = "macos")]
fn platform_status(
    scheduler: &OsDatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    let manifest_path = macos_manifest_path()?;
    let manifest_exists = manifest_path.exists();
    let loaded = macos_launchctl_list().unwrap_or(false);
    let installed = manifest_exists && loaded;
    let message = if installed {
        "macOS LaunchAgent is installed and loaded for background database backups.".to_string()
    } else if manifest_exists {
        "macOS LaunchAgent manifest exists but is not currently loaded.".to_string()
    } else {
        "macOS LaunchAgent is not installed.".to_string()
    };

    Ok(scheduler.status(installed, Some(manifest_path), message))
}

#[cfg(target_os = "macos")]
fn platform_install(
    scheduler: &OsDatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    let manifest_path = macos_manifest_path()?;
    let logs_dir = macos_logs_dir()?;

    fs::create_dir_all(&logs_dir).map_err(|error| {
        AppError::Infrastructure(format!("failed to create scheduler log directory: {error}"))
    })?;
    write_macos_launch_agent(
        &manifest_path,
        &scheduler.executable_path,
        &logs_dir.join("database-backups.out.log"),
        &logs_dir.join("database-backups.err.log"),
    )?;
    let manifest_arg = manifest_path.to_string_lossy().into_owned();
    let load_result = macos_launchctl(&["load", "-w", manifest_arg.as_str()]);
    let loaded = macos_launchctl_list().unwrap_or(false);
    let message = match load_result {
        Ok(output) if output.exit_code == Some(0) && loaded => {
            "macOS LaunchAgent installed and loaded for background database backups.".to_string()
        }
        Ok(output) => format!(
            "macOS LaunchAgent manifest installed; launchctl returned exit {:?}. {}",
            output.exit_code,
            output.stderr.trim()
        ),
        Err(error) => format!(
            "macOS LaunchAgent manifest installed, but launchctl load could not be verified: {error}"
        ),
    };

    Ok(scheduler.status(
        manifest_path.exists() && loaded,
        Some(manifest_path),
        message,
    ))
}

#[cfg(target_os = "macos")]
fn platform_uninstall(
    scheduler: &OsDatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    let manifest_path = macos_manifest_path()?;

    if manifest_path.exists() {
        let manifest_arg = manifest_path.to_string_lossy().into_owned();
        let _ = macos_launchctl(&["unload", "-w", manifest_arg.as_str()]);
        fs::remove_file(&manifest_path).map_err(|error| {
            AppError::Infrastructure(format!("failed to remove macOS LaunchAgent: {error}"))
        })?;
    }

    Ok(scheduler.status(
        false,
        Some(manifest_path),
        "macOS LaunchAgent removed for background database backups.".to_string(),
    ))
}

#[cfg(target_os = "windows")]
fn platform_status(
    scheduler: &OsDatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    let output = windows_schtasks(&["/Query", "/TN", SCHEDULER_LABEL, "/FO", "LIST"]);
    let installed = output
        .as_ref()
        .is_ok_and(|process_output| process_output.exit_code == Some(0));
    let message = if installed {
        "Windows scheduled task is installed for background database backups.".to_string()
    } else {
        "Windows scheduled task is not installed.".to_string()
    };

    Ok(scheduler.status(installed, None, message))
}

#[cfg(target_os = "windows")]
fn platform_install(
    scheduler: &OsDatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    let task_command = format!("\"{}\" {}", scheduler.executable_path.display(), CLI_ARG);
    let output = windows_schtasks(&[
        "/Create",
        "/TN",
        SCHEDULER_LABEL,
        "/SC",
        "MINUTE",
        "/MO",
        "5",
        "/TR",
        &task_command,
        "/F",
    ])?;
    let installed = output.exit_code == Some(0);
    let message = if installed {
        "Windows scheduled task installed for background database backups.".to_string()
    } else {
        format!(
            "Windows scheduled task install returned exit {:?}: {}",
            output.exit_code,
            output.stderr.trim()
        )
    };

    Ok(scheduler.status(installed, None, message))
}

#[cfg(target_os = "windows")]
fn platform_uninstall(
    scheduler: &OsDatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    let _ = windows_schtasks(&["/Delete", "/TN", SCHEDULER_LABEL, "/F"]);

    Ok(scheduler.status(
        false,
        None,
        "Windows scheduled task removed for background database backups.".to_string(),
    ))
}

#[cfg(target_os = "linux")]
fn platform_status(
    scheduler: &OsDatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    let (service_path, timer_path) = linux_unit_paths()?;
    let unit_files_exist = service_path.exists() && timer_path.exists();
    let enabled = linux_systemctl(&["--user", "is-enabled", linux_timer_unit_name()])
        .map(|output| output.exit_code == Some(0))
        .unwrap_or(false);
    let active = linux_systemctl(&["--user", "is-active", linux_timer_unit_name()])
        .map(|output| output.exit_code == Some(0))
        .unwrap_or(false);
    let installed = unit_files_exist && enabled && active;
    let message = if installed {
        "Linux systemd user timer is installed and active for background database backups."
            .to_string()
    } else if unit_files_exist {
        "Linux systemd user unit files exist but the timer is not active.".to_string()
    } else {
        "Linux systemd user timer is not installed.".to_string()
    };

    Ok(scheduler.status(installed, Some(timer_path), message))
}

#[cfg(target_os = "linux")]
fn platform_install(
    scheduler: &OsDatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    let (service_path, timer_path) = linux_unit_paths()?;

    if let Some(parent) = service_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::Infrastructure(format!("failed to create systemd user directory: {error}"))
        })?;
    }

    write_linux_systemd_service(&service_path, &scheduler.executable_path)?;
    write_linux_systemd_timer(&timer_path)?;
    let _ = linux_systemctl(&["--user", "daemon-reload"])?;
    let enable_result = linux_systemctl(&["--user", "enable", "--now", linux_timer_unit_name()]);
    let active = linux_systemctl(&["--user", "is-active", linux_timer_unit_name()])
        .map(|output| output.exit_code == Some(0))
        .unwrap_or(false);
    let message = match enable_result {
        Ok(output) if output.exit_code == Some(0) && active => {
            "Linux systemd user timer installed and active for background database backups."
                .to_string()
        }
        Ok(output) => format!(
            "Linux systemd user timer files installed; systemctl returned exit {:?}. {}",
            output.exit_code,
            output.stderr.trim()
        ),
        Err(error) => {
            format!("Linux systemd user timer files installed, but activation failed: {error}")
        }
    };

    Ok(scheduler.status(
        service_path.exists() && timer_path.exists() && active,
        Some(timer_path),
        message,
    ))
}

#[cfg(target_os = "linux")]
fn platform_uninstall(
    scheduler: &OsDatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    let (service_path, timer_path) = linux_unit_paths()?;

    let _ = linux_systemctl(&["--user", "disable", "--now", linux_timer_unit_name()]);

    if timer_path.exists() {
        fs::remove_file(&timer_path).map_err(|error| {
            AppError::Infrastructure(format!("failed to remove Linux systemd timer: {error}"))
        })?;
    }

    if service_path.exists() {
        fs::remove_file(&service_path).map_err(|error| {
            AppError::Infrastructure(format!("failed to remove Linux systemd service: {error}"))
        })?;
    }

    let _ = linux_systemctl(&["--user", "daemon-reload"]);

    Ok(scheduler.status(
        false,
        Some(timer_path),
        "Linux systemd user timer removed for background database backups.".to_string(),
    ))
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_status(
    scheduler: &OsDatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    Ok(scheduler.status(
        false,
        None,
        "OS-level background backup scheduler is not implemented for this platform.".to_string(),
    ))
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_install(
    scheduler: &OsDatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    Ok(scheduler.status(
        false,
        None,
        "OS-level background backup scheduler is not implemented for this platform.".to_string(),
    ))
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_uninstall(
    scheduler: &OsDatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    Ok(scheduler.status(
        false,
        None,
        "OS-level background backup scheduler is not implemented for this platform.".to_string(),
    ))
}

#[cfg(target_os = "macos")]
fn macos_manifest_path() -> AppResult<PathBuf> {
    let base_dirs = BaseDirs::new().ok_or_else(|| {
        AppError::Configuration("failed to resolve user home directory".to_string())
    })?;

    Ok(base_dirs
        .home_dir()
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{SCHEDULER_LABEL}.plist")))
}

#[cfg(target_os = "macos")]
fn macos_logs_dir() -> AppResult<PathBuf> {
    let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
        AppError::Configuration("failed to resolve scheduler log directory".to_string())
    })?;

    Ok(project_dirs.data_local_dir().join("logs"))
}

#[cfg(target_os = "macos")]
fn write_macos_launch_agent(
    manifest_path: &Path,
    executable_path: &Path,
    stdout_path: &Path,
    stderr_path: &Path,
) -> AppResult<()> {
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::Infrastructure(format!("failed to create LaunchAgent directory: {error}"))
        })?;
    }

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{executable}</string>
    <string>{cli_arg}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>StartInterval</key>
  <integer>{interval}</integer>
  <key>StandardOutPath</key>
  <string>{stdout}</string>
  <key>StandardErrorPath</key>
  <string>{stderr}</string>
</dict>
</plist>
"#,
        cli_arg = CLI_ARG,
        executable = escape_xml(&executable_path.to_string_lossy()),
        interval = SCHEDULE_INTERVAL_SECONDS,
        label = SCHEDULER_LABEL,
        stderr = escape_xml(&stderr_path.to_string_lossy()),
        stdout = escape_xml(&stdout_path.to_string_lossy()),
    );

    fs::write(manifest_path, plist).map_err(|error| {
        AppError::Infrastructure(format!("failed to write macOS LaunchAgent: {error}"))
    })
}

#[cfg(target_os = "macos")]
fn macos_launchctl(args: &[&str]) -> AppResult<ProcessOutput> {
    let program = launchctl_path();
    let runner = CommandRunner::new(
        CommandPolicy::deny_all()
            .allow_program_paths([program.clone()])
            .with_default_timeout(Duration::from_secs(10)),
    );

    runner.execute(
        ProcessCommand::new(program.to_string_lossy().into_owned())
            .args(args.iter().copied())
            .timeout(Duration::from_secs(10)),
    )
}

#[cfg(target_os = "macos")]
fn macos_launchctl_list() -> AppResult<bool> {
    let output = macos_launchctl(&["list", SCHEDULER_LABEL])?;
    Ok(output.exit_code == Some(0))
}

#[cfg(target_os = "macos")]
fn launchctl_path() -> PathBuf {
    let bin_path = PathBuf::from("/bin/launchctl");

    if bin_path.exists() {
        bin_path
    } else {
        PathBuf::from("/usr/bin/launchctl")
    }
}

#[cfg(target_os = "windows")]
fn windows_schtasks(args: &[&str]) -> AppResult<ProcessOutput> {
    let runner = CommandRunner::new(
        CommandPolicy::deny_all()
            .allow_program_names(["schtasks", "schtasks.exe"])
            .with_default_timeout(Duration::from_secs(20)),
    );

    runner.execute(
        ProcessCommand::new("schtasks")
            .args(args.iter().copied())
            .timeout(Duration::from_secs(20)),
    )
}

#[cfg(target_os = "linux")]
fn linux_unit_paths() -> AppResult<(PathBuf, PathBuf)> {
    let base_dirs = BaseDirs::new().ok_or_else(|| {
        AppError::Configuration("failed to resolve user home directory".to_string())
    })?;
    let systemd_user_dir = base_dirs
        .home_dir()
        .join(".config")
        .join("systemd")
        .join("user");

    Ok((
        systemd_user_dir.join(linux_service_unit_name()),
        systemd_user_dir.join(linux_timer_unit_name()),
    ))
}

#[cfg(target_os = "linux")]
fn write_linux_systemd_service(service_path: &Path, executable_path: &Path) -> AppResult<()> {
    let unit = format!(
        r#"[Unit]
Description=AxiomPHP database backup runner
Documentation=https://github.com/ParkPawapon/axiom

[Service]
Type=oneshot
ExecStart={executable} {cli_arg}
"#,
        cli_arg = CLI_ARG,
        executable = systemd_quote_arg(&executable_path.to_string_lossy()),
    );

    fs::write(service_path, unit).map_err(|error| {
        AppError::Infrastructure(format!("failed to write Linux systemd service: {error}"))
    })
}

#[cfg(target_os = "linux")]
fn write_linux_systemd_timer(timer_path: &Path) -> AppResult<()> {
    let unit = format!(
        r#"[Unit]
Description=AxiomPHP scheduled database backups

[Timer]
OnBootSec=5min
OnUnitActiveSec={interval}s
Persistent=true
Unit={service_unit}

[Install]
WantedBy=timers.target
"#,
        interval = SCHEDULE_INTERVAL_SECONDS,
        service_unit = linux_service_unit_name(),
    );

    fs::write(timer_path, unit).map_err(|error| {
        AppError::Infrastructure(format!("failed to write Linux systemd timer: {error}"))
    })
}

#[cfg(target_os = "linux")]
fn linux_systemctl(args: &[&str]) -> AppResult<ProcessOutput> {
    let program = systemctl_path();
    let runner = CommandRunner::new(
        CommandPolicy::deny_all()
            .allow_program_paths([program.clone()])
            .with_default_timeout(Duration::from_secs(20)),
    );

    runner.execute(
        ProcessCommand::new(program.to_string_lossy().into_owned())
            .args(args.iter().copied())
            .timeout(Duration::from_secs(20)),
    )
}

#[cfg(target_os = "linux")]
fn systemctl_path() -> PathBuf {
    let usr_bin_path = PathBuf::from("/usr/bin/systemctl");

    if usr_bin_path.exists() {
        usr_bin_path
    } else {
        PathBuf::from("/bin/systemctl")
    }
}

#[cfg(target_os = "linux")]
fn linux_service_unit_name() -> &'static str {
    "dev.axiomphp.database-backups.service"
}

#[cfg(target_os = "linux")]
fn linux_timer_unit_name() -> &'static str {
    "dev.axiomphp.database-backups.timer"
}

#[cfg(target_os = "linux")]
fn systemd_quote_arg(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('%', "%%")
    )
}

fn platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unsupported"
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

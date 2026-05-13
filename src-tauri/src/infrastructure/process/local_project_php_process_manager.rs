use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;

use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_process::{ProjectPhpProcessState, ProjectPhpProcessStatus};
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand};
use crate::ports::project_php_process_manager::{
    ProjectPhpProcessManager, StartProjectPhpProcessRequest,
};
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_port::validate_port;

const PHP_PROJECT_PROCESS_STARTUP_TIMEOUT: Duration = Duration::from_secs(3);
const PHP_PROJECT_PROCESS_PORT_SCAN_LIMIT: u16 = 128;

#[derive(Debug)]
pub struct LocalProjectPhpProcessManager {
    workspace_root: PathBuf,
    processes: Mutex<BTreeMap<String, ManagedPhpProcess>>,
}

#[derive(Debug)]
struct ManagedPhpProcess {
    child: Child,
    php_version: RuntimeVersion,
    port: u16,
    document_root: PathBuf,
    log_file: PathBuf,
    started_at: DateTime<Utc>,
}

impl LocalProjectPhpProcessManager {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application data directory".to_string())
        })?;

        Ok(Self::with_workspace_root(
            project_dirs.data_local_dir().join("project-processes"),
        ))
    }

    pub fn with_workspace_root(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            processes: Mutex::new(BTreeMap::new()),
        }
    }

    fn prepare_document_root(&self, project_id: &ProjectId) -> AppResult<PathBuf> {
        let document_root = self.workspace_root.join(&project_id.0).join("public");

        fs::create_dir_all(&document_root).map_err(|error| {
            AppError::Infrastructure(format!("failed to create project document root: {error}"))
        })?;

        Ok(document_root)
    }

    fn log_file_path(&self, project_id: &ProjectId) -> PathBuf {
        self.workspace_root
            .join(&project_id.0)
            .join("php-server.log")
    }
}

impl ProjectPhpProcessManager for LocalProjectPhpProcessManager {
    fn start_php_process(
        &self,
        request: StartProjectPhpProcessRequest,
    ) -> AppResult<ProjectPhpProcessStatus> {
        let mut processes = self
            .processes
            .lock()
            .map_err(|_error| AppError::Unexpected)?;

        if let Some(status) = status_for_existing_process(&mut processes, &request.project_id)? {
            if status.state == ProjectPhpProcessState::Running {
                return Ok(status);
            }
        }

        let document_root = self.prepare_document_root(&request.project_id)?;
        let log_file = self.log_file_path(&request.project_id);
        let port = find_available_loopback_port(preferred_project_port(&request.project_id)?)?;
        let bind_address = format!("127.0.0.1:{port}");
        let php_binary_path = PathBuf::from(&request.php_binary.path.0);

        let process_command = ProcessCommand::new(php_binary_path.to_string_lossy().into_owned())
            .args([
                "-S".to_string(),
                bind_address.clone(),
                "-t".to_string(),
                document_root.to_string_lossy().into_owned(),
            ])
            .current_dir(document_root.clone());

        let policy = CommandPolicy::deny_all()
            .allow_program_paths([php_binary_path.clone()])
            .with_default_timeout(PHP_PROJECT_PROCESS_STARTUP_TIMEOUT);
        policy.validate(&process_command)?;

        let log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .map_err(|error| {
                AppError::Infrastructure(format!("failed to open PHP process log file: {error}"))
            })?;
        let log_for_stderr = log.try_clone().map_err(|error| {
            AppError::Infrastructure(format!("failed to clone PHP process log file: {error}"))
        })?;

        let mut command = Command::new(&php_binary_path);
        command.args(&process_command.args);
        command.current_dir(&document_root);
        command.stdin(Stdio::null());
        command.stdout(Stdio::from(log));
        command.stderr(Stdio::from(log_for_stderr));
        command.env_clear();
        apply_minimal_project_environment(&mut command);

        tracing::info!(
            project_id = %request.project_id.0,
            php_version = %request.php_version.as_str(),
            port,
            "starting PHP project process"
        );

        let mut child = command.spawn().map_err(|error| {
            AppError::Infrastructure(format!("failed to start PHP project process: {error}"))
        })?;

        wait_for_php_process_ready(&mut child, port).inspect_err(|_error| {
            let _ = child.kill();
            let _ = child.wait();
        })?;

        let process = ManagedPhpProcess {
            child,
            php_version: request.php_version,
            port,
            document_root,
            log_file,
            started_at: Utc::now(),
        };
        let status = running_status(request.project_id.clone(), &process);

        processes.insert(request.project_id.0, process);

        Ok(status)
    }

    fn stop_php_process(&self, project_id: &ProjectId) -> AppResult<ProjectPhpProcessStatus> {
        let mut processes = self
            .processes
            .lock()
            .map_err(|_error| AppError::Unexpected)?;
        let Some(mut process) = processes.remove(&project_id.0) else {
            return Ok(ProjectPhpProcessStatus::stopped(project_id.clone()));
        };

        process.child.kill().map_err(|error| {
            AppError::Infrastructure(format!("failed to stop PHP project process: {error}"))
        })?;
        process.child.wait().map_err(|error| {
            AppError::Infrastructure(format!("failed to wait for PHP project process: {error}"))
        })?;

        tracing::info!(project_id = %project_id.0, "stopped PHP project process");

        Ok(ProjectPhpProcessStatus {
            status_message: "PHP project process stopped.".to_string(),
            ..ProjectPhpProcessStatus::stopped(project_id.clone())
        })
    }

    fn get_php_process_status(&self, project_id: &ProjectId) -> AppResult<ProjectPhpProcessStatus> {
        let mut processes = self
            .processes
            .lock()
            .map_err(|_error| AppError::Unexpected)?;

        Ok(status_for_existing_process(&mut processes, project_id)?
            .unwrap_or_else(|| ProjectPhpProcessStatus::stopped(project_id.clone())))
    }
}

impl Drop for LocalProjectPhpProcessManager {
    fn drop(&mut self) {
        if let Ok(mut processes) = self.processes.lock() {
            for (_project_id, mut process) in std::mem::take(&mut *processes) {
                let _ = process.child.kill();
                let _ = process.child.wait();
            }
        }
    }
}

fn status_for_existing_process(
    processes: &mut BTreeMap<String, ManagedPhpProcess>,
    project_id: &ProjectId,
) -> AppResult<Option<ProjectPhpProcessStatus>> {
    let Some(process) = processes.get_mut(&project_id.0) else {
        return Ok(None);
    };

    if let Some(status) = process.child.try_wait().map_err(|error| {
        AppError::Infrastructure(format!("failed to inspect PHP project process: {error}"))
    })? {
        let port = process.port;
        let log_file = process.log_file.clone();
        processes.remove(&project_id.0);

        return Ok(Some(ProjectPhpProcessStatus {
            project_id: project_id.clone(),
            state: ProjectPhpProcessState::Failed,
            pid: None,
            php_version: None,
            port: Some(port),
            url: None,
            document_root: None,
            log_file: Some(log_file.to_string_lossy().into_owned()),
            started_at: None,
            status_message: format!(
                "PHP project process exited unexpectedly with status {}.",
                status
                    .code()
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ),
        }));
    }

    Ok(Some(running_status(project_id.clone(), process)))
}

fn running_status(project_id: ProjectId, process: &ManagedPhpProcess) -> ProjectPhpProcessStatus {
    ProjectPhpProcessStatus {
        project_id,
        state: ProjectPhpProcessState::Running,
        pid: Some(process.child.id()),
        php_version: Some(process.php_version.clone()),
        port: Some(process.port),
        url: Some(format!("http://127.0.0.1:{}", process.port)),
        document_root: Some(process.document_root.to_string_lossy().into_owned()),
        log_file: Some(process.log_file.to_string_lossy().into_owned()),
        started_at: Some(process.started_at),
        status_message: "PHP project process is running on loopback only.".to_string(),
    }
}

fn preferred_project_port(project_id: &ProjectId) -> AppResult<u16> {
    let offset = project_id
        .0
        .bytes()
        .fold(0_u16, |total, byte| total.wrapping_add(u16::from(byte)))
        % 200;

    validate_port(8500 + offset)
}

fn find_available_loopback_port(preferred_port: u16) -> AppResult<u16> {
    for offset in 0..PHP_PROJECT_PROCESS_PORT_SCAN_LIMIT {
        let port = preferred_port.saturating_add(offset);

        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return validate_port(port);
        }
    }

    Err(AppError::Infrastructure(
        "failed to find an available loopback port for the PHP project process".to_string(),
    ))
}

fn wait_for_php_process_ready(child: &mut Child, port: u16) -> AppResult<()> {
    let started_at = Instant::now();

    while started_at.elapsed() < PHP_PROJECT_PROCESS_STARTUP_TIMEOUT {
        if let Some(status) = child.try_wait().map_err(|error| {
            AppError::Infrastructure(format!("failed to inspect PHP project startup: {error}"))
        })? {
            return Err(AppError::Infrastructure(format!(
                "PHP project process exited during startup with status {}",
                status
                    .code()
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            )));
        }

        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }

        thread::sleep(Duration::from_millis(50));
    }

    Err(AppError::Infrastructure(
        "PHP project process did not become ready before the startup timeout".to_string(),
    ))
}

fn apply_minimal_project_environment(command: &mut Command) {
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
        "TEMP",
        "TMP",
        "WINDIR",
    ]
}

#[cfg(not(windows))]
fn minimal_environment_keys() -> &'static [&'static str] {
    &["PATH", "TMPDIR"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferred_project_port_is_stable_and_nonzero() {
        let project_id = ProjectId("current-project".to_string());

        let first = preferred_project_port(&project_id).expect("port should validate");
        let second = preferred_project_port(&project_id).expect("port should validate");

        assert_eq!(first, second);
        assert!(first >= 8500);
    }

    #[test]
    fn stopped_status_contains_no_process_details() {
        let project_id = ProjectId("current-project".to_string());
        let status = ProjectPhpProcessStatus::stopped(project_id);

        assert_eq!(status.state, ProjectPhpProcessState::Stopped);
        assert!(status.pid.is_none());
        assert!(status.url.is_none());
    }
}

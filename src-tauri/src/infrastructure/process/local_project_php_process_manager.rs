use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use sysinfo::{Pid, Process, ProcessesToUpdate, System};

use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_path::ProjectPath;
use crate::domain::project::project_process::{ProjectPhpProcessState, ProjectPhpProcessStatus};
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand};
use crate::ports::project_php_process_manager::{
    ProjectPhpProcessManager, StartProjectPhpProcessRequest,
};
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_path::validate_existing_directory_path;
use crate::shared::validation::validate_port::validate_port;

use super::project_process_registry::{PersistedProjectProcessRecord, ProjectProcessRegistry};

const PHP_PROJECT_PROCESS_STARTUP_TIMEOUT: Duration = Duration::from_secs(3);
const PHP_PROJECT_PROCESS_PORT_SCAN_LIMIT: u16 = 128;

#[derive(Debug)]
pub struct LocalProjectPhpProcessManager {
    workspace_root: PathBuf,
    registry: ProjectProcessRegistry,
    processes: Mutex<BTreeMap<String, ProjectProcessEntry>>,
}

#[derive(Debug)]
enum ProjectProcessEntry {
    Managed(ManagedPhpProcess),
    Recovered(PersistedProjectProcessRecord),
}

#[derive(Debug)]
struct ManagedPhpProcess {
    child: Child,
    php_version: RuntimeVersion,
    php_binary_path: PathBuf,
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

        Self::with_workspace_root(project_dirs.data_local_dir().join("project-processes"))
    }

    pub fn with_workspace_root(workspace_root: PathBuf) -> AppResult<Self> {
        let registry = ProjectProcessRegistry::new(workspace_root.join("process-registry.json"));
        let recovered_processes = registry
            .load_records()?
            .into_iter()
            .map(|record| {
                (
                    record.project_id.0.clone(),
                    ProjectProcessEntry::Recovered(record),
                )
            })
            .collect();

        Ok(Self {
            workspace_root,
            registry,
            processes: Mutex::new(recovered_processes),
        })
    }

    fn resolve_document_root(&self, document_root: &ProjectPath) -> AppResult<PathBuf> {
        validate_existing_directory_path(&document_root.0)
    }

    fn prepare_log_file_path(&self, project_id: &ProjectId) -> AppResult<PathBuf> {
        let log_directory = self.workspace_root.join(&project_id.0);

        fs::create_dir_all(&log_directory).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to create PHP process log directory: {error}"
            ))
        })?;

        Ok(log_directory.join("php-server.log"))
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

        if let Some(status) =
            status_for_existing_process(&self.registry, &mut processes, &request.project_id)?
        {
            if status.state == ProjectPhpProcessState::Running {
                return Ok(status);
            }
        }

        let document_root = self.resolve_document_root(&request.document_root)?;
        let log_file = self.prepare_log_file_path(&request.project_id)?;
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

        let mut process = ManagedPhpProcess {
            child,
            php_version: request.php_version,
            php_binary_path,
            port,
            document_root,
            log_file,
            started_at: Utc::now(),
        };
        let status = running_status(request.project_id.clone(), &process);
        let record = persisted_record_from_managed_process(&request.project_id, &process);

        if let Err(error) = self.registry.upsert(&record) {
            let _ = process.child.kill();
            let _ = process.child.wait();
            return Err(error);
        }

        processes.insert(request.project_id.0, ProjectProcessEntry::Managed(process));

        Ok(status)
    }

    fn stop_php_process(&self, project_id: &ProjectId) -> AppResult<ProjectPhpProcessStatus> {
        let mut processes = self
            .processes
            .lock()
            .map_err(|_error| AppError::Unexpected)?;
        let Some(process_entry) = processes.remove(&project_id.0) else {
            self.registry.remove(project_id)?;
            return Ok(ProjectPhpProcessStatus::stopped(project_id.clone()));
        };

        match process_entry {
            ProjectProcessEntry::Managed(mut process) => {
                process.child.kill().map_err(|error| {
                    AppError::Infrastructure(format!("failed to stop PHP project process: {error}"))
                })?;
                process.child.wait().map_err(|error| {
                    AppError::Infrastructure(format!(
                        "failed to wait for PHP project process: {error}"
                    ))
                })?;
            }
            ProjectProcessEntry::Recovered(record) => {
                stop_recovered_process(&record)?;
            }
        }

        self.registry.remove(project_id)?;

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

        Ok(
            status_for_existing_process(&self.registry, &mut processes, project_id)?
                .unwrap_or_else(|| ProjectPhpProcessStatus::stopped(project_id.clone())),
        )
    }
}

impl Drop for LocalProjectPhpProcessManager {
    fn drop(&mut self) {
        if let Ok(mut processes) = self.processes.lock() {
            for (project_id, process_entry) in std::mem::take(&mut *processes) {
                if let ProjectProcessEntry::Managed(mut process) = process_entry {
                    let _ = process.child.kill();
                    let _ = process.child.wait();
                    let _ = self.registry.remove(&ProjectId(project_id));
                }
            }
        }
    }
}

fn status_for_existing_process(
    registry: &ProjectProcessRegistry,
    processes: &mut BTreeMap<String, ProjectProcessEntry>,
    project_id: &ProjectId,
) -> AppResult<Option<ProjectPhpProcessStatus>> {
    let Some(process_entry) = processes.get_mut(&project_id.0) else {
        return Ok(None);
    };

    match process_entry {
        ProjectProcessEntry::Managed(process) => {
            if let Some(status) = process.child.try_wait().map_err(|error| {
                AppError::Infrastructure(format!("failed to inspect PHP project process: {error}"))
            })? {
                let failed_status = failed_status_from_managed_exit(project_id, process, &status);
                processes.remove(&project_id.0);
                registry.remove(project_id)?;

                return Ok(Some(failed_status));
            }

            let mut record = persisted_record_from_managed_process(project_id, process);
            record.last_seen_at = Utc::now();
            registry.upsert(&record)?;

            Ok(Some(running_status(project_id.clone(), process)))
        }
        ProjectProcessEntry::Recovered(record) => {
            if recovered_process_matches_record(record) {
                record.last_seen_at = Utc::now();
                registry.upsert(record)?;

                return Ok(Some(running_status_from_registry_record(record)));
            }

            let failed_status = failed_status_from_registry_record(record);
            processes.remove(&project_id.0);
            registry.remove(project_id)?;

            Ok(Some(failed_status))
        }
    }
}

fn persisted_record_from_managed_process(
    project_id: &ProjectId,
    process: &ManagedPhpProcess,
) -> PersistedProjectProcessRecord {
    PersistedProjectProcessRecord {
        project_id: project_id.clone(),
        pid: process.child.id(),
        php_version: process.php_version.clone(),
        php_binary_path: process.php_binary_path.to_string_lossy().into_owned(),
        port: process.port,
        url: format!("http://127.0.0.1:{}", process.port),
        document_root: process.document_root.to_string_lossy().into_owned(),
        log_file: process.log_file.to_string_lossy().into_owned(),
        started_at: process.started_at,
        last_seen_at: Utc::now(),
    }
}

fn failed_status_from_managed_exit(
    project_id: &ProjectId,
    process: &ManagedPhpProcess,
    exit_status: &ExitStatus,
) -> ProjectPhpProcessStatus {
    ProjectPhpProcessStatus {
        project_id: project_id.clone(),
        state: ProjectPhpProcessState::Failed,
        pid: None,
        php_version: None,
        port: Some(process.port),
        url: None,
        document_root: Some(process.document_root.to_string_lossy().into_owned()),
        log_file: Some(process.log_file.to_string_lossy().into_owned()),
        started_at: None,
        status_message: format!(
            "PHP project process exited unexpectedly with status {}.",
            exit_status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ),
    }
}

fn failed_status_from_registry_record(
    record: &PersistedProjectProcessRecord,
) -> ProjectPhpProcessStatus {
    ProjectPhpProcessStatus {
        project_id: record.project_id.clone(),
        state: ProjectPhpProcessState::Failed,
        pid: None,
        php_version: Some(record.php_version.clone()),
        port: Some(record.port),
        url: None,
        document_root: Some(record.document_root.clone()),
        log_file: Some(record.log_file.clone()),
        started_at: Some(record.started_at),
        status_message: "Recovered PHP project process registry entry is stale; the process is no longer running or no longer matches the recorded command.".to_string(),
    }
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

fn running_status_from_registry_record(
    record: &PersistedProjectProcessRecord,
) -> ProjectPhpProcessStatus {
    ProjectPhpProcessStatus {
        project_id: record.project_id.clone(),
        state: ProjectPhpProcessState::Running,
        pid: Some(record.pid),
        php_version: Some(record.php_version.clone()),
        port: Some(record.port),
        url: Some(record.url.clone()),
        document_root: Some(record.document_root.clone()),
        log_file: Some(record.log_file.clone()),
        started_at: Some(record.started_at),
        status_message:
            "Recovered PHP project process is running on loopback from persisted registry state."
                .to_string(),
    }
}

fn recovered_process_matches_record(record: &PersistedProjectProcessRecord) -> bool {
    let mut system = System::new_all();
    let pid = Pid::from_u32(record.pid);
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

    let Some(process) = system.process(pid) else {
        return false;
    };

    process_matches_registry_record(process, record)
        && TcpStream::connect(("127.0.0.1", record.port)).is_ok()
}

fn stop_recovered_process(record: &PersistedProjectProcessRecord) -> AppResult<()> {
    let mut system = System::new_all();
    let pid = Pid::from_u32(record.pid);
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

    let Some(process) = system.process(pid) else {
        return Ok(());
    };

    if !process_matches_registry_record(process, record) {
        return Err(AppError::Validation(
            "refusing to stop recovered PHP process because the PID no longer matches the registry"
                .to_string(),
        ));
    }

    if !process.kill() {
        return Err(AppError::Infrastructure(
            "failed to stop recovered PHP project process".to_string(),
        ));
    }

    Ok(())
}

fn process_matches_registry_record(
    process: &Process,
    record: &PersistedProjectProcessRecord,
) -> bool {
    executable_matches_record(process, record)
        && working_directory_matches_record(process, record)
        && command_line_matches_record(process, record)
}

fn executable_matches_record(process: &Process, record: &PersistedProjectProcessRecord) -> bool {
    let Some(executable_path) = process.exe() else {
        return false;
    };

    paths_match(executable_path, Path::new(&record.php_binary_path))
}

fn working_directory_matches_record(
    process: &Process,
    record: &PersistedProjectProcessRecord,
) -> bool {
    let Some(current_directory) = process.cwd() else {
        return false;
    };

    paths_match(current_directory, Path::new(&record.document_root))
}

fn command_line_matches_record(process: &Process, record: &PersistedProjectProcessRecord) -> bool {
    let command_parts = process
        .cmd()
        .iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let bind_address = format!("127.0.0.1:{}", record.port);

    command_parts.iter().any(|part| part == "-S")
        && command_parts.iter().any(|part| part == &bind_address)
        && command_parts.iter().any(|part| part == "-t")
        && command_parts
            .iter()
            .any(|part| paths_match(Path::new(part), Path::new(&record.document_root)))
}

fn paths_match(left: &Path, right: &Path) -> bool {
    let normalized_left = left.canonicalize().unwrap_or_else(|_| left.to_path_buf());
    let normalized_right = right.canonicalize().unwrap_or_else(|_| right.to_path_buf());

    normalized_left == normalized_right
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

    fn persisted_process_record(
        project_id: &str,
        pid: u32,
        runtime_dir: &Path,
        document_root: &Path,
    ) -> PersistedProjectProcessRecord {
        PersistedProjectProcessRecord {
            project_id: ProjectId(project_id.to_string()),
            pid,
            php_version: RuntimeVersion::trusted("8.4"),
            php_binary_path: "/usr/local/bin/php8.4".to_string(),
            port: 8501,
            url: "http://127.0.0.1:8501".to_string(),
            document_root: document_root.to_string_lossy().into_owned(),
            log_file: runtime_dir
                .join(project_id)
                .join("php-server.log")
                .to_string_lossy()
                .into_owned(),
            started_at: Utc::now(),
            last_seen_at: Utc::now(),
        }
    }

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

    #[test]
    fn resolves_real_project_document_root_without_creating_placeholder() {
        let base_dir = std::env::temp_dir().join(format!(
            "axiomphp-real-document-root-{}",
            uuid::Uuid::new_v4()
        ));
        let runtime_dir = base_dir.join("runtime");
        let document_root = base_dir.join("project").join("public");
        fs::create_dir_all(&document_root).expect("document root should be created");

        let manager = LocalProjectPhpProcessManager::with_workspace_root(runtime_dir.clone())
            .expect("manager should initialize");
        let resolved = manager
            .resolve_document_root(&ProjectPath(document_root.to_string_lossy().into_owned()))
            .expect("document root should resolve");

        assert_eq!(
            resolved,
            document_root
                .canonicalize()
                .expect("document root should canonicalize")
        );
        assert!(!runtime_dir.join("current-project").join("public").exists());

        fs::remove_dir_all(base_dir).expect("test directory should be removed");
    }

    #[test]
    fn rejects_missing_project_document_root() {
        let runtime_dir = std::env::temp_dir().join(format!(
            "axiomphp-missing-document-root-{}",
            uuid::Uuid::new_v4()
        ));
        let manager = LocalProjectPhpProcessManager::with_workspace_root(runtime_dir.clone())
            .expect("manager should initialize");

        let result = manager.resolve_document_root(&ProjectPath(
            runtime_dir.join("missing").to_string_lossy().into_owned(),
        ));

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn loads_persisted_process_records_for_crash_recovery() {
        let base_dir = std::env::temp_dir().join(format!(
            "axiomphp-process-recovery-{}",
            uuid::Uuid::new_v4()
        ));
        let runtime_dir = base_dir.join("runtime");
        let document_root = base_dir.join("project").join("public");
        fs::create_dir_all(&document_root).expect("document root should be created");
        let registry = ProjectProcessRegistry::new(runtime_dir.join("process-registry.json"));
        registry
            .upsert(&persisted_process_record(
                "current-project",
                std::process::id(),
                &runtime_dir,
                &document_root,
            ))
            .expect("record should persist");

        let manager = LocalProjectPhpProcessManager::with_workspace_root(runtime_dir.clone())
            .expect("manager should initialize");
        let processes = manager
            .processes
            .lock()
            .expect("process registry lock should be available");

        assert!(matches!(
            processes.get("current-project"),
            Some(ProjectProcessEntry::Recovered(_))
        ));

        fs::remove_dir_all(base_dir).expect("test directory should be removed");
    }

    #[test]
    fn clears_stale_recovered_registry_record_on_status_check() {
        let base_dir = std::env::temp_dir().join(format!(
            "axiomphp-stale-process-recovery-{}",
            uuid::Uuid::new_v4()
        ));
        let runtime_dir = base_dir.join("runtime");
        let document_root = base_dir.join("project").join("public");
        fs::create_dir_all(&document_root).expect("document root should be created");
        let registry = ProjectProcessRegistry::new(runtime_dir.join("process-registry.json"));
        registry
            .upsert(&persisted_process_record(
                "current-project",
                std::process::id(),
                &runtime_dir,
                &document_root,
            ))
            .expect("record should persist");

        let manager = LocalProjectPhpProcessManager::with_workspace_root(runtime_dir.clone())
            .expect("manager should initialize");
        let status = manager
            .get_php_process_status(&ProjectId("current-project".to_string()))
            .expect("status should resolve");

        assert_eq!(status.state, ProjectPhpProcessState::Failed);
        assert!(
            ProjectProcessRegistry::new(runtime_dir.join("process-registry.json"))
                .load_records()
                .expect("records should load")
                .is_empty()
        );

        fs::remove_dir_all(base_dir).expect("test directory should be removed");
    }
}

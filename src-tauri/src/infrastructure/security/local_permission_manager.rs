use std::fs::{self, OpenOptions};
use std::path::PathBuf;

use directories::ProjectDirs;

use crate::domain::security::security_status::SecurityPermissionStatus;
use crate::infrastructure::certificates::local_certificate_manager::LocalCertificateManager;
use crate::infrastructure::networking::hosts_file_adapter::HostsFileAdapter;
use crate::infrastructure::services::adapters::executable_resolver::ExecutableResolver;
use crate::ports::permission_manager::PermissionManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug, Clone)]
pub struct LocalPermissionManager {
    hosts_file_path: PathBuf,
    certificate_authority_path: PathBuf,
    audit_log_dir: PathBuf,
}

impl LocalPermissionManager {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application data directory".to_string())
        })?;
        let certificate_manager = LocalCertificateManager::new()?;
        let hosts_adapter = HostsFileAdapter::new()?;

        Ok(Self {
            hosts_file_path: hosts_adapter.hosts_file_path(),
            certificate_authority_path: certificate_manager.certificate_authority_path(),
            audit_log_dir: project_dirs.data_local_dir().join("security").join("audit"),
        })
    }
}

impl PermissionManager for LocalPermissionManager {
    fn inspect_security_permissions(&self) -> AppResult<SecurityPermissionStatus> {
        let host_file_writable = OpenOptions::new()
            .append(true)
            .open(&self.hosts_file_path)
            .is_ok();
        let audit_log_writable = fs::create_dir_all(&self.audit_log_dir)
            .and_then(|_| {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(self.audit_log_dir.join(".permission-check"))
                    .map(|_| ())
            })
            .is_ok();
        let certificate_store_available = if cfg!(target_os = "macos") {
            ExecutableResolver::from_env().resolve("security").is_some()
        } else if cfg!(windows) {
            ExecutableResolver::from_env().resolve("certutil").is_some()
        } else {
            false
        };

        Ok(SecurityPermissionStatus {
            hosts_file_path: self.hosts_file_path.to_string_lossy().into_owned(),
            host_file_writable,
            certificate_store_available,
            certificate_authority_path: self
                .certificate_authority_path
                .to_string_lossy()
                .into_owned(),
            audit_log_writable,
            elevation_supported: cfg!(target_os = "macos") || cfg!(windows),
            status_message:
                "Security permission status was inspected without modifying privileged resources."
                    .to_string(),
        })
    }
}

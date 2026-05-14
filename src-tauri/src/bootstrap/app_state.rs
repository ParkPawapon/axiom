use std::fmt;
use std::sync::Arc;

use crate::infrastructure::certificates::local_certificate_manager::LocalCertificateManager;
use crate::infrastructure::databases::local_database_provisioner::LocalDatabaseProvisioner;
use crate::infrastructure::databases::managed_database_dependency_manager::ManagedDatabaseDependencyManager;
use crate::infrastructure::logging::file_audit_logger::FileAuditLogger;
use crate::infrastructure::logging::file_log_reader::FileLogReader;
use crate::infrastructure::networking::hosts_file_adapter::HostsFileAdapter;
use crate::infrastructure::persistence::file_database_provisioning_repository::FileDatabaseProvisioningRepository;
use crate::infrastructure::persistence::file_project_repository::FileProjectRepository;
use crate::infrastructure::persistence::file_project_runtime_repository::FileProjectRuntimeRepository;
use crate::infrastructure::process::local_project_php_process_manager::LocalProjectPhpProcessManager;
use crate::infrastructure::runtimes::package_manager_php_installer::PackageManagerPhpInstaller;
use crate::infrastructure::runtimes::php_binary_detector::PhpBinaryDetector;
use crate::infrastructure::secure_storage::keychain_storage::KeychainStorage;
use crate::infrastructure::security::local_permission_manager::LocalPermissionManager;
use crate::infrastructure::services::local_service_manager::LocalServiceManager;
use crate::ports::audit_logger::AuditLogger;
use crate::ports::certificate_manager::CertificateManager;
use crate::ports::database_dependency_manager::DatabaseDependencyManager;
use crate::ports::database_provisioner::DatabaseProvisioner;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::ports::hosts_file_manager::HostsFileManager;
use crate::ports::log_reader::LogReader;
use crate::ports::permission_manager::PermissionManager;
use crate::ports::php_runtime_detector::PhpRuntimeDetector;
use crate::ports::php_runtime_installer::PhpRuntimeInstaller;
use crate::ports::project_php_process_manager::ProjectPhpProcessManager;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::ports::secure_storage::SecureStorage;
use crate::ports::service_manager::ServiceManager;
use crate::shared::result::app_result::AppResult;

#[derive(Clone)]
pub struct AppState {
    pub app_name: &'static str,
    audit_logger: Arc<dyn AuditLogger>,
    certificate_manager: Arc<dyn CertificateManager>,
    database_dependency_manager: Arc<dyn DatabaseDependencyManager>,
    database_provisioner: Arc<dyn DatabaseProvisioner>,
    database_provisioning_repository: Arc<dyn DatabaseProvisioningRepository>,
    hosts_file_manager: Arc<dyn HostsFileManager>,
    log_reader: Arc<dyn LogReader>,
    permission_manager: Arc<dyn PermissionManager>,
    php_runtime_detector: Arc<dyn PhpRuntimeDetector>,
    php_runtime_installer: Arc<dyn PhpRuntimeInstaller>,
    project_php_process_manager: Arc<dyn ProjectPhpProcessManager>,
    project_repository: Arc<dyn ProjectRepository>,
    project_runtime_repository: Arc<dyn ProjectRuntimeRepository>,
    service_manager: Arc<dyn ServiceManager>,
}

impl AppState {
    pub fn new() -> AppResult<Self> {
        let project_runtime_repository =
            Arc::new(FileProjectRuntimeRepository::new()?) as Arc<dyn ProjectRuntimeRepository>;
        let project_repository =
            Arc::new(FileProjectRepository::new()?) as Arc<dyn ProjectRepository>;
        let secure_storage = Arc::new(KeychainStorage::new()) as Arc<dyn SecureStorage>;
        let database_provisioner = Arc::new(LocalDatabaseProvisioner::new(secure_storage)?)
            as Arc<dyn DatabaseProvisioner>;
        let database_provisioning_repository = Arc::new(FileDatabaseProvisioningRepository::new()?)
            as Arc<dyn DatabaseProvisioningRepository>;
        let database_dependency_manager =
            Arc::new(ManagedDatabaseDependencyManager::new()) as Arc<dyn DatabaseDependencyManager>;

        Ok(Self {
            app_name: "AxiomPHP",
            audit_logger: Arc::new(FileAuditLogger::new()?),
            certificate_manager: Arc::new(LocalCertificateManager::new()?),
            database_dependency_manager,
            database_provisioner,
            database_provisioning_repository,
            hosts_file_manager: Arc::new(HostsFileAdapter::new()?),
            log_reader: Arc::new(FileLogReader::new()?),
            permission_manager: Arc::new(LocalPermissionManager::new()?),
            php_runtime_detector: Arc::new(PhpBinaryDetector::new()),
            php_runtime_installer: Arc::new(PackageManagerPhpInstaller::new()),
            project_php_process_manager: Arc::new(LocalProjectPhpProcessManager::new()?),
            project_repository,
            project_runtime_repository,
            service_manager: Arc::new(LocalServiceManager::new()),
        })
    }

    pub fn php_runtime_detector(&self) -> &dyn PhpRuntimeDetector {
        self.php_runtime_detector.as_ref()
    }

    pub fn audit_logger(&self) -> &dyn AuditLogger {
        self.audit_logger.as_ref()
    }

    pub fn certificate_manager(&self) -> &dyn CertificateManager {
        self.certificate_manager.as_ref()
    }

    pub fn database_provisioner(&self) -> &dyn DatabaseProvisioner {
        self.database_provisioner.as_ref()
    }

    pub fn database_dependency_manager(&self) -> &dyn DatabaseDependencyManager {
        self.database_dependency_manager.as_ref()
    }

    pub fn database_provisioning_repository(&self) -> &dyn DatabaseProvisioningRepository {
        self.database_provisioning_repository.as_ref()
    }

    pub fn hosts_file_manager(&self) -> &dyn HostsFileManager {
        self.hosts_file_manager.as_ref()
    }

    pub fn log_reader(&self) -> &dyn LogReader {
        self.log_reader.as_ref()
    }

    pub fn php_runtime_installer(&self) -> &dyn PhpRuntimeInstaller {
        self.php_runtime_installer.as_ref()
    }

    pub fn permission_manager(&self) -> &dyn PermissionManager {
        self.permission_manager.as_ref()
    }

    pub fn project_php_process_manager(&self) -> &dyn ProjectPhpProcessManager {
        self.project_php_process_manager.as_ref()
    }

    pub fn project_repository(&self) -> &dyn ProjectRepository {
        self.project_repository.as_ref()
    }

    pub fn project_runtime_repository(&self) -> &dyn ProjectRuntimeRepository {
        self.project_runtime_repository.as_ref()
    }

    pub fn service_manager(&self) -> &dyn ServiceManager {
        self.service_manager.as_ref()
    }
}

impl fmt::Debug for AppState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AppState")
            .field("app_name", &self.app_name)
            .finish_non_exhaustive()
    }
}

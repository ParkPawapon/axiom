use std::fmt;
use std::sync::Arc;

use crate::infrastructure::logging::file_log_reader::FileLogReader;
use crate::infrastructure::persistence::file_project_repository::FileProjectRepository;
use crate::infrastructure::persistence::file_project_runtime_repository::FileProjectRuntimeRepository;
use crate::infrastructure::process::local_project_php_process_manager::LocalProjectPhpProcessManager;
use crate::infrastructure::runtimes::package_manager_php_installer::PackageManagerPhpInstaller;
use crate::infrastructure::runtimes::php_binary_detector::PhpBinaryDetector;
use crate::infrastructure::services::local_service_manager::LocalServiceManager;
use crate::ports::log_reader::LogReader;
use crate::ports::php_runtime_detector::PhpRuntimeDetector;
use crate::ports::php_runtime_installer::PhpRuntimeInstaller;
use crate::ports::project_php_process_manager::ProjectPhpProcessManager;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::ports::service_manager::ServiceManager;
use crate::shared::result::app_result::AppResult;

#[derive(Clone)]
pub struct AppState {
    pub app_name: &'static str,
    log_reader: Arc<dyn LogReader>,
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

        Ok(Self {
            app_name: "AxiomPHP",
            log_reader: Arc::new(FileLogReader::new()?),
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

    pub fn log_reader(&self) -> &dyn LogReader {
        self.log_reader.as_ref()
    }

    pub fn php_runtime_installer(&self) -> &dyn PhpRuntimeInstaller {
        self.php_runtime_installer.as_ref()
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

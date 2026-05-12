use std::fmt;
use std::sync::Arc;

use crate::infrastructure::persistence::file_project_runtime_repository::FileProjectRuntimeRepository;
use crate::infrastructure::services::local_service_manager::LocalServiceManager;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::ports::service_manager::ServiceManager;
use crate::shared::result::app_result::AppResult;

#[derive(Clone)]
pub struct AppState {
    pub app_name: &'static str,
    project_runtime_repository: Arc<dyn ProjectRuntimeRepository>,
    service_manager: Arc<dyn ServiceManager>,
}

impl AppState {
    pub fn new() -> AppResult<Self> {
        let project_runtime_repository =
            Arc::new(FileProjectRuntimeRepository::new()?) as Arc<dyn ProjectRuntimeRepository>;

        Ok(Self {
            app_name: "AxiomPHP",
            project_runtime_repository,
            service_manager: Arc::new(LocalServiceManager::new()),
        })
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

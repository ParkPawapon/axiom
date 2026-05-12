use std::fmt;
use std::sync::Arc;

use crate::infrastructure::services::local_service_manager::LocalServiceManager;
use crate::ports::service_manager::ServiceManager;

#[derive(Clone)]
pub struct AppState {
    pub app_name: &'static str,
    service_manager: Arc<dyn ServiceManager>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            app_name: "AxiomPHP",
            service_manager: Arc::new(LocalServiceManager::new()),
        }
    }

    pub fn service_manager(&self) -> &dyn ServiceManager {
        self.service_manager.as_ref()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
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

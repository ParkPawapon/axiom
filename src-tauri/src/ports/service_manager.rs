use crate::domain::service::service::Service;
use crate::domain::service::service_action::ServiceActionOutcome;
use crate::shared::result::app_result::AppResult;

pub trait ServiceManager: Send + Sync {
    fn list_services(&self) -> AppResult<Vec<Service>>;
    fn get_service_status(&self, service_id: &str) -> AppResult<Service>;
    fn start_service(&self, service_id: &str) -> AppResult<ServiceActionOutcome>;
    fn stop_service(&self, service_id: &str) -> AppResult<ServiceActionOutcome>;
    fn restart_service(&self, service_id: &str) -> AppResult<ServiceActionOutcome>;
}

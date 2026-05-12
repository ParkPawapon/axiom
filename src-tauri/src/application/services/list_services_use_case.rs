use crate::domain::service::service::Service;
use crate::ports::service_manager::ServiceManager;
use crate::shared::result::app_result::AppResult;

pub fn list_services(service_manager: &dyn ServiceManager) -> AppResult<Vec<Service>> {
    service_manager.list_services()
}

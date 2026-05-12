use crate::domain::service::service::Service;
use crate::ports::service_manager::ServiceManager;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_service_id::validate_service_id;

pub fn get_service_status(
    service_manager: &dyn ServiceManager,
    service_id: &str,
) -> AppResult<Service> {
    let service_id = validate_service_id(service_id)?;

    service_manager.get_service_status(service_id)
}

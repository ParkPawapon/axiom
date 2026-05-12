use crate::domain::service::service_action::ServiceActionOutcome;
use crate::ports::service_manager::ServiceManager;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_service_id::validate_service_id;

pub fn restart_service(
    service_manager: &dyn ServiceManager,
    service_id: &str,
) -> AppResult<ServiceActionOutcome> {
    let service_id = validate_service_id(service_id)?;

    service_manager.restart_service(service_id)
}

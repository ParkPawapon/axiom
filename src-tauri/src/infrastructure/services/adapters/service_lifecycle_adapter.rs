use crate::shared::result::app_result::AppResult;

use super::service_status_adapter::ServiceProbeResult;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ServiceLifecycleActionResult {
    pub executed: bool,
    pub message: String,
    pub probe: ServiceProbeResult,
}

impl ServiceLifecycleActionResult {
    pub fn completed(message: impl Into<String>, probe: ServiceProbeResult) -> Self {
        Self {
            executed: true,
            message: message.into(),
            probe,
        }
    }

    pub fn blocked(message: impl Into<String>, probe: ServiceProbeResult) -> Self {
        Self {
            executed: false,
            message: message.into(),
            probe,
        }
    }
}

pub trait ServiceLifecycleAdapter {
    fn lifecycle_probe(&self) -> ServiceProbeResult;

    fn start(&self) -> AppResult<ServiceLifecycleActionResult>;

    fn stop(&self) -> AppResult<ServiceLifecycleActionResult>;

    fn restart(&self) -> AppResult<ServiceLifecycleActionResult>;
}

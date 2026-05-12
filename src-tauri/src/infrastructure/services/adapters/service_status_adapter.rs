use crate::domain::service::service_status::ServiceStatus;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ServiceProbeResult {
    pub status: ServiceStatus,
    pub status_message: String,
    pub can_start: bool,
    pub can_stop: bool,
    pub can_restart: bool,
}

impl ServiceProbeResult {
    pub fn detected(message: impl Into<String>) -> Self {
        Self {
            status: ServiceStatus::Detected,
            status_message: message.into(),
            can_start: false,
            can_stop: false,
            can_restart: false,
        }
    }

    pub fn not_configured(message: impl Into<String>) -> Self {
        Self {
            status: ServiceStatus::NotConfigured,
            status_message: message.into(),
            can_start: false,
            can_stop: false,
            can_restart: false,
        }
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            status: ServiceStatus::Failed,
            status_message: message.into(),
            can_start: false,
            can_stop: false,
            can_restart: false,
        }
    }
}

pub trait ServiceStatusAdapter {
    fn probe(&self) -> ServiceProbeResult;
}

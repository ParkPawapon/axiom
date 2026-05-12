use super::service_status::ServiceStatus;
use super::service_type::ServiceType;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    pub name: String,
    pub service_type: ServiceType,
    pub status: ServiceStatus,
    pub description: String,
    pub status_message: String,
    pub can_start: bool,
    pub can_stop: bool,
    pub can_restart: bool,
}

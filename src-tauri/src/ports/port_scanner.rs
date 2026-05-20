use crate::domain::system::system_status::PortCheckResult;
use crate::shared::result::app_result::AppResult;

pub trait PortScanner: Send + Sync {
    fn check_loopback_port(&self, port: u16) -> AppResult<PortCheckResult>;
}

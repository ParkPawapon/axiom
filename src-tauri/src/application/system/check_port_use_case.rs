use crate::domain::system::system_status::PortCheckResult;
use crate::ports::port_scanner::PortScanner;
use crate::shared::result::app_result::AppResult;

pub fn check_port(port_scanner: &dyn PortScanner, port: u16) -> AppResult<PortCheckResult> {
    port_scanner.check_loopback_port(port)
}

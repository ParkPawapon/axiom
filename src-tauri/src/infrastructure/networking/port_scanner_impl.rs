use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

use chrono::Utc;

use crate::domain::system::system_status::{PortCheckResult, PortCheckState};
use crate::ports::port_scanner::PortScanner;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_port::validate_port;

#[derive(Debug, Clone, Copy, Default)]
pub struct PortScannerImpl;

impl PortScannerImpl {
    pub fn new() -> Self {
        Self
    }
}

impl PortScanner for PortScannerImpl {
    fn check_loopback_port(&self, port: u16) -> AppResult<PortCheckResult> {
        let port = validate_port(port)?;
        let bind_address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
        let checked_at = Utc::now();

        match TcpListener::bind(bind_address) {
            Ok(listener) => {
                drop(listener);
                Ok(PortCheckResult {
                    port,
                    bind_address: bind_address.to_string(),
                    state: PortCheckState::Available,
                    checked_at,
                    status_message: format!("Port {port} is available on 127.0.0.1."),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => Ok(PortCheckResult {
                port,
                bind_address: bind_address.to_string(),
                state: PortCheckState::InUse,
                checked_at,
                status_message: format!("Port {port} is already in use on 127.0.0.1."),
            }),
            Err(error) => Err(AppError::Infrastructure(format!(
                "failed to check loopback port {port}: {error}"
            ))),
        }
    }
}

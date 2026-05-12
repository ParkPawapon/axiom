use super::service_status_adapter::{ServiceProbeResult, ServiceStatusAdapter};
use super::version_command_adapter::{VersionCommandAdapter, VersionCommandCandidate};

const MYSQL_CANDIDATES: &[VersionCommandCandidate] = &[VersionCommandCandidate {
    program_name: "mysql",
    args: &["--version"],
    display_name: "MySQL client",
}];

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct MysqlServiceAdapter;

impl MysqlServiceAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ServiceStatusAdapter for MysqlServiceAdapter {
    fn probe(&self) -> ServiceProbeResult {
        VersionCommandAdapter::new("MySQL", MYSQL_CANDIDATES).probe()
    }
}

use super::service_status_adapter::{ServiceProbeResult, ServiceStatusAdapter};
use super::version_command_adapter::{VersionCommandAdapter, VersionCommandCandidate};

const POSTGRESQL_CANDIDATES: &[VersionCommandCandidate] = &[VersionCommandCandidate {
    program_name: "psql",
    args: &["--version"],
    display_name: "PostgreSQL client",
}];

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct PostgresqlServiceAdapter;

impl PostgresqlServiceAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ServiceStatusAdapter for PostgresqlServiceAdapter {
    fn probe(&self) -> ServiceProbeResult {
        VersionCommandAdapter::new("PostgreSQL", POSTGRESQL_CANDIDATES).probe()
    }
}

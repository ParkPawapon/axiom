use super::service_status_adapter::{ServiceProbeResult, ServiceStatusAdapter};
use super::version_command_adapter::{VersionCommandAdapter, VersionCommandCandidate};

const REVERSE_PROXY_CANDIDATES: &[VersionCommandCandidate] = &[
    VersionCommandCandidate {
        program_name: "caddy",
        args: &["version"],
        display_name: "Caddy",
    },
    VersionCommandCandidate {
        program_name: "nginx",
        args: &["-v"],
        display_name: "Nginx",
    },
];

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct ReverseProxyAdapter;

impl ReverseProxyAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ServiceStatusAdapter for ReverseProxyAdapter {
    fn probe(&self) -> ServiceProbeResult {
        VersionCommandAdapter::new("Reverse proxy", REVERSE_PROXY_CANDIDATES).probe()
    }
}

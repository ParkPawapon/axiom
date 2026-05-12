use super::service_status_adapter::{ServiceProbeResult, ServiceStatusAdapter};
use super::version_command_adapter::{VersionCommandAdapter, VersionCommandCandidate};

const DOCKER_CANDIDATES: &[VersionCommandCandidate] = &[VersionCommandCandidate {
    program_name: "docker",
    args: &["--version"],
    display_name: "Docker client",
}];

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct DockerServiceAdapter;

impl DockerServiceAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ServiceStatusAdapter for DockerServiceAdapter {
    fn probe(&self) -> ServiceProbeResult {
        VersionCommandAdapter::new("Docker", DOCKER_CANDIDATES).probe()
    }
}

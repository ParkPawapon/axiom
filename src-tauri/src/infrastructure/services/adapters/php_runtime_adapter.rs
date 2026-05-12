use super::service_status_adapter::{ServiceProbeResult, ServiceStatusAdapter};
use super::version_command_adapter::{VersionCommandAdapter, VersionCommandCandidate};

const PHP_CANDIDATES: &[VersionCommandCandidate] = &[VersionCommandCandidate {
    program_name: "php",
    args: &["--version"],
    display_name: "PHP",
}];

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct PhpRuntimeAdapter;

impl PhpRuntimeAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ServiceStatusAdapter for PhpRuntimeAdapter {
    fn probe(&self) -> ServiceProbeResult {
        VersionCommandAdapter::new("PHP runtime", PHP_CANDIDATES).probe()
    }
}

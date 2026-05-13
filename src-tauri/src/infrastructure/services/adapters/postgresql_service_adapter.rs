use crate::domain::service::service_status::ServiceStatus;
use crate::infrastructure::services::adapters::service_lifecycle_adapter::{
    ServiceLifecycleActionResult, ServiceLifecycleAdapter,
};
use crate::shared::result::app_result::AppResult;

use super::service_status_adapter::{ServiceProbeResult, ServiceStatusAdapter};
use super::version_command_adapter::{VersionCommandAdapter, VersionCommandCandidate};

const POSTGRESQL_CANDIDATES: &[VersionCommandCandidate] = &[VersionCommandCandidate {
    program_name: "psql",
    args: &["--version"],
    display_name: "PostgreSQL client",
}];

#[cfg(target_os = "macos")]
const POSTGRESQL_LAUNCHD_SERVICES:
    &[crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition] = &[
    crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition {
        label: "homebrew.mxcl.postgresql@17",
    },
    crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition {
        label: "homebrew.mxcl.postgresql@16",
    },
    crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition {
        label: "homebrew.mxcl.postgresql@15",
    },
    crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition {
        label: "homebrew.mxcl.postgresql@14",
    },
    crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition {
        label: "homebrew.mxcl.postgresql",
    },
];

#[cfg(windows)]
const POSTGRESQL_WINDOWS_SERVICES:
    &[crate::platform::windows::service_adapter::WindowsServiceDefinition] = &[
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "postgresql-x64-17",
    },
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "postgresql-x64-16",
    },
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "postgresql-x64-15",
    },
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "postgresql-x64-14",
    },
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "postgresql",
    },
];

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct PostgresqlServiceAdapter;

impl PostgresqlServiceAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ServiceStatusAdapter for PostgresqlServiceAdapter {
    fn probe(&self) -> ServiceProbeResult {
        service_probe_with_cli_fallback(platform_probe(), "PostgreSQL", POSTGRESQL_CANDIDATES)
    }
}

impl ServiceLifecycleAdapter for PostgresqlServiceAdapter {
    fn lifecycle_probe(&self) -> ServiceProbeResult {
        self.probe()
    }

    fn start(&self) -> AppResult<ServiceLifecycleActionResult> {
        platform_start()
    }

    fn stop(&self) -> AppResult<ServiceLifecycleActionResult> {
        platform_stop()
    }

    fn restart(&self) -> AppResult<ServiceLifecycleActionResult> {
        platform_restart()
    }
}

fn service_probe_with_cli_fallback(
    platform_probe: ServiceProbeResult,
    service_name: &'static str,
    candidates: &'static [VersionCommandCandidate],
) -> ServiceProbeResult {
    if platform_probe.status == ServiceStatus::NotConfigured {
        return VersionCommandAdapter::new(service_name, candidates).probe();
    }

    platform_probe
}

#[cfg(target_os = "macos")]
fn platform_adapter() -> crate::platform::macos::service_adapter::MacosServiceAdapter {
    crate::platform::macos::service_adapter::MacosServiceAdapter::new(
        "PostgreSQL",
        POSTGRESQL_LAUNCHD_SERVICES,
    )
}

#[cfg(windows)]
fn platform_adapter() -> crate::platform::windows::service_adapter::WindowsServiceAdapter {
    crate::platform::windows::service_adapter::WindowsServiceAdapter::new(
        "PostgreSQL",
        POSTGRESQL_WINDOWS_SERVICES,
    )
}

#[cfg(any(target_os = "macos", windows))]
fn platform_probe() -> ServiceProbeResult {
    platform_adapter().probe()
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_probe() -> ServiceProbeResult {
    ServiceProbeResult::not_configured(
        "PostgreSQL lifecycle is not supported on this operating system.",
    )
}

#[cfg(any(target_os = "macos", windows))]
fn platform_start() -> AppResult<ServiceLifecycleActionResult> {
    platform_adapter().start()
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_start() -> AppResult<ServiceLifecycleActionResult> {
    Ok(ServiceLifecycleActionResult::blocked(
        "PostgreSQL lifecycle is not supported on this operating system.",
        platform_probe(),
    ))
}

#[cfg(any(target_os = "macos", windows))]
fn platform_stop() -> AppResult<ServiceLifecycleActionResult> {
    platform_adapter().stop()
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_stop() -> AppResult<ServiceLifecycleActionResult> {
    Ok(ServiceLifecycleActionResult::blocked(
        "PostgreSQL lifecycle is not supported on this operating system.",
        platform_probe(),
    ))
}

#[cfg(any(target_os = "macos", windows))]
fn platform_restart() -> AppResult<ServiceLifecycleActionResult> {
    platform_adapter().restart()
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_restart() -> AppResult<ServiceLifecycleActionResult> {
    Ok(ServiceLifecycleActionResult::blocked(
        "PostgreSQL lifecycle is not supported on this operating system.",
        platform_probe(),
    ))
}

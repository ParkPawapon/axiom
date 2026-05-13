use crate::domain::service::service_status::ServiceStatus;
use crate::infrastructure::services::adapters::service_lifecycle_adapter::{
    ServiceLifecycleActionResult, ServiceLifecycleAdapter,
};
use crate::shared::result::app_result::AppResult;

use super::service_status_adapter::{ServiceProbeResult, ServiceStatusAdapter};
use super::version_command_adapter::{VersionCommandAdapter, VersionCommandCandidate};

const MYSQL_CANDIDATES: &[VersionCommandCandidate] = &[VersionCommandCandidate {
    program_name: "mysql",
    args: &["--version"],
    display_name: "MySQL client",
}];

#[cfg(target_os = "macos")]
const MYSQL_LAUNCHD_SERVICES:
    &[crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition] = &[
    crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition {
        label: "homebrew.mxcl.mysql",
    },
    crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition {
        label: "homebrew.mxcl.mysql@8.4",
    },
    crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition {
        label: "homebrew.mxcl.mysql@8.3",
    },
    crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition {
        label: "homebrew.mxcl.mysql@8.2",
    },
    crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition {
        label: "homebrew.mxcl.mysql@8.1",
    },
    crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition {
        label: "homebrew.mxcl.mysql@8.0",
    },
    crate::platform::macos::service_adapter::MacosLaunchdServiceDefinition {
        label: "homebrew.mxcl.mysql@5.7",
    },
];

#[cfg(windows)]
const MYSQL_WINDOWS_SERVICES:
    &[crate::platform::windows::service_adapter::WindowsServiceDefinition] = &[
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "MySQL84",
    },
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "MySQL83",
    },
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "MySQL80",
    },
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "MySQL57",
    },
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "MySQL",
    },
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "mysql",
    },
];

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct MysqlServiceAdapter;

impl MysqlServiceAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ServiceStatusAdapter for MysqlServiceAdapter {
    fn probe(&self) -> ServiceProbeResult {
        service_probe_with_cli_fallback(platform_probe(), "MySQL", MYSQL_CANDIDATES)
    }
}

impl ServiceLifecycleAdapter for MysqlServiceAdapter {
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
        "MySQL",
        MYSQL_LAUNCHD_SERVICES,
    )
}

#[cfg(windows)]
fn platform_adapter() -> crate::platform::windows::service_adapter::WindowsServiceAdapter {
    crate::platform::windows::service_adapter::WindowsServiceAdapter::new(
        "MySQL",
        MYSQL_WINDOWS_SERVICES,
    )
}

#[cfg(any(target_os = "macos", windows))]
fn platform_probe() -> ServiceProbeResult {
    platform_adapter().probe()
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_probe() -> ServiceProbeResult {
    ServiceProbeResult::not_configured("MySQL lifecycle is not supported on this operating system.")
}

#[cfg(any(target_os = "macos", windows))]
fn platform_start() -> AppResult<ServiceLifecycleActionResult> {
    platform_adapter().start()
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_start() -> AppResult<ServiceLifecycleActionResult> {
    Ok(ServiceLifecycleActionResult::blocked(
        "MySQL lifecycle is not supported on this operating system.",
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
        "MySQL lifecycle is not supported on this operating system.",
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
        "MySQL lifecycle is not supported on this operating system.",
        platform_probe(),
    ))
}

use crate::domain::service::service_status::ServiceStatus;
use crate::infrastructure::docker::docker_cli_client::DockerCliClient;
use crate::infrastructure::services::adapters::service_lifecycle_adapter::{
    ServiceLifecycleActionResult, ServiceLifecycleAdapter,
};
use crate::ports::docker_client::DockerClient;
use crate::shared::result::app_result::AppResult;

use super::service_status_adapter::{ServiceProbeResult, ServiceStatusAdapter};

#[cfg(windows)]
const DOCKER_WINDOWS_SERVICES:
    &[crate::platform::windows::service_adapter::WindowsServiceDefinition] = &[
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "com.docker.service",
    },
    crate::platform::windows::service_adapter::WindowsServiceDefinition {
        service_name: "docker",
    },
];

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct DockerServiceAdapter;

impl DockerServiceAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ServiceStatusAdapter for DockerServiceAdapter {
    fn probe(&self) -> ServiceProbeResult {
        match DockerCliClient::new().probe_engine() {
            Ok(probe) if probe.engine_running => ServiceProbeResult::running(probe.status_message),
            Ok(probe) if probe.cli_found => ServiceProbeResult::stopped(probe.status_message),
            Ok(probe) => ServiceProbeResult::not_configured(probe.status_message),
            Err(error) => {
                ServiceProbeResult::failed(format!("Docker diagnostics failed safely: {error}"))
            }
        }
    }
}

impl ServiceLifecycleAdapter for DockerServiceAdapter {
    fn lifecycle_probe(&self) -> ServiceProbeResult {
        self.probe()
    }

    fn start(&self) -> AppResult<ServiceLifecycleActionResult> {
        if self.probe().status == ServiceStatus::Running {
            if let Some(message) = DockerCliClient::new().start_configured_compose_project()? {
                return Ok(ServiceLifecycleActionResult::completed(
                    message,
                    DockerServiceAdapter::new().probe(),
                ));
            }
        }

        platform_start()
    }

    fn stop(&self) -> AppResult<ServiceLifecycleActionResult> {
        if self.probe().status == ServiceStatus::Running {
            if let Some(message) = DockerCliClient::new().stop_configured_compose_project()? {
                return Ok(ServiceLifecycleActionResult::completed(
                    message,
                    DockerServiceAdapter::new().probe(),
                ));
            }
        }

        platform_stop()
    }

    fn restart(&self) -> AppResult<ServiceLifecycleActionResult> {
        if self.probe().status == ServiceStatus::Running {
            let client = DockerCliClient::new();
            if let Some(stop_message) = client.stop_configured_compose_project()? {
                let start_message =
                    client
                        .start_configured_compose_project()?
                        .unwrap_or_else(|| {
                            "Managed Docker Compose project start was not configured.".to_string()
                        });

                return Ok(ServiceLifecycleActionResult::completed(
                    format!("{stop_message} {start_message}"),
                    DockerServiceAdapter::new().probe(),
                ));
            }
        }

        platform_restart()
    }
}

#[cfg(target_os = "macos")]
fn platform_start() -> AppResult<ServiceLifecycleActionResult> {
    crate::platform::macos::service_adapter::MacosDockerDesktopAdapter::start().map(refresh_probe)
}

#[cfg(windows)]
fn platform_start() -> AppResult<ServiceLifecycleActionResult> {
    docker_windows_adapter().start().map(refresh_probe)
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_start() -> AppResult<ServiceLifecycleActionResult> {
    Ok(ServiceLifecycleActionResult::blocked(
        "Docker lifecycle is not supported on this operating system.",
        DockerServiceAdapter::new().probe(),
    ))
}

#[cfg(target_os = "macos")]
fn platform_stop() -> AppResult<ServiceLifecycleActionResult> {
    crate::platform::macos::service_adapter::MacosDockerDesktopAdapter::stop().map(refresh_probe)
}

#[cfg(windows)]
fn platform_stop() -> AppResult<ServiceLifecycleActionResult> {
    docker_windows_adapter().stop().map(refresh_probe)
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_stop() -> AppResult<ServiceLifecycleActionResult> {
    Ok(ServiceLifecycleActionResult::blocked(
        "Docker lifecycle is not supported on this operating system.",
        DockerServiceAdapter::new().probe(),
    ))
}

#[cfg(target_os = "macos")]
fn platform_restart() -> AppResult<ServiceLifecycleActionResult> {
    crate::platform::macos::service_adapter::MacosDockerDesktopAdapter::restart().map(refresh_probe)
}

#[cfg(windows)]
fn platform_restart() -> AppResult<ServiceLifecycleActionResult> {
    docker_windows_adapter().restart().map(refresh_probe)
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_restart() -> AppResult<ServiceLifecycleActionResult> {
    Ok(ServiceLifecycleActionResult::blocked(
        "Docker lifecycle is not supported on this operating system.",
        DockerServiceAdapter::new().probe(),
    ))
}

#[cfg(windows)]
fn docker_windows_adapter() -> crate::platform::windows::service_adapter::WindowsServiceAdapter {
    crate::platform::windows::service_adapter::WindowsServiceAdapter::new(
        "Docker",
        DOCKER_WINDOWS_SERVICES,
    )
}

fn refresh_probe(mut result: ServiceLifecycleActionResult) -> ServiceLifecycleActionResult {
    if result.probe.status != ServiceStatus::NotConfigured {
        result.probe = DockerServiceAdapter::new().probe();
    }

    result
}

use crate::domain::service::service::Service;
use crate::domain::service::service_action::{
    ServiceAction, ServiceActionOutcome, ServiceActionState,
};
use crate::domain::service::service_type::ServiceType;
use crate::infrastructure::services::adapters::docker_service_adapter::DockerServiceAdapter;
use crate::infrastructure::services::adapters::mysql_service_adapter::MysqlServiceAdapter;
use crate::infrastructure::services::adapters::php_runtime_adapter::PhpRuntimeAdapter;
use crate::infrastructure::services::adapters::postgresql_service_adapter::PostgresqlServiceAdapter;
use crate::infrastructure::services::adapters::reverse_proxy_adapter::ReverseProxyAdapter;
use crate::infrastructure::services::adapters::service_lifecycle_adapter::{
    ServiceLifecycleActionResult, ServiceLifecycleAdapter,
};
use crate::infrastructure::services::adapters::service_status_adapter::{
    ServiceProbeResult, ServiceStatusAdapter,
};
use crate::ports::service_manager::ServiceManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug, Clone, Copy)]
struct ServiceDefinition {
    id: &'static str,
    name: &'static str,
    service_type: ServiceType,
    description: &'static str,
    required_driver: &'static str,
    adapter_kind: ServiceAdapterKind,
}

#[derive(Debug, Clone, Copy)]
enum ServiceAdapterKind {
    Docker,
    Mysql,
    Php,
    Postgresql,
    ReverseProxy,
}

#[derive(Debug, Default)]
pub struct LocalServiceManager;

const SERVICE_CATALOG: &[ServiceDefinition] = &[
    ServiceDefinition {
        id: "php-runtime",
        name: "PHP Runtime",
        service_type: ServiceType::Php,
        description: "Per-project PHP process control for local development.",
        required_driver: "PHP runtime driver",
        adapter_kind: ServiceAdapterKind::Php,
    },
    ServiceDefinition {
        id: "mysql",
        name: "MySQL",
        service_type: ServiceType::Mysql,
        description: "Local MySQL service lifecycle through launchd on macOS or Windows Service Control.",
        required_driver: "MySQL launchd or Windows service driver",
        adapter_kind: ServiceAdapterKind::Mysql,
    },
    ServiceDefinition {
        id: "postgresql",
        name: "PostgreSQL",
        service_type: ServiceType::Postgresql,
        description: "Local PostgreSQL service lifecycle through launchd on macOS or Windows Service Control.",
        required_driver: "PostgreSQL launchd or Windows service driver",
        adapter_kind: ServiceAdapterKind::Postgresql,
    },
    ServiceDefinition {
        id: "reverse-proxy",
        name: "Reverse Proxy",
        service_type: ServiceType::ReverseProxy,
        description: "Local reverse proxy lifecycle for Caddy or Nginx through OS service adapters.",
        required_driver: "reverse proxy launchd or Windows service driver",
        adapter_kind: ServiceAdapterKind::ReverseProxy,
    },
    ServiceDefinition {
        id: "docker",
        name: "Docker",
        service_type: ServiceType::Docker,
        description: "Docker engine and Compose orchestration diagnostics through the Docker CLI boundary.",
        required_driver: "Docker Desktop or Windows Docker service driver",
        adapter_kind: ServiceAdapterKind::Docker,
    },
];

impl LocalServiceManager {
    pub fn new() -> Self {
        Self
    }

    fn find_definition(&self, service_id: &str) -> AppResult<&'static ServiceDefinition> {
        SERVICE_CATALOG
            .iter()
            .find(|definition| definition.id == service_id)
            .ok_or_else(|| AppError::NotFound(format!("service `{service_id}` is not registered")))
    }

    fn service_from_definition(definition: &ServiceDefinition) -> Service {
        let probe = Self::probe_definition(definition);

        Self::service_from_probe(definition, probe)
    }

    fn service_from_probe(definition: &ServiceDefinition, probe: ServiceProbeResult) -> Service {
        Service {
            id: definition.id.to_string(),
            name: definition.name.to_string(),
            service_type: definition.service_type,
            status: probe.status,
            description: definition.description.to_string(),
            status_message: probe.status_message,
            can_start: probe.can_start,
            can_stop: probe.can_stop,
            can_restart: probe.can_restart,
        }
    }

    fn probe_definition(definition: &ServiceDefinition) -> ServiceProbeResult {
        match definition.adapter_kind {
            ServiceAdapterKind::Docker => DockerServiceAdapter::new().lifecycle_probe(),
            ServiceAdapterKind::Mysql => MysqlServiceAdapter::new().lifecycle_probe(),
            ServiceAdapterKind::Php => PhpRuntimeAdapter::new().probe(),
            ServiceAdapterKind::Postgresql => PostgresqlServiceAdapter::new().lifecycle_probe(),
            ServiceAdapterKind::ReverseProxy => ReverseProxyAdapter::new().lifecycle_probe(),
        }
    }

    fn run_lifecycle_action(
        definition: &ServiceDefinition,
        action: ServiceAction,
    ) -> AppResult<ServiceLifecycleActionResult> {
        match (definition.adapter_kind, action) {
            (ServiceAdapterKind::Docker, ServiceAction::Start) => DockerServiceAdapter::new().start(),
            (ServiceAdapterKind::Docker, ServiceAction::Stop) => DockerServiceAdapter::new().stop(),
            (ServiceAdapterKind::Docker, ServiceAction::Restart) => {
                DockerServiceAdapter::new().restart()
            }
            (ServiceAdapterKind::Mysql, ServiceAction::Start) => MysqlServiceAdapter::new().start(),
            (ServiceAdapterKind::Mysql, ServiceAction::Stop) => MysqlServiceAdapter::new().stop(),
            (ServiceAdapterKind::Mysql, ServiceAction::Restart) => {
                MysqlServiceAdapter::new().restart()
            }
            (ServiceAdapterKind::Postgresql, ServiceAction::Start) => {
                PostgresqlServiceAdapter::new().start()
            }
            (ServiceAdapterKind::Postgresql, ServiceAction::Stop) => {
                PostgresqlServiceAdapter::new().stop()
            }
            (ServiceAdapterKind::Postgresql, ServiceAction::Restart) => {
                PostgresqlServiceAdapter::new().restart()
            }
            (ServiceAdapterKind::ReverseProxy, ServiceAction::Start) => {
                ReverseProxyAdapter::new().start()
            }
            (ServiceAdapterKind::ReverseProxy, ServiceAction::Stop) => {
                ReverseProxyAdapter::new().stop()
            }
            (ServiceAdapterKind::ReverseProxy, ServiceAction::Restart) => {
                ReverseProxyAdapter::new().restart()
            }
            (ServiceAdapterKind::Php, _) => Ok(ServiceLifecycleActionResult::blocked(
                "PHP runtime service control is project-scoped. Use project process controls instead of global PHP service lifecycle.",
                Self::probe_definition(definition),
            )),
        }
    }

    fn action_outcome(
        &self,
        service_id: &str,
        action: ServiceAction,
    ) -> AppResult<ServiceActionOutcome> {
        let definition = self.find_definition(service_id)?;
        let lifecycle_result = Self::run_lifecycle_action(definition, action)?;
        let service = Self::service_from_probe(definition, lifecycle_result.probe);
        let state = if lifecycle_result.executed {
            ServiceActionState::Completed
        } else {
            ServiceActionState::Blocked
        };

        Ok(ServiceActionOutcome {
            action,
            state,
            service,
            message: if lifecycle_result.executed {
                lifecycle_result.message
            } else {
                format!(
                    "{} {}",
                    definition.required_driver, lifecycle_result.message
                )
            },
        })
    }
}

impl ServiceManager for LocalServiceManager {
    fn list_services(&self) -> AppResult<Vec<Service>> {
        Ok(SERVICE_CATALOG
            .iter()
            .map(Self::service_from_definition)
            .collect())
    }

    fn get_service_status(&self, service_id: &str) -> AppResult<Service> {
        self.find_definition(service_id)
            .map(Self::service_from_definition)
    }

    fn start_service(&self, service_id: &str) -> AppResult<ServiceActionOutcome> {
        self.action_outcome(service_id, ServiceAction::Start)
    }

    fn stop_service(&self, service_id: &str) -> AppResult<ServiceActionOutcome> {
        self.action_outcome(service_id, ServiceAction::Stop)
    }

    fn restart_service(&self, service_id: &str) -> AppResult<ServiceActionOutcome> {
        self.action_outcome(service_id, ServiceAction::Restart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_expected_service_boundaries() {
        let manager = LocalServiceManager::new();

        let services = manager
            .list_services()
            .expect("service catalog should load");

        assert_eq!(services.len(), 5);
        assert!(services.iter().any(|service| service.id == "mysql"));
        assert!(services.iter().any(|service| service.id == "postgresql"));
        assert!(services.iter().any(|service| service.id == "reverse-proxy"));
        assert!(services.iter().any(|service| service.id == "docker"));
    }

    #[test]
    fn blocks_global_php_runtime_lifecycle_actions() {
        let manager = LocalServiceManager::new();

        let outcome = manager
            .start_service("php-runtime")
            .expect("known service should return an outcome");

        assert_eq!(outcome.state, ServiceActionState::Blocked);
        assert!(!outcome.service.can_start);
        assert!(outcome.message.contains("project-scoped"));
    }

    #[test]
    fn rejects_unknown_services() {
        let manager = LocalServiceManager::new();

        let result = manager.get_service_status("unknown-service");

        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}

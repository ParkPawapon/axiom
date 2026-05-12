use crate::domain::service::service::Service;
use crate::domain::service::service_action::{
    ServiceAction, ServiceActionOutcome, ServiceActionState,
};
use crate::domain::service::service_status::ServiceStatus;
use crate::domain::service::service_type::ServiceType;
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
    },
    ServiceDefinition {
        id: "mysql",
        name: "MySQL",
        service_type: ServiceType::Mysql,
        description: "Local MySQL service lifecycle boundary.",
        required_driver: "MySQL service driver",
    },
    ServiceDefinition {
        id: "postgresql",
        name: "PostgreSQL",
        service_type: ServiceType::Postgresql,
        description: "Local PostgreSQL service lifecycle boundary.",
        required_driver: "PostgreSQL service driver",
    },
    ServiceDefinition {
        id: "reverse-proxy",
        name: "Reverse Proxy",
        service_type: ServiceType::ReverseProxy,
        description: "Local domain routing and HTTPS entrypoint boundary.",
        required_driver: "reverse proxy driver",
    },
    ServiceDefinition {
        id: "docker",
        name: "Docker",
        service_type: ServiceType::Docker,
        description: "Docker-backed service orchestration boundary.",
        required_driver: "Docker client driver",
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
        Service {
            id: definition.id.to_string(),
            name: definition.name.to_string(),
            service_type: definition.service_type,
            status: ServiceStatus::NotConfigured,
            description: definition.description.to_string(),
            status_message: format!(
                "{} is not configured yet. Configure a production service adapter before enabling lifecycle actions.",
                definition.required_driver
            ),
            can_start: false,
            can_stop: false,
            can_restart: false,
        }
    }

    fn blocked_outcome(
        &self,
        service_id: &str,
        action: ServiceAction,
    ) -> AppResult<ServiceActionOutcome> {
        let definition = self.find_definition(service_id)?;
        let service = Self::service_from_definition(definition);

        Ok(ServiceActionOutcome {
            action,
            state: ServiceActionState::Blocked,
            service,
            message: format!(
                "{} is not configured. The request was validated, but no OS-level action was executed.",
                definition.required_driver
            ),
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
        self.blocked_outcome(service_id, ServiceAction::Start)
    }

    fn stop_service(&self, service_id: &str) -> AppResult<ServiceActionOutcome> {
        self.blocked_outcome(service_id, ServiceAction::Stop)
    }

    fn restart_service(&self, service_id: &str) -> AppResult<ServiceActionOutcome> {
        self.blocked_outcome(service_id, ServiceAction::Restart)
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
        assert!(services.iter().all(|service| !service.can_start));
        assert!(services
            .iter()
            .all(|service| service.status == ServiceStatus::NotConfigured));
    }

    #[test]
    fn blocks_lifecycle_actions_without_driver() {
        let manager = LocalServiceManager::new();

        let outcome = manager
            .start_service("php-runtime")
            .expect("known service should return an outcome");

        assert_eq!(outcome.state, ServiceActionState::Blocked);
        assert_eq!(outcome.service.status, ServiceStatus::NotConfigured);
    }

    #[test]
    fn rejects_unknown_services() {
        let manager = LocalServiceManager::new();

        let result = manager.get_service_status("unknown-service");

        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}

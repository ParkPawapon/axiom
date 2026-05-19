use std::collections::BTreeMap;
use std::fs;
use std::sync::{Arc, Mutex};

use axiomphp_lib::domain::docker::docker_project::{
    DockerComposeProfile, DockerProjectComposeRequest, DockerProjectImageOverride,
    DockerProjectResourceLimits,
};
use axiomphp_lib::domain::project::project::Project;
use axiomphp_lib::domain::project::project_id::ProjectId;
use axiomphp_lib::domain::project::project_path::ProjectPath;
use axiomphp_lib::infrastructure::docker::docker_compose_generator::image_is_digest_pinned;
use axiomphp_lib::infrastructure::docker::project_docker_orchestrator::ProjectDockerOrchestrator;
use axiomphp_lib::ports::docker_project_orchestrator::DockerProjectOrchestrator as _;
use axiomphp_lib::ports::secure_storage::SecureStorage;
use axiomphp_lib::shared::result::app_result::AppResult;
use chrono::Utc;
use uuid::Uuid;

#[derive(Default)]
struct MemorySecureStorage {
    secrets: Mutex<BTreeMap<(String, String), String>>,
}

impl SecureStorage for MemorySecureStorage {
    fn store_secret(&self, namespace: &str, key: &str, secret: &str) -> AppResult<()> {
        let mut secrets = self.secrets.lock().expect("memory secure storage poisoned");
        secrets.insert((namespace.to_string(), key.to_string()), secret.to_string());
        Ok(())
    }

    fn get_secret(&self, namespace: &str, key: &str) -> AppResult<Option<String>> {
        let secrets = self.secrets.lock().expect("memory secure storage poisoned");
        Ok(secrets
            .get(&(namespace.to_string(), key.to_string()))
            .cloned())
    }

    fn delete_secret(&self, namespace: &str, key: &str) -> AppResult<()> {
        let mut secrets = self.secrets.lock().expect("memory secure storage poisoned");
        secrets.remove(&(namespace.to_string(), key.to_string()));
        Ok(())
    }
}

#[test]
fn starts_php_docker_container_in_isolated_project_context_when_enabled(
) -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("AXIOM_RUN_DOCKER_INTEGRATION_TEST")
        .ok()
        .as_deref()
        != Some("1")
    {
        eprintln!("skipping Docker integration test; set AXIOM_RUN_DOCKER_INTEGRATION_TEST=1");
        return Ok(());
    }

    let php_image = std::env::var("AXIOM_DOCKER_INTEGRATION_PHP_IMAGE").expect(
        "AXIOM_DOCKER_INTEGRATION_PHP_IMAGE must be a digest-pinned PHP image for integration tests",
    );
    assert!(
        image_is_digest_pinned(&php_image),
        "integration image must use @sha256:<digest>"
    );

    let suffix = Uuid::new_v4().simple().to_string();
    let project_id = format!("docker-it-{}", &suffix[..12]);
    let workspace = std::env::temp_dir().join(format!("axiom-{project_id}"));
    let document_root = workspace.join("document-root");
    let compose_root = workspace.join("compose-root");

    fs::create_dir_all(&document_root)?;
    fs::write(document_root.join("index.php"), "<?php echo 'ok';\n")?;

    let now = Utc::now();
    let project = Project {
        id: ProjectId(project_id),
        name: "Docker integration project".to_string(),
        document_root: ProjectPath(document_root.to_string_lossy().into_owned()),
        created_at: now,
        updated_at: now,
    };
    let request = DockerProjectComposeRequest {
        project_id: project.id.clone(),
        profiles: vec![DockerComposeProfile::Php],
        image_overrides: vec![DockerProjectImageOverride {
            profile: DockerComposeProfile::Php,
            image: php_image,
        }],
        resource_limits: DockerProjectResourceLimits {
            cpus: Some(0.5),
            memory_mb: Some(256),
        },
    };
    let orchestrator = ProjectDockerOrchestrator::with_base_dir(
        Arc::new(MemorySecureStorage::default()),
        compose_root,
    );

    let plan = orchestrator.generate_compose_plan(&project, &request)?;
    assert!(
        plan.compose_file_written,
        "compose generation blocked unexpectedly: {:?}",
        plan.image_trust
    );

    let start_result = orchestrator.start_project(&project, &request)?;
    assert!(start_result.plan.compose_file_written);
    assert!(start_result.runtime.engine_running);
    assert!(start_result
        .runtime
        .containers
        .iter()
        .any(|container| container.service_name == "php"));

    let _ = orchestrator.stop_project(&project);
    let _ = orchestrator.remove_project_volumes(&project);
    let _ = fs::remove_dir_all(&workspace);

    Ok(())
}

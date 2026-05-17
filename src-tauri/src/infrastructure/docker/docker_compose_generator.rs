use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::project::project::Project;
use crate::domain::project::project_docker::ProjectDockerComposeConfig;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_path::validate_existing_directory_path;
use crate::shared::validation::validate_project_id::validate_project_id;

const CONTAINER_DOCUMENT_ROOT: &str = "/workspace";
const CONTAINER_PORT: u16 = 8080;
const SERVICE_NAME: &str = "php";

#[derive(Debug, Clone)]
pub struct DockerComposeGenerator {
    compose_root: PathBuf,
}

impl DockerComposeGenerator {
    pub fn new(compose_root: PathBuf) -> Self {
        Self { compose_root }
    }

    pub fn project_runtime_dir(&self, project_id: &str) -> AppResult<PathBuf> {
        let project_id = validate_project_id(project_id)?;

        Ok(self.compose_root.join(project_id))
    }

    pub fn project_compose_file(&self, project_id: &str) -> AppResult<PathBuf> {
        Ok(self.project_runtime_dir(project_id)?.join("compose.yaml"))
    }

    pub fn compose_project_name(&self, project_id: &str) -> AppResult<String> {
        let project_id = validate_project_id(project_id)?;
        let mut name = format!("axiomphp-{project_id}")
            .bytes()
            .map(|byte| match byte {
                b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => byte as char,
                _ => '-',
            })
            .collect::<String>();

        name.truncate(63);

        Ok(name.trim_matches(['-', '_']).to_string())
    }

    pub fn generate(
        &self,
        project: &Project,
        php_version: &RuntimeVersion,
    ) -> AppResult<ProjectDockerComposeConfig> {
        let document_root = validate_existing_directory_path(&project.document_root.0)?;
        let project_id = validate_project_id(&project.id.0)?;
        let project_runtime_dir = self.project_runtime_dir(project_id)?;
        let compose_file = self.project_compose_file(project_id)?;
        let compose_project_name = self.compose_project_name(project_id)?;
        let php_image = php_image_for_version(php_version);

        fs::create_dir_all(&project_runtime_dir).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to create Docker project runtime directory: {error}"
            ))
        })?;

        fs::write(
            &compose_file,
            compose_yaml(
                &project.id.0,
                &project.name,
                &document_root,
                &php_image,
                &compose_project_name,
            )?,
        )
        .map_err(|error| {
            AppError::Infrastructure(format!("failed to write project Compose file: {error}"))
        })?;

        Ok(ProjectDockerComposeConfig {
            project_id: project.id.clone(),
            compose_project_name,
            compose_file_path: compose_file.to_string_lossy().into_owned(),
            project_runtime_dir: project_runtime_dir.to_string_lossy().into_owned(),
            document_root: document_root.to_string_lossy().into_owned(),
            php_image,
            service_name: SERVICE_NAME.to_string(),
            container_document_root: CONTAINER_DOCUMENT_ROOT.to_string(),
            container_port: CONTAINER_PORT,
            status_message:
                "Project Docker Compose file was generated in the app-owned runtime directory."
                    .to_string(),
        })
    }
}

pub fn php_image_for_version(version: &RuntimeVersion) -> String {
    format!("php:{}-cli", version.as_str())
}

fn compose_yaml(
    project_id: &str,
    project_name: &str,
    document_root: &Path,
    php_image: &str,
    compose_project_name: &str,
) -> AppResult<String> {
    Ok(format!(
        "name: {}\nservices:\n  {SERVICE_NAME}:\n    image: {}\n    container_name: {}\n    working_dir: {CONTAINER_DOCUMENT_ROOT}\n    command:\n      - php\n      - -S\n      - 0.0.0.0:{CONTAINER_PORT}\n      - -t\n      - {CONTAINER_DOCUMENT_ROOT}\n    ports:\n      - {}\n    volumes:\n      - type: bind\n        source: {}\n        target: {CONTAINER_DOCUMENT_ROOT}\n        read_only: false\n    labels:\n      dev.axiomphp.project-id: {}\n      dev.axiomphp.project-name: {}\n    restart: \"no\"\n",
        yaml_scalar(compose_project_name)?,
        yaml_scalar(php_image)?,
        yaml_scalar(&format!("{compose_project_name}-{SERVICE_NAME}"))?,
        yaml_scalar(&format!("127.0.0.1::{CONTAINER_PORT}"))?,
        yaml_scalar(&document_root.to_string_lossy())?,
        yaml_scalar(project_id)?,
        yaml_scalar(project_name)?,
    ))
}

fn yaml_scalar(value: &str) -> AppResult<String> {
    serde_json::to_string(value).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to serialize Docker Compose scalar: {error}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::Utc;
    use uuid::Uuid;

    use crate::domain::project::project_id::ProjectId;
    use crate::domain::project::project_path::ProjectPath;

    use super::*;

    #[test]
    fn generates_project_scoped_compose_file() {
        let temp_dir = std::env::temp_dir().join(format!("axiom-docker-{}", Uuid::new_v4()));
        let document_root = temp_dir.join("public");
        let compose_root = temp_dir.join("compose");
        fs::create_dir_all(&document_root).expect("document root");
        let generator = DockerComposeGenerator::new(compose_root);
        let project = Project {
            id: ProjectId("demo-project".to_string()),
            name: "Demo Project".to_string(),
            document_root: ProjectPath(document_root.to_string_lossy().into_owned()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let config = generator
            .generate(&project, &RuntimeVersion::trusted("8.4"))
            .expect("compose file should generate");
        let contents = fs::read_to_string(&config.compose_file_path).expect("compose file");

        assert_eq!(config.compose_project_name, "axiomphp-demo-project");
        assert!(contents.contains("php:8.4-cli"));
        assert!(contents.contains("dev.axiomphp.project-id"));

        let _ = fs::remove_dir_all(temp_dir);
    }
}

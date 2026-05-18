use std::collections::BTreeMap;
use std::path::Path;

use crate::domain::docker::docker_project::{
    DockerComposeProfile, DockerImageTrustEvaluation, DockerProjectServicePlan,
    DockerProjectVolumePlan,
};
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const PHP_CONTAINER_PORT: u16 = 8000;
const MYSQL_CONTAINER_PORT: u16 = 3306;
const POSTGRES_CONTAINER_PORT: u16 = 5432;
const REVERSE_PROXY_CONTAINER_PORT: u16 = 80;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DockerComposeGenerationInput {
    pub project_id: String,
    pub document_root: String,
    pub compose_project_name: String,
    pub env_file_name: String,
    pub reverse_proxy_config_file_name: String,
    pub profiles: Vec<DockerComposeProfile>,
    pub images: BTreeMap<DockerComposeProfile, String>,
    pub ports: DockerProjectPorts,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DockerProjectPorts {
    pub mysql_host_port: u16,
    pub postgres_host_port: u16,
    pub reverse_proxy_host_port: u16,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DockerComposeGenerationOutput {
    pub compose_yaml: String,
    pub reverse_proxy_config: Option<String>,
    pub services: Vec<DockerProjectServicePlan>,
    pub volumes: Vec<DockerProjectVolumePlan>,
    pub image_trust: Vec<DockerImageTrustEvaluation>,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct DockerComposeGenerator;

impl DockerComposeGenerator {
    pub fn generate(
        &self,
        input: DockerComposeGenerationInput,
    ) -> AppResult<DockerComposeGenerationOutput> {
        validate_document_root(&input.document_root)?;

        let profiles = normalize_profiles(&input.profiles);
        let image_trust = profiles
            .iter()
            .map(|profile| {
                image_trust_evaluation(*profile, required_image(&input.images, *profile))
            })
            .collect::<Vec<_>>();
        let mut services = Vec::new();
        let mut volumes = Vec::new();
        let mut yaml = String::new();

        yaml.push_str("name: ");
        yaml.push_str(&quote_yaml(&input.compose_project_name));
        yaml.push_str("\nservices:\n");

        if profiles.contains(&DockerComposeProfile::Php) {
            let image = required_image(&input.images, DockerComposeProfile::Php).to_string();
            services.push(DockerProjectServicePlan {
                profile: DockerComposeProfile::Php,
                service_name: "php".to_string(),
                image: image.clone(),
                host_port: None,
                container_port: Some(PHP_CONTAINER_PORT),
                status_message: "PHP development server service is included.".to_string(),
            });
            yaml.push_str(&php_service_yaml(&image, &input.document_root));
        }

        if profiles.contains(&DockerComposeProfile::Mysql) {
            let image = required_image(&input.images, DockerComposeProfile::Mysql).to_string();
            let volume_name = mysql_volume_name(&input.project_id);
            services.push(DockerProjectServicePlan {
                profile: DockerComposeProfile::Mysql,
                service_name: "mysql".to_string(),
                image: image.clone(),
                host_port: Some(input.ports.mysql_host_port),
                container_port: Some(MYSQL_CONTAINER_PORT),
                status_message: "Project-specific MySQL service profile is included.".to_string(),
            });
            volumes.push(DockerProjectVolumePlan {
                name: volume_name.clone(),
                service_name: "mysql".to_string(),
                mount_path: "/var/lib/mysql".to_string(),
                created: false,
            });
            yaml.push_str(&mysql_service_yaml(
                &image,
                input.ports.mysql_host_port,
                &volume_name,
            ));
        }

        if profiles.contains(&DockerComposeProfile::Postgresql) {
            let image = required_image(&input.images, DockerComposeProfile::Postgresql).to_string();
            let volume_name = postgres_volume_name(&input.project_id);
            services.push(DockerProjectServicePlan {
                profile: DockerComposeProfile::Postgresql,
                service_name: "postgres".to_string(),
                image: image.clone(),
                host_port: Some(input.ports.postgres_host_port),
                container_port: Some(POSTGRES_CONTAINER_PORT),
                status_message: "Project-specific PostgreSQL service profile is included."
                    .to_string(),
            });
            volumes.push(DockerProjectVolumePlan {
                name: volume_name.clone(),
                service_name: "postgres".to_string(),
                mount_path: "/var/lib/postgresql/data".to_string(),
                created: false,
            });
            yaml.push_str(&postgres_service_yaml(
                &image,
                input.ports.postgres_host_port,
                &volume_name,
            ));
        }

        let reverse_proxy_config = profiles
            .contains(&DockerComposeProfile::ReverseProxy)
            .then(|| reverse_proxy_config());

        if profiles.contains(&DockerComposeProfile::ReverseProxy) {
            let image =
                required_image(&input.images, DockerComposeProfile::ReverseProxy).to_string();
            services.push(DockerProjectServicePlan {
                profile: DockerComposeProfile::ReverseProxy,
                service_name: "reverse-proxy".to_string(),
                image: image.clone(),
                host_port: Some(input.ports.reverse_proxy_host_port),
                container_port: Some(REVERSE_PROXY_CONTAINER_PORT),
                status_message: "Project-specific reverse proxy profile is included.".to_string(),
            });
            yaml.push_str(&reverse_proxy_service_yaml(
                &image,
                input.ports.reverse_proxy_host_port,
                &input.reverse_proxy_config_file_name,
            ));
        }

        if !volumes.is_empty() {
            yaml.push_str("volumes:\n");
            for volume in &volumes {
                yaml.push_str("  ");
                yaml.push_str(&volume.name);
                yaml.push_str(":\n");
                yaml.push_str("    labels:\n");
                yaml.push_str("      dev.axiomphp.project-id: ");
                yaml.push_str(&quote_yaml(&input.project_id));
                yaml.push('\n');
            }
        }

        yaml.push_str("networks:\n  default:\n    name: ");
        yaml.push_str(&quote_yaml(&format!(
            "{}_network",
            input.compose_project_name
        )));
        yaml.push('\n');

        Ok(DockerComposeGenerationOutput {
            compose_yaml: yaml,
            reverse_proxy_config,
            services,
            volumes,
            image_trust,
        })
    }
}

pub fn normalize_profiles(profiles: &[DockerComposeProfile]) -> Vec<DockerComposeProfile> {
    let mut normalized = profiles.to_vec();

    if normalized.is_empty() {
        normalized.push(DockerComposeProfile::Php);
    }

    if normalized.contains(&DockerComposeProfile::ReverseProxy)
        && !normalized.contains(&DockerComposeProfile::Php)
    {
        normalized.push(DockerComposeProfile::Php);
    }

    normalized.sort();
    normalized.dedup();
    normalized
}

pub fn mysql_volume_name(project_id: &str) -> String {
    format!("axiom_{}_mysql_data", volume_safe_project_id(project_id))
}

pub fn postgres_volume_name(project_id: &str) -> String {
    format!("axiom_{}_postgres_data", volume_safe_project_id(project_id))
}

pub fn image_trust_evaluation(
    profile: DockerComposeProfile,
    image: &str,
) -> DockerImageTrustEvaluation {
    let pinned_by_digest = image_is_digest_pinned(image);
    let allowed = pinned_by_digest;
    let status_message = if allowed {
        "Image reference is pinned by sha256 digest.".to_string()
    } else {
        "Image reference is blocked until configured with an immutable @sha256 digest.".to_string()
    };

    DockerImageTrustEvaluation {
        profile,
        image: image.to_string(),
        pinned_by_digest,
        allowed,
        status_message,
    }
}

pub fn image_is_digest_pinned(image: &str) -> bool {
    let Some((_name, digest)) = image.rsplit_once("@sha256:") else {
        return false;
    };

    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn php_service_yaml(image: &str, document_root: &str) -> String {
    format!(
        r#"  php:
    image: {image}
    profiles: ["php"]
    working_dir: /workspace
    command: ["php", "-S", "0.0.0.0:8000", "-t", "/workspace"]
    expose:
      - "8000"
    volumes:
      - type: bind
        source: {document_root}
        target: /workspace
    labels:
      dev.axiomphp.service: "php"
"#,
        image = quote_yaml(image),
        document_root = quote_yaml(document_root),
    )
}

fn mysql_service_yaml(image: &str, host_port: u16, volume_name: &str) -> String {
    format!(
        r#"  mysql:
    image: {image}
    profiles: ["mysql"]
    ports:
      - "127.0.0.1:{host_port}:3306"
    environment:
      MYSQL_DATABASE: "${{AXIOM_MYSQL_DATABASE}}"
      MYSQL_USER: "${{AXIOM_MYSQL_USER}}"
      MYSQL_PASSWORD: "${{AXIOM_MYSQL_PASSWORD}}"
      MYSQL_ROOT_PASSWORD: "${{AXIOM_MYSQL_ROOT_PASSWORD}}"
    volumes:
      - {volume_name}:/var/lib/mysql
    labels:
      dev.axiomphp.service: "mysql"
"#,
        image = quote_yaml(image),
    )
}

fn postgres_service_yaml(image: &str, host_port: u16, volume_name: &str) -> String {
    format!(
        r#"  postgres:
    image: {image}
    profiles: ["postgresql"]
    ports:
      - "127.0.0.1:{host_port}:5432"
    environment:
      POSTGRES_DB: "${{AXIOM_POSTGRES_DATABASE}}"
      POSTGRES_USER: "${{AXIOM_POSTGRES_USER}}"
      POSTGRES_PASSWORD: "${{AXIOM_POSTGRES_PASSWORD}}"
    volumes:
      - {volume_name}:/var/lib/postgresql/data
    labels:
      dev.axiomphp.service: "postgresql"
"#,
        image = quote_yaml(image),
    )
}

fn reverse_proxy_service_yaml(image: &str, host_port: u16, config_file_name: &str) -> String {
    format!(
        r#"  reverse-proxy:
    image: {image}
    profiles: ["reverse-proxy"]
    depends_on:
      - php
    ports:
      - "127.0.0.1:{host_port}:80"
    volumes:
      - ./{config_file_name}:/etc/nginx/conf.d/default.conf:ro
    labels:
      dev.axiomphp.service: "reverse-proxy"
"#,
        image = quote_yaml(image),
    )
}

fn reverse_proxy_config() -> String {
    r#"server {
    listen 80;
    server_name _;

    location / {
        proxy_pass http://php:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
"#
    .to_string()
}

fn required_image(
    images: &BTreeMap<DockerComposeProfile, String>,
    profile: DockerComposeProfile,
) -> &str {
    images
        .get(&profile)
        .map(String::as_str)
        .unwrap_or_else(|| default_image(profile))
}

fn default_image(profile: DockerComposeProfile) -> &'static str {
    match profile {
        DockerComposeProfile::Mysql => "mysql:8.4",
        DockerComposeProfile::Php => "php:8.4-cli",
        DockerComposeProfile::Postgresql => "postgres:17",
        DockerComposeProfile::ReverseProxy => "nginx:1.27-alpine",
    }
}

fn validate_document_root(document_root: &str) -> AppResult<()> {
    let path = Path::new(document_root);

    if !path.is_absolute() || !path.is_dir() {
        return Err(AppError::Validation(
            "project document root must be an existing absolute directory".to_string(),
        ));
    }

    Ok(())
}

fn quote_yaml(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn volume_safe_project_id(project_id: &str) -> String {
    project_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unpinned_images() {
        let evaluation = image_trust_evaluation(DockerComposeProfile::Php, "php:8.4-cli");

        assert!(!evaluation.allowed);
        assert!(!evaluation.pinned_by_digest);
    }

    #[test]
    fn accepts_sha256_pinned_images() {
        let image = format!("php:8.4-cli@sha256:{}", "a".repeat(64));
        let evaluation = image_trust_evaluation(DockerComposeProfile::Php, &image);

        assert!(evaluation.allowed);
        assert!(evaluation.pinned_by_digest);
    }

    #[test]
    fn reverse_proxy_implies_php_profile() {
        let profiles = normalize_profiles(&[DockerComposeProfile::ReverseProxy]);

        assert!(profiles.contains(&DockerComposeProfile::Php));
        assert!(profiles.contains(&DockerComposeProfile::ReverseProxy));
    }
}

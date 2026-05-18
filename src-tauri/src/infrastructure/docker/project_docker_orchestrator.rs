use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use directories::ProjectDirs;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::docker::docker_project::{
    DockerComposeProfile, DockerDiagnosticCheck, DockerDiagnosticsReport,
    DockerProjectActionResult, DockerProjectComposePlan, DockerProjectContainerStatus,
    DockerProjectLogReadResult, DockerProjectRuntimeStatus, DockerProjectVolumeLifecycleResult,
    DockerProjectVolumePlan,
};
use crate::domain::project::project::Project;
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::docker::docker_compose_generator::{
    mysql_volume_name, normalize_profiles, postgres_volume_name, DockerComposeGenerationInput,
    DockerComposeGenerator, DockerProjectPorts,
};
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::infrastructure::services::adapters::executable_resolver::ExecutableResolver;
use crate::ports::docker_project_orchestrator::DockerProjectOrchestrator;
use crate::ports::process_manager::ProcessManager;
use crate::ports::secure_storage::SecureStorage;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

const DOCKER_TIMEOUT: Duration = Duration::from_secs(8);
const COMPOSE_TIMEOUT: Duration = Duration::from_secs(120);
const DOCKER_OUTPUT_LIMIT_BYTES: usize = 256 * 1024;
const DOCKER_SECRET_NAMESPACE: &str = "docker";
const LABEL_PROJECT_ID: &str = "dev.axiomphp.project-id";

#[derive(Clone)]
pub struct ProjectDockerOrchestrator {
    secure_storage: Arc<dyn SecureStorage>,
    generator: DockerComposeGenerator,
    base_dir: PathBuf,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ProjectDockerPaths {
    root_dir: PathBuf,
    compose_file: PathBuf,
    env_file: PathBuf,
    reverse_proxy_config_file: PathBuf,
}

impl ProjectDockerOrchestrator {
    pub fn new(secure_storage: Arc<dyn SecureStorage>) -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application data directory".to_string())
        })?;

        Ok(Self {
            secure_storage,
            generator: DockerComposeGenerator,
            base_dir: project_dirs
                .data_local_dir()
                .join("docker")
                .join("projects"),
        })
    }

    pub fn with_base_dir(secure_storage: Arc<dyn SecureStorage>, base_dir: PathBuf) -> Self {
        Self {
            secure_storage,
            generator: DockerComposeGenerator,
            base_dir,
        }
    }

    fn paths_for_project(&self, project: &Project) -> AppResult<ProjectDockerPaths> {
        let safe_project_id = validate_project_id(&project.id.0)?;
        let root_dir = self.base_dir.join(safe_project_id);

        Ok(ProjectDockerPaths {
            compose_file: root_dir.join("compose.yaml"),
            env_file: root_dir.join("compose.env"),
            reverse_proxy_config_file: root_dir.join("reverse-proxy.conf"),
            root_dir,
        })
    }

    fn run_docker(
        &self,
        args: impl IntoIterator<Item = impl Into<String>>,
        timeout: Duration,
    ) -> AppResult<ProcessOutput> {
        let docker_path = resolve_docker()?;
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([docker_path.clone()])
                .with_default_timeout(timeout)
                .with_max_output_bytes(DOCKER_OUTPUT_LIMIT_BYTES),
        );

        runner.execute(
            ProcessCommand::new(docker_path.to_string_lossy().into_owned())
                .args(args)
                .timeout(timeout),
        )
    }

    fn docker_available(&self) -> bool {
        self.run_docker(
            ["info", "--format", "{{json .ServerVersion}}"],
            DOCKER_TIMEOUT,
        )
        .map(|output| output.exit_code == Some(0) && !output.timed_out)
        .unwrap_or(false)
    }

    fn write_compose_files(
        &self,
        project: &Project,
        profiles: &[DockerComposeProfile],
    ) -> AppResult<DockerProjectComposePlan> {
        let paths = self.paths_for_project(project)?;
        let normalized_profiles = normalize_profiles(profiles);
        let images = configured_images();
        let compose_project_name = compose_project_name(&project.id.0);
        let ports = deterministic_ports(&project.id.0);

        let generation = self.generator.generate(DockerComposeGenerationInput {
            project_id: project.id.0.clone(),
            document_root: project.document_root.0.clone(),
            compose_project_name: compose_project_name.clone(),
            env_file_name: paths
                .env_file
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("compose.env")
                .to_string(),
            reverse_proxy_config_file_name: paths
                .reverse_proxy_config_file
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("reverse-proxy.conf")
                .to_string(),
            profiles: normalized_profiles.clone(),
            images,
            ports: ports.clone(),
        })?;

        let mut diagnostics = image_diagnostics(&generation.image_trust);
        let can_write_compose = generation.image_trust.iter().all(|trust| trust.allowed);
        let mut compose_file_written = false;

        if can_write_compose {
            fs::create_dir_all(&paths.root_dir).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to create Docker project directory: {error}"
                ))
            })?;
            fs::write(&paths.compose_file, generation.compose_yaml).map_err(|error| {
                AppError::Infrastructure(format!("failed to write Docker Compose file: {error}"))
            })?;
            fs::write(
                &paths.env_file,
                self.compose_env_contents(project, &normalized_profiles, ports)?,
            )
            .map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to write Docker Compose env file: {error}"
                ))
            })?;
            harden_secret_file_permissions(&paths.env_file)?;

            if let Some(config) = generation.reverse_proxy_config {
                fs::write(&paths.reverse_proxy_config_file, config).map_err(|error| {
                    AppError::Infrastructure(format!(
                        "failed to write Docker reverse proxy config: {error}"
                    ))
                })?;
            }

            compose_file_written = true;
            diagnostics.push("Compose files were written with digest-pinned images.".to_string());
        } else {
            diagnostics.push(
                "Compose files were not written because one or more image references are not digest-pinned."
                    .to_string(),
            );
        }

        let reverse_proxy_config_path = normalized_profiles
            .contains(&DockerComposeProfile::ReverseProxy)
            .then(|| {
                paths
                    .reverse_proxy_config_file
                    .to_string_lossy()
                    .into_owned()
            });
        let status_message = if compose_file_written {
            "Project Docker Compose plan generated.".to_string()
        } else {
            "Project Docker Compose plan prepared but blocked by image trust policy.".to_string()
        };

        Ok(DockerProjectComposePlan {
            project_id: project.id.clone(),
            project_name: project.name.clone(),
            compose_project_name,
            compose_file_path: paths.compose_file.to_string_lossy().into_owned(),
            compose_file_written,
            env_file_path: paths.env_file.to_string_lossy().into_owned(),
            reverse_proxy_config_path,
            profiles: normalized_profiles,
            services: generation.services,
            volumes: generation.volumes,
            image_trust: generation.image_trust,
            diagnostics,
            generated_at: Utc::now(),
            status_message,
        })
    }

    fn compose_env_contents(
        &self,
        project: &Project,
        profiles: &[DockerComposeProfile],
        ports: DockerProjectPorts,
    ) -> AppResult<String> {
        let database_name = database_name(&project.id.0);
        let database_user = database_user(&project.id.0);
        let mut env_file = String::new();

        env_file.push_str(&format!("AXIOM_PROJECT_ID={}\n", project.id.0));
        env_file.push_str(&format!("AXIOM_MYSQL_DATABASE={database_name}\n"));
        env_file.push_str(&format!("AXIOM_MYSQL_USER={database_user}\n"));
        env_file.push_str(&format!("AXIOM_POSTGRES_DATABASE={database_name}\n"));
        env_file.push_str(&format!("AXIOM_POSTGRES_USER={database_user}\n"));
        env_file.push_str(&format!(
            "AXIOM_MYSQL_HOST_PORT={}\n",
            ports.mysql_host_port
        ));
        env_file.push_str(&format!(
            "AXIOM_POSTGRES_HOST_PORT={}\n",
            ports.postgres_host_port
        ));
        env_file.push_str(&format!(
            "AXIOM_REVERSE_PROXY_HOST_PORT={}\n",
            ports.reverse_proxy_host_port
        ));

        if profiles.contains(&DockerComposeProfile::Mysql) {
            env_file.push_str(&format!(
                "AXIOM_MYSQL_PASSWORD={}\n",
                self.secret(project, "mysql-password")?
            ));
            env_file.push_str(&format!(
                "AXIOM_MYSQL_ROOT_PASSWORD={}\n",
                self.secret(project, "mysql-root-password")?
            ));
        }

        if profiles.contains(&DockerComposeProfile::Postgresql) {
            env_file.push_str(&format!(
                "AXIOM_POSTGRES_PASSWORD={}\n",
                self.secret(project, "postgres-password")?
            ));
        }

        Ok(env_file)
    }

    fn secret(&self, project: &Project, suffix: &str) -> AppResult<String> {
        let key = format!("{}-{suffix}", project.id.0);

        if let Some(secret) = self
            .secure_storage
            .get_secret(DOCKER_SECRET_NAMESPACE, &key)?
        {
            return Ok(secret);
        }

        let secret = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        self.secure_storage
            .store_secret(DOCKER_SECRET_NAMESPACE, &key, &secret)?;

        Ok(secret)
    }

    fn compose_args(
        &self,
        project: &Project,
        profiles: &[DockerComposeProfile],
    ) -> AppResult<Vec<String>> {
        let paths = self.paths_for_project(project)?;
        if !paths.compose_file.is_file() || !paths.env_file.is_file() {
            return Err(AppError::Validation(
                "project Docker Compose files are not generated yet".to_string(),
            ));
        }

        let mut args = vec![
            "compose".to_string(),
            "--env-file".to_string(),
            paths.env_file.to_string_lossy().into_owned(),
            "--project-name".to_string(),
            compose_project_name(&project.id.0),
            "--file".to_string(),
            paths.compose_file.to_string_lossy().into_owned(),
        ];

        for profile in normalize_profiles(profiles) {
            args.push("--profile".to_string());
            args.push(profile.compose_profile().to_string());
        }

        Ok(args)
    }
}

impl DockerProjectOrchestrator for ProjectDockerOrchestrator {
    fn diagnostics(&self) -> AppResult<DockerDiagnosticsReport> {
        let mut checks = Vec::new();
        let docker_path = resolve_docker();
        let cli_found = docker_path.is_ok();

        checks.push(DockerDiagnosticCheck {
            name: "Docker CLI".to_string(),
            healthy: cli_found,
            status_message: docker_path
                .as_ref()
                .map(|path| format!("Docker CLI resolved at {}.", path.to_string_lossy()))
                .unwrap_or_else(|error| format!("Docker CLI is unavailable: {error}")),
        });

        if !cli_found {
            return Ok(DockerDiagnosticsReport {
                cli_found: false,
                engine_running: false,
                compose_available: false,
                docker_context: None,
                checks,
                status_message: "Docker CLI is not available. Install or open Docker Desktop before running project containers.".to_string(),
            });
        }

        let engine = self.run_docker(
            ["info", "--format", "{{json .ServerVersion}}"],
            DOCKER_TIMEOUT,
        )?;
        let engine_running = engine.exit_code == Some(0) && !engine.timed_out;
        checks.push(DockerDiagnosticCheck {
            name: "Docker engine".to_string(),
            healthy: engine_running,
            status_message: if engine_running {
                "Docker engine responded to diagnostics.".to_string()
            } else {
                format!(
                    "Docker engine is not ready. {} Open Docker Desktop and wait for the engine to finish setup.",
                    summarize_output(&engine)
                )
            },
        });

        let compose = self.run_docker(["compose", "version", "--short"], DOCKER_TIMEOUT)?;
        let compose_available = compose.exit_code == Some(0) && !compose.timed_out;
        checks.push(DockerDiagnosticCheck {
            name: "Docker Compose".to_string(),
            healthy: compose_available,
            status_message: if compose_available {
                format!("Docker Compose is available: {}.", compose.stdout.trim())
            } else {
                format!(
                    "Docker Compose diagnostics failed. {}",
                    summarize_output(&compose)
                )
            },
        });

        let context = self.run_docker(["context", "show"], DOCKER_TIMEOUT).ok();
        let docker_context = context
            .as_ref()
            .filter(|output| output.exit_code == Some(0) && !output.timed_out)
            .map(|output| output.stdout.trim().to_string())
            .filter(|value| !value.is_empty());

        checks.push(DockerDiagnosticCheck {
            name: "Docker context".to_string(),
            healthy: docker_context.is_some(),
            status_message: docker_context
                .as_ref()
                .map(|context| format!("Docker context `{context}` is selected."))
                .unwrap_or_else(|| "Docker context could not be read.".to_string()),
        });

        let status_message = if engine_running && compose_available {
            "Docker Desktop runtime diagnostics are ready for project orchestration.".to_string()
        } else {
            "Docker diagnostics are incomplete after engine reset; open Docker Desktop and retry when setup completes.".to_string()
        };

        Ok(DockerDiagnosticsReport {
            cli_found,
            engine_running,
            compose_available,
            docker_context,
            checks,
            status_message,
        })
    }

    fn generate_compose_plan(
        &self,
        project: &Project,
        profiles: &[DockerComposeProfile],
    ) -> AppResult<DockerProjectComposePlan> {
        self.write_compose_files(project, profiles)
    }

    fn get_runtime_status(&self, project: &Project) -> AppResult<DockerProjectRuntimeStatus> {
        let paths = self.paths_for_project(project)?;
        let engine_running = self.docker_available();
        let compose_file_exists = paths.compose_file.is_file() && paths.env_file.is_file();
        let mut diagnostics = Vec::new();
        let mut containers = Vec::new();

        if engine_running && compose_file_exists {
            let mut args = self.compose_args(project, &[])?;
            args.extend(["ps".to_string(), "--format".to_string(), "json".to_string()]);
            let output = self.run_docker(args, DOCKER_TIMEOUT)?;

            if output.exit_code == Some(0) && !output.timed_out {
                containers = parse_compose_containers(&output.stdout);
            } else {
                diagnostics.push(format!(
                    "Compose status failed safely. {}",
                    summarize_output(&output)
                ));
            }
        } else if !engine_running {
            diagnostics.push("Docker engine is not ready.".to_string());
        } else {
            diagnostics.push("Compose files are not generated yet.".to_string());
        }

        let volumes = self.project_volumes(project)?;
        let status_message = if engine_running {
            "Project Docker runtime status checked.".to_string()
        } else {
            "Docker engine is not running.".to_string()
        };

        Ok(DockerProjectRuntimeStatus {
            project_id: project.id.clone(),
            compose_project_name: compose_project_name(&project.id.0),
            engine_running,
            compose_file_exists,
            containers,
            volumes,
            diagnostics,
            checked_at: Utc::now(),
            status_message,
        })
    }

    fn start_project(
        &self,
        project: &Project,
        profiles: &[DockerComposeProfile],
    ) -> AppResult<DockerProjectActionResult> {
        let plan = self.write_compose_files(project, profiles)?;
        ensure_plan_is_startable(&plan)?;
        let mut args = self.compose_args(project, &plan.profiles)?;
        args.extend([
            "up".to_string(),
            "--detach".to_string(),
            "--remove-orphans".to_string(),
        ]);
        let output = self.run_docker(args, COMPOSE_TIMEOUT)?;
        ensure_successful_output("docker compose up", &output)?;
        let runtime = self.get_runtime_status(project)?;

        Ok(DockerProjectActionResult {
            project_id: project.id.clone(),
            action: "start".to_string(),
            plan,
            runtime,
            status_message: "Project Docker services started.".to_string(),
        })
    }

    fn stop_project(&self, project: &Project) -> AppResult<DockerProjectActionResult> {
        let plan = self.write_compose_files(project, &[])?;
        let mut args = self.compose_args(project, &[])?;
        args.extend(["down".to_string(), "--remove-orphans".to_string()]);
        let output = self.run_docker(args, COMPOSE_TIMEOUT)?;
        ensure_successful_output("docker compose down", &output)?;
        let runtime = self.get_runtime_status(project)?;

        Ok(DockerProjectActionResult {
            project_id: project.id.clone(),
            action: "stop".to_string(),
            plan,
            runtime,
            status_message: "Project Docker services stopped.".to_string(),
        })
    }

    fn restart_project(
        &self,
        project: &Project,
        profiles: &[DockerComposeProfile],
    ) -> AppResult<DockerProjectActionResult> {
        let _ = self.stop_project(project);
        let mut result = self.start_project(project, profiles)?;
        result.action = "restart".to_string();
        result.status_message = "Project Docker services restarted.".to_string();
        Ok(result)
    }

    fn ensure_project_volumes(
        &self,
        project: &Project,
        profiles: &[DockerComposeProfile],
    ) -> AppResult<DockerProjectVolumeLifecycleResult> {
        let volumes = planned_volumes(project, profiles);

        for volume in &volumes {
            let output = self.run_docker(
                [
                    "volume".to_string(),
                    "create".to_string(),
                    "--label".to_string(),
                    format!("{LABEL_PROJECT_ID}={}", project.id.0),
                    volume.name.clone(),
                ],
                DOCKER_TIMEOUT,
            )?;
            ensure_successful_output("docker volume create", &output)?;
        }

        Ok(DockerProjectVolumeLifecycleResult {
            project_id: project.id.clone(),
            volumes: volumes
                .into_iter()
                .map(|mut volume| {
                    volume.created = true;
                    volume
                })
                .collect(),
            status_message: "Project Docker volumes are present.".to_string(),
        })
    }

    fn remove_project_volumes(
        &self,
        project: &Project,
    ) -> AppResult<DockerProjectVolumeLifecycleResult> {
        let volumes = self.project_volumes(project)?;

        for volume in &volumes {
            let output = self.run_docker(
                [
                    "volume".to_string(),
                    "rm".to_string(),
                    "--force".to_string(),
                    volume.name.clone(),
                ],
                DOCKER_TIMEOUT,
            )?;
            ensure_successful_output("docker volume rm", &output)?;
        }

        Ok(DockerProjectVolumeLifecycleResult {
            project_id: project.id.clone(),
            volumes: volumes
                .into_iter()
                .map(|mut volume| {
                    volume.created = false;
                    volume
                })
                .collect(),
            status_message: "Project Docker volumes were removed.".to_string(),
        })
    }

    fn read_project_logs(
        &self,
        project: &Project,
        tail_lines: u16,
    ) -> AppResult<DockerProjectLogReadResult> {
        let safe_tail = tail_lines.clamp(10, 1_000);
        let mut args = self.compose_args(project, &[])?;
        args.extend([
            "logs".to_string(),
            "--no-color".to_string(),
            "--tail".to_string(),
            safe_tail.to_string(),
        ]);
        let output = self.run_docker(args, DOCKER_TIMEOUT)?;
        ensure_successful_output("docker compose logs", &output)?;
        let lines = output
            .stdout
            .lines()
            .map(sanitize_log_line)
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>();

        Ok(DockerProjectLogReadResult {
            project_id: project.id.clone(),
            lines,
            truncated: output.stdout_truncated || output.stderr_truncated,
            status_message: "Project Docker logs read through the backend boundary.".to_string(),
        })
    }
}

impl ProjectDockerOrchestrator {
    fn project_volumes(&self, project: &Project) -> AppResult<Vec<DockerProjectVolumePlan>> {
        if !self.docker_available() {
            return Ok(Vec::new());
        }

        let output = self.run_docker(
            [
                "volume".to_string(),
                "ls".to_string(),
                "--filter".to_string(),
                format!("label={LABEL_PROJECT_ID}={}", project.id.0),
                "--format".to_string(),
                "{{.Name}}".to_string(),
            ],
            DOCKER_TIMEOUT,
        )?;

        if output.exit_code != Some(0) || output.timed_out {
            return Ok(Vec::new());
        }

        Ok(output
            .stdout
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|name| DockerProjectVolumePlan {
                name: name.to_string(),
                service_name: volume_service_name(name),
                mount_path: volume_mount_path(name),
                created: true,
            })
            .collect())
    }
}

fn configured_images() -> BTreeMap<DockerComposeProfile, String> {
    [
        (DockerComposeProfile::Php, "AXIOM_DOCKER_PHP_IMAGE"),
        (DockerComposeProfile::Mysql, "AXIOM_DOCKER_MYSQL_IMAGE"),
        (
            DockerComposeProfile::Postgresql,
            "AXIOM_DOCKER_POSTGRES_IMAGE",
        ),
        (
            DockerComposeProfile::ReverseProxy,
            "AXIOM_DOCKER_REVERSE_PROXY_IMAGE",
        ),
    ]
    .into_iter()
    .filter_map(|(profile, key)| {
        env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(|value| (profile, value))
    })
    .collect()
}

fn planned_volumes(
    project: &Project,
    profiles: &[DockerComposeProfile],
) -> Vec<DockerProjectVolumePlan> {
    let profiles = normalize_profiles(profiles);
    let mut volumes = Vec::new();

    if profiles.contains(&DockerComposeProfile::Mysql) {
        volumes.push(DockerProjectVolumePlan {
            name: mysql_volume_name(&project.id.0),
            service_name: "mysql".to_string(),
            mount_path: "/var/lib/mysql".to_string(),
            created: false,
        });
    }

    if profiles.contains(&DockerComposeProfile::Postgresql) {
        volumes.push(DockerProjectVolumePlan {
            name: postgres_volume_name(&project.id.0),
            service_name: "postgres".to_string(),
            mount_path: "/var/lib/postgresql/data".to_string(),
            created: false,
        });
    }

    volumes
}

fn ensure_plan_is_startable(plan: &DockerProjectComposePlan) -> AppResult<()> {
    if !plan.compose_file_written {
        return Err(AppError::PermissionDenied(
            "Docker Compose start is blocked until all selected images are digest-pinned"
                .to_string(),
        ));
    }

    let blocked = plan
        .image_trust
        .iter()
        .filter(|trust| !trust.allowed)
        .map(|trust| format!("{}: {}", trust.profile.as_key(), trust.image))
        .collect::<Vec<_>>();

    if !blocked.is_empty() {
        return Err(AppError::PermissionDenied(format!(
            "Docker image trust policy blocked unpinned images: {}",
            blocked.join(", ")
        )));
    }

    Ok(())
}

fn image_diagnostics(
    image_trust: &[crate::domain::docker::docker_project::DockerImageTrustEvaluation],
) -> Vec<String> {
    image_trust
        .iter()
        .map(|trust| {
            format!(
                "{} image `{}`: {}",
                trust.profile.as_key(),
                trust.image,
                trust.status_message
            )
        })
        .collect()
}

fn deterministic_ports(project_id: &str) -> DockerProjectPorts {
    let hash = project_id.bytes().fold(0_u16, |acc, byte| {
        acc.wrapping_mul(31).wrapping_add(byte as u16)
    }) % 1_000;

    DockerProjectPorts {
        mysql_host_port: 33_060 + hash,
        postgres_host_port: 54_320 + hash,
        reverse_proxy_host_port: 18_080 + hash,
    }
}

fn compose_project_name(project_id: &str) -> String {
    format!("axiom_{}", project_id.replace('-', "_"))
}

fn database_name(project_id: &str) -> String {
    format!("ax_{}", project_id.replace('-', "_"))
        .chars()
        .take(63)
        .collect()
}

fn database_user(project_id: &str) -> String {
    format!("ax_{}", project_id.replace('-', "_"))
        .chars()
        .take(31)
        .collect()
}

fn resolve_docker() -> AppResult<PathBuf> {
    ExecutableResolver::from_env()
        .resolve("docker")
        .ok_or_else(|| {
            AppError::NotFound("Docker CLI executable was not found on PATH".to_string())
        })
}

fn parse_compose_containers(contents: &str) -> Vec<DockerProjectContainerStatus> {
    let trimmed = contents.trim();

    if trimmed.is_empty() {
        return Vec::new();
    }

    if let Ok(Value::Array(items)) = serde_json::from_str::<Value>(trimmed) {
        return items.iter().filter_map(container_from_json).collect();
    }

    trimmed
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|value| container_from_json(&value))
        .collect()
}

fn container_from_json(value: &Value) -> Option<DockerProjectContainerStatus> {
    let name = value
        .get("Name")
        .or_else(|| value.get("Name"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let service_name = value
        .get("Service")
        .or_else(|| value.get("ServiceName"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let state = value
        .get("State")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let status = value
        .get("Status")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    Some(DockerProjectContainerStatus {
        name,
        service_name,
        state,
        status,
    })
}

fn ensure_successful_output(label: &str, output: &ProcessOutput) -> AppResult<()> {
    if output.timed_out {
        return Err(AppError::Infrastructure(format!("{label} timed out")));
    }

    if output.exit_code == Some(0) {
        return Ok(());
    }

    Err(AppError::Infrastructure(format!(
        "{label} failed. {}",
        summarize_output(output)
    )))
}

fn summarize_output(output: &ProcessOutput) -> String {
    let text = if output.stderr.trim().is_empty() {
        output.stdout.trim()
    } else {
        output.stderr.trim()
    };

    if text.is_empty() {
        "No Docker diagnostic output was returned.".to_string()
    } else {
        text.lines().rev().take(4).collect::<Vec<_>>().join("\n")
    }
}

fn sanitize_log_line(line: &str) -> String {
    let lowercase = line.to_ascii_lowercase();
    let sensitive_markers = [
        "access_key",
        "api_key",
        "apikey",
        "auth",
        "credential",
        "password",
        "private_key",
        "secret",
        "token",
    ];

    if sensitive_markers
        .iter()
        .any(|marker| lowercase.contains(marker))
    {
        return "[redacted sensitive docker log line]".to_string();
    }

    line.to_string()
}

fn volume_service_name(volume_name: &str) -> String {
    if volume_name.contains("_mysql_") {
        "mysql".to_string()
    } else if volume_name.contains("_postgres_") {
        "postgres".to_string()
    } else {
        "unknown".to_string()
    }
}

fn volume_mount_path(volume_name: &str) -> String {
    if volume_name.contains("_mysql_") {
        "/var/lib/mysql".to_string()
    } else if volume_name.contains("_postgres_") {
        "/var/lib/postgresql/data".to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg(unix)]
fn harden_secret_file_permissions(path: &Path) -> AppResult<()> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, permissions).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to harden Docker env file permissions: {error}"
        ))
    })
}

#[cfg(not(unix))]
fn harden_secret_file_permissions(_path: &Path) -> AppResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_line_delimited_compose_ps_json() {
        let containers = parse_compose_containers(
            r#"{"Name":"axiom-app-php-1","Service":"php","State":"running","Status":"Up"}
{"Name":"axiom-app-mysql-1","Service":"mysql","State":"exited","Status":"Exited"}"#,
        );

        assert_eq!(containers.len(), 2);
        assert_eq!(containers[0].service_name, "php");
        assert_eq!(containers[1].state, "exited");
    }

    #[test]
    fn log_sanitizer_redacts_secret_lines() {
        assert_eq!(
            sanitize_log_line("MYSQL_PASSWORD=secret"),
            "[redacted sensitive docker log line]"
        );
    }
}

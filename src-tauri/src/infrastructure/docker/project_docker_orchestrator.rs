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
    DockerComposeProfile, DockerDiagnosticCheck, DockerDiagnosticsReport, DockerImagePinResolution,
    DockerImagePinResolutionReport, DockerImageTrustEvaluation, DockerProjectActionResult,
    DockerProjectComposePlan, DockerProjectComposeRequest, DockerProjectContainerStatus,
    DockerProjectImageOverride, DockerProjectLogReadResult, DockerProjectResourceLimits,
    DockerProjectRuntimeStatus, DockerProjectVolumeLifecycleResult, DockerProjectVolumePlan,
    DockerRegistryTrustMetadata,
};
use crate::domain::project::project::Project;
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::docker::docker_compose_generator::{
    default_image, image_is_digest_pinned, mysql_volume_name, normalize_profiles,
    postgres_volume_name, redis_volume_name, DockerComposeGenerationInput, DockerComposeGenerator,
    DockerProjectPorts,
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
const IMAGE_INSPECT_TIMEOUT: Duration = Duration::from_secs(90);
const DOCKER_SECRET_NAMESPACE: &str = "docker";
const LABEL_PROJECT_ID: &str = "dev.axiomphp.project-id";
const DEFAULT_ALLOWED_REGISTRIES: &[&str] = &["docker.io", "registry-1.docker.io"];

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
        let mut command_args = Vec::new();

        if let Some(context) = configured_docker_context()? {
            command_args.push("--context".to_string());
            command_args.push(context);
        }

        command_args.extend(args.into_iter().map(Into::into));
        let runner = CommandRunner::new(
            CommandPolicy::deny_all()
                .allow_program_paths([docker_path.clone()])
                .with_default_timeout(timeout)
                .with_max_output_bytes(DOCKER_OUTPUT_LIMIT_BYTES),
        );

        runner.execute(
            ProcessCommand::new(docker_path.to_string_lossy().into_owned())
                .args(command_args)
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
        request: &DockerProjectComposeRequest,
    ) -> AppResult<DockerProjectComposePlan> {
        let paths = self.paths_for_project(project)?;
        validate_resource_limits(request.resource_limits)?;
        let normalized_profiles = normalize_profiles(&request.profiles);
        let images = configured_images(&request.image_overrides)?;
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
            images: images.clone(),
            ports: ports.clone(),
            resource_limits: request.resource_limits,
        })?;

        let image_trust = self.evaluate_image_trust(&normalized_profiles, &images);
        let mut diagnostics = image_diagnostics(&image_trust);
        let can_write_compose = image_trust.iter().all(|trust| trust.allowed);
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
            image_trust,
            resource_limits: request.resource_limits,
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

    fn resolve_image_pins(
        &self,
        request: &DockerProjectComposeRequest,
    ) -> AppResult<DockerImagePinResolutionReport> {
        validate_project_id(&request.project_id.0)?;
        let profiles = normalize_profiles(&request.profiles);
        let images = configured_images(&request.image_overrides)?;
        let mut diagnostics = Vec::new();
        let mut resolutions = Vec::new();

        for profile in profiles {
            let source_image = required_image_for_profile(&images, profile).to_string();
            match self.inspect_registry_metadata(&source_image) {
                Ok(metadata) => {
                    let pinned_image = pin_image_reference(&source_image, &metadata.digest);
                    diagnostics.push(format!(
                        "{} image resolved to {}.",
                        profile.as_key(),
                        metadata.digest
                    ));
                    resolutions.push(DockerImagePinResolution {
                        profile,
                        source_image,
                        pinned_image,
                        metadata,
                        status_message: "Image digest resolved from registry metadata.".to_string(),
                    });
                }
                Err(error) => diagnostics.push(format!(
                    "{} image `{}` could not be resolved: {error}",
                    profile.as_key(),
                    source_image
                )),
            }
        }

        let status_message = if resolutions.is_empty() {
            "No Docker image digests were resolved.".to_string()
        } else {
            format!("Resolved {} Docker image digest(s).", resolutions.len())
        };

        Ok(DockerImagePinResolutionReport {
            resolutions,
            diagnostics,
            status_message,
        })
    }

    fn generate_compose_plan(
        &self,
        project: &Project,
        request: &DockerProjectComposeRequest,
    ) -> AppResult<DockerProjectComposePlan> {
        self.write_compose_files(project, request)
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
        request: &DockerProjectComposeRequest,
    ) -> AppResult<DockerProjectActionResult> {
        let plan = self.write_compose_files(project, request)?;
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
        let request = default_project_request(project);
        let plan = self.write_compose_files(project, &request)?;
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
        request: &DockerProjectComposeRequest,
    ) -> AppResult<DockerProjectActionResult> {
        let _ = self.stop_project(project);
        let mut result = self.start_project(project, request)?;
        result.action = "restart".to_string();
        result.status_message = "Project Docker services restarted.".to_string();
        Ok(result)
    }

    fn ensure_project_volumes(
        &self,
        project: &Project,
        request: &DockerProjectComposeRequest,
    ) -> AppResult<DockerProjectVolumeLifecycleResult> {
        let volumes = planned_volumes(project, &request.profiles);

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

    fn evaluate_image_trust(
        &self,
        profiles: &[DockerComposeProfile],
        images: &BTreeMap<DockerComposeProfile, String>,
    ) -> Vec<DockerImageTrustEvaluation> {
        profiles
            .iter()
            .map(|profile| {
                let image = required_image_for_profile(images, *profile);
                let pinned_by_digest = image_is_digest_pinned(image);

                match self.inspect_registry_metadata(image) {
                    Ok(metadata) => {
                        let digest_matches = image_digest(image)
                            .map(|digest| digest == metadata.digest)
                            .unwrap_or(false);
                        let allowed =
                            pinned_by_digest && digest_matches && metadata.allowed_registry;
                        let status_message = if allowed {
                            "Image reference is digest-pinned and verified against registry metadata."
                                .to_string()
                        } else if !pinned_by_digest {
                            "Image reference is resolved from registry metadata but must be pinned before start."
                                .to_string()
                        } else if !metadata.allowed_registry {
                            "Image registry is not allowed by Docker trust policy.".to_string()
                        } else {
                            "Image digest did not match registry metadata.".to_string()
                        };

                        DockerImageTrustEvaluation {
                            profile: *profile,
                            image: image.to_string(),
                            pinned_by_digest,
                            registry_allowed: metadata.allowed_registry,
                            metadata_verified: digest_matches,
                            allowed,
                            metadata: Some(metadata),
                            status_message,
                        }
                    }
                    Err(error) => DockerImageTrustEvaluation {
                        profile: *profile,
                        image: image.to_string(),
                        pinned_by_digest,
                        registry_allowed: image_registry_allowed(image),
                        metadata_verified: false,
                        allowed: false,
                        metadata: None,
                        status_message: format!(
                            "Image registry metadata verification failed: {error}"
                        ),
                    },
                }
            })
            .collect()
    }

    fn inspect_registry_metadata(&self, image: &str) -> AppResult<DockerRegistryTrustMetadata> {
        validate_image_reference(image)?;
        let output = self.run_docker(
            [
                "buildx".to_string(),
                "imagetools".to_string(),
                "inspect".to_string(),
                image.to_string(),
                "--format".to_string(),
                "{\"digest\":\"{{.Manifest.Digest}}\",\"mediaType\":\"{{.Manifest.MediaType}}\",\"platformCount\":{{len .Manifest.Manifests}}}".to_string(),
            ],
            IMAGE_INSPECT_TIMEOUT,
        )?;

        if output.exit_code == Some(0) && !output.timed_out {
            return parse_registry_metadata(image, &output.stdout);
        }

        let fallback = self.run_docker(
            [
                "buildx".to_string(),
                "imagetools".to_string(),
                "inspect".to_string(),
                image.to_string(),
            ],
            IMAGE_INSPECT_TIMEOUT,
        )?;
        ensure_successful_output("docker buildx imagetools inspect", &fallback)?;

        parse_text_registry_metadata(image, &fallback.stdout)
    }
}

fn configured_images(
    image_overrides: &[DockerProjectImageOverride],
) -> AppResult<BTreeMap<DockerComposeProfile, String>> {
    let mut images = [
        (DockerComposeProfile::Mailpit, "AXIOM_DOCKER_MAILPIT_IMAGE"),
        (DockerComposeProfile::Php, "AXIOM_DOCKER_PHP_IMAGE"),
        (DockerComposeProfile::Mysql, "AXIOM_DOCKER_MYSQL_IMAGE"),
        (
            DockerComposeProfile::Postgresql,
            "AXIOM_DOCKER_POSTGRES_IMAGE",
        ),
        (DockerComposeProfile::Redis, "AXIOM_DOCKER_REDIS_IMAGE"),
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
    .collect::<BTreeMap<_, _>>();

    for image_override in image_overrides {
        validate_image_reference(&image_override.image)?;
        images.insert(image_override.profile, image_override.image.clone());
    }

    Ok(images)
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

    if profiles.contains(&DockerComposeProfile::Redis) {
        volumes.push(DockerProjectVolumePlan {
            name: redis_volume_name(&project.id.0),
            service_name: "redis".to_string(),
            mount_path: "/data".to_string(),
            created: false,
        });
    }

    volumes
}

fn ensure_plan_is_startable(plan: &DockerProjectComposePlan) -> AppResult<()> {
    if !plan.compose_file_written {
        return Err(AppError::PermissionDenied(
            "Docker Compose start is blocked until all selected images are digest-pinned and registry metadata is verified"
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
            "Docker image trust policy blocked images: {}",
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
        redis_host_port: 63_790 + hash,
        mailpit_smtp_host_port: 10_250 + hash,
        mailpit_web_host_port: 18_250 + hash,
        postgres_host_port: 54_320 + hash,
        reverse_proxy_host_port: 18_080 + hash,
    }
}

fn default_project_request(project: &Project) -> DockerProjectComposeRequest {
    DockerProjectComposeRequest {
        project_id: project.id.clone(),
        profiles: Vec::new(),
        image_overrides: Vec::new(),
        resource_limits: DockerProjectResourceLimits::default(),
    }
}

fn required_image_for_profile(
    images: &BTreeMap<DockerComposeProfile, String>,
    profile: DockerComposeProfile,
) -> &str {
    images
        .get(&profile)
        .map(String::as_str)
        .unwrap_or_else(|| default_image(profile))
}

fn validate_resource_limits(resource_limits: DockerProjectResourceLimits) -> AppResult<()> {
    if let Some(cpus) = resource_limits.cpus {
        if !cpus.is_finite() || !(0.1..=16.0).contains(&cpus) {
            return Err(AppError::Validation(
                "Docker CPU limit must be between 0.10 and 16.00".to_string(),
            ));
        }
    }

    if let Some(memory_mb) = resource_limits.memory_mb {
        if !(128..=65_536).contains(&memory_mb) {
            return Err(AppError::Validation(
                "Docker memory limit must be between 128 MB and 65536 MB".to_string(),
            ));
        }
    }

    Ok(())
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

fn validate_image_reference(image: &str) -> AppResult<()> {
    let trimmed = image.trim();

    if trimmed.is_empty() || trimmed != image || image.len() > 240 {
        return Err(AppError::Validation(
            "Docker image reference is invalid".to_string(),
        ));
    }

    if image.starts_with('-')
        || image.contains("..")
        || image.contains("://")
        || !image.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '/' | '_' | '-' | ':' | '@')
        })
    {
        return Err(AppError::Validation(
            "Docker image reference contains unsupported characters".to_string(),
        ));
    }

    Ok(())
}

fn parse_registry_metadata(image: &str, contents: &str) -> AppResult<DockerRegistryTrustMetadata> {
    let value = serde_json::from_str::<Value>(contents.trim()).map_err(|error| {
        AppError::Infrastructure(format!(
            "Docker registry metadata was not valid JSON: {error}"
        ))
    })?;
    let manifest = value
        .get("Manifest")
        .or_else(|| value.get("manifest"))
        .unwrap_or(&value);
    let digest = json_string(manifest, &["Digest", "digest"])
        .or_else(|| json_string(&value, &["Digest", "digest"]))
        .ok_or_else(|| {
            AppError::Infrastructure(
                "Docker registry metadata did not include a digest".to_string(),
            )
        })?;
    let media_type = json_string(manifest, &["MediaType", "mediaType"])
        .or_else(|| json_string(&value, &["MediaType", "mediaType"]))
        .unwrap_or_else(|| "unknown".to_string());
    let platform_count = json_usize(manifest, &["PlatformCount", "platformCount"])
        .or_else(|| json_usize(&value, &["PlatformCount", "platformCount"]))
        .or_else(|| json_array_len(manifest, &["Manifests", "manifests"]))
        .or_else(|| json_array_len(&value, &["Manifests", "manifests"]))
        .unwrap_or(1);

    Ok(metadata_from_parts(
        image,
        digest,
        media_type,
        platform_count,
    ))
}

fn parse_text_registry_metadata(
    image: &str,
    contents: &str,
) -> AppResult<DockerRegistryTrustMetadata> {
    let digest = contents
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("Digest:")
                .map(str::trim)
                .map(str::to_string)
        })
        .ok_or_else(|| {
            AppError::Infrastructure(
                "Docker registry metadata did not include a digest".to_string(),
            )
        })?;
    let media_type = contents
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("MediaType:")
                .map(str::trim)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unknown".to_string());
    let platform_count = contents
        .lines()
        .filter(|line| line.trim_start().starts_with("Platform:"))
        .count()
        .max(1);

    Ok(metadata_from_parts(
        image,
        digest,
        media_type,
        platform_count,
    ))
}

fn metadata_from_parts(
    image: &str,
    digest: String,
    media_type: String,
    platform_count: usize,
) -> DockerRegistryTrustMetadata {
    let image_reference = parse_image_reference(image);
    let allowed_registry = allowed_registries()
        .iter()
        .any(|registry| registry == &image_reference.registry);
    let status_message = if allowed_registry {
        "Registry metadata was resolved from an allowed registry.".to_string()
    } else {
        format!(
            "Registry `{}` is not in the Docker trust allowlist.",
            image_reference.registry
        )
    };

    DockerRegistryTrustMetadata {
        registry: image_reference.registry,
        repository: image_reference.repository,
        reference: image_reference.reference,
        digest: normalize_digest(&digest),
        media_type,
        platform_count,
        allowed_registry,
        status_message,
    }
}

fn json_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::to_string)
}

fn json_array_len(value: &Value, keys: &[&str]) -> Option<usize> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_array))
        .map(Vec::len)
}

fn json_usize(value: &Value, keys: &[&str]) -> Option<usize> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_u64))
        .and_then(|value| usize::try_from(value).ok())
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ParsedImageReference {
    registry: String,
    repository: String,
    reference: String,
}

fn parse_image_reference(image: &str) -> ParsedImageReference {
    let without_digest = image
        .split_once('@')
        .map(|(left, _right)| left)
        .unwrap_or(image);
    let digest_reference = image
        .split_once('@')
        .map(|(_left, right)| right.to_string());
    let parts = without_digest.split('/').collect::<Vec<_>>();
    let first = parts.first().copied().unwrap_or_default();
    let has_explicit_registry =
        parts.len() > 1 && (first.contains('.') || first.contains(':') || first == "localhost");
    let (registry, repository_with_tag) = if has_explicit_registry {
        (
            first.to_string(),
            parts.get(1..).unwrap_or_default().join("/"),
        )
    } else {
        ("docker.io".to_string(), without_digest.to_string())
    };
    let repository_with_library = if registry == "docker.io" && !repository_with_tag.contains('/') {
        format!("library/{repository_with_tag}")
    } else {
        repository_with_tag
    };
    let (repository, tag) = split_repository_tag(&repository_with_library);
    let reference = digest_reference
        .or_else(|| tag.map(str::to_string))
        .unwrap_or_else(|| "latest".to_string());

    ParsedImageReference {
        registry,
        repository,
        reference,
    }
}

fn split_repository_tag(repository_with_tag: &str) -> (String, Option<&str>) {
    let last_slash = repository_with_tag.rfind('/');
    let last_colon = repository_with_tag.rfind(':');

    if let Some(colon_index) = last_colon {
        if last_slash.is_none_or(|slash_index| colon_index > slash_index) {
            return (
                repository_with_tag[..colon_index].to_string(),
                Some(&repository_with_tag[colon_index + 1..]),
            );
        }
    }

    (repository_with_tag.to_string(), None)
}

fn pin_image_reference(image: &str, digest: &str) -> String {
    if image_is_digest_pinned(image) {
        return image.to_string();
    }

    format!("{}@{}", image, normalize_digest(digest))
}

fn image_digest(image: &str) -> Option<String> {
    image
        .split_once('@')
        .map(|(_left, digest)| normalize_digest(digest))
}

fn normalize_digest(digest: &str) -> String {
    let trimmed = digest.trim();

    if trimmed.starts_with("sha256:") {
        trimmed.to_string()
    } else {
        format!("sha256:{trimmed}")
    }
}

fn image_registry_allowed(image: &str) -> bool {
    let image_reference = parse_image_reference(image);

    allowed_registries()
        .iter()
        .any(|registry| registry == &image_reference.registry)
}

fn allowed_registries() -> Vec<String> {
    env::var("AXIOM_DOCKER_ALLOWED_REGISTRIES")
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|registries| !registries.is_empty())
        .unwrap_or_else(|| {
            DEFAULT_ALLOWED_REGISTRIES
                .iter()
                .map(|registry| (*registry).to_string())
                .collect()
        })
}

fn configured_docker_context() -> AppResult<Option<String>> {
    let Some(context) = env::var("AXIOM_DOCKER_CONTEXT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    if context
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
        return Ok(Some(context));
    }

    Err(AppError::Validation(
        "Docker context name contains unsupported characters".to_string(),
    ))
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
    } else if volume_name.contains("_redis_") {
        "redis".to_string()
    } else {
        "unknown".to_string()
    }
}

fn volume_mount_path(volume_name: &str) -> String {
    if volume_name.contains("_mysql_") {
        "/var/lib/mysql".to_string()
    } else if volume_name.contains("_postgres_") {
        "/var/lib/postgresql/data".to_string()
    } else if volume_name.contains("_redis_") {
        "/data".to_string()
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

    #[test]
    fn parses_docker_hub_shorthand_image_references() {
        let reference = parse_image_reference("redis:7-alpine");

        assert_eq!(reference.registry, "docker.io");
        assert_eq!(reference.repository, "library/redis");
        assert_eq!(reference.reference, "7-alpine");
    }

    #[test]
    fn pins_image_references_without_losing_tags() {
        let digest = format!("sha256:{}", "a".repeat(64));
        let pinned = pin_image_reference("php:8.4-cli", &digest);

        assert_eq!(pinned, format!("php:8.4-cli@sha256:{}", "a".repeat(64)));
    }

    #[test]
    fn rejects_unsafe_resource_limits() {
        assert!(validate_resource_limits(DockerProjectResourceLimits {
            cpus: Some(0.05),
            memory_mb: None,
        })
        .is_err());
        assert!(validate_resource_limits(DockerProjectResourceLimits {
            cpus: None,
            memory_mb: Some(64),
        })
        .is_err());
    }

    #[test]
    fn recognizes_redis_volume_metadata() {
        assert_eq!(volume_service_name("axiom_project_redis_data"), "redis");
        assert_eq!(volume_mount_path("axiom_project_redis_data"), "/data");
    }
}

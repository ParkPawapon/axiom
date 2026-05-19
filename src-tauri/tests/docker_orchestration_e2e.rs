#![cfg(unix)]

mod support;

use std::fs;
use std::sync::Arc;

use axiomphp_lib::domain::docker::docker_project::{
    DockerComposeProfile, DockerProjectComposeRequest, DockerProjectImageOverride,
    DockerProjectResourceLimits,
};
use axiomphp_lib::domain::project::project_id::ProjectId;
use axiomphp_lib::infrastructure::docker::project_docker_orchestrator::ProjectDockerOrchestrator;
use axiomphp_lib::ports::docker_project_orchestrator::DockerProjectOrchestrator as _;
use support::{env_lock, project, MemorySecureStorage, PathGuard, TestEnvironment};

#[test]
fn project_docker_orchestration_runs_through_allowlisted_cli_boundary() {
    let _env = env_lock();
    let test_env = TestEnvironment::new("docker-orchestration-e2e");
    let bin_dir = test_env.path("bin");
    let document_root = test_env.path("document-root");
    let compose_root = test_env.path("compose-root");
    fs::create_dir_all(&bin_dir).expect("fake bin");
    fs::create_dir_all(&document_root).expect("document root");
    fs::write(document_root.join("index.php"), "<?php echo 'ok';\n").expect("project file");
    let docker_log = test_env.path("docker.log");
    let docker_log_script_path = docker_log.to_string_lossy().into_owned();
    let docker_script = r#"#!/bin/sh
printf '%s\n' "$*" >> '__LOG__'
if [ "$1" = "info" ]; then
  printf '"24.0.0"\n'
  exit 0
fi
if [ "$1" = "context" ] && [ "$2" = "show" ]; then
  printf 'default\n'
  exit 0
fi
if [ "$1" = "buildx" ] && [ "$2" = "imagetools" ] && [ "$3" = "inspect" ]; then
  image="$4"
  digest="${image##*@sha256:}"
  printf '{"digest":"sha256:%s","mediaType":"application/vnd.oci.image.index.v1+json","platformCount":2}\n' "$digest"
  exit 0
fi
if [ "$1" = "compose" ] && [ "$2" = "version" ]; then
  printf '2.24.0\n'
  exit 0
fi
if [ "$1" = "compose" ]; then
  for arg in "$@"; do
    if [ "$arg" = "ps" ]; then
      printf '[{"Name":"axiom-php-1","Service":"php","State":"running","Status":"Up"}]\n'
      exit 0
    fi
    if [ "$arg" = "logs" ]; then
      printf 'php ready\npassword=do-not-leak\n'
      exit 0
    fi
  done
  exit 0
fi
if [ "$1" = "volume" ] && [ "$2" = "create" ]; then
  last=""
  for arg in "$@"; do
    last="$arg"
  done
  printf '%s\n' "$last"
  exit 0
fi
if [ "$1" = "volume" ] && [ "$2" = "ls" ]; then
  printf 'axiom_docker_orchestration_e2e_mysql_data\naxiom_docker_orchestration_e2e_postgres_data\naxiom_docker_orchestration_e2e_redis_data\n'
  exit 0
fi
if [ "$1" = "volume" ] && [ "$2" = "rm" ]; then
  exit 0
fi
exit 0
"#
    .replace("__LOG__", &docker_log_script_path);
    support::write_executable(&bin_dir.join("docker"), &docker_script);
    let _path = PathGuard::prepend(&bin_dir);
    let docker = ProjectDockerOrchestrator::with_base_dir(
        Arc::new(MemorySecureStorage::default()),
        compose_root.clone(),
    );
    let project = project("docker-orchestration-e2e", document_root);
    let request = compose_request(&project.id);

    let diagnostics = docker.diagnostics().expect("docker diagnostics");
    assert!(diagnostics.cli_found);
    assert!(diagnostics.engine_running);
    assert!(diagnostics.compose_available);

    let pins = docker
        .resolve_image_pins(&request)
        .expect("image pin resolution");
    assert_eq!(pins.resolutions.len(), request.profiles.len());

    let plan = docker
        .generate_compose_plan(&project, &request)
        .expect("compose plan");
    assert!(plan.compose_file_written);
    assert_eq!(plan.services.len(), 6);
    assert!(plan.image_trust.iter().all(|trust| trust.allowed));

    let volumes = docker
        .ensure_project_volumes(&project, &request)
        .expect("volume lifecycle");
    assert_eq!(volumes.volumes.len(), 3);

    let start = docker.start_project(&project, &request).expect("start");
    assert_eq!(start.action, "start");
    assert!(start.runtime.engine_running);
    assert!(start
        .runtime
        .containers
        .iter()
        .any(|container| container.service_name == "php"));

    let logs = docker.read_project_logs(&project, 50).expect("docker logs");
    assert!(logs.lines.iter().any(|line| line.contains("php ready")));
    assert!(logs
        .lines
        .iter()
        .any(|line| line == "[redacted sensitive docker log line]"
            || line == "[redacted sensitive process output]"));

    let restart = docker.restart_project(&project, &request).expect("restart");
    assert_eq!(restart.action, "restart");

    let stop = docker.stop_project(&project).expect("stop");
    assert_eq!(stop.action, "stop");

    let removed = docker
        .remove_project_volumes(&project)
        .expect("remove volumes");
    assert!(removed.volumes.iter().all(|volume| !volume.created));

    let log = fs::read_to_string(docker_log).expect("fake docker log");
    assert!(log.contains("compose"));
    assert!(log.contains("buildx imagetools inspect"));
}

fn compose_request(project_id: &ProjectId) -> DockerProjectComposeRequest {
    let digest = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let profiles = vec![
        DockerComposeProfile::Php,
        DockerComposeProfile::Mysql,
        DockerComposeProfile::Postgresql,
        DockerComposeProfile::Redis,
        DockerComposeProfile::Mailpit,
        DockerComposeProfile::ReverseProxy,
    ];
    let image_overrides = [
        (
            DockerComposeProfile::Php,
            format!("docker.io/library/php:8.4-cli@sha256:{digest}"),
        ),
        (
            DockerComposeProfile::Mysql,
            format!("docker.io/library/mysql:8.4@sha256:{digest}"),
        ),
        (
            DockerComposeProfile::Postgresql,
            format!("docker.io/library/postgres:17@sha256:{digest}"),
        ),
        (
            DockerComposeProfile::Redis,
            format!("docker.io/library/redis:7-alpine@sha256:{digest}"),
        ),
        (
            DockerComposeProfile::Mailpit,
            format!("docker.io/axllent/mailpit:v1.22@sha256:{digest}"),
        ),
        (
            DockerComposeProfile::ReverseProxy,
            format!("docker.io/library/nginx:1.27-alpine@sha256:{digest}"),
        ),
    ]
    .into_iter()
    .map(|(profile, image)| DockerProjectImageOverride { profile, image })
    .collect();

    DockerProjectComposeRequest {
        project_id: project_id.clone(),
        profiles,
        image_overrides,
        resource_limits: DockerProjectResourceLimits {
            cpus: Some(1.0),
            memory_mb: Some(512),
        },
    }
}

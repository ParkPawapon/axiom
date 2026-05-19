mod support;

use std::collections::BTreeMap;
use std::fs;

use axiomphp_lib::domain::docker::docker_project::{
    DockerComposeProfile, DockerProjectResourceLimits,
};
use axiomphp_lib::infrastructure::docker::docker_compose_generator::{
    DockerComposeGenerationInput, DockerComposeGenerator, DockerProjectPorts,
};
use support::TestEnvironment;

#[test]
fn compose_generation_covers_all_project_service_profiles() {
    let test_env = TestEnvironment::new("docker-compose-e2e");
    let document_root = test_env.path("document-root");
    fs::create_dir_all(&document_root).expect("document root");
    let digest = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let images = [
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
    .collect::<BTreeMap<_, _>>();
    let output = DockerComposeGenerator
        .generate(DockerComposeGenerationInput {
            project_id: "docker-compose-e2e".to_string(),
            document_root: document_root.to_string_lossy().into_owned(),
            compose_project_name: "axiom_docker_compose_e2e".to_string(),
            env_file_name: "compose.env".to_string(),
            reverse_proxy_config_file_name: "reverse-proxy.conf".to_string(),
            profiles: vec![
                DockerComposeProfile::Php,
                DockerComposeProfile::Mysql,
                DockerComposeProfile::Postgresql,
                DockerComposeProfile::Redis,
                DockerComposeProfile::Mailpit,
                DockerComposeProfile::ReverseProxy,
            ],
            images,
            ports: DockerProjectPorts {
                mysql_host_port: 33061,
                redis_host_port: 63791,
                mailpit_smtp_host_port: 10251,
                mailpit_web_host_port: 8026,
                postgres_host_port: 54321,
                reverse_proxy_host_port: 18081,
            },
            resource_limits: DockerProjectResourceLimits {
                cpus: Some(0.5),
                memory_mb: Some(512),
            },
        })
        .expect("compose generation");

    assert_eq!(output.services.len(), 6);
    assert_eq!(output.volumes.len(), 3);
    assert!(output.compose_yaml.contains("mem_limit: \"512m\""));
    assert!(output.compose_yaml.contains("cpus: \"0.50\""));
    assert!(output.compose_yaml.contains("reverse-proxy"));
    assert!(output.reverse_proxy_config.is_some());
    assert!(output
        .image_trust
        .iter()
        .all(|trust| trust.pinned_by_digest));
}

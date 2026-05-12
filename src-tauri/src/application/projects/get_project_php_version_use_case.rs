use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_php_version::ProjectPhpVersionConfig;
use crate::domain::runtime::php_runtime::{default_php_version, supported_php_versions_catalog};
use crate::ports::php_runtime_detector::PhpRuntimeDetector;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn get_project_php_version(
    repository: &dyn ProjectRuntimeRepository,
    detector: &dyn PhpRuntimeDetector,
    project_id: &str,
) -> AppResult<ProjectPhpVersionConfig> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let selection = repository.get_php_selection(&project_id)?;
    let selected_php_version = selection
        .as_ref()
        .map(|selection| selection.php_version.clone())
        .unwrap_or_else(default_php_version);
    let selected_php_binary = if let Some(selection) = selection {
        detector.detect_php_binary(&selection.php_version)?
    } else {
        None
    };
    let mut available_php_versions = Vec::new();

    for runtime in supported_php_versions_catalog() {
        let detected_binary = detector.detect_php_binary(&runtime.version)?;
        let runtime = if let Some(binary) = detected_binary {
            runtime.with_detected_binary(binary)
        } else {
            runtime
        };

        available_php_versions.push(runtime);
    }

    Ok(ProjectPhpVersionConfig {
        project_id,
        selected_php_version,
        selected_php_binary,
        available_php_versions,
        status_message: "PHP switching is enabled only for PHP binaries detected on this machine. Missing or end-of-life versions require explicit manual installation confirmation before they can be selected.".to_string(),
    })
}

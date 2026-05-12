use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_php_version::ProjectPhpVersionConfig;
use crate::domain::runtime::php_runtime::{default_php_version, supported_php_versions};
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn get_project_php_version(
    repository: &dyn ProjectRuntimeRepository,
    project_id: &str,
) -> AppResult<ProjectPhpVersionConfig> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let selected_php_version = repository
        .get_php_version(&project_id)?
        .unwrap_or_else(default_php_version);

    Ok(ProjectPhpVersionConfig {
        project_id,
        selected_php_version,
        available_php_versions: supported_php_versions(),
        status_message: "PHP version preference is stored for this project. Runtime installation and process switching remain disabled until runtime management is implemented.".to_string(),
    })
}

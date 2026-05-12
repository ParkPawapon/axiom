use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_php_version::ProjectPhpInstallPlan;
use crate::domain::runtime::php_runtime::is_supported_php_version;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn request_project_php_install(
    repository: &dyn ProjectRuntimeRepository,
    project_id: &str,
    php_version: &str,
) -> AppResult<ProjectPhpInstallPlan> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let php_version = RuntimeVersion::new(php_version)?;

    if !is_supported_php_version(&php_version) {
        return Err(AppError::Validation(format!(
            "PHP {} is not in the supported project runtime catalog",
            php_version.as_str()
        )));
    }

    repository.record_php_install_request(&project_id, &php_version)?;

    Ok(ProjectPhpInstallPlan {
        project_id,
        php_version: php_version.clone(),
        requires_manual_confirmation: true,
        warning_message: format!(
            "PHP {} requires manual installation confirmation. End-of-life PHP branches should only be installed for isolated legacy projects.",
            php_version.as_str()
        ),
        status_message: "AxiomPHP recorded the install request but did not run an installer. Install or register the PHP binary through a trusted runtime source, then refresh this project before switching.".to_string(),
    })
}

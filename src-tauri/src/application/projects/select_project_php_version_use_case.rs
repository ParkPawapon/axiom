use crate::application::projects::get_project_php_version_use_case;
use crate::domain::project::project_php_version::ProjectPhpVersionConfig;
use crate::domain::runtime::php_runtime::is_supported_php_version;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn select_project_php_version(
    repository: &dyn ProjectRuntimeRepository,
    project_id: &str,
    php_version: &str,
) -> AppResult<ProjectPhpVersionConfig> {
    let project_id = validate_project_id(project_id)?;
    let php_version = RuntimeVersion::new(php_version)?;

    if !is_supported_php_version(&php_version) {
        return Err(AppError::Validation(format!(
            "PHP {} is not in the supported project runtime catalog",
            php_version.as_str()
        )));
    }

    repository.save_php_version(
        &crate::domain::project::project_id::ProjectId(project_id.to_string()),
        &php_version,
    )?;

    get_project_php_version_use_case::get_project_php_version(repository, project_id)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::domain::project::project_id::ProjectId;
    use crate::ports::project_runtime_repository::ProjectRuntimeRepository;

    #[derive(Debug, Default)]
    struct MemoryProjectRuntimeRepository {
        selected: Mutex<Option<RuntimeVersion>>,
    }

    impl ProjectRuntimeRepository for MemoryProjectRuntimeRepository {
        fn get_php_version(&self, _project_id: &ProjectId) -> AppResult<Option<RuntimeVersion>> {
            Ok(self
                .selected
                .lock()
                .map_err(|_error| AppError::Unexpected)?
                .clone())
        }

        fn save_php_version(
            &self,
            _project_id: &ProjectId,
            version: &RuntimeVersion,
        ) -> AppResult<()> {
            *self
                .selected
                .lock()
                .map_err(|_error| AppError::Unexpected)? = Some(version.clone());
            Ok(())
        }
    }

    #[test]
    fn saves_supported_php_version_selection() {
        let repository = MemoryProjectRuntimeRepository::default();

        let config =
            select_project_php_version(&repository, "current-project", "8.4").expect("valid save");

        assert_eq!(config.selected_php_version.as_str(), "8.4");
        assert_eq!(config.project_id.0, "current-project");
    }

    #[test]
    fn rejects_unsupported_php_version_selection() {
        let repository = MemoryProjectRuntimeRepository::default();

        let result = select_project_php_version(&repository, "current-project", "7.4");

        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}

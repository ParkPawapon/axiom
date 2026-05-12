use crate::application::projects::get_project_php_version_use_case;
use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_php_version::{
    ProjectPhpRuntimeSelection, ProjectPhpVersionConfig,
};
use crate::domain::runtime::php_runtime::is_supported_php_version;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::ports::php_runtime_detector::PhpRuntimeDetector;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn select_project_php_version(
    repository: &dyn ProjectRuntimeRepository,
    detector: &dyn PhpRuntimeDetector,
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

    let detected_binary = detector
        .detect_php_binary(&php_version)?
        .ok_or_else(|| {
            AppError::Validation(format!(
                "PHP {} is not installed or not discoverable on PATH. Confirm manual installation before switching this project.",
                php_version.as_str()
            ))
        })?;
    let project_id = ProjectId(project_id.to_string());

    repository.save_php_selection(
        &project_id,
        &ProjectPhpRuntimeSelection {
            php_version,
            php_binary_path: detected_binary.path,
        },
    )?;

    get_project_php_version_use_case::get_project_php_version(repository, detector, &project_id.0)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::domain::project::project_id::ProjectId;
    use crate::domain::runtime::php_runtime::DetectedPhpBinary;
    use crate::domain::runtime::runtime_path::RuntimePath;
    use crate::ports::php_runtime_detector::PhpRuntimeDetector;
    use crate::ports::project_runtime_repository::ProjectRuntimeRepository;

    #[derive(Debug, Default)]
    struct MemoryProjectRuntimeRepository {
        selected: Mutex<Option<ProjectPhpRuntimeSelection>>,
    }

    impl ProjectRuntimeRepository for MemoryProjectRuntimeRepository {
        fn get_php_selection(
            &self,
            _project_id: &ProjectId,
        ) -> AppResult<Option<ProjectPhpRuntimeSelection>> {
            Ok(self
                .selected
                .lock()
                .map_err(|_error| AppError::Unexpected)?
                .clone())
        }

        fn save_php_selection(
            &self,
            _project_id: &ProjectId,
            selection: &ProjectPhpRuntimeSelection,
        ) -> AppResult<()> {
            *self
                .selected
                .lock()
                .map_err(|_error| AppError::Unexpected)? = Some(selection.clone());
            Ok(())
        }

        fn record_php_install_request(
            &self,
            _project_id: &ProjectId,
            _version: &RuntimeVersion,
        ) -> AppResult<()> {
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    struct MemoryPhpRuntimeDetector;

    impl PhpRuntimeDetector for MemoryPhpRuntimeDetector {
        fn detect_php_binary(
            &self,
            version: &RuntimeVersion,
        ) -> AppResult<Option<DetectedPhpBinary>> {
            Ok(Some(DetectedPhpBinary {
                version: version.clone(),
                path: RuntimePath(format!("/usr/local/bin/php{}", version.as_str())),
                display_name: format!("php{} test binary", version.as_str()),
            }))
        }
    }

    #[test]
    fn saves_supported_php_version_selection() {
        let repository = MemoryProjectRuntimeRepository::default();
        let detector = MemoryPhpRuntimeDetector;

        let config = select_project_php_version(&repository, &detector, "current-project", "8.4")
            .expect("valid save");

        assert_eq!(config.selected_php_version.as_str(), "8.4");
        assert_eq!(config.project_id.0, "current-project");
    }

    #[test]
    fn rejects_unsupported_php_version_selection() {
        let repository = MemoryProjectRuntimeRepository::default();
        let detector = MemoryPhpRuntimeDetector;

        let result = select_project_php_version(&repository, &detector, "current-project", "4.4");

        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}

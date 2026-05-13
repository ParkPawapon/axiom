use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_process::ProjectPhpProcessStatus;
use crate::ports::php_runtime_detector::PhpRuntimeDetector;
use crate::ports::project_php_process_manager::{
    ProjectPhpProcessManager, StartProjectPhpProcessRequest,
};
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn start_project_php_process(
    repository: &dyn ProjectRuntimeRepository,
    detector: &dyn PhpRuntimeDetector,
    process_manager: &dyn ProjectPhpProcessManager,
    project_id: &str,
) -> AppResult<ProjectPhpProcessStatus> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let selection = repository.get_php_selection(&project_id)?.ok_or_else(|| {
        AppError::Validation(
            "select and save a PHP binary before starting the project process".to_string(),
        )
    })?;
    let php_binary = detector
        .detect_php_binary(&selection.php_version)?
        .ok_or_else(|| {
            AppError::Validation(format!(
                "PHP {} is not installed or not discoverable on PATH",
                selection.php_version.as_str()
            ))
        })?;

    process_manager.start_php_process(StartProjectPhpProcessRequest {
        project_id,
        php_version: selection.php_version,
        php_binary,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::domain::project::project_php_version::ProjectPhpRuntimeSelection;
    use crate::domain::project::project_process::{
        ProjectPhpProcessState, ProjectPhpProcessStatus,
    };
    use crate::domain::runtime::php_runtime::DetectedPhpBinary;
    use crate::domain::runtime::runtime_path::RuntimePath;
    use crate::domain::runtime::runtime_version::RuntimeVersion;

    #[derive(Debug, Default)]
    struct MemoryProjectRuntimeRepository {
        selection: Mutex<Option<ProjectPhpRuntimeSelection>>,
    }

    impl ProjectRuntimeRepository for MemoryProjectRuntimeRepository {
        fn get_php_selection(
            &self,
            _project_id: &ProjectId,
        ) -> AppResult<Option<ProjectPhpRuntimeSelection>> {
            Ok(self
                .selection
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
                .selection
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
                display_name: format!("PHP {}", version.as_str()),
            }))
        }
    }

    #[derive(Debug, Default)]
    struct MemoryProjectPhpProcessManager;

    impl ProjectPhpProcessManager for MemoryProjectPhpProcessManager {
        fn start_php_process(
            &self,
            request: StartProjectPhpProcessRequest,
        ) -> AppResult<ProjectPhpProcessStatus> {
            Ok(ProjectPhpProcessStatus {
                project_id: request.project_id,
                state: ProjectPhpProcessState::Running,
                pid: Some(42),
                php_version: Some(request.php_version),
                port: Some(8500),
                url: Some("http://127.0.0.1:8500".to_string()),
                document_root: Some("/tmp/axiomphp/current-project/public".to_string()),
                log_file: Some("/tmp/axiomphp/current-project/php-server.log".to_string()),
                started_at: None,
                status_message: "running".to_string(),
            })
        }

        fn stop_php_process(&self, project_id: &ProjectId) -> AppResult<ProjectPhpProcessStatus> {
            Ok(ProjectPhpProcessStatus::stopped(project_id.clone()))
        }

        fn get_php_process_status(
            &self,
            project_id: &ProjectId,
        ) -> AppResult<ProjectPhpProcessStatus> {
            Ok(ProjectPhpProcessStatus::stopped(project_id.clone()))
        }
    }

    #[test]
    fn rejects_start_without_selected_php_binary() {
        let repository = MemoryProjectRuntimeRepository::default();
        let detector = MemoryPhpRuntimeDetector;
        let process_manager = MemoryProjectPhpProcessManager;

        let result =
            start_project_php_process(&repository, &detector, &process_manager, "current-project");

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn starts_with_selected_php_binary() {
        let repository = MemoryProjectRuntimeRepository::default();
        let detector = MemoryPhpRuntimeDetector;
        let process_manager = MemoryProjectPhpProcessManager;
        repository
            .save_php_selection(
                &ProjectId("current-project".to_string()),
                &ProjectPhpRuntimeSelection {
                    php_version: RuntimeVersion::trusted("8.4"),
                    php_binary_path: RuntimePath("/usr/local/bin/php8.4".to_string()),
                },
            )
            .expect("selection should save");

        let status =
            start_project_php_process(&repository, &detector, &process_manager, "current-project")
                .expect("process should start");

        assert_eq!(status.state, ProjectPhpProcessState::Running);
        assert_eq!(status.php_version.expect("version").as_str(), "8.4");
    }
}

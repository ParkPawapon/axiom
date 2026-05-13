use crate::domain::project::project_process::ProjectPhpProcessStatus;
use crate::ports::php_runtime_detector::PhpRuntimeDetector;
use crate::ports::project_php_process_manager::ProjectPhpProcessManager;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::shared::result::app_result::AppResult;

use super::start_project_php_process_use_case::build_start_project_php_process_request;

pub fn restart_project_php_process(
    project_repository: &dyn ProjectRepository,
    runtime_repository: &dyn ProjectRuntimeRepository,
    detector: &dyn PhpRuntimeDetector,
    process_manager: &dyn ProjectPhpProcessManager,
    project_id: &str,
) -> AppResult<ProjectPhpProcessStatus> {
    let request = build_start_project_php_process_request(
        project_repository,
        runtime_repository,
        detector,
        project_id,
    )?;

    process_manager.restart_php_process(request)
}

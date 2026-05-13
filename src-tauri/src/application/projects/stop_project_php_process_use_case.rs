use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_process::ProjectPhpProcessStatus;
use crate::ports::project_php_process_manager::ProjectPhpProcessManager;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn stop_project_php_process(
    process_manager: &dyn ProjectPhpProcessManager,
    project_id: &str,
) -> AppResult<ProjectPhpProcessStatus> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());

    process_manager.stop_php_process(&project_id)
}

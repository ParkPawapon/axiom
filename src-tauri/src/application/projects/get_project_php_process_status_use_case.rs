use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_process::ProjectPhpProcessStatus;
use crate::ports::project_php_process_manager::ProjectPhpProcessManager;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn get_project_php_process_status(
    process_manager: &dyn ProjectPhpProcessManager,
    project_id: &str,
) -> AppResult<ProjectPhpProcessStatus> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());

    process_manager.get_php_process_status(&project_id)
}

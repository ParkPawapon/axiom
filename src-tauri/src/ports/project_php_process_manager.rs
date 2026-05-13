use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_process::ProjectPhpProcessStatus;
use crate::domain::runtime::php_runtime::DetectedPhpBinary;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::shared::result::app_result::AppResult;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StartProjectPhpProcessRequest {
    pub project_id: ProjectId,
    pub php_version: RuntimeVersion,
    pub php_binary: DetectedPhpBinary,
}

pub trait ProjectPhpProcessManager: Send + Sync {
    fn start_php_process(
        &self,
        request: StartProjectPhpProcessRequest,
    ) -> AppResult<ProjectPhpProcessStatus>;

    fn stop_php_process(&self, project_id: &ProjectId) -> AppResult<ProjectPhpProcessStatus>;

    fn get_php_process_status(&self, project_id: &ProjectId) -> AppResult<ProjectPhpProcessStatus>;
}

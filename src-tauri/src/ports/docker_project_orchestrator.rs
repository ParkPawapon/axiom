use crate::domain::docker::docker_project::{
    DockerComposeProfile, DockerDiagnosticsReport, DockerProjectActionResult,
    DockerProjectComposePlan, DockerProjectLogReadResult, DockerProjectRuntimeStatus,
    DockerProjectVolumeLifecycleResult,
};
use crate::domain::project::project::Project;
use crate::shared::result::app_result::AppResult;

pub trait DockerProjectOrchestrator: Send + Sync {
    fn diagnostics(&self) -> AppResult<DockerDiagnosticsReport>;

    fn generate_compose_plan(
        &self,
        project: &Project,
        profiles: &[DockerComposeProfile],
    ) -> AppResult<DockerProjectComposePlan>;

    fn get_runtime_status(&self, project: &Project) -> AppResult<DockerProjectRuntimeStatus>;

    fn start_project(
        &self,
        project: &Project,
        profiles: &[DockerComposeProfile],
    ) -> AppResult<DockerProjectActionResult>;

    fn stop_project(&self, project: &Project) -> AppResult<DockerProjectActionResult>;

    fn restart_project(
        &self,
        project: &Project,
        profiles: &[DockerComposeProfile],
    ) -> AppResult<DockerProjectActionResult>;

    fn ensure_project_volumes(
        &self,
        project: &Project,
        profiles: &[DockerComposeProfile],
    ) -> AppResult<DockerProjectVolumeLifecycleResult>;

    fn remove_project_volumes(
        &self,
        project: &Project,
    ) -> AppResult<DockerProjectVolumeLifecycleResult>;

    fn read_project_logs(
        &self,
        project: &Project,
        tail_lines: u16,
    ) -> AppResult<DockerProjectLogReadResult>;
}

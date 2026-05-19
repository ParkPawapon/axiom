use crate::domain::docker::docker_project::{
    DockerDiagnosticsReport, DockerImagePinResolutionReport, DockerProjectActionResult,
    DockerProjectComposePlan, DockerProjectComposeRequest, DockerProjectLogReadResult,
    DockerProjectRuntimeStatus, DockerProjectVolumeLifecycleResult,
};
use crate::domain::project::project::Project;
use crate::shared::result::app_result::AppResult;

pub trait DockerProjectOrchestrator: Send + Sync {
    fn diagnostics(&self) -> AppResult<DockerDiagnosticsReport>;

    fn resolve_image_pins(
        &self,
        request: &DockerProjectComposeRequest,
    ) -> AppResult<DockerImagePinResolutionReport>;

    fn generate_compose_plan(
        &self,
        project: &Project,
        request: &DockerProjectComposeRequest,
    ) -> AppResult<DockerProjectComposePlan>;

    fn get_runtime_status(&self, project: &Project) -> AppResult<DockerProjectRuntimeStatus>;

    fn start_project(
        &self,
        project: &Project,
        request: &DockerProjectComposeRequest,
    ) -> AppResult<DockerProjectActionResult>;

    fn stop_project(&self, project: &Project) -> AppResult<DockerProjectActionResult>;

    fn restart_project(
        &self,
        project: &Project,
        request: &DockerProjectComposeRequest,
    ) -> AppResult<DockerProjectActionResult>;

    fn ensure_project_volumes(
        &self,
        project: &Project,
        request: &DockerProjectComposeRequest,
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

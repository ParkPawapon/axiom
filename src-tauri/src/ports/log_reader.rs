use crate::domain::logs::project_log::ProjectLogReadResult;
use crate::domain::project::project_id::ProjectId;
use crate::shared::result::app_result::AppResult;

pub trait LogReader: Send + Sync {
    fn read_project_process_log(
        &self,
        project_id: &ProjectId,
        max_lines: usize,
        query: Option<&str>,
    ) -> AppResult<ProjectLogReadResult>;
}

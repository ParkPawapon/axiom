use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_php_version::ProjectPhpRuntimeSelection;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::shared::result::app_result::AppResult;

pub trait ProjectRuntimeRepository: Send + Sync {
    fn get_php_selection(
        &self,
        project_id: &ProjectId,
    ) -> AppResult<Option<ProjectPhpRuntimeSelection>>;
    fn save_php_selection(
        &self,
        project_id: &ProjectId,
        selection: &ProjectPhpRuntimeSelection,
    ) -> AppResult<()>;
    fn record_php_install_request(
        &self,
        project_id: &ProjectId,
        version: &RuntimeVersion,
    ) -> AppResult<()>;
}

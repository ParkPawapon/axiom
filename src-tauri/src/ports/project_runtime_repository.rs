use crate::domain::project::project_id::ProjectId;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::shared::result::app_result::AppResult;

pub trait ProjectRuntimeRepository: Send + Sync {
    fn get_php_version(&self, project_id: &ProjectId) -> AppResult<Option<RuntimeVersion>>;
    fn save_php_version(&self, project_id: &ProjectId, version: &RuntimeVersion) -> AppResult<()>;
}

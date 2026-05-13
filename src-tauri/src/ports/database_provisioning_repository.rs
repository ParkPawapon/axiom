use crate::domain::database::database_config::ProjectDatabaseProfile;
use crate::domain::database::database_type::DatabaseType;
use crate::domain::project::project_id::ProjectId;
use crate::shared::result::app_result::AppResult;

pub trait DatabaseProvisioningRepository: Send + Sync {
    fn list_profiles(&self, project_id: &ProjectId) -> AppResult<Vec<ProjectDatabaseProfile>>;

    fn get_profile(
        &self,
        project_id: &ProjectId,
        database_type: DatabaseType,
    ) -> AppResult<Option<ProjectDatabaseProfile>>;

    fn save_profile(&self, profile: ProjectDatabaseProfile) -> AppResult<ProjectDatabaseProfile>;
}

use crate::domain::database::database_config::DatabaseBackupRemoteDestination;
use crate::domain::database::database_type::DatabaseType;
use crate::domain::project::project_id::ProjectId;
use crate::shared::result::app_result::AppResult;

pub trait DatabaseBackupDestinationRepository: Send + Sync {
    fn list_destinations(
        &self,
        project_id: &ProjectId,
    ) -> AppResult<Vec<DatabaseBackupRemoteDestination>>;

    fn get_destination(
        &self,
        project_id: &ProjectId,
        database_type: DatabaseType,
    ) -> AppResult<Option<DatabaseBackupRemoteDestination>>;

    fn save_destination(
        &self,
        destination: DatabaseBackupRemoteDestination,
    ) -> AppResult<DatabaseBackupRemoteDestination>;
}

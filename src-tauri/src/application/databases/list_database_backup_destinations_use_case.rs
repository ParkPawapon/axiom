use crate::domain::database::database_config::DatabaseBackupRemoteDestination;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_backup_destination_repository::DatabaseBackupDestinationRepository;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn list_database_backup_destinations(
    backup_destination_repository: &dyn DatabaseBackupDestinationRepository,
    project_id: &str,
) -> AppResult<Vec<DatabaseBackupRemoteDestination>> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());

    backup_destination_repository.list_destinations(&project_id)
}

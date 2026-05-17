use chrono::{DateTime, Utc};

use crate::domain::database::database_config::{DatabaseBackupMetadata, ProjectDatabaseProfile};
use crate::shared::result::app_result::AppResult;

pub trait DatabaseBackupCatalog: Send + Sync {
    fn latest_backup_at_or_before(
        &self,
        profile: &ProjectDatabaseProfile,
        target_time: DateTime<Utc>,
    ) -> AppResult<Option<DatabaseBackupMetadata>>;
}

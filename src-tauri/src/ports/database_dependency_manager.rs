use crate::domain::database::database_config::{
    ManagedDatabaseDependencyReport, ManagedDatabaseServiceReport, PhpMyAdminAccess,
    ProjectDatabaseProfile,
};
use crate::domain::database::database_type::DatabaseType;
use crate::shared::result::app_result::AppResult;

pub trait DatabaseDependencyManager: Send + Sync {
    fn ensure_database_dependencies(
        &self,
        database_type: DatabaseType,
    ) -> AppResult<ManagedDatabaseDependencyReport>;

    fn start_database_service(
        &self,
        database_type: DatabaseType,
    ) -> AppResult<ManagedDatabaseServiceReport>;

    fn configure_phpmyadmin(
        &self,
        profile: &ProjectDatabaseProfile,
    ) -> AppResult<Option<PhpMyAdminAccess>>;
}

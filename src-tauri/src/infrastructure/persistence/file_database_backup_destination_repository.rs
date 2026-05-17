use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use directories::ProjectDirs;

use crate::domain::database::database_config::DatabaseBackupRemoteDestination;
use crate::domain::database::database_type::DatabaseType;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_backup_destination_repository::DatabaseBackupDestinationRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug)]
pub struct FileDatabaseBackupDestinationRepository {
    storage_path: PathBuf,
    write_lock: Mutex<()>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseBackupDestinationStore {
    destinations: BTreeMap<String, DatabaseBackupRemoteDestination>,
}

impl FileDatabaseBackupDestinationRepository {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application config directory".to_string())
        })?;

        Ok(Self::with_storage_path(
            project_dirs
                .config_dir()
                .join("database-backup-destinations.json"),
        ))
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            write_lock: Mutex::new(()),
        }
    }

    fn load_store(&self) -> AppResult<DatabaseBackupDestinationStore> {
        if !self.storage_path.exists() {
            return Ok(DatabaseBackupDestinationStore::default());
        }

        let contents = fs::read_to_string(&self.storage_path).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to read database backup destination configuration: {error}"
            ))
        })?;

        serde_json::from_str(&contents).map_err(|error| {
            AppError::Configuration(format!(
                "database backup destination configuration is invalid: {error}"
            ))
        })
    }

    fn save_store_unlocked(&self, store: &DatabaseBackupDestinationStore) -> AppResult<()> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to create database backup destination directory: {error}"
                ))
            })?;
        }

        let payload = serde_json::to_vec_pretty(store).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to serialize database backup destinations: {error}"
            ))
        })?;
        let temporary_path = self.storage_path.with_extension("json.tmp");

        fs::write(&temporary_path, payload).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to write database backup destinations: {error}"
            ))
        })?;
        if self.storage_path.exists() {
            fs::remove_file(&self.storage_path).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to replace database backup destinations: {error}"
                ))
            })?;
        }
        fs::rename(&temporary_path, &self.storage_path).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to commit database backup destinations: {error}"
            ))
        })
    }
}

impl DatabaseBackupDestinationRepository for FileDatabaseBackupDestinationRepository {
    fn list_destinations(
        &self,
        project_id: &ProjectId,
    ) -> AppResult<Vec<DatabaseBackupRemoteDestination>> {
        let mut destinations = self
            .load_store()?
            .destinations
            .into_values()
            .filter(|destination| destination.project_id == *project_id)
            .collect::<Vec<_>>();

        destinations.sort_by_key(|destination| destination.database_type.as_key());
        Ok(destinations)
    }

    fn get_destination(
        &self,
        project_id: &ProjectId,
        database_type: DatabaseType,
    ) -> AppResult<Option<DatabaseBackupRemoteDestination>> {
        Ok(self
            .load_store()?
            .destinations
            .get(&destination_key(project_id, database_type))
            .cloned())
    }

    fn save_destination(
        &self,
        destination: DatabaseBackupRemoteDestination,
    ) -> AppResult<DatabaseBackupRemoteDestination> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_error| AppError::Unexpected)?;
        let mut store = self.load_store()?;

        store.destinations.insert(
            destination_key(&destination.project_id, destination.database_type),
            destination.clone(),
        );
        self.save_store_unlocked(&store)?;

        Ok(destination)
    }
}

fn destination_key(project_id: &ProjectId, database_type: DatabaseType) -> String {
    format!("{}:{}", project_id.0, database_type.as_key())
}

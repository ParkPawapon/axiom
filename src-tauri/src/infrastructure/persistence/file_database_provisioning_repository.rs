use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use directories::ProjectDirs;

use crate::domain::database::database_config::ProjectDatabaseProfile;
use crate::domain::database::database_type::DatabaseType;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug)]
pub struct FileDatabaseProvisioningRepository {
    storage_path: PathBuf,
    write_lock: Mutex<()>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseProvisioningStore {
    profiles: BTreeMap<String, ProjectDatabaseProfile>,
}

impl FileDatabaseProvisioningRepository {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application config directory".to_string())
        })?;

        Ok(Self::with_storage_path(
            project_dirs.config_dir().join("database-provisioning.json"),
        ))
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            write_lock: Mutex::new(()),
        }
    }

    fn load_store(&self) -> AppResult<DatabaseProvisioningStore> {
        if !self.storage_path.exists() {
            return Ok(DatabaseProvisioningStore::default());
        }

        let contents = fs::read_to_string(&self.storage_path).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to read database provisioning configuration: {error}"
            ))
        })?;

        serde_json::from_str(&contents).map_err(|error| {
            AppError::Configuration(format!(
                "database provisioning configuration is invalid: {error}"
            ))
        })
    }

    fn save_store_unlocked(&self, store: &DatabaseProvisioningStore) -> AppResult<()> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to create database provisioning configuration directory: {error}"
                ))
            })?;
        }

        let payload = serde_json::to_vec_pretty(store).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to serialize database provisioning configuration: {error}"
            ))
        })?;
        let temporary_path = self.storage_path.with_extension("json.tmp");

        fs::write(&temporary_path, payload).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to write database provisioning configuration: {error}"
            ))
        })?;

        if self.storage_path.exists() {
            fs::remove_file(&self.storage_path).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to replace database provisioning configuration: {error}"
                ))
            })?;
        }

        fs::rename(&temporary_path, &self.storage_path).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to commit database provisioning configuration: {error}"
            ))
        })
    }
}

impl DatabaseProvisioningRepository for FileDatabaseProvisioningRepository {
    fn list_profiles(&self, project_id: &ProjectId) -> AppResult<Vec<ProjectDatabaseProfile>> {
        let mut profiles = self
            .load_store()?
            .profiles
            .into_values()
            .filter(|profile| profile.project_id == *project_id)
            .collect::<Vec<_>>();

        profiles.sort_by_key(|profile| profile.database_type.as_key());

        Ok(profiles)
    }

    fn get_profile(
        &self,
        project_id: &ProjectId,
        database_type: DatabaseType,
    ) -> AppResult<Option<ProjectDatabaseProfile>> {
        Ok(self
            .load_store()?
            .profiles
            .get(&profile_key(project_id, database_type))
            .cloned())
    }

    fn save_profile(&self, profile: ProjectDatabaseProfile) -> AppResult<ProjectDatabaseProfile> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_error| AppError::Unexpected)?;
        let mut store = self.load_store()?;

        store.profiles.insert(
            profile_key(&profile.project_id, profile.database_type),
            profile.clone(),
        );
        self.save_store_unlocked(&store)?;

        Ok(profile)
    }
}

fn profile_key(project_id: &ProjectId, database_type: DatabaseType) -> String {
    format!("{}:{}", project_id.0, database_type.as_key())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::database::database_config::DatabaseProvisioningStatus;

    #[test]
    fn persists_database_profiles() {
        let storage_path = std::env::temp_dir().join(format!(
            "axiom-database-provisioning-{}.json",
            uuid::Uuid::new_v4()
        ));
        let repository =
            FileDatabaseProvisioningRepository::with_storage_path(storage_path.clone());
        let project_id = ProjectId("project-one".to_string());
        let now = Utc::now();

        repository
            .save_profile(ProjectDatabaseProfile {
                project_id: project_id.clone(),
                database_type: DatabaseType::Mysql,
                database_name: "ax_project_one_mysql".to_string(),
                username: "ax_project_one".to_string(),
                host: "127.0.0.1".to_string(),
                port: 3306,
                data_dir: "/tmp/data".to_string(),
                backup_dir: "/tmp/backups".to_string(),
                migration_dir: "/tmp/migrations".to_string(),
                admin_url: Some("http://127.0.0.1/phpmyadmin?db=ax_project_one_mysql".to_string()),
                status: DatabaseProvisioningStatus::Pending,
                status_message: "pending".to_string(),
                applied_migrations: Vec::new(),
                created_at: now,
                updated_at: now,
            })
            .expect("profile should persist");

        let profiles = repository
            .list_profiles(&project_id)
            .expect("profiles should load");

        assert_eq!(profiles.len(), 1);

        let _ = fs::remove_file(storage_path);
    }
}

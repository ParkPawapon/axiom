use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use directories::ProjectDirs;

use crate::domain::database::database_config::DatabaseBackupPolicy;
use crate::domain::database::database_type::DatabaseType;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_backup_policy_repository::DatabaseBackupPolicyRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug)]
pub struct FileDatabaseBackupPolicyRepository {
    storage_path: PathBuf,
    write_lock: Mutex<()>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseBackupPolicyStore {
    policies: BTreeMap<String, DatabaseBackupPolicy>,
}

impl FileDatabaseBackupPolicyRepository {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application config directory".to_string())
        })?;

        Ok(Self::with_storage_path(
            project_dirs
                .config_dir()
                .join("database-backup-policies.json"),
        ))
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            write_lock: Mutex::new(()),
        }
    }

    fn load_store(&self) -> AppResult<DatabaseBackupPolicyStore> {
        if !self.storage_path.exists() {
            return Ok(DatabaseBackupPolicyStore::default());
        }

        let contents = fs::read_to_string(&self.storage_path).map_err(|error| {
            AppError::Infrastructure(format!("failed to read database backup policies: {error}"))
        })?;

        serde_json::from_str(&contents).map_err(|error| {
            AppError::Configuration(format!("database backup policies are invalid: {error}"))
        })
    }

    fn save_store_unlocked(&self, store: &DatabaseBackupPolicyStore) -> AppResult<()> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to create database backup policy directory: {error}"
                ))
            })?;
        }

        let payload = serde_json::to_vec_pretty(store).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to serialize database backup policies: {error}"
            ))
        })?;
        let temporary_path = self.storage_path.with_extension("json.tmp");

        fs::write(&temporary_path, payload).map_err(|error| {
            AppError::Infrastructure(format!("failed to write database backup policies: {error}"))
        })?;

        if self.storage_path.exists() {
            fs::remove_file(&self.storage_path).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to replace database backup policies: {error}"
                ))
            })?;
        }

        fs::rename(&temporary_path, &self.storage_path).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to commit database backup policies: {error}"
            ))
        })
    }
}

impl DatabaseBackupPolicyRepository for FileDatabaseBackupPolicyRepository {
    fn list_policies(&self, project_id: &ProjectId) -> AppResult<Vec<DatabaseBackupPolicy>> {
        let mut policies = self
            .load_store()?
            .policies
            .into_values()
            .filter(|policy| policy.project_id == *project_id)
            .collect::<Vec<_>>();

        policies.sort_by_key(|policy| policy.database_type.as_key());

        Ok(policies)
    }

    fn list_all_policies(&self) -> AppResult<Vec<DatabaseBackupPolicy>> {
        Ok(self.load_store()?.policies.into_values().collect())
    }

    fn get_policy(
        &self,
        project_id: &ProjectId,
        database_type: DatabaseType,
    ) -> AppResult<Option<DatabaseBackupPolicy>> {
        Ok(self
            .load_store()?
            .policies
            .get(&policy_key(project_id, database_type))
            .cloned())
    }

    fn save_policy(&self, policy: DatabaseBackupPolicy) -> AppResult<DatabaseBackupPolicy> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_error| AppError::Unexpected)?;
        let mut store = self.load_store()?;

        store.policies.insert(
            policy_key(&policy.project_id, policy.database_type),
            policy.clone(),
        );
        self.save_store_unlocked(&store)?;

        Ok(policy)
    }
}

fn policy_key(project_id: &ProjectId, database_type: DatabaseType) -> String {
    format!("{}:{}", project_id.0, database_type.as_key())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::domain::database::database_config::{
        DatabaseBackupCompression, DatabaseBackupEncryption,
    };

    use super::*;

    #[test]
    fn persists_backup_policies() {
        let storage_path = std::env::temp_dir().join(format!(
            "axiom-database-backup-policies-{}.json",
            uuid::Uuid::new_v4()
        ));
        let repository =
            FileDatabaseBackupPolicyRepository::with_storage_path(storage_path.clone());
        let project_id = ProjectId("project-one".to_string());

        repository
            .save_policy(DatabaseBackupPolicy {
                project_id: project_id.clone(),
                database_type: DatabaseType::Mysql,
                enabled: true,
                interval_minutes: 60,
                retention_days: 30,
                compression: DatabaseBackupCompression::Gzip,
                encryption: DatabaseBackupEncryption::Aes256Gcm,
                last_run_at: None,
                next_run_at: Some(Utc::now()),
                updated_at: Utc::now(),
            })
            .expect("policy should persist");

        let policies = repository
            .list_policies(&project_id)
            .expect("policies should load");

        assert_eq!(policies.len(), 1);

        let _ = fs::remove_file(storage_path);
    }
}

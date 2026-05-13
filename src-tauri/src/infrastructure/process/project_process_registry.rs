use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{DateTime, Utc};

use crate::domain::project::project_id::ProjectId;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const PROCESS_REGISTRY_SCHEMA_VERSION: u16 = 1;

#[derive(Debug)]
pub struct ProjectProcessRegistry {
    storage_path: PathBuf,
    write_lock: Mutex<()>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedProjectProcessRecord {
    pub project_id: ProjectId,
    pub pid: u32,
    pub php_version: RuntimeVersion,
    pub php_binary_path: String,
    pub port: u16,
    pub url: String,
    pub document_root: String,
    pub log_file: String,
    pub started_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectProcessRegistryStore {
    schema_version: u16,
    processes: BTreeMap<String, PersistedProjectProcessRecord>,
}

impl ProjectProcessRegistry {
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            write_lock: Mutex::new(()),
        }
    }

    pub fn load_records(&self) -> AppResult<Vec<PersistedProjectProcessRecord>> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_error| AppError::Unexpected)?;

        Ok(self
            .load_store_unlocked()?
            .processes
            .into_values()
            .collect())
    }

    pub fn upsert(&self, record: &PersistedProjectProcessRecord) -> AppResult<()> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_error| AppError::Unexpected)?;
        let mut store = self.load_store_unlocked()?;

        store
            .processes
            .insert(record.project_id.0.clone(), record.clone());

        self.save_store_unlocked(&store)
    }

    pub fn remove(&self, project_id: &ProjectId) -> AppResult<()> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_error| AppError::Unexpected)?;
        let mut store = self.load_store_unlocked()?;

        store.processes.remove(&project_id.0);

        self.save_store_unlocked(&store)
    }

    fn load_store_unlocked(&self) -> AppResult<ProjectProcessRegistryStore> {
        if !self.storage_path.exists() {
            return Ok(ProjectProcessRegistryStore {
                schema_version: PROCESS_REGISTRY_SCHEMA_VERSION,
                processes: BTreeMap::new(),
            });
        }

        let contents = fs::read_to_string(&self.storage_path).map_err(|error| {
            AppError::Infrastructure(format!("failed to read PHP process registry: {error}"))
        })?;

        match serde_json::from_str::<ProjectProcessRegistryStore>(&contents) {
            Ok(mut store) => {
                if store.schema_version == 0 {
                    store.schema_version = PROCESS_REGISTRY_SCHEMA_VERSION;
                }

                Ok(store)
            }
            Err(error) => {
                self.quarantine_corrupt_registry()?;
                tracing::warn!(
                    ?error,
                    registry = %self.storage_path.display(),
                    "quarantined corrupt PHP process registry"
                );

                Ok(ProjectProcessRegistryStore {
                    schema_version: PROCESS_REGISTRY_SCHEMA_VERSION,
                    processes: BTreeMap::new(),
                })
            }
        }
    }

    fn save_store_unlocked(&self, store: &ProjectProcessRegistryStore) -> AppResult<()> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to create PHP process registry directory: {error}"
                ))
            })?;
        }

        let mut normalized_store = ProjectProcessRegistryStore {
            schema_version: PROCESS_REGISTRY_SCHEMA_VERSION,
            processes: store.processes.clone(),
        };
        normalized_store
            .processes
            .retain(|project_id, record| project_id == &record.project_id.0);

        let contents = serde_json::to_string_pretty(&normalized_store).map_err(|error| {
            AppError::Infrastructure(format!("failed to serialize PHP process registry: {error}"))
        })?;
        let temporary_path = temporary_registry_path(&self.storage_path);

        fs::write(&temporary_path, contents).map_err(|error| {
            AppError::Infrastructure(format!("failed to write PHP process registry: {error}"))
        })?;

        fs::rename(&temporary_path, &self.storage_path).map_err(|error| {
            AppError::Infrastructure(format!("failed to replace PHP process registry: {error}"))
        })
    }

    fn quarantine_corrupt_registry(&self) -> AppResult<()> {
        let quarantine_path = self
            .storage_path
            .with_extension(format!("corrupt-{}.json", Utc::now().timestamp_millis()));

        fs::rename(&self.storage_path, quarantine_path).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to quarantine corrupt PHP process registry: {error}"
            ))
        })
    }
}

fn temporary_registry_path(storage_path: &Path) -> PathBuf {
    storage_path.with_extension("tmp")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "axiomphp-process-registry-{name}-{}.json",
            uuid::Uuid::new_v4()
        ))
    }

    fn process_record(project_id: &str, port: u16) -> PersistedProjectProcessRecord {
        PersistedProjectProcessRecord {
            project_id: ProjectId(project_id.to_string()),
            pid: u32::from(port),
            php_version: RuntimeVersion::trusted("8.4"),
            php_binary_path: "/usr/local/bin/php8.4".to_string(),
            port,
            url: format!("http://127.0.0.1:{port}"),
            document_root: format!("/tmp/{project_id}/public"),
            log_file: format!("/tmp/{project_id}/php-server.log"),
            started_at: Utc::now(),
            last_seen_at: Utc::now(),
        }
    }

    #[test]
    fn persists_multiple_project_process_records() {
        let storage_path = test_registry_path("multiple");
        let registry = ProjectProcessRegistry::new(storage_path.clone());

        registry
            .upsert(&process_record("alpha-project", 8501))
            .expect("first record should persist");
        registry
            .upsert(&process_record("beta-project", 8502))
            .expect("second record should persist");

        let restored_registry = ProjectProcessRegistry::new(storage_path.clone());
        let records = restored_registry
            .load_records()
            .expect("records should load");

        assert_eq!(records.len(), 2);
        assert!(records
            .iter()
            .any(|record| record.project_id.0 == "alpha-project"));
        assert!(records
            .iter()
            .any(|record| record.project_id.0 == "beta-project"));

        fs::remove_file(storage_path).expect("registry should be removed");
    }

    #[test]
    fn removes_project_process_record() {
        let storage_path = test_registry_path("remove");
        let registry = ProjectProcessRegistry::new(storage_path.clone());

        registry
            .upsert(&process_record("alpha-project", 8501))
            .expect("record should persist");
        registry
            .remove(&ProjectId("alpha-project".to_string()))
            .expect("record should be removed");

        assert!(registry
            .load_records()
            .expect("records should load")
            .is_empty());

        fs::remove_file(storage_path).expect("registry should be removed");
    }
}

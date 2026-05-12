use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use directories::ProjectDirs;

use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_php_version::ProjectPhpRuntimeSelection;
use crate::domain::runtime::runtime_path::RuntimePath;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug)]
pub struct FileProjectRuntimeRepository {
    storage_path: PathBuf,
    write_lock: Mutex<()>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectRuntimeStore {
    projects: BTreeMap<String, StoredProjectRuntimePreference>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredProjectRuntimePreference {
    php_version: RuntimeVersion,
    #[serde(default)]
    php_binary_path: Option<RuntimePath>,
    #[serde(default)]
    install_requests: Vec<StoredProjectRuntimeInstallRequest>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredProjectRuntimeInstallRequest {
    php_version: RuntimeVersion,
    requested_at: DateTime<Utc>,
}

impl FileProjectRuntimeRepository {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application config directory".to_string())
        })?;

        Ok(Self::with_storage_path(
            project_dirs.config_dir().join("project-runtime.json"),
        ))
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            write_lock: Mutex::new(()),
        }
    }

    fn load_store(&self) -> AppResult<ProjectRuntimeStore> {
        if !self.storage_path.exists() {
            return Ok(ProjectRuntimeStore::default());
        }

        let contents = fs::read_to_string(&self.storage_path).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to read project runtime preferences: {error}"
            ))
        })?;

        serde_json::from_str(&contents).map_err(|error| {
            AppError::Configuration(format!("project runtime preferences are invalid: {error}"))
        })
    }

    fn save_store(&self, store: &ProjectRuntimeStore) -> AppResult<()> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_error| AppError::Unexpected)?;

        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to create project runtime preference directory: {error}"
                ))
            })?;
        }

        let payload = serde_json::to_vec_pretty(store).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to serialize project runtime preferences: {error}"
            ))
        })?;
        let temporary_path = self.storage_path.with_extension("json.tmp");

        fs::write(&temporary_path, payload).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to write project runtime preference file: {error}"
            ))
        })?;

        if self.storage_path.exists() {
            fs::remove_file(&self.storage_path).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to replace project runtime preference file: {error}"
                ))
            })?;
        }

        fs::rename(&temporary_path, &self.storage_path).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to commit project runtime preference file: {error}"
            ))
        })
    }
}

impl ProjectRuntimeRepository for FileProjectRuntimeRepository {
    fn get_php_selection(
        &self,
        project_id: &ProjectId,
    ) -> AppResult<Option<ProjectPhpRuntimeSelection>> {
        Ok(self
            .load_store()?
            .projects
            .get(&project_id.0)
            .and_then(|preference| {
                Some(ProjectPhpRuntimeSelection {
                    php_version: preference.php_version.clone(),
                    php_binary_path: preference.php_binary_path.clone()?,
                })
            }))
    }

    fn save_php_selection(
        &self,
        project_id: &ProjectId,
        selection: &ProjectPhpRuntimeSelection,
    ) -> AppResult<()> {
        let mut store = self.load_store()?;
        let install_requests = store
            .projects
            .get(&project_id.0)
            .map(|preference| preference.install_requests.clone())
            .unwrap_or_default();

        store.projects.insert(
            project_id.0.clone(),
            StoredProjectRuntimePreference {
                php_version: selection.php_version.clone(),
                php_binary_path: Some(selection.php_binary_path.clone()),
                install_requests,
                updated_at: Utc::now(),
            },
        );

        self.save_store(&store)
    }

    fn record_php_install_request(
        &self,
        project_id: &ProjectId,
        version: &RuntimeVersion,
    ) -> AppResult<()> {
        let mut store = self.load_store()?;
        let preference = store
            .projects
            .entry(project_id.0.clone())
            .or_insert_with(|| StoredProjectRuntimePreference {
                php_version: version.clone(),
                php_binary_path: None,
                install_requests: Vec::new(),
                updated_at: Utc::now(),
            });

        preference.php_version = version.clone();
        preference
            .install_requests
            .push(StoredProjectRuntimeInstallRequest {
                php_version: version.clone(),
                requested_at: Utc::now(),
            });
        preference.updated_at = Utc::now();

        self.save_store(&store)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn persists_project_php_version_preferences() {
        let storage_path =
            std::env::temp_dir().join(format!("axiomphp-project-runtime-{}.json", Uuid::new_v4()));
        let repository = FileProjectRuntimeRepository::with_storage_path(storage_path.clone());
        let project_id = ProjectId("current-project".to_string());
        let version = RuntimeVersion::trusted("8.4");
        let selection = ProjectPhpRuntimeSelection {
            php_version: version.clone(),
            php_binary_path: RuntimePath("/usr/local/bin/php8.4".to_string()),
        };

        repository
            .save_php_selection(&project_id, &selection)
            .expect("preference should save");

        let persisted = repository
            .get_php_selection(&project_id)
            .expect("preference should load");

        assert_eq!(persisted, Some(selection));

        let _ = fs::remove_file(storage_path);
    }
}

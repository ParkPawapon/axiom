use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::Utc;
use directories::ProjectDirs;
use uuid::Uuid;

use crate::domain::project::project::Project;
use crate::domain::project::project_config::{CreateProjectRequest, UpdateProjectRequest};
use crate::domain::project::project_id::ProjectId;
use crate::ports::project_repository::ProjectRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug)]
pub struct FileProjectRepository {
    storage_path: PathBuf,
    write_lock: Mutex<()>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectStore {
    projects: BTreeMap<String, Project>,
}

impl FileProjectRepository {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application config directory".to_string())
        })?;

        Ok(Self::with_storage_path(
            project_dirs.config_dir().join("projects.json"),
        ))
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            write_lock: Mutex::new(()),
        }
    }

    fn load_store(&self) -> AppResult<ProjectStore> {
        if !self.storage_path.exists() {
            return Ok(ProjectStore::default());
        }

        let contents = fs::read_to_string(&self.storage_path).map_err(|error| {
            AppError::Infrastructure(format!("failed to read project configuration: {error}"))
        })?;

        serde_json::from_str(&contents).map_err(|error| {
            AppError::Configuration(format!("project configuration is invalid: {error}"))
        })
    }

    fn save_store_unlocked(&self, store: &ProjectStore) -> AppResult<()> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to create project configuration directory: {error}"
                ))
            })?;
        }

        let payload = serde_json::to_vec_pretty(store).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to serialize project configuration: {error}"
            ))
        })?;
        let temporary_path = self.storage_path.with_extension("json.tmp");

        fs::write(&temporary_path, payload).map_err(|error| {
            AppError::Infrastructure(format!("failed to write project configuration: {error}"))
        })?;

        if self.storage_path.exists() {
            fs::remove_file(&self.storage_path).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to replace project configuration: {error}"
                ))
            })?;
        }

        fs::rename(&temporary_path, &self.storage_path).map_err(|error| {
            AppError::Infrastructure(format!("failed to commit project configuration: {error}"))
        })
    }
}

impl ProjectRepository for FileProjectRepository {
    fn list_projects(&self) -> AppResult<Vec<Project>> {
        let mut projects = self
            .load_store()?
            .projects
            .into_values()
            .collect::<Vec<_>>();

        projects.sort_by(|left, right| {
            left.name
                .to_ascii_lowercase()
                .cmp(&right.name.to_ascii_lowercase())
                .then_with(|| left.id.0.cmp(&right.id.0))
        });

        Ok(projects)
    }

    fn get_project(&self, project_id: &ProjectId) -> AppResult<Option<Project>> {
        Ok(self.load_store()?.projects.get(&project_id.0).cloned())
    }

    fn create_project(&self, request: CreateProjectRequest) -> AppResult<Project> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_error| AppError::Unexpected)?;
        let mut store = self.load_store()?;

        ensure_document_root_is_unique(&store, None, &request.document_root.0)?;

        let id = unique_project_id(&store, &request.name);
        let now = Utc::now();
        let project = Project {
            id: id.clone(),
            name: request.name,
            document_root: request.document_root,
            created_at: now,
            updated_at: now,
        };

        store.projects.insert(id.0, project.clone());
        self.save_store_unlocked(&store)?;

        Ok(project)
    }

    fn update_project(
        &self,
        project_id: &ProjectId,
        request: UpdateProjectRequest,
    ) -> AppResult<Project> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_error| AppError::Unexpected)?;
        let mut store = self.load_store()?;

        ensure_document_root_is_unique(&store, Some(project_id), &request.document_root.0)?;

        let existing = store.projects.get_mut(&project_id.0).ok_or_else(|| {
            AppError::NotFound(format!("project `{}` was not found", project_id.0))
        })?;
        existing.name = request.name;
        existing.document_root = request.document_root;
        existing.updated_at = Utc::now();
        let project = existing.clone();

        self.save_store_unlocked(&store)?;

        Ok(project)
    }

    fn delete_project(&self, project_id: &ProjectId) -> AppResult<()> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_error| AppError::Unexpected)?;
        let mut store = self.load_store()?;

        store.projects.remove(&project_id.0).ok_or_else(|| {
            AppError::NotFound(format!("project `{}` was not found", project_id.0))
        })?;

        self.save_store_unlocked(&store)
    }
}

fn ensure_document_root_is_unique(
    store: &ProjectStore,
    current_project_id: Option<&ProjectId>,
    document_root: &str,
) -> AppResult<()> {
    let duplicate = store.projects.values().any(|project| {
        current_project_id.is_none_or(|project_id| project.id != *project_id)
            && project.document_root.0 == document_root
    });

    if duplicate {
        return Err(AppError::Validation(
            "project document root is already registered".to_string(),
        ));
    }

    Ok(())
}

fn unique_project_id(store: &ProjectStore, name: &str) -> ProjectId {
    loop {
        let suffix = Uuid::new_v4().simple().to_string();
        let id = ProjectId(format!(
            "{}-{}",
            project_id_prefix(name),
            suffix.chars().take(8).collect::<String>()
        ));

        if !store.projects.contains_key(&id.0) {
            return id;
        }
    }
}

fn project_id_prefix(name: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_dash = false;

    for character in name.trim().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_was_dash = false;
        } else if !previous_was_dash && !slug.is_empty() {
            slug.push('-');
            previous_was_dash = true;
        }

        if slug.len() >= 48 {
            break;
        }
    }

    let slug = slug.trim_matches('-');

    if slug.is_empty() {
        "project".to_string()
    } else {
        slug.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::domain::project::project_path::ProjectPath;

    #[test]
    fn persists_project_crud() {
        let temp_dir = std::env::temp_dir().join(format!("axiomphp-projects-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let storage_path = temp_dir.join("projects.json");
        let document_root = temp_dir.join("public");
        fs::create_dir_all(&document_root).expect("document root");
        let repository = FileProjectRepository::with_storage_path(storage_path);

        let created = repository
            .create_project(CreateProjectRequest {
                name: "Demo Site".to_string(),
                document_root: ProjectPath(document_root.to_string_lossy().into_owned()),
            })
            .expect("project should save");

        assert_eq!(repository.list_projects().expect("list").len(), 1);

        let updated = repository
            .update_project(
                &created.id,
                UpdateProjectRequest {
                    name: "Demo Site Updated".to_string(),
                    document_root: ProjectPath(document_root.to_string_lossy().into_owned()),
                },
            )
            .expect("project should update");

        assert_eq!(updated.name, "Demo Site Updated");

        repository
            .delete_project(&created.id)
            .expect("project should delete");

        assert!(repository.list_projects().expect("list").is_empty());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn rejects_duplicate_document_roots() {
        let temp_dir = std::env::temp_dir().join(format!("axiomphp-projects-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let document_root = temp_dir.join("public");
        fs::create_dir_all(&document_root).expect("document root");
        let repository = FileProjectRepository::with_storage_path(temp_dir.join("projects.json"));

        repository
            .create_project(CreateProjectRequest {
                name: "One".to_string(),
                document_root: ProjectPath(document_root.to_string_lossy().into_owned()),
            })
            .expect("first project should save");
        let duplicate = repository.create_project(CreateProjectRequest {
            name: "Two".to_string(),
            document_root: ProjectPath(document_root.to_string_lossy().into_owned()),
        });

        assert!(matches!(duplicate, Err(AppError::Validation(_))));

        let _ = fs::remove_dir_all(temp_dir);
    }
}

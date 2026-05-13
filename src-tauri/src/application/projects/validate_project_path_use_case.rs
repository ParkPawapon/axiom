use crate::domain::project::project_path::ProjectPath;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_path::validate_existing_directory_path;

pub fn validate_project_path(document_root: &str) -> AppResult<ProjectPath> {
    let document_root = validate_existing_directory_path(document_root)?;

    Ok(ProjectPath(document_root.to_string_lossy().into_owned()))
}

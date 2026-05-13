use crate::domain::logs::project_log::ProjectLogReadResult;
use crate::domain::project::project_id::ProjectId;
use crate::ports::log_reader::LogReader;
use crate::ports::project_repository::ProjectRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

const DEFAULT_MAX_LOG_LINES: usize = 300;
const MAX_LOG_LINES: usize = 1_000;
const MAX_LOG_QUERY_LENGTH: usize = 120;

pub fn read_project_logs(
    project_repository: &dyn ProjectRepository,
    log_reader: &dyn LogReader,
    project_id: &str,
    max_lines: Option<usize>,
    query: Option<String>,
) -> AppResult<ProjectLogReadResult> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());

    project_repository
        .get_project(&project_id)?
        .ok_or_else(|| AppError::NotFound(format!("project `{}` was not found", project_id.0)))?;

    let max_lines = max_lines
        .unwrap_or(DEFAULT_MAX_LOG_LINES)
        .clamp(1, MAX_LOG_LINES);
    let query = normalize_query(query)?;

    log_reader.read_project_process_log(&project_id, max_lines, query.as_deref())
}

fn normalize_query(query: Option<String>) -> AppResult<Option<String>> {
    let Some(query) = query else {
        return Ok(None);
    };
    let trimmed = query.trim();

    if trimmed.is_empty() {
        return Ok(None);
    }

    if trimmed.chars().any(char::is_control) {
        return Err(AppError::Validation(
            "log query must not contain control characters".to_string(),
        ));
    }

    if trimmed.len() > MAX_LOG_QUERY_LENGTH {
        return Err(AppError::Validation(format!(
            "log query must be {MAX_LOG_QUERY_LENGTH} characters or fewer"
        )));
    }

    Ok(Some(trimmed.to_string()))
}

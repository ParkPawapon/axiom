use std::collections::BTreeSet;
use std::thread;

use serde::Serialize;

use crate::domain::project::project_process::ProjectPhpProcessStatus;
use crate::ports::php_runtime_detector::PhpRuntimeDetector;
use crate::ports::project_php_process_manager::ProjectPhpProcessManager;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::error::error_code::ErrorCode;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

use super::restart_project_php_process_use_case::restart_project_php_process;
use super::start_project_php_process_use_case::start_project_php_process;
use super::stop_project_php_process_use_case::stop_project_php_process;

const MAX_PROJECT_PROCESS_BATCH_SIZE: usize = 12;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPhpProcessActionResult {
    pub project_id: String,
    pub succeeded: bool,
    pub status: Option<ProjectPhpProcessStatus>,
    pub error_code: Option<ErrorCode>,
    pub error_message: Option<String>,
}

pub fn start_project_php_processes(
    project_repository: &dyn ProjectRepository,
    runtime_repository: &dyn ProjectRuntimeRepository,
    detector: &dyn PhpRuntimeDetector,
    process_manager: &dyn ProjectPhpProcessManager,
    project_ids: &[String],
) -> AppResult<Vec<ProjectPhpProcessActionResult>> {
    run_project_process_batch(project_ids, |project_id| {
        start_project_php_process(
            project_repository,
            runtime_repository,
            detector,
            process_manager,
            project_id,
        )
    })
}

pub fn stop_project_php_processes(
    process_manager: &dyn ProjectPhpProcessManager,
    project_ids: &[String],
) -> AppResult<Vec<ProjectPhpProcessActionResult>> {
    run_project_process_batch(project_ids, |project_id| {
        stop_project_php_process(process_manager, project_id)
    })
}

pub fn restart_project_php_processes(
    project_repository: &dyn ProjectRepository,
    runtime_repository: &dyn ProjectRuntimeRepository,
    detector: &dyn PhpRuntimeDetector,
    process_manager: &dyn ProjectPhpProcessManager,
    project_ids: &[String],
) -> AppResult<Vec<ProjectPhpProcessActionResult>> {
    run_project_process_batch(project_ids, |project_id| {
        restart_project_php_process(
            project_repository,
            runtime_repository,
            detector,
            process_manager,
            project_id,
        )
    })
}

fn run_project_process_batch<F>(
    project_ids: &[String],
    action: F,
) -> AppResult<Vec<ProjectPhpProcessActionResult>>
where
    F: Fn(&str) -> AppResult<ProjectPhpProcessStatus> + Sync,
{
    let project_ids = normalize_project_ids(project_ids)?;

    thread::scope(|scope| {
        let handles = project_ids
            .into_iter()
            .map(|project_id| {
                let action = &action;
                scope.spawn(move || {
                    let result = action(&project_id);
                    action_result(project_id, result)
                })
            })
            .collect::<Vec<_>>();

        Ok(handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_panic| ProjectPhpProcessActionResult {
                        project_id: "unknown".to_string(),
                        succeeded: false,
                        status: None,
                        error_code: Some(ErrorCode::Unexpected),
                        error_message: Some("project process action panicked".to_string()),
                    })
            })
            .collect())
    })
}

fn normalize_project_ids(project_ids: &[String]) -> AppResult<Vec<String>> {
    if project_ids.is_empty() {
        return Err(AppError::Validation(
            "select at least one project process action target".to_string(),
        ));
    }

    if project_ids.len() > MAX_PROJECT_PROCESS_BATCH_SIZE {
        return Err(AppError::Validation(format!(
            "project process actions are limited to {MAX_PROJECT_PROCESS_BATCH_SIZE} projects at a time"
        )));
    }

    let mut unique_project_ids = BTreeSet::new();

    for project_id in project_ids {
        unique_project_ids.insert(validate_project_id(project_id)?.to_string());
    }

    Ok(unique_project_ids.into_iter().collect())
}

fn action_result(
    project_id: String,
    result: AppResult<ProjectPhpProcessStatus>,
) -> ProjectPhpProcessActionResult {
    match result {
        Ok(status) => ProjectPhpProcessActionResult {
            project_id,
            succeeded: true,
            status: Some(status),
            error_code: None,
            error_message: None,
        },
        Err(error) => ProjectPhpProcessActionResult {
            project_id,
            succeeded: false,
            status: None,
            error_code: Some(error.code()),
            error_message: Some(error.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_project_batches() {
        let result = normalize_project_ids(&[]);

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn deduplicates_project_batch_ids() {
        let result = normalize_project_ids(&[
            "alpha-project".to_string(),
            "alpha-project".to_string(),
            "beta-project".to_string(),
        ])
        .expect("ids should validate");

        assert_eq!(result, vec!["alpha-project", "beta-project"]);
    }
}

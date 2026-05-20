use crate::domain::logs::project_log::ProjectLogReadResult;
use crate::ports::log_reader::LogReader;
use crate::ports::project_repository::ProjectRepository;
use crate::shared::result::app_result::AppResult;

use super::read_logs_use_case::read_project_logs;

pub fn stream_project_logs(
    project_repository: &dyn ProjectRepository,
    log_reader: &dyn LogReader,
    project_id: &str,
    max_lines: Option<usize>,
    last_line_number: Option<u64>,
    query: Option<String>,
) -> AppResult<ProjectLogReadResult> {
    let mut result =
        read_project_logs(project_repository, log_reader, project_id, max_lines, query)?;
    let Some(last_line_number) = last_line_number else {
        result.status_message = format!(
            "{} Streaming cursor was initialized.",
            result.status_message
        );
        return Ok(result);
    };

    result
        .entries
        .retain(|entry| entry.line_number > last_line_number);
    result.returned_lines = result.entries.len();
    result.status_message = if result.returned_lines == 0 {
        format!("No new log lines after cursor {last_line_number}.")
    } else {
        format!(
            "Streaming {} new project log line(s) after cursor {last_line_number}.",
            result.returned_lines
        )
    };

    Ok(result)
}

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

use directories::ProjectDirs;

use crate::domain::logs::log_entry::LogEntry;
use crate::domain::logs::log_source::LogSource;
use crate::domain::logs::project_log::ProjectLogReadResult;
use crate::domain::project::project_id::ProjectId;
use crate::ports::log_reader::LogReader;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const MAX_LOG_READ_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct FileLogReader {
    project_process_root: PathBuf,
}

impl FileLogReader {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application data directory".to_string())
        })?;

        Ok(Self::with_project_process_root(
            project_dirs.data_local_dir().join("project-processes"),
        ))
    }

    pub fn with_project_process_root(project_process_root: PathBuf) -> Self {
        Self {
            project_process_root,
        }
    }

    fn project_log_path(&self, project_id: &ProjectId) -> PathBuf {
        self.project_process_root
            .join(&project_id.0)
            .join("php-server.log")
    }
}

impl LogReader for FileLogReader {
    fn read_project_process_log(
        &self,
        project_id: &ProjectId,
        max_lines: usize,
        query: Option<&str>,
    ) -> AppResult<ProjectLogReadResult> {
        let log_file = self.project_log_path(project_id);
        let log_file_display = log_file.to_string_lossy().into_owned();

        if !log_file.exists() {
            return Ok(ProjectLogReadResult {
                project_id: project_id.clone(),
                log_file: log_file_display,
                entries: Vec::new(),
                returned_lines: 0,
                file_size_bytes: 0,
                truncated: false,
                status_message: "No PHP process log file exists for this project yet.".to_string(),
            });
        }

        let mut file = File::open(&log_file).map_err(|error| {
            AppError::Infrastructure(format!("failed to open project log file: {error}"))
        })?;
        let file_size_bytes = file
            .metadata()
            .map_err(|error| {
                AppError::Infrastructure(format!("failed to inspect project log file: {error}"))
            })?
            .len();
        let truncated_by_size = file_size_bytes > MAX_LOG_READ_BYTES;

        if truncated_by_size {
            file.seek(SeekFrom::Start(file_size_bytes - MAX_LOG_READ_BYTES))
                .map_err(|error| {
                    AppError::Infrastructure(format!("failed to seek project log file: {error}"))
                })?;
        }

        let mut reader = BufReader::new(file);

        if truncated_by_size {
            let mut discarded_partial_line = String::new();
            reader
                .read_line(&mut discarded_partial_line)
                .map_err(|error| {
                    AppError::Infrastructure(format!(
                        "failed to align project log file to a line boundary: {error}"
                    ))
                })?;
        }

        let mut visible_lines = VecDeque::with_capacity(max_lines);
        let mut line_number = 0_u64;
        let mut matched_lines = 0_usize;
        let mut dropped_by_line_limit = false;
        let normalized_query = query.map(str::to_ascii_lowercase);

        for line in reader.lines() {
            let line = line.map_err(|error| {
                AppError::Infrastructure(format!("failed to read project log file: {error}"))
            })?;
            line_number += 1;

            if normalized_query
                .as_ref()
                .is_some_and(|query| !line.to_ascii_lowercase().contains(query))
            {
                continue;
            }

            matched_lines += 1;

            if visible_lines.len() == max_lines {
                visible_lines.pop_front();
                dropped_by_line_limit = true;
            }

            visible_lines.push_back((line_number, line));
        }

        let entries = visible_lines
            .into_iter()
            .map(|(line_number, line)| {
                LogEntry::from_raw_line(line_number, LogSource(project_id.0.clone()), line)
            })
            .collect::<Vec<_>>();
        let returned_lines = entries.len();
        let truncated = truncated_by_size || dropped_by_line_limit;
        let status_message = if returned_lines == 0 {
            "The project log file exists, but no lines match the current filter.".to_string()
        } else if truncated {
            format!("Showing the latest {returned_lines} of {matched_lines} matching project log lines.")
        } else {
            format!("Showing {returned_lines} project log lines.")
        };

        Ok(ProjectLogReadResult {
            project_id: project_id.clone(),
            log_file: log_file_display,
            entries,
            returned_lines,
            file_size_bytes,
            truncated,
            status_message,
        })
    }
}

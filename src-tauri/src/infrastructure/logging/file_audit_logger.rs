use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use chrono::{Duration, Utc};
use directories::ProjectDirs;

use crate::domain::security::audit_log::{
    AuditLogEntry, AuditLogReadResult, AuditLogRetentionResult,
};
use crate::ports::audit_logger::AuditLogger;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const DEFAULT_RETENTION_DAYS: u16 = 30;
const MAX_AUDIT_ENTRIES: usize = 1_000;

#[derive(Debug, Clone)]
pub struct FileAuditLogger {
    log_file: PathBuf,
    retention_days: u16,
}

impl FileAuditLogger {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application data directory".to_string())
        })?;

        Ok(Self {
            log_file: project_dirs
                .data_local_dir()
                .join("security")
                .join("audit")
                .join("audit.ndjson"),
            retention_days: DEFAULT_RETENTION_DAYS,
        })
    }

    pub fn with_log_file(log_file: PathBuf, retention_days: u16) -> Self {
        Self {
            log_file,
            retention_days,
        }
    }

    fn ensure_parent_dir(&self) -> AppResult<()> {
        let parent = self.log_file.parent().ok_or_else(|| {
            AppError::Configuration("audit log path does not have a parent directory".to_string())
        })?;

        fs::create_dir_all(parent).map_err(|error| {
            AppError::Infrastructure(format!("failed to create audit log directory: {error}"))
        })
    }

    fn read_all_entries(&self) -> AppResult<Vec<AuditLogEntry>> {
        if !self.log_file.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.log_file).map_err(|error| {
            AppError::Infrastructure(format!("failed to open audit log file: {error}"))
        })?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|error| {
                AppError::Infrastructure(format!("failed to read audit log file: {error}"))
            })?;
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(entry) = serde_json::from_str::<AuditLogEntry>(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn write_entries(&self, entries: &[AuditLogEntry]) -> AppResult<()> {
        self.ensure_parent_dir()?;
        let mut file = File::create(&self.log_file).map_err(|error| {
            AppError::Infrastructure(format!("failed to rewrite audit log file: {error}"))
        })?;

        for entry in entries {
            let line = serde_json::to_string(entry).map_err(|error| {
                AppError::Infrastructure(format!("failed to serialize audit log entry: {error}"))
            })?;
            writeln!(file, "{line}").map_err(|error| {
                AppError::Infrastructure(format!("failed to write audit log entry: {error}"))
            })?;
        }

        Ok(())
    }
}

impl AuditLogger for FileAuditLogger {
    fn record(&self, entry: AuditLogEntry) -> AppResult<()> {
        self.ensure_parent_dir()?;
        let line = serde_json::to_string(&entry).map_err(|error| {
            AppError::Infrastructure(format!("failed to serialize audit log entry: {error}"))
        })?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .map_err(|error| {
                AppError::Infrastructure(format!("failed to open audit log file: {error}"))
            })?;

        writeln!(file, "{line}").map_err(|error| {
            AppError::Infrastructure(format!("failed to write audit log entry: {error}"))
        })
    }

    fn read(&self, max_entries: usize) -> AppResult<AuditLogReadResult> {
        let max_entries = max_entries.clamp(1, MAX_AUDIT_ENTRIES);
        let entries = self.read_all_entries()?;
        let total_entries = entries.len();
        let mut visible_entries = VecDeque::with_capacity(max_entries);

        for entry in entries {
            if visible_entries.len() == max_entries {
                visible_entries.pop_front();
            }
            visible_entries.push_back(entry);
        }

        let entries = visible_entries.into_iter().rev().collect::<Vec<_>>();
        let returned_entries = entries.len();

        Ok(AuditLogReadResult {
            entries,
            returned_entries,
            retention_days: self.retention_days,
            log_file: self.log_file.to_string_lossy().into_owned(),
            truncated: total_entries > returned_entries,
            status_message: format!("Showing {returned_entries} security audit entries."),
        })
    }

    fn prune(&self, retention_days: u16) -> AppResult<AuditLogRetentionResult> {
        let retention_days = retention_days.clamp(1, 365);
        let cutoff = Utc::now() - Duration::days(i64::from(retention_days));
        let entries = self.read_all_entries()?;
        let original_count = entries.len();
        let retained_entries = entries
            .into_iter()
            .filter(|entry| entry.timestamp >= cutoff)
            .collect::<Vec<_>>();
        let retained_count = retained_entries.len();

        self.write_entries(&retained_entries)?;

        Ok(AuditLogRetentionResult {
            removed_entries: original_count.saturating_sub(retained_count),
            retained_entries: retained_count,
            retention_days,
            log_file: self.log_file.to_string_lossy().into_owned(),
            status_message: format!("Audit log retention applied for {retention_days} days."),
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::domain::security::audit_log::AuditSeverity;

    use super::*;

    #[test]
    fn records_and_reads_audit_entries() {
        let log_file = std::env::temp_dir().join(format!("axiom-audit-{}.ndjson", Uuid::new_v4()));
        let logger = FileAuditLogger::with_log_file(log_file.clone(), 30);

        logger
            .record(AuditLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                actor: "local-user".to_string(),
                operation: "test".to_string(),
                resource: "resource".to_string(),
                severity: AuditSeverity::Info,
                status: "completed".to_string(),
                message: "recorded".to_string(),
            })
            .expect("audit entry should be recorded");

        let result = logger.read(10).expect("audit log should be readable");

        assert_eq!(result.returned_entries, 1);
        let _ = fs::remove_file(log_file);
    }
}

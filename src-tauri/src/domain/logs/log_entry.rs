use super::log_level::LogLevel;
use super::log_source::LogSource;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: String,
    pub line_number: u64,
    pub level: LogLevel,
    pub source: LogSource,
    pub message: String,
    pub raw: String,
}

impl LogEntry {
    pub fn from_raw_line(line_number: u64, source: LogSource, raw: String) -> Self {
        let level = infer_level(&raw);
        let message = strip_php_timestamp(&raw).trim().to_string();

        Self {
            id: format!("{}-{line_number}", source.0),
            line_number,
            level,
            source,
            message: if message.is_empty() {
                raw.clone()
            } else {
                message
            },
            raw,
        }
    }
}

fn infer_level(line: &str) -> LogLevel {
    let line = line.to_ascii_lowercase();

    if line.contains("fatal") || line.contains("error") || line.contains("failed") {
        return LogLevel::Error;
    }

    if line.contains("warning") || line.contains("warn") || line.contains("deprecated") {
        return LogLevel::Warn;
    }

    if line.contains("debug") {
        return LogLevel::Debug;
    }

    LogLevel::Info
}

fn strip_php_timestamp(line: &str) -> &str {
    let Some(stripped) = line.strip_prefix('[') else {
        return line;
    };

    let Some((_timestamp, message)) = stripped.split_once("] ") else {
        return line;
    };

    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_error_level_from_php_log_line() {
        let entry = LogEntry::from_raw_line(
            12,
            LogSource("project".to_string()),
            "[Wed May 13 10:00:00 2026] PHP Fatal error: boom".to_string(),
        );

        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.message, "PHP Fatal error: boom");
    }
}

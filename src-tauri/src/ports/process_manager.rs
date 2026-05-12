use crate::domain::security::command_policy::{ProcessCommand, ProcessOutput};
use crate::shared::result::app_result::AppResult;

pub trait ProcessManager: Send + Sync {
    fn execute(&self, command: ProcessCommand) -> AppResult<ProcessOutput>;
}

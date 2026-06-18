use crate::domain::control_center::setup_diagnostic::SetupDiagnosticsReport;
use crate::shared::result::app_result::AppResult;

use super::get_control_center_summary_use_case::{
    get_control_center_summary, ControlCenterDependencies,
};

pub fn get_setup_diagnostics(
    dependencies: ControlCenterDependencies<'_>,
    selected_project_id: Option<String>,
) -> AppResult<SetupDiagnosticsReport> {
    Ok(get_control_center_summary(dependencies, selected_project_id)?.setup)
}

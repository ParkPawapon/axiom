use chrono::Utc;

use crate::domain::system::system_status::PermissionCheckResult;
use crate::ports::permission_manager::PermissionManager;
use crate::shared::result::app_result::AppResult;

pub fn check_permissions(
    permission_manager: &dyn PermissionManager,
) -> AppResult<PermissionCheckResult> {
    let permissions = permission_manager.inspect_security_permissions()?;
    let status_message = permissions.status_message.clone();

    Ok(PermissionCheckResult {
        permissions,
        checked_at: Utc::now(),
        status_message,
    })
}

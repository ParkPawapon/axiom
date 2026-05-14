use crate::domain::security::security_status::SecurityPermissionStatus;
use crate::ports::permission_manager::PermissionManager;
use crate::shared::result::app_result::AppResult;

pub fn get_security_status(
    permission_manager: &dyn PermissionManager,
) -> AppResult<SecurityPermissionStatus> {
    permission_manager.inspect_security_permissions()
}

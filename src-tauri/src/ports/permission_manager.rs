use crate::domain::security::security_status::SecurityPermissionStatus;
use crate::shared::result::app_result::AppResult;

pub trait PermissionManager: Send + Sync {
    fn inspect_security_permissions(&self) -> AppResult<SecurityPermissionStatus>;
}

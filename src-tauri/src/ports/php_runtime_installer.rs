use crate::domain::project::project_php_version::PhpRuntimeInstallProvider;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::shared::result::app_result::AppResult;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PhpRuntimeInstallReport {
    pub provider: PhpRuntimeInstallProvider,
    pub package_name: String,
    pub status_message: String,
}

pub trait PhpRuntimeInstaller: Send + Sync {
    fn install_php_runtime(&self, version: &RuntimeVersion) -> AppResult<PhpRuntimeInstallReport>;
}

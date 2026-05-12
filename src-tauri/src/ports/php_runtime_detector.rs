use crate::domain::runtime::php_runtime::DetectedPhpBinary;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::shared::result::app_result::AppResult;

pub trait PhpRuntimeDetector: Send + Sync {
    fn detect_php_binary(&self, version: &RuntimeVersion) -> AppResult<Option<DetectedPhpBinary>>;
}

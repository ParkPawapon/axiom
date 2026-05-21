use crate::domain::runtime::php_runtime::{supported_php_versions_catalog, PhpRuntime};
use crate::ports::php_runtime_detector::PhpRuntimeDetector;
use crate::shared::result::app_result::AppResult;

pub fn detect_php_runtimes(detector: &dyn PhpRuntimeDetector) -> AppResult<Vec<PhpRuntime>> {
    supported_php_versions_catalog()
        .into_iter()
        .map(
            |runtime| match detector.detect_php_binary(&runtime.version)? {
                Some(binary) => Ok(runtime.with_detected_binary(binary)),
                None => Ok(runtime),
            },
        )
        .collect()
}

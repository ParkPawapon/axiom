use crate::domain::runtime::php_runtime::{
    is_supported_php_version, supported_php_versions_catalog,
};
use crate::domain::runtime::runtime_validation::RuntimeValidationResult;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::ports::php_runtime_detector::PhpRuntimeDetector;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub fn validate_runtime(
    detector: &dyn PhpRuntimeDetector,
    php_version: &str,
) -> AppResult<RuntimeValidationResult> {
    let php_version = RuntimeVersion::new(php_version)?;

    if !is_supported_php_version(&php_version) {
        return Err(AppError::Validation(format!(
            "PHP {} is not in the supported runtime catalog",
            php_version.as_str()
        )));
    }

    let runtime = supported_php_versions_catalog()
        .into_iter()
        .find(|runtime| runtime.version == php_version)
        .ok_or(AppError::Unexpected)?;
    let detected_binary = detector.detect_php_binary(&php_version)?;
    let runtime = detected_binary
        .clone()
        .map(|binary| runtime.clone().with_detected_binary(binary))
        .unwrap_or(runtime);
    let valid = detected_binary.is_some();
    let status_message = if valid {
        format!(
            "PHP {} is installed and can be selected.",
            php_version.as_str()
        )
    } else {
        format!(
            "PHP {} is supported but no matching binary was detected.",
            php_version.as_str()
        )
    };

    Ok(RuntimeValidationResult {
        runtime,
        detected_binary,
        valid,
        status_message,
    })
}

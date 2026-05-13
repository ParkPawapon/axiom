use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_php_version::{
    ProjectPhpInstallResult, ProjectPhpRuntimeSelection,
};
use crate::domain::runtime::php_runtime::is_supported_php_version;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::ports::php_runtime_detector::PhpRuntimeDetector;
use crate::ports::php_runtime_installer::PhpRuntimeInstaller;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn install_project_php_runtime(
    repository: &dyn ProjectRuntimeRepository,
    detector: &dyn PhpRuntimeDetector,
    installer: &dyn PhpRuntimeInstaller,
    project_id: &str,
    php_version: &str,
) -> AppResult<ProjectPhpInstallResult> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let php_version = RuntimeVersion::new(php_version)?;

    if !is_supported_php_version(&php_version) {
        return Err(AppError::Validation(format!(
            "PHP {} is not in the supported project runtime catalog",
            php_version.as_str()
        )));
    }

    repository.record_php_install_request(&project_id, &php_version)?;

    if let Some(binary) = detector.detect_php_binary(&php_version)? {
        repository.save_php_selection(
            &project_id,
            &ProjectPhpRuntimeSelection {
                php_version: php_version.clone(),
                php_binary_path: binary.path.clone(),
            },
        )?;

        return Ok(ProjectPhpInstallResult {
            project_id,
            php_version,
            provider: provider_for_current_platform()?,
            package_name: "already-installed".to_string(),
            selected_php_binary: Some(binary),
            diagnostics: Vec::new(),
            rollback: None,
            status_message:
                "Matching PHP binary is already installed. Project runtime selection was updated."
                    .to_string(),
        });
    }

    let install_report = installer.install_php_runtime(&php_version)?;
    let selected_php_binary = detector.detect_php_binary(&php_version)?;

    if let Some(binary) = &selected_php_binary {
        repository.save_php_selection(
            &project_id,
            &ProjectPhpRuntimeSelection {
                php_version: php_version.clone(),
                php_binary_path: binary.path.clone(),
            },
        )?;
    }

    let status_message = if selected_php_binary.is_some() {
        format!(
            "{} Matching PHP binary was detected after installation and selected for this project.",
            install_report.status_message
        )
    } else {
        format!(
            "{} Installation completed, but the PHP binary is not discoverable yet. Restart AxiomPHP or add the installed binary directory to PATH before switching.",
            install_report.status_message
        )
    };

    Ok(ProjectPhpInstallResult {
        project_id,
        php_version,
        provider: install_report.provider,
        package_name: install_report.package_name,
        selected_php_binary,
        diagnostics: install_report.diagnostics,
        rollback: install_report.rollback,
        status_message,
    })
}

fn provider_for_current_platform(
) -> AppResult<crate::domain::project::project_php_version::PhpRuntimeInstallProvider> {
    if cfg!(target_os = "macos") {
        return Ok(
            crate::domain::project::project_php_version::PhpRuntimeInstallProvider::Homebrew,
        );
    }

    if cfg!(windows) {
        return Ok(crate::domain::project::project_php_version::PhpRuntimeInstallProvider::Scoop);
    }

    Err(AppError::Configuration(
        "automatic PHP installation is currently supported only on macOS with Homebrew and Windows with Scoop".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::application::projects::get_project_php_version_use_case;
    use crate::domain::project::project_id::ProjectId;
    use crate::domain::project::project_php_version::PhpRuntimeInstallProvider;
    use crate::domain::runtime::php_runtime::DetectedPhpBinary;
    use crate::domain::runtime::runtime_path::RuntimePath;
    use crate::ports::php_runtime_installer::PhpRuntimeInstallReport;

    #[derive(Debug, Default)]
    struct MemoryProjectRuntimeRepository {
        selected: Mutex<Option<ProjectPhpRuntimeSelection>>,
        install_requests: Mutex<Vec<RuntimeVersion>>,
    }

    impl ProjectRuntimeRepository for MemoryProjectRuntimeRepository {
        fn get_php_selection(
            &self,
            _project_id: &ProjectId,
        ) -> AppResult<Option<ProjectPhpRuntimeSelection>> {
            Ok(self
                .selected
                .lock()
                .map_err(|_error| AppError::Unexpected)?
                .clone())
        }

        fn save_php_selection(
            &self,
            _project_id: &ProjectId,
            selection: &ProjectPhpRuntimeSelection,
        ) -> AppResult<()> {
            *self
                .selected
                .lock()
                .map_err(|_error| AppError::Unexpected)? = Some(selection.clone());
            Ok(())
        }

        fn record_php_install_request(
            &self,
            _project_id: &ProjectId,
            version: &RuntimeVersion,
        ) -> AppResult<()> {
            self.install_requests
                .lock()
                .map_err(|_error| AppError::Unexpected)?
                .push(version.clone());
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    struct MemoryPhpRuntimeDetector {
        detected: Mutex<bool>,
    }

    impl PhpRuntimeDetector for MemoryPhpRuntimeDetector {
        fn detect_php_binary(
            &self,
            version: &RuntimeVersion,
        ) -> AppResult<Option<DetectedPhpBinary>> {
            if !*self
                .detected
                .lock()
                .map_err(|_error| AppError::Unexpected)?
            {
                return Ok(None);
            }

            Ok(Some(DetectedPhpBinary {
                version: version.clone(),
                path: RuntimePath(format!("/usr/local/bin/php{}", version.as_str())),
                display_name: format!("php{} test binary", version.as_str()),
            }))
        }
    }

    #[derive(Debug)]
    struct MemoryPhpRuntimeInstaller<'a> {
        detector: &'a MemoryPhpRuntimeDetector,
    }

    impl PhpRuntimeInstaller for MemoryPhpRuntimeInstaller<'_> {
        fn install_php_runtime(
            &self,
            version: &RuntimeVersion,
        ) -> AppResult<PhpRuntimeInstallReport> {
            *self
                .detector
                .detected
                .lock()
                .map_err(|_error| AppError::Unexpected)? = true;

            Ok(PhpRuntimeInstallReport {
                provider: PhpRuntimeInstallProvider::Homebrew,
                package_name: format!("php@{}", version.as_str()),
                diagnostics: Vec::new(),
                rollback: None,
                status_message: "installed".to_string(),
            })
        }
    }

    #[test]
    fn installs_and_selects_detected_binary() {
        let repository = MemoryProjectRuntimeRepository::default();
        let detector = MemoryPhpRuntimeDetector::default();
        let installer = MemoryPhpRuntimeInstaller {
            detector: &detector,
        };

        let result = install_project_php_runtime(
            &repository,
            &detector,
            &installer,
            "current-project",
            "8.4",
        )
        .expect("install should succeed");

        assert_eq!(result.php_version.as_str(), "8.4");
        assert!(result.selected_php_binary.is_some());

        let config = get_project_php_version_use_case::get_project_php_version(
            &repository,
            &detector,
            "current-project",
        )
        .expect("config should load");

        assert_eq!(config.selected_php_version.as_str(), "8.4");
    }
}

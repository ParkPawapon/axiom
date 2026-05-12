use super::runtime_version::RuntimeVersion;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhpRuntime {
    pub version: RuntimeVersion,
    pub label: String,
    pub support_phase: PhpVersionSupportPhase,
    pub recommended: bool,
    pub installed: bool,
    pub binary_display_name: Option<String>,
    pub can_switch: bool,
    pub requires_manual_install_confirmation: bool,
    pub lifecycle_warning: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PhpVersionSupportPhase {
    Active,
    EndOfLife,
    Security,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedPhpBinary {
    pub version: RuntimeVersion,
    pub path: super::runtime_path::RuntimePath,
    pub display_name: String,
}

impl PhpRuntime {
    pub fn with_detected_binary(mut self, binary: DetectedPhpBinary) -> Self {
        self.installed = true;
        self.binary_display_name = Some(binary.display_name);
        self.can_switch = true;
        self
    }
}

pub fn supported_php_versions_catalog() -> Vec<PhpRuntime> {
    vec![
        php_runtime("8.5", PhpVersionSupportPhase::Active, true),
        php_runtime("8.4", PhpVersionSupportPhase::Active, false),
        php_runtime("8.3", PhpVersionSupportPhase::Security, false),
        php_runtime("8.2", PhpVersionSupportPhase::Security, false),
        php_runtime("8.1", PhpVersionSupportPhase::EndOfLife, false),
        php_runtime("8.0", PhpVersionSupportPhase::EndOfLife, false),
        php_runtime("7.4", PhpVersionSupportPhase::EndOfLife, false),
        php_runtime("7.3", PhpVersionSupportPhase::EndOfLife, false),
        php_runtime("7.2", PhpVersionSupportPhase::EndOfLife, false),
        php_runtime("7.1", PhpVersionSupportPhase::EndOfLife, false),
        php_runtime("7.0", PhpVersionSupportPhase::EndOfLife, false),
        php_runtime("5.6", PhpVersionSupportPhase::EndOfLife, false),
    ]
}

pub fn default_php_version() -> RuntimeVersion {
    RuntimeVersion::trusted("8.5")
}

pub fn is_supported_php_version(version: &RuntimeVersion) -> bool {
    supported_php_versions_catalog()
        .iter()
        .any(|runtime| runtime.version == *version)
}

fn php_runtime(
    version: &'static str,
    support_phase: PhpVersionSupportPhase,
    recommended: bool,
) -> PhpRuntime {
    PhpRuntime {
        version: RuntimeVersion::trusted(version),
        label: format!("PHP {version}"),
        support_phase,
        recommended,
        installed: false,
        binary_display_name: None,
        can_switch: false,
        requires_manual_install_confirmation: requires_manual_install_confirmation(version),
        lifecycle_warning: lifecycle_warning(version, support_phase),
    }
}

fn requires_manual_install_confirmation(version: &str) -> bool {
    version_major(version).is_some_and(|major| major <= 8)
}

fn lifecycle_warning(
    version: &'static str,
    support_phase: PhpVersionSupportPhase,
) -> Option<String> {
    match support_phase {
        PhpVersionSupportPhase::Active => None,
        PhpVersionSupportPhase::Security => Some(format!(
            "PHP {version} is in security support. Use it only when the project requires this branch."
        )),
        PhpVersionSupportPhase::EndOfLife => Some(format!(
            "PHP {version} is end-of-life and no longer receives official PHP security updates."
        )),
    }
}

fn version_major(version: &str) -> Option<u16> {
    version.split('.').next()?.parse().ok()
}

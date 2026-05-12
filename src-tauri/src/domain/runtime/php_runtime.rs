use super::runtime_version::RuntimeVersion;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhpRuntime {
    pub version: RuntimeVersion,
    pub label: String,
    pub support_phase: PhpVersionSupportPhase,
    pub recommended: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PhpVersionSupportPhase {
    Active,
    Security,
}

pub fn supported_php_versions() -> Vec<PhpRuntime> {
    vec![
        php_runtime("8.5", PhpVersionSupportPhase::Active, true),
        php_runtime("8.4", PhpVersionSupportPhase::Active, false),
        php_runtime("8.3", PhpVersionSupportPhase::Security, false),
        php_runtime("8.2", PhpVersionSupportPhase::Security, false),
    ]
}

pub fn default_php_version() -> RuntimeVersion {
    RuntimeVersion::trusted("8.5")
}

pub fn is_supported_php_version(version: &RuntimeVersion) -> bool {
    supported_php_versions()
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
    }
}

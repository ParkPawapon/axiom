use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExecutableResolver {
    search_paths: Vec<PathBuf>,
}

impl ExecutableResolver {
    pub fn from_env() -> Self {
        let search_paths = env::var_os("PATH")
            .map(|paths| env::split_paths(&paths).collect())
            .unwrap_or_default();

        Self { search_paths }
    }

    #[cfg(test)]
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Self { search_paths }
    }

    pub fn resolve_first(&self, program_names: &[&str]) -> Option<PathBuf> {
        program_names
            .iter()
            .find_map(|program_name| self.resolve(program_name))
    }

    pub fn resolve(&self, program_name: &str) -> Option<PathBuf> {
        if program_name.trim().is_empty()
            || program_name.contains(std::path::MAIN_SEPARATOR)
            || program_name.contains('/')
            || program_name.contains('\\')
        {
            return None;
        }

        self.search_paths.iter().find_map(|directory| {
            candidate_names(program_name)
                .into_iter()
                .find_map(|candidate| {
                    let candidate_path = directory.join(candidate);
                    executable_path(&candidate_path)
                })
        })
    }
}

fn executable_path(path: &Path) -> Option<PathBuf> {
    let metadata = path.metadata().ok()?;

    if !metadata.is_file() || !is_executable(&metadata) {
        return None;
    }

    path.canonicalize()
        .ok()
        .or_else(|| Some(path.to_path_buf()))
}

#[cfg(unix)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;

    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &std::fs::Metadata) -> bool {
    true
}

#[cfg(windows)]
fn candidate_names(program_name: &str) -> Vec<String> {
    if Path::new(program_name).extension().is_some() {
        return vec![program_name.to_string()];
    }

    vec![format!("{program_name}.exe")]
}

#[cfg(not(windows))]
fn candidate_names(program_name: &str) -> Vec<String> {
    vec![program_name.to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_program_names_with_path_separators() {
        let resolver = ExecutableResolver::new(vec![PathBuf::from("/usr/bin")]);

        assert!(resolver.resolve("../php").is_none());
        assert!(resolver.resolve("/usr/bin/php").is_none());
    }
}

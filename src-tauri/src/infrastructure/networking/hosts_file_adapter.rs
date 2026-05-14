use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::Utc;
use directories::ProjectDirs;

use crate::domain::networking::host_entry::{HostFileEntry, HostFileUpdateResult};
use crate::domain::security::elevation::{PermissionElevationKind, PermissionElevationRequest};
use crate::ports::hosts_file_manager::HostsFileManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const BEGIN_MARKER: &str = "# AxiomPHP managed hosts begin";
const END_MARKER: &str = "# AxiomPHP managed hosts end";

#[derive(Debug, Clone)]
pub struct HostsFileAdapter {
    hosts_file_path: PathBuf,
    security_dir: PathBuf,
}

impl HostsFileAdapter {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application data directory".to_string())
        })?;

        Ok(Self {
            hosts_file_path: default_hosts_file_path(),
            security_dir: project_dirs.data_local_dir().join("security"),
        })
    }

    pub fn with_paths(hosts_file_path: PathBuf, security_dir: PathBuf) -> Self {
        Self {
            hosts_file_path,
            security_dir,
        }
    }

    pub fn hosts_file_path(&self) -> PathBuf {
        self.hosts_file_path.clone()
    }

    fn backup_dir(&self) -> PathBuf {
        self.security_dir.join("hosts-backups")
    }

    fn pending_dir(&self) -> PathBuf {
        self.security_dir.join("pending-hosts")
    }

    fn build_next_content(current_content: &str, entry: &HostFileEntry) -> (String, bool) {
        let managed_line = format!("{} {}\n", entry.address, entry.domain);
        let managed_block = format!("{BEGIN_MARKER}\n{managed_line}{END_MARKER}\n");

        if current_content.lines().any(|line| {
            line.split_whitespace().collect::<Vec<_>>()
                == [entry.address.as_str(), entry.domain.as_str()]
        }) {
            return (current_content.to_string(), false);
        }

        if let (Some(begin), Some(end)) = (
            current_content.find(BEGIN_MARKER),
            current_content.find(END_MARKER),
        ) {
            let end_index = end + END_MARKER.len();
            let mut next = String::new();
            next.push_str(&current_content[..begin]);
            next.push_str(&managed_block);
            next.push_str(current_content[end_index..].trim_start_matches('\n'));
            return (ensure_trailing_newline(next), true);
        }

        let mut next = ensure_trailing_newline(current_content.to_string());
        if !next.ends_with("\n\n") {
            next.push('\n');
        }
        next.push_str(&managed_block);

        (next, true)
    }

    fn write_support_files(
        &self,
        current_content: &str,
        next_content: &str,
    ) -> AppResult<(String, String)> {
        fs::create_dir_all(self.backup_dir()).map_err(|error| {
            AppError::Infrastructure(format!("failed to create hosts backup directory: {error}"))
        })?;
        fs::create_dir_all(self.pending_dir()).map_err(|error| {
            AppError::Infrastructure(format!("failed to create pending hosts directory: {error}"))
        })?;

        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let backup_path = self.backup_dir().join(format!("hosts-{timestamp}.bak"));
        let prepared_hosts_path = self
            .pending_dir()
            .join(format!("hosts-{timestamp}.prepared"));

        fs::write(&backup_path, current_content).map_err(|error| {
            AppError::Infrastructure(format!("failed to write hosts backup: {error}"))
        })?;
        fs::write(&prepared_hosts_path, next_content).map_err(|error| {
            AppError::Infrastructure(format!("failed to write prepared hosts file: {error}"))
        })?;

        Ok((
            backup_path.to_string_lossy().into_owned(),
            prepared_hosts_path.to_string_lossy().into_owned(),
        ))
    }

    fn elevation_request(&self, prepared_hosts_path: &str) -> PermissionElevationRequest {
        PermissionElevationRequest {
            kind: PermissionElevationKind::HostFileWrite,
            title: "Administrator approval required".to_string(),
            reason: "The operating system hosts file is protected and requires administrator approval to update local development domains.".to_string(),
            command_preview: host_file_elevation_command(prepared_hosts_path, &self.hosts_file_path),
            requires_admin: true,
            status_message: "Review the prepared hosts file before applying it with administrator privileges.".to_string(),
        }
    }
}

impl HostsFileManager for HostsFileAdapter {
    fn apply_entry(&self, entry: HostFileEntry) -> AppResult<HostFileUpdateResult> {
        let hosts_file_display = self.hosts_file_path.to_string_lossy().into_owned();
        let current_content = fs::read_to_string(&self.hosts_file_path).map_err(|error| {
            AppError::Infrastructure(format!("failed to read hosts file: {error}"))
        })?;
        let (next_content, changed) = Self::build_next_content(&current_content, &entry);

        if !changed {
            return Ok(HostFileUpdateResult {
                entry,
                hosts_file_path: hosts_file_display,
                backup_path: None,
                prepared_hosts_path: None,
                updated: false,
                requires_elevation: false,
                elevation: None,
                status_message: "Hosts file already contains the requested local mapping."
                    .to_string(),
            });
        }

        let (backup_path, prepared_hosts_path) =
            self.write_support_files(&current_content, &next_content)?;

        match fs::write(&self.hosts_file_path, next_content) {
            Ok(()) => Ok(HostFileUpdateResult {
                entry,
                hosts_file_path: hosts_file_display,
                backup_path: Some(backup_path),
                prepared_hosts_path: Some(prepared_hosts_path),
                updated: true,
                requires_elevation: false,
                elevation: None,
                status_message: "Hosts file was updated with the managed local domain mapping."
                    .to_string(),
            }),
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                let elevation = self.elevation_request(&prepared_hosts_path);
                Ok(HostFileUpdateResult {
                    entry,
                    hosts_file_path: hosts_file_display,
                    backup_path: Some(backup_path),
                    prepared_hosts_path: Some(prepared_hosts_path),
                    updated: false,
                    requires_elevation: true,
                    elevation: Some(elevation),
                    status_message:
                        "Hosts file update is prepared and waiting for administrator approval."
                            .to_string(),
                })
            }
            Err(error) => Err(AppError::Infrastructure(format!(
                "failed to write hosts file: {error}"
            ))),
        }
    }
}

fn default_hosts_file_path() -> PathBuf {
    if cfg!(windows) {
        std::env::var_os("SystemRoot")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:\\Windows"))
            .join("System32")
            .join("drivers")
            .join("etc")
            .join("hosts")
    } else {
        PathBuf::from("/etc/hosts")
    }
}

fn ensure_trailing_newline(mut content: String) -> String {
    if !content.ends_with('\n') {
        content.push('\n');
    }

    content
}

fn host_file_elevation_command(prepared_hosts_path: &str, hosts_file_path: &Path) -> Vec<String> {
    if cfg!(windows) {
        return vec![
            "Run an elevated terminal.".to_string(),
            format!(
                "copy /Y \"{}\" \"{}\"",
                prepared_hosts_path,
                hosts_file_path.to_string_lossy()
            ),
        ];
    }

    vec![
        "Review the prepared file before applying it.".to_string(),
        format!(
            "sudo install -m 0644 \"{}\" \"{}\"",
            prepared_hosts_path,
            hosts_file_path.to_string_lossy()
        ),
    ]
}

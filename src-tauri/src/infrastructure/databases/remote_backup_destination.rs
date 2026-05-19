use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::domain::database::database_config::{
    DatabaseBackupRemoteCopyReceipt, DatabaseBackupRemoteDestination,
    DatabaseBackupRemoteDestinationProvider, DatabaseBackupResult,
};
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::infrastructure::services::adapters::executable_resolver::ExecutableResolver;
use crate::ports::process_manager::ProcessManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const REMOTE_COPY_TIMEOUT: Duration = Duration::from_secs(180);
const REMOTE_COPY_OUTPUT_LIMIT_BYTES: usize = 64 * 1024;
const R2_ENDPOINT_URL_ENV: &str = "AXIOM_R2_ENDPOINT_URL";

pub fn copy_backup_to_remote_destination(
    result: &DatabaseBackupResult,
    destination: &DatabaseBackupRemoteDestination,
) -> AppResult<Vec<DatabaseBackupRemoteCopyReceipt>> {
    if !destination.enabled {
        return Ok(Vec::new());
    }

    match destination.provider {
        DatabaseBackupRemoteDestinationProvider::LocalPath => {
            copy_backup_to_local_destination(result, destination)
        }
        DatabaseBackupRemoteDestinationProvider::S3 => {
            copy_backup_with_aws_cli(result, destination, None)
        }
        DatabaseBackupRemoteDestinationProvider::R2 => {
            let endpoint_url = std::env::var(R2_ENDPOINT_URL_ENV).map_err(|_| {
                AppError::Configuration(format!(
                    "{R2_ENDPOINT_URL_ENV} is required for R2 backup destinations"
                ))
            })?;

            copy_backup_with_aws_cli(result, destination, Some(endpoint_url.trim().to_string()))
        }
        DatabaseBackupRemoteDestinationProvider::Gcs => {
            copy_backup_with_gcloud(result, destination)
        }
        DatabaseBackupRemoteDestinationProvider::Sftp => copy_backup_with_scp(result, destination),
    }
}

fn copy_backup_to_local_destination(
    result: &DatabaseBackupResult,
    destination: &DatabaseBackupRemoteDestination,
) -> AppResult<Vec<DatabaseBackupRemoteCopyReceipt>> {
    let destination_root = validate_local_destination_path(&destination.destination_path)?;
    let scoped_destination = destination_root
        .join(&destination.project_id.0)
        .join(destination.database_type.as_key());

    fs::create_dir_all(&scoped_destination).map_err(|error| {
        AppError::Infrastructure(format!("failed to create backup destination: {error}"))
    })?;

    let mut copied_paths = Vec::new();
    for source_path in backup_artifact_paths(result) {
        let source_path = validate_source_artifact(&source_path)?;
        let copied_path = copy_one_local(&source_path, &scoped_destination)?;
        copied_paths.push(remote_copy_receipt(
            destination,
            &source_path,
            copied_path.to_string_lossy().into_owned(),
            true,
        )?);
    }

    Ok(copied_paths)
}

fn copy_backup_with_aws_cli(
    result: &DatabaseBackupResult,
    destination: &DatabaseBackupRemoteDestination,
    endpoint_url: Option<String>,
) -> AppResult<Vec<DatabaseBackupRemoteCopyReceipt>> {
    validate_cloud_destination_uri(&destination.destination_path, "s3://")?;
    let aws_path = resolve_required_executable("aws")?;
    let runner = remote_runner(&aws_path);
    let mut copied_paths = Vec::new();

    for source_path in backup_artifact_paths(result) {
        let source_path = validate_source_artifact(&source_path)?;
        let target = remote_target_uri(destination, &source_path)?;
        let mut args = Vec::new();

        if let Some(endpoint_url) = &endpoint_url {
            args.extend(["--endpoint-url".to_string(), endpoint_url.clone()]);
        }
        args.extend([
            "s3".to_string(),
            "cp".to_string(),
            source_path.to_string_lossy().into_owned(),
            target.clone(),
        ]);
        ensure_successful_output(
            "aws s3 backup copy",
            run_remote_copy(&runner, &aws_path, args)?,
        )?;
        copied_paths.push(remote_copy_receipt(
            destination,
            &source_path,
            target,
            true,
        )?);
    }

    Ok(copied_paths)
}

fn copy_backup_with_gcloud(
    result: &DatabaseBackupResult,
    destination: &DatabaseBackupRemoteDestination,
) -> AppResult<Vec<DatabaseBackupRemoteCopyReceipt>> {
    validate_cloud_destination_uri(&destination.destination_path, "gs://")?;
    let gcloud_path = resolve_required_executable("gcloud")?;
    let runner = remote_runner(&gcloud_path);
    let mut copied_paths = Vec::new();

    for source_path in backup_artifact_paths(result) {
        let source_path = validate_source_artifact(&source_path)?;
        let target = remote_target_uri(destination, &source_path)?;

        ensure_successful_output(
            "gcloud storage backup copy",
            run_remote_copy(
                &runner,
                &gcloud_path,
                [
                    "storage".to_string(),
                    "cp".to_string(),
                    source_path.to_string_lossy().into_owned(),
                    target.clone(),
                ],
            )?,
        )?;
        copied_paths.push(remote_copy_receipt(
            destination,
            &source_path,
            target,
            true,
        )?);
    }

    Ok(copied_paths)
}

fn copy_backup_with_scp(
    result: &DatabaseBackupResult,
    destination: &DatabaseBackupRemoteDestination,
) -> AppResult<Vec<DatabaseBackupRemoteCopyReceipt>> {
    validate_cloud_destination_uri(&destination.destination_path, "sftp://")?;
    let scp_path = resolve_required_executable("scp")?;
    let runner = remote_runner(&scp_path);
    let mut copied_paths = Vec::new();

    for source_path in backup_artifact_paths(result) {
        let source_path = validate_source_artifact(&source_path)?;
        let target_uri = remote_target_uri(destination, &source_path)?;
        let scp_target = sftp_uri_to_scp_target(&target_uri)?;

        ensure_successful_output(
            "sftp backup copy",
            run_remote_copy(
                &runner,
                &scp_path,
                [source_path.to_string_lossy().into_owned(), scp_target],
            )?,
        )?;
        copied_paths.push(remote_copy_receipt(
            destination,
            &source_path,
            target_uri,
            true,
        )?);
    }

    Ok(copied_paths)
}

fn backup_artifact_paths(result: &DatabaseBackupResult) -> Vec<String> {
    let mut paths = vec![result.backup_path.clone()];

    if let Some(metadata_path) = &result.metadata_path {
        paths.push(metadata_path.clone());
    }

    if let Some(signature_path) = &result.signature_path {
        paths.push(signature_path.clone());
    }

    paths
}

fn validate_local_destination_path(path: &str) -> AppResult<PathBuf> {
    let path = Path::new(path.trim());

    if !path.is_absolute() {
        return Err(AppError::Validation(
            "backup destination path must be absolute".to_string(),
        ));
    }

    if path.exists() && !path.is_dir() {
        return Err(AppError::Validation(
            "backup destination path must be a directory".to_string(),
        ));
    }

    Ok(path.to_path_buf())
}

fn validate_cloud_destination_uri(value: &str, prefix: &str) -> AppResult<()> {
    let value = value.trim();

    if value.as_bytes().contains(&0) || value.chars().any(char::is_control) {
        return Err(AppError::Validation(
            "backup destination URI must not contain null bytes or control characters".to_string(),
        ));
    }

    if !value.starts_with(prefix) || value.len() <= prefix.len() {
        return Err(AppError::Validation(format!(
            "backup destination URI must start with {prefix}"
        )));
    }

    Ok(())
}

fn copy_one_local(source_path: &Path, destination_dir: &Path) -> AppResult<PathBuf> {
    let file_name = source_path
        .file_name()
        .ok_or_else(|| AppError::Validation("backup artifact has no file name".to_string()))?;
    let destination_path = destination_dir.join(file_name);

    fs::copy(&source_path, &destination_path).map_err(|error| {
        AppError::Infrastructure(format!("failed to copy backup artifact: {error}"))
    })?;

    Ok(destination_path)
}

fn validate_source_artifact(source_path: &str) -> AppResult<PathBuf> {
    let source_path = Path::new(source_path);
    let source_path = source_path.canonicalize().map_err(|error| {
        AppError::Validation(format!(
            "backup artifact path must exist before copy: {error}"
        ))
    })?;

    if !source_path.is_file() {
        return Err(AppError::Validation(
            "backup artifact path must point to a file".to_string(),
        ));
    }

    Ok(source_path)
}

fn remote_target_uri(
    destination: &DatabaseBackupRemoteDestination,
    source_path: &Path,
) -> AppResult<String> {
    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::Validation("backup artifact has no file name".to_string()))?;
    let base = destination.destination_path.trim().trim_end_matches('/');

    Ok(format!(
        "{}/{}/{}/{}",
        base,
        destination.project_id.0,
        destination.database_type.as_key(),
        file_name
    ))
}

fn sftp_uri_to_scp_target(uri: &str) -> AppResult<String> {
    let value = uri.strip_prefix("sftp://").ok_or_else(|| {
        AppError::Validation("SFTP backup destination must start with sftp://".to_string())
    })?;
    let Some((host, path)) = value.split_once('/') else {
        return Err(AppError::Validation(
            "SFTP backup destination must include a remote path".to_string(),
        ));
    };

    if host.trim().is_empty() || path.trim().is_empty() {
        return Err(AppError::Validation(
            "SFTP backup destination host and path are required".to_string(),
        ));
    }

    Ok(format!("{host}:/{path}"))
}

fn remote_copy_receipt(
    destination: &DatabaseBackupRemoteDestination,
    source_path: &Path,
    remote_uri: String,
    verified: bool,
) -> AppResult<DatabaseBackupRemoteCopyReceipt> {
    Ok(DatabaseBackupRemoteCopyReceipt {
        provider: destination.provider,
        artifact_path: source_path.to_string_lossy().into_owned(),
        remote_uri,
        sha256: sha256_file_hex(source_path)?,
        size_bytes: source_path
            .metadata()
            .map(|metadata| metadata.len())
            .map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to inspect copied backup artifact: {error}"
                ))
            })?,
        copied_at: Utc::now(),
        verified,
        status_message:
            "Remote backup artifact was copied and recorded with local integrity metadata."
                .to_string(),
    })
}

fn sha256_file_hex(path: &Path) -> AppResult<String> {
    let mut file = fs::File::open(path).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to open backup artifact for hashing: {error}"
        ))
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read_count = file.read(&mut buffer).map_err(|error| {
            AppError::Infrastructure(format!("failed to hash backup artifact: {error}"))
        })?;

        if read_count == 0 {
            break;
        }

        hasher.update(&buffer[..read_count]);
    }

    Ok(hex_encode(&hasher.finalize()))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }

    encoded
}

fn resolve_required_executable(name: &str) -> AppResult<PathBuf> {
    ExecutableResolver::from_env()
        .resolve(name)
        .ok_or_else(|| AppError::NotFound(format!("{name} CLI executable was not found on PATH")))
}

fn remote_runner(program_path: &Path) -> CommandRunner {
    CommandRunner::new(
        CommandPolicy::deny_all()
            .allow_program_paths([program_path.to_path_buf()])
            .with_default_timeout(REMOTE_COPY_TIMEOUT)
            .with_max_output_bytes(REMOTE_COPY_OUTPUT_LIMIT_BYTES),
    )
}

fn run_remote_copy(
    runner: &CommandRunner,
    program_path: &Path,
    args: impl IntoIterator<Item = impl Into<String>>,
) -> AppResult<ProcessOutput> {
    runner.execute(
        ProcessCommand::new(program_path.to_string_lossy().into_owned())
            .args(args)
            .timeout(REMOTE_COPY_TIMEOUT),
    )
}

fn ensure_successful_output(label: &str, output: ProcessOutput) -> AppResult<()> {
    if output.timed_out {
        return Err(AppError::Infrastructure(format!("{label} timed out")));
    }

    if output.exit_code == Some(0) {
        return Ok(());
    }

    let diagnostic = if output.stderr.trim().is_empty() {
        output.stdout.trim()
    } else {
        output.stderr.trim()
    };

    Err(AppError::Infrastructure(format!(
        "{label} failed: {}",
        if diagnostic.is_empty() {
            "no diagnostic output was returned"
        } else {
            diagnostic
        }
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::database::database_type::DatabaseType;
    use crate::domain::project::project_id::ProjectId;

    #[test]
    fn builds_scoped_s3_destination_uri() {
        let source = PathBuf::from("/tmp/demo.sql.gz.enc");
        let destination = DatabaseBackupRemoteDestination {
            project_id: ProjectId("demo".to_string()),
            database_type: DatabaseType::Mysql,
            provider: DatabaseBackupRemoteDestinationProvider::S3,
            enabled: true,
            destination_path: "s3://bucket/prefix".to_string(),
            updated_at: chrono::Utc::now(),
        };

        let uri = remote_target_uri(&destination, &source).expect("uri");

        assert_eq!(uri, "s3://bucket/prefix/demo/mysql/demo.sql.gz.enc");
    }

    #[test]
    fn converts_sftp_uri_to_scp_target() {
        let target =
            sftp_uri_to_scp_target("sftp://user@example.com/backups/demo.sql").expect("scp target");

        assert_eq!(target, "user@example.com:/backups/demo.sql");
    }

    #[test]
    fn remote_copy_receipts_include_integrity_metadata() {
        let temp_dir =
            std::env::temp_dir().join(format!("axiom-remote-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let artifact = temp_dir.join("demo.sql");
        fs::write(&artifact, "select 1;").expect("artifact");
        let destination = DatabaseBackupRemoteDestination {
            project_id: ProjectId("demo".to_string()),
            database_type: DatabaseType::Mysql,
            provider: DatabaseBackupRemoteDestinationProvider::S3,
            enabled: true,
            destination_path: "s3://bucket/prefix".to_string(),
            updated_at: chrono::Utc::now(),
        };

        let receipt = remote_copy_receipt(
            &destination,
            &artifact,
            "s3://bucket/prefix/demo/mysql/demo.sql".to_string(),
            true,
        )
        .expect("receipt");

        assert_eq!(
            receipt.provider,
            DatabaseBackupRemoteDestinationProvider::S3
        );
        assert_eq!(receipt.size_bytes, 9);
        assert!(receipt.verified);

        let _ = fs::remove_dir_all(temp_dir);
    }
}

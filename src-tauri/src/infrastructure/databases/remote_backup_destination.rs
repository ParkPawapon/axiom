use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{Datelike, Utc};
use hmac::{Hmac, KeyInit as HmacKeyInit, Mac};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_LENGTH, HOST};
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
const AWS_ACCESS_KEY_ID_ENV: &str = "AWS_ACCESS_KEY_ID";
const AWS_SECRET_ACCESS_KEY_ENV: &str = "AWS_SECRET_ACCESS_KEY";
const AWS_SESSION_TOKEN_ENV: &str = "AWS_SESSION_TOKEN";
const AWS_REGION_ENV: &str = "AWS_REGION";
const AWS_DEFAULT_REGION_ENV: &str = "AWS_DEFAULT_REGION";
const R2_ACCESS_KEY_ID_ENV: &str = "AXIOM_R2_ACCESS_KEY_ID";
const R2_SECRET_ACCESS_KEY_ENV: &str = "AXIOM_R2_SECRET_ACCESS_KEY";
const GCS_ACCESS_TOKEN_ENV: &str = "AXIOM_GCS_ACCESS_TOKEN";
const GOOGLE_OAUTH_ACCESS_TOKEN_ENV: &str = "GOOGLE_OAUTH_ACCESS_TOKEN";

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
            copy_backup_with_s3_native(result, destination, None)
                .or_else(|_| copy_backup_with_aws_cli(result, destination, None))
        }
        DatabaseBackupRemoteDestinationProvider::R2 => {
            let endpoint_url = std::env::var(R2_ENDPOINT_URL_ENV).map_err(|_| {
                AppError::Configuration(format!(
                    "{R2_ENDPOINT_URL_ENV} is required for R2 backup destinations"
                ))
            })?;
            let endpoint_url = endpoint_url.trim().to_string();

            copy_backup_with_s3_native(result, destination, Some(endpoint_url.clone()))
                .or_else(|_| copy_backup_with_aws_cli(result, destination, Some(endpoint_url)))
        }
        DatabaseBackupRemoteDestinationProvider::Gcs => {
            copy_backup_with_gcs_native(result, destination)
                .or_else(|_| copy_backup_with_gcloud(result, destination))
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

fn copy_backup_with_s3_native(
    result: &DatabaseBackupResult,
    destination: &DatabaseBackupRemoteDestination,
    endpoint_url: Option<String>,
) -> AppResult<Vec<DatabaseBackupRemoteCopyReceipt>> {
    validate_cloud_destination_uri(&destination.destination_path, "s3://")?;
    let credentials = S3Credentials::from_env(destination.provider)?;
    let client = Client::builder()
        .timeout(REMOTE_COPY_TIMEOUT)
        .build()
        .map_err(|error| {
            AppError::Infrastructure(format!("failed to initialize S3 client: {error}"))
        })?;
    let mut copied_paths = Vec::new();

    for source_path in backup_artifact_paths(result) {
        let source_path = validate_source_artifact(&source_path)?;
        let target = remote_target_uri(destination, &source_path)?;
        let target_parts = parse_s3_uri(&target)?;
        let bytes = fs::read(&source_path).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to read backup artifact for S3 upload: {error}"
            ))
        })?;
        let request = S3PutRequest::new(&target_parts, endpoint_url.as_deref(), &credentials)?;
        let payload_sha256 = hex_encode(&Sha256::digest(&bytes));
        let headers = s3_signed_headers(&request, &credentials, &payload_sha256, bytes.len())?;
        let response = client
            .put(&request.url)
            .headers(headers)
            .body(bytes)
            .send()
            .map_err(|error| {
                AppError::Infrastructure(format!("native S3 backup upload failed: {error}"))
            })?;

        if !response.status().is_success() {
            return Err(AppError::Infrastructure(format!(
                "native S3 backup upload returned HTTP {}",
                response.status()
            )));
        }

        copied_paths.push(remote_copy_receipt(
            destination,
            &source_path,
            target,
            true,
        )?);
    }

    Ok(copied_paths)
}

fn copy_backup_with_gcs_native(
    result: &DatabaseBackupResult,
    destination: &DatabaseBackupRemoteDestination,
) -> AppResult<Vec<DatabaseBackupRemoteCopyReceipt>> {
    validate_cloud_destination_uri(&destination.destination_path, "gs://")?;
    let access_token = env_value(GCS_ACCESS_TOKEN_ENV)
        .or_else(|| env_value(GOOGLE_OAUTH_ACCESS_TOKEN_ENV))
        .ok_or_else(|| {
            AppError::Configuration(format!(
                "{GCS_ACCESS_TOKEN_ENV} or {GOOGLE_OAUTH_ACCESS_TOKEN_ENV} is required for native GCS uploads"
            ))
        })?;
    let client = Client::builder()
        .timeout(REMOTE_COPY_TIMEOUT)
        .build()
        .map_err(|error| {
            AppError::Infrastructure(format!("failed to initialize GCS client: {error}"))
        })?;
    let mut copied_paths = Vec::new();

    for source_path in backup_artifact_paths(result) {
        let source_path = validate_source_artifact(&source_path)?;
        let target = remote_target_uri(destination, &source_path)?;
        let target_parts = parse_gcs_uri(&target)?;
        let bytes = fs::read(&source_path).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to read backup artifact for GCS upload: {error}"
            ))
        })?;
        let url = format!(
            "https://storage.googleapis.com/upload/storage/v1/b/{}/o?uploadType=media&name={}",
            percent_encode_query_value(&target_parts.bucket),
            percent_encode_query_value(&target_parts.key)
        );
        let response = client
            .post(url)
            .bearer_auth(&access_token)
            .header("content-type", "application/octet-stream")
            .body(bytes)
            .send()
            .map_err(|error| {
                AppError::Infrastructure(format!("native GCS backup upload failed: {error}"))
            })?;

        if !response.status().is_success() {
            return Err(AppError::Infrastructure(format!(
                "native GCS backup upload returned HTTP {}",
                response.status()
            )));
        }

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

#[derive(Debug, Clone, Eq, PartialEq)]
struct CloudObjectUri {
    bucket: String,
    key: String,
}

fn parse_s3_uri(uri: &str) -> AppResult<CloudObjectUri> {
    parse_cloud_object_uri(uri, "s3://", "S3")
}

fn parse_gcs_uri(uri: &str) -> AppResult<CloudObjectUri> {
    parse_cloud_object_uri(uri, "gs://", "GCS")
}

fn parse_cloud_object_uri(uri: &str, prefix: &str, label: &str) -> AppResult<CloudObjectUri> {
    validate_cloud_destination_uri(uri, prefix)?;
    let value = uri.trim().strip_prefix(prefix).ok_or_else(|| {
        AppError::Validation(format!("{label} destination must start with {prefix}"))
    })?;
    let Some((bucket, key)) = value.split_once('/') else {
        return Err(AppError::Validation(format!(
            "{label} destination must include a bucket and object key"
        )));
    };

    if !valid_cloud_bucket_name(bucket) || key.trim().is_empty() {
        return Err(AppError::Validation(format!(
            "{label} destination bucket or object key is invalid"
        )));
    }

    Ok(CloudObjectUri {
        bucket: bucket.to_string(),
        key: key.to_string(),
    })
}

fn valid_cloud_bucket_name(bucket: &str) -> bool {
    (3..=63).contains(&bucket.len())
        && bucket.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '-')
        })
        && bucket
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphanumeric())
        && bucket
            .chars()
            .last()
            .is_some_and(|character| character.is_ascii_alphanumeric())
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct S3Credentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: Option<String>,
    region: String,
}

impl S3Credentials {
    fn from_env(provider: DatabaseBackupRemoteDestinationProvider) -> AppResult<Self> {
        let access_key_id = match provider {
            DatabaseBackupRemoteDestinationProvider::R2 => {
                env_value(R2_ACCESS_KEY_ID_ENV).or_else(|| env_value(AWS_ACCESS_KEY_ID_ENV))
            }
            _ => env_value(AWS_ACCESS_KEY_ID_ENV),
        }
        .ok_or_else(|| {
            AppError::Configuration(format!(
                "{AWS_ACCESS_KEY_ID_ENV} is required for native S3-compatible uploads"
            ))
        })?;
        let secret_access_key = match provider {
            DatabaseBackupRemoteDestinationProvider::R2 => {
                env_value(R2_SECRET_ACCESS_KEY_ENV).or_else(|| env_value(AWS_SECRET_ACCESS_KEY_ENV))
            }
            _ => env_value(AWS_SECRET_ACCESS_KEY_ENV),
        }
        .ok_or_else(|| {
            AppError::Configuration(format!(
                "{AWS_SECRET_ACCESS_KEY_ENV} is required for native S3-compatible uploads"
            ))
        })?;
        let region = env_value(AWS_REGION_ENV)
            .or_else(|| env_value(AWS_DEFAULT_REGION_ENV))
            .unwrap_or_else(|| {
                if provider == DatabaseBackupRemoteDestinationProvider::R2 {
                    "auto".to_string()
                } else {
                    "us-east-1".to_string()
                }
            });

        validate_s3_credential_part(&access_key_id, "S3 access key id")?;
        validate_s3_credential_part(&secret_access_key, "S3 secret access key")?;
        validate_s3_credential_part(&region, "S3 region")?;

        Ok(Self {
            access_key_id,
            secret_access_key,
            session_token: env_value(AWS_SESSION_TOKEN_ENV),
            region,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct S3PutRequest {
    url: String,
    host: String,
    canonical_uri: String,
}

impl S3PutRequest {
    fn new(
        target: &CloudObjectUri,
        endpoint_url: Option<&str>,
        credentials: &S3Credentials,
    ) -> AppResult<Self> {
        if let Some(endpoint_url) = endpoint_url {
            let endpoint = endpoint_url.trim().trim_end_matches('/');
            if !endpoint.starts_with("https://") {
                return Err(AppError::Validation(
                    "S3-compatible native upload endpoint must use HTTPS".to_string(),
                ));
            }
            let host = endpoint.trim_start_matches("https://").to_string();
            let canonical_uri = format!(
                "/{}/{}",
                percent_encode_path_segment(&target.bucket),
                percent_encode_path(&target.key)
            );

            return Ok(Self {
                url: format!("{endpoint}{canonical_uri}"),
                host,
                canonical_uri,
            });
        }

        let host = format!("{}.s3.{}.amazonaws.com", target.bucket, credentials.region);
        let canonical_uri = format!("/{}", percent_encode_path(&target.key));

        Ok(Self {
            url: format!("https://{host}{canonical_uri}"),
            host,
            canonical_uri,
        })
    }
}

fn s3_signed_headers(
    request: &S3PutRequest,
    credentials: &S3Credentials,
    payload_sha256: &str,
    payload_len: usize,
) -> AppResult<HeaderMap> {
    let now = Utc::now();
    let date_stamp = format!("{:04}{:02}{:02}", now.year(), now.month(), now.day());
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let mut canonical_headers = format!(
        "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
        request.host, payload_sha256, amz_date
    );
    let mut signed_headers = "host;x-amz-content-sha256;x-amz-date".to_string();

    if let Some(session_token) = &credentials.session_token {
        canonical_headers.push_str("x-amz-security-token:");
        canonical_headers.push_str(session_token);
        canonical_headers.push('\n');
        signed_headers.push_str(";x-amz-security-token");
    }

    let canonical_request = format!(
        "PUT\n{}\n\n{}{}\n{}",
        request.canonical_uri, canonical_headers, signed_headers, payload_sha256
    );
    let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, credentials.region);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date,
        credential_scope,
        hex_encode(&Sha256::digest(canonical_request.as_bytes()))
    );
    let signing_key = aws_v4_signing_key(
        &credentials.secret_access_key,
        &date_stamp,
        &credentials.region,
    )?;
    let signature = hmac_sha256_hex(&signing_key, string_to_sign.as_bytes())?;
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        credentials.access_key_id, credential_scope, signed_headers, signature
    );
    let mut headers = HeaderMap::new();

    headers.insert(HOST, header_value(&request.host)?);
    headers.insert(
        HeaderName::from_static("x-amz-content-sha256"),
        header_value(payload_sha256)?,
    );
    headers.insert(
        HeaderName::from_static("x-amz-date"),
        header_value(&amz_date)?,
    );
    headers.insert(CONTENT_LENGTH, header_value(&payload_len.to_string())?);
    headers.insert(AUTHORIZATION, header_value(&authorization)?);
    if let Some(session_token) = &credentials.session_token {
        headers.insert(
            HeaderName::from_static("x-amz-security-token"),
            header_value(session_token)?,
        );
    }

    Ok(headers)
}

fn aws_v4_signing_key(secret: &str, date: &str, region: &str) -> AppResult<Vec<u8>> {
    let date_key = hmac_sha256(format!("AWS4{secret}").as_bytes(), date.as_bytes())?;
    let region_key = hmac_sha256(&date_key, region.as_bytes())?;
    let service_key = hmac_sha256(&region_key, b"s3")?;

    hmac_sha256(&service_key, b"aws4_request")
}

fn hmac_sha256(key: &[u8], value: &[u8]) -> AppResult<Vec<u8>> {
    let mut mac = <Hmac<Sha256> as HmacKeyInit>::new_from_slice(key).map_err(|error| {
        AppError::Infrastructure(format!("failed to initialize HMAC signer: {error}"))
    })?;
    mac.update(value);

    Ok(mac.finalize().into_bytes().to_vec())
}

fn hmac_sha256_hex(key: &[u8], value: &[u8]) -> AppResult<String> {
    hmac_sha256(key, value).map(|digest| hex_encode(&digest))
}

fn header_value(value: &str) -> AppResult<HeaderValue> {
    HeaderValue::from_str(value)
        .map_err(|error| AppError::Validation(format!("HTTP header value is invalid: {error}")))
}

fn validate_s3_credential_part(value: &str, label: &str) -> AppResult<()> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(AppError::Validation(format!(
            "{label} must not be empty or contain control characters"
        )));
    }

    Ok(())
}

fn copy_one_local(source_path: &Path, destination_dir: &Path) -> AppResult<PathBuf> {
    let file_name = source_path
        .file_name()
        .ok_or_else(|| AppError::Validation("backup artifact has no file name".to_string()))?;
    let destination_path = destination_dir.join(file_name);

    fs::copy(source_path, &destination_path).map_err(|error| {
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

fn env_value(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn percent_encode_path(path: &str) -> String {
    path.split('/')
        .map(percent_encode_path_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn percent_encode_path_segment(value: &str) -> String {
    percent_encode_with(value, true)
}

fn percent_encode_query_value(value: &str) -> String {
    percent_encode_with(value, false)
}

fn percent_encode_with(value: &str, keep_path_safe: bool) -> String {
    let mut encoded = String::new();

    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'.' | b'_' | b'~')
            || (keep_path_safe && byte == b'/')
        {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(char::from(b"0123456789ABCDEF"[(byte >> 4) as usize]));
            encoded.push(char::from(b"0123456789ABCDEF"[(byte & 0x0f) as usize]));
        }
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

    #[test]
    fn parses_scoped_cloud_object_uri() {
        let uri = parse_s3_uri("s3://bucket-name/prefix/demo.sql.gz.enc").expect("s3 uri");

        assert_eq!(uri.bucket, "bucket-name");
        assert_eq!(uri.key, "prefix/demo.sql.gz.enc");
    }

    #[test]
    fn percent_encodes_gcs_object_names() {
        assert_eq!(
            percent_encode_query_value("demo/mysql/a file.sql.gz.enc"),
            "demo%2Fmysql%2Fa%20file.sql.gz.enc"
        );
    }
}

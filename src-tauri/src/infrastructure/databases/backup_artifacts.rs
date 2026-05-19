use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration as StdDuration;

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{Aes256Gcm, KeyInit as AesKeyInit, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use chrono::{Duration, Utc};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use hmac::{Hmac, KeyInit as HmacKeyInit, Mac};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::database::database_config::{
    DatabaseBackupArtifactTrustEnrollmentResult, DatabaseBackupCompression,
    DatabaseBackupEncryption, DatabaseBackupKeyManagementStatus, DatabaseBackupKmsEnvelope,
    DatabaseBackupMetadata, DatabaseBackupOptions, DatabaseBackupTrustBundle,
    DatabaseBackupTrustExportResult, DatabaseBackupTrustImportResult, ProjectDatabaseProfile,
};
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::infrastructure::services::adapters::executable_resolver::ExecutableResolver;
use crate::ports::process_manager::ProcessManager;
use crate::ports::secure_storage::SecureStorage;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const BACKUP_SECRET_NAMESPACE: &str = "database-backup";
const BACKUP_ENCRYPTION_KEY: &str = "managed-backup-key";
const BACKUP_SIGNING_KEY: &str = "managed-backup-signing-key";
const BACKUP_TRUST_FINGERPRINTS_KEY: &str = "trusted-backup-signing-fingerprints";
const BACKUP_ENCRYPTION_KEY_ENV: &str = "AXIOM_BACKUP_ENCRYPTION_KEY_B64";
const BACKUP_SIGNING_KEY_ENV: &str = "AXIOM_BACKUP_SIGNING_KEY_B64";
const BACKUP_KMS_PROVIDER_ENV: &str = "AXIOM_BACKUP_KMS_PROVIDER";
const BACKUP_KMS_KEY_ID_ENV: &str = "AXIOM_BACKUP_KMS_KEY_ID";
const ENCRYPTION_MAGIC: &[u8] = b"AXIOMDB1";
const KMS_ENVELOPE_MAGIC: &[u8] = b"AXIOMDBK2";
const AES_256_KEY_BYTES: usize = 32;
const AES_GCM_NONCE_BYTES: usize = 12;
const MIN_RETENTION_DAYS: u16 = 1;
const MAX_RETENTION_DAYS: u16 = 365;
const KMS_COMMAND_TIMEOUT: StdDuration = StdDuration::from_secs(60);
const KMS_OUTPUT_LIMIT_BYTES: usize = 128 * 1024;

#[derive(Debug, Clone)]
pub struct FinalizedBackupArtifact {
    pub backup_path: PathBuf,
    pub metadata_path: PathBuf,
    pub signature_path: PathBuf,
    pub compressed: bool,
    pub encrypted: bool,
    pub size_bytes: u64,
    pub pruned_backup_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PreparedRestoreArtifact {
    pub source_path: PathBuf,
    pub sql_path: PathBuf,
    pub decrypted: bool,
    pub decompressed: bool,
    pub signature_verified: bool,
    pub temporary_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseBackupSignature {
    algorithm: String,
    key_source: String,
    #[serde(default)]
    key_fingerprint: String,
    backup_path: String,
    metadata_path: String,
    signature: String,
    signed_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct EncryptedBackupArtifact {
    path: PathBuf,
    kms_envelope: Option<DatabaseBackupKmsEnvelope>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct KmsEnvelopeHeader {
    version: u16,
    provider: String,
    key_id: String,
    encrypted_data_key: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct KmsEncryptedDataKey {
    provider: String,
    key_id: String,
    ciphertext_b64: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ExternalKmsConfig {
    provider: String,
    key_id: String,
}

pub fn backup_key_management_status(
    secure_storage: &dyn SecureStorage,
) -> AppResult<DatabaseBackupKeyManagementStatus> {
    let encryption_key_source = key_source(
        secure_storage,
        BACKUP_ENCRYPTION_KEY_ENV,
        BACKUP_ENCRYPTION_KEY,
    )?;
    let signing_key_source =
        key_source(secure_storage, BACKUP_SIGNING_KEY_ENV, BACKUP_SIGNING_KEY)?;
    let trusted_signing_key_fingerprints = trusted_signing_key_fingerprints(secure_storage)?;
    let external_kms_provider = env_value(BACKUP_KMS_PROVIDER_ENV);
    let external_kms_key_id = env_value(BACKUP_KMS_KEY_ID_ENV);

    Ok(DatabaseBackupKeyManagementStatus {
        encryption_key_source,
        signing_key_source,
        external_kms_provider,
        external_kms_key_id,
        trusted_signing_key_fingerprints,
        status_message: "Backup key management status was inspected without exposing key material."
            .to_string(),
    })
}

pub fn export_backup_trust_bundle(
    secure_storage: &dyn SecureStorage,
    output_dir: &str,
) -> AppResult<DatabaseBackupTrustExportResult> {
    let output_dir = validate_output_directory(output_dir)?;
    let (signing_key, _source) = signing_key(secure_storage)?;
    let signing_key_fingerprint = key_fingerprint(&signing_key);
    let bundle = DatabaseBackupTrustBundle {
        version: 1,
        algorithm: "hmac-sha256-key-fingerprint".to_string(),
        signing_key_fingerprint: signing_key_fingerprint.clone(),
        artifact_sha256: None,
        source_machine: source_machine_label(),
        exported_at: Utc::now(),
    };
    let trust_bundle_path = output_dir.join(format!(
        "axiomphp-backup-trust-{}.json",
        signing_key_fingerprint.chars().take(16).collect::<String>()
    ));
    let payload = serde_json::to_vec_pretty(&bundle).map_err(|error| {
        AppError::Infrastructure(format!("failed to serialize backup trust bundle: {error}"))
    })?;

    fs::write(&trust_bundle_path, payload).map_err(|error| {
        AppError::Infrastructure(format!("failed to write backup trust bundle: {error}"))
    })?;

    Ok(DatabaseBackupTrustExportResult {
        trust_bundle_path: trust_bundle_path.to_string_lossy().into_owned(),
        signing_key_fingerprint,
        status_message: "Backup trust bundle was exported without secret key material.".to_string(),
    })
}

pub fn import_backup_trust_bundle(
    secure_storage: &dyn SecureStorage,
    trust_bundle_path: &str,
) -> AppResult<DatabaseBackupTrustImportResult> {
    let trust_bundle_path = validate_trust_bundle_path(trust_bundle_path)?;
    let payload = fs::read_to_string(&trust_bundle_path).map_err(|error| {
        AppError::Infrastructure(format!("failed to read backup trust bundle: {error}"))
    })?;
    let bundle: DatabaseBackupTrustBundle = serde_json::from_str(&payload).map_err(|error| {
        AppError::Configuration(format!("backup trust bundle is invalid: {error}"))
    })?;

    if bundle.algorithm != "hmac-sha256-key-fingerprint" || bundle.version != 1 {
        return Err(AppError::Validation(
            "backup trust bundle uses an unsupported format".to_string(),
        ));
    }

    enroll_trusted_signing_fingerprint(secure_storage, &bundle.signing_key_fingerprint)?;

    if let Some(artifact_sha256) = &bundle.artifact_sha256 {
        enroll_trusted_artifact_hash(secure_storage, artifact_sha256)?;
    }

    Ok(DatabaseBackupTrustImportResult {
        trust_bundle_path: trust_bundle_path.to_string_lossy().into_owned(),
        trusted_signing_key_fingerprint: bundle.signing_key_fingerprint,
        status_message: "Backup signing key fingerprint was enrolled as trusted.".to_string(),
    })
}

pub fn enroll_backup_artifact_trust(
    secure_storage: &dyn SecureStorage,
    backup_path: &str,
) -> AppResult<DatabaseBackupArtifactTrustEnrollmentResult> {
    let backup_path = validate_restore_path(backup_path)?;
    let artifact_sha256 = sha256_file_hex(&backup_path)?;
    let signature_fingerprint = read_signature(&backup_path)?
        .map(|signature| signature.key_fingerprint)
        .filter(|fingerprint| !fingerprint.trim().is_empty());

    enroll_trusted_artifact_hash(secure_storage, &artifact_sha256)?;

    if let Some(fingerprint) = &signature_fingerprint {
        enroll_trusted_signing_fingerprint(secure_storage, fingerprint)?;
    }

    Ok(DatabaseBackupArtifactTrustEnrollmentResult {
        backup_path: backup_path.to_string_lossy().into_owned(),
        artifact_sha256,
        trusted_signing_key_fingerprint: signature_fingerprint,
        status_message: "Backup artifact hash was enrolled for cross-machine restore trust."
            .to_string(),
    })
}

pub fn normalize_backup_options(
    mut options: DatabaseBackupOptions,
) -> AppResult<DatabaseBackupOptions> {
    if !(MIN_RETENTION_DAYS..=MAX_RETENTION_DAYS).contains(&options.retention_days) {
        return Err(AppError::Validation(format!(
            "backup retention must be between {MIN_RETENTION_DAYS} and {MAX_RETENTION_DAYS} days"
        )));
    }

    if matches!(options.encryption, DatabaseBackupEncryption::Aes256Gcm)
        && matches!(options.compression, DatabaseBackupCompression::None)
    {
        options.compression = DatabaseBackupCompression::Gzip;
    }

    Ok(options)
}

pub fn finalize_backup_artifact(
    profile: &ProjectDatabaseProfile,
    secure_storage: &dyn SecureStorage,
    raw_backup_path: PathBuf,
    options: DatabaseBackupOptions,
) -> AppResult<FinalizedBackupArtifact> {
    let options = normalize_backup_options(options)?;
    let created_at = Utc::now();
    let mut current_path = raw_backup_path;
    let mut compressed = false;
    let mut encrypted = false;
    let mut kms_envelope = None;

    if options.compression == DatabaseBackupCompression::Gzip {
        current_path = compress_backup(&current_path)?;
        compressed = true;
    }

    if options.encryption == DatabaseBackupEncryption::Aes256Gcm {
        let encrypted_artifact = encrypt_backup(secure_storage, &current_path)?;
        current_path = encrypted_artifact.path;
        kms_envelope = encrypted_artifact.kms_envelope;
        encrypted = true;
    }

    let size_bytes = file_size(&current_path)?;
    let metadata_path = metadata_path_for(&current_path);
    let metadata = DatabaseBackupMetadata {
        project_id: profile.project_id.clone(),
        database_type: profile.database_type,
        backup_path: current_path.to_string_lossy().into_owned(),
        metadata_path: metadata_path.to_string_lossy().into_owned(),
        compression: options.compression,
        encryption: options.encryption,
        compressed,
        encrypted,
        size_bytes,
        kms_envelope,
        created_at,
    };

    write_metadata(&metadata_path, &metadata)?;
    let signature_path = write_signature(secure_storage, &current_path, &metadata_path)?;

    let pruned_backup_paths = prune_backups(profile, &current_path, options.retention_days)?;

    Ok(FinalizedBackupArtifact {
        backup_path: current_path,
        metadata_path,
        signature_path,
        compressed,
        encrypted,
        size_bytes,
        pruned_backup_paths,
    })
}

pub fn prepare_restore_artifact(
    profile: &ProjectDatabaseProfile,
    secure_storage: &dyn SecureStorage,
    backup_path: &str,
) -> AppResult<PreparedRestoreArtifact> {
    let source_path = validate_restore_path(backup_path)?;
    let restore_work_dir = restore_work_dir(profile)?;
    fs::create_dir_all(&restore_work_dir).map_err(|error| {
        AppError::Infrastructure(format!("failed to create restore work directory: {error}"))
    })?;
    lock_private_directory(&restore_work_dir)?;

    let mut current_path = source_path.clone();
    let mut temporary_paths = Vec::new();
    let mut decrypted = false;
    let mut decompressed = false;
    let signature_verified = verify_signature_if_present(secure_storage, &source_path)?;

    if path_has_extension(&current_path, "enc") {
        let decrypted_bytes = decrypt_backup(secure_storage, &current_path)?;
        let decrypted_path = restore_work_dir.join(format!(
            "{}.decrypted{}",
            Uuid::new_v4(),
            if is_gzip_bytes(&decrypted_bytes) {
                ".sql.gz"
            } else {
                ".sql"
            }
        ));
        fs::write(&decrypted_path, decrypted_bytes).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to write decrypted restore artifact: {error}"
            ))
        })?;
        current_path = decrypted_path.clone();
        temporary_paths.push(decrypted_path);
        decrypted = true;
    }

    if path_has_extension(&current_path, "gz") || file_has_gzip_magic(&current_path)? {
        let decompressed_path = restore_work_dir.join(format!("{}.restore.sql", Uuid::new_v4()));
        decompress_backup(&current_path, &decompressed_path)?;
        current_path = decompressed_path.clone();
        temporary_paths.push(decompressed_path);
        decompressed = true;
    }

    if !path_has_extension(&current_path, "sql") {
        return Err(AppError::Validation(
            "restore backup must resolve to a .sql file after decrypt/decompress".to_string(),
        ));
    }

    Ok(PreparedRestoreArtifact {
        source_path,
        sql_path: current_path,
        decrypted,
        decompressed,
        signature_verified,
        temporary_paths,
    })
}

pub fn cleanup_restore_artifact(artifact: &PreparedRestoreArtifact) {
    for temporary_path in &artifact.temporary_paths {
        let _ = fs::remove_file(temporary_path);
    }
}

fn compress_backup(path: &Path) -> AppResult<PathBuf> {
    let compressed_path = path.with_extension("sql.gz");
    let mut input = File::open(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to open backup for compression: {error}"))
    })?;
    let output = File::create(&compressed_path).map_err(|error| {
        AppError::Infrastructure(format!("failed to create compressed backup: {error}"))
    })?;
    let mut encoder = GzEncoder::new(output, Compression::default());

    io::copy(&mut input, &mut encoder).map_err(|error| {
        AppError::Infrastructure(format!("failed to compress database backup: {error}"))
    })?;
    encoder.finish().map_err(|error| {
        AppError::Infrastructure(format!("failed to finish compressed backup: {error}"))
    })?;
    fs::remove_file(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to remove uncompressed backup: {error}"))
    })?;

    Ok(compressed_path)
}

fn decompress_backup(path: &Path, output_path: &Path) -> AppResult<()> {
    let input = File::open(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to open backup for decompression: {error}"))
    })?;
    let mut decoder = GzDecoder::new(input);
    let mut output = File::create(output_path).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to create decompressed restore file: {error}"
        ))
    })?;

    io::copy(&mut decoder, &mut output).map_err(|error| {
        AppError::Infrastructure(format!("failed to decompress database backup: {error}"))
    })?;

    Ok(())
}

fn encrypt_backup(
    secure_storage: &dyn SecureStorage,
    path: &Path,
) -> AppResult<EncryptedBackupArtifact> {
    let encrypted_path = encrypted_path_for(path);
    let plaintext = fs::read(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to read backup for encryption: {error}"))
    })?;
    let (ciphertext, kms_envelope) = encrypt_bytes(secure_storage, &plaintext)?;

    fs::write(&encrypted_path, ciphertext).map_err(|error| {
        AppError::Infrastructure(format!("failed to write encrypted backup: {error}"))
    })?;
    fs::remove_file(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to remove plaintext backup: {error}"))
    })?;

    Ok(EncryptedBackupArtifact {
        path: encrypted_path,
        kms_envelope,
    })
}

fn decrypt_backup(secure_storage: &dyn SecureStorage, path: &Path) -> AppResult<Vec<u8>> {
    let payload = fs::read(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to read encrypted backup: {error}"))
    })?;

    decrypt_bytes(secure_storage, &payload)
}

fn encrypt_bytes(
    secure_storage: &dyn SecureStorage,
    plaintext: &[u8],
) -> AppResult<(Vec<u8>, Option<DatabaseBackupKmsEnvelope>)> {
    if let Some(config) = external_kms_config()? {
        let mut data_key = [0_u8; AES_256_KEY_BYTES];
        OsRng.fill_bytes(&mut data_key);
        let encrypted_data_key = encrypt_data_key_with_kms(&config, &data_key)?;
        let (payload, envelope) =
            encrypt_kms_envelope_payload(plaintext, data_key, encrypted_data_key)?;

        return Ok((payload, Some(envelope)));
    }

    let key = backup_key(secure_storage)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|error| {
        AppError::Infrastructure(format!("failed to initialize backup cipher: {error}"))
    })?;
    let mut nonce = [0_u8; AES_GCM_NONCE_BYTES];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|error| AppError::Infrastructure(format!("backup encryption failed: {error}")))?;
    let mut payload = Vec::with_capacity(ENCRYPTION_MAGIC.len() + nonce.len() + ciphertext.len());

    payload.extend_from_slice(ENCRYPTION_MAGIC);
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&ciphertext);

    Ok((payload, None))
}

fn decrypt_bytes(secure_storage: &dyn SecureStorage, payload: &[u8]) -> AppResult<Vec<u8>> {
    if payload.starts_with(KMS_ENVELOPE_MAGIC) {
        return decrypt_kms_envelope_payload(payload);
    }

    if payload.len() <= ENCRYPTION_MAGIC.len() + AES_GCM_NONCE_BYTES
        || &payload[..ENCRYPTION_MAGIC.len()] != ENCRYPTION_MAGIC
    {
        return Err(AppError::Validation(
            "encrypted backup has an unsupported format".to_string(),
        ));
    }

    let key = backup_key(secure_storage)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|error| {
        AppError::Infrastructure(format!("failed to initialize backup cipher: {error}"))
    })?;
    let nonce_start = ENCRYPTION_MAGIC.len();
    let ciphertext_start = nonce_start + AES_GCM_NONCE_BYTES;

    cipher
        .decrypt(
            Nonce::from_slice(&payload[nonce_start..ciphertext_start]),
            &payload[ciphertext_start..],
        )
        .map_err(|error| AppError::PermissionDenied(format!("backup decryption failed: {error}")))
}

fn encrypt_kms_envelope_payload(
    plaintext: &[u8],
    data_key: [u8; AES_256_KEY_BYTES],
    encrypted_data_key: KmsEncryptedDataKey,
) -> AppResult<(Vec<u8>, DatabaseBackupKmsEnvelope)> {
    let cipher = Aes256Gcm::new_from_slice(&data_key).map_err(|error| {
        AppError::Infrastructure(format!("failed to initialize KMS backup cipher: {error}"))
    })?;
    let mut nonce = [0_u8; AES_GCM_NONCE_BYTES];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|error| {
            AppError::Infrastructure(format!("KMS envelope backup encryption failed: {error}"))
        })?;
    let encrypted_data_key_bytes = STANDARD
        .decode(&encrypted_data_key.ciphertext_b64)
        .map_err(|error| {
            AppError::Infrastructure(format!("KMS encrypted data key is invalid base64: {error}"))
        })?;
    let encrypted_data_key_fingerprint = hex_encode(&Sha256::digest(&encrypted_data_key_bytes));
    let header = KmsEnvelopeHeader {
        version: 1,
        provider: encrypted_data_key.provider.clone(),
        key_id: encrypted_data_key.key_id.clone(),
        encrypted_data_key: encrypted_data_key.ciphertext_b64,
    };
    let header = serde_json::to_vec(&header).map_err(|error| {
        AppError::Infrastructure(format!("failed to serialize KMS envelope header: {error}"))
    })?;
    let header_len = u32::try_from(header.len())
        .map_err(|_| AppError::Infrastructure("KMS envelope header is too large".to_string()))?;
    let mut payload = Vec::with_capacity(
        KMS_ENVELOPE_MAGIC.len() + 4 + header.len() + nonce.len() + ciphertext.len(),
    );

    payload.extend_from_slice(KMS_ENVELOPE_MAGIC);
    payload.extend_from_slice(&header_len.to_be_bytes());
    payload.extend_from_slice(&header);
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&ciphertext);

    Ok((
        payload,
        DatabaseBackupKmsEnvelope {
            provider: encrypted_data_key.provider,
            key_id: encrypted_data_key.key_id,
            encrypted_data_key_fingerprint,
            status_message: "Backup was encrypted with an external KMS-wrapped data key."
                .to_string(),
        },
    ))
}

fn decrypt_kms_envelope_payload(payload: &[u8]) -> AppResult<Vec<u8>> {
    let header_len_start = KMS_ENVELOPE_MAGIC.len();
    let header_len_end = header_len_start + 4;

    if payload.len() <= header_len_end + AES_GCM_NONCE_BYTES {
        return Err(AppError::Validation(
            "KMS encrypted backup has an unsupported format".to_string(),
        ));
    }

    let header_len = u32::from_be_bytes(
        payload[header_len_start..header_len_end]
            .try_into()
            .map_err(|_| {
                AppError::Validation("KMS envelope header length is invalid".to_string())
            })?,
    ) as usize;
    let header_end = header_len_end + header_len;
    let nonce_end = header_end + AES_GCM_NONCE_BYTES;

    if payload.len() <= nonce_end {
        return Err(AppError::Validation(
            "KMS encrypted backup payload is truncated".to_string(),
        ));
    }

    let header: KmsEnvelopeHeader = serde_json::from_slice(&payload[header_len_end..header_end])
        .map_err(|error| {
            AppError::Configuration(format!("KMS envelope header is invalid: {error}"))
        })?;

    if header.version != 1 {
        return Err(AppError::Validation(
            "KMS encrypted backup uses an unsupported envelope version".to_string(),
        ));
    }

    let data_key = decrypt_data_key_with_kms(&header)?;
    let cipher = Aes256Gcm::new_from_slice(&data_key).map_err(|error| {
        AppError::Infrastructure(format!("failed to initialize KMS backup cipher: {error}"))
    })?;

    cipher
        .decrypt(
            Nonce::from_slice(&payload[header_end..nonce_end]),
            &payload[nonce_end..],
        )
        .map_err(|error| {
            AppError::PermissionDenied(format!("KMS envelope backup decryption failed: {error}"))
        })
}

fn backup_key(secure_storage: &dyn SecureStorage) -> AppResult<[u8; AES_256_KEY_BYTES]> {
    if let Some(key) = external_key(BACKUP_ENCRYPTION_KEY_ENV, "backup encryption")? {
        return Ok(key);
    }

    if let Some(secret) =
        secure_storage.get_secret(BACKUP_SECRET_NAMESPACE, BACKUP_ENCRYPTION_KEY)?
    {
        let decoded = STANDARD.decode(secret).map_err(|error| {
            AppError::Configuration(format!("stored backup encryption key is invalid: {error}"))
        })?;
        return decoded.try_into().map_err(|_| {
            AppError::Configuration("stored backup encryption key has invalid length".to_string())
        });
    }

    let mut key = [0_u8; AES_256_KEY_BYTES];
    OsRng.fill_bytes(&mut key);
    secure_storage.store_secret(
        BACKUP_SECRET_NAMESPACE,
        BACKUP_ENCRYPTION_KEY,
        &STANDARD.encode(key),
    )?;

    Ok(key)
}

fn signing_key(secure_storage: &dyn SecureStorage) -> AppResult<([u8; AES_256_KEY_BYTES], String)> {
    if let Some(key) = external_key(BACKUP_SIGNING_KEY_ENV, "backup signing")? {
        return Ok((key, "environment".to_string()));
    }

    if let Some(secret) = secure_storage.get_secret(BACKUP_SECRET_NAMESPACE, BACKUP_SIGNING_KEY)? {
        let decoded = STANDARD.decode(secret).map_err(|error| {
            AppError::Configuration(format!("stored backup signing key is invalid: {error}"))
        })?;
        let key = decoded.try_into().map_err(|_| {
            AppError::Configuration("stored backup signing key has invalid length".to_string())
        })?;
        return Ok((key, "secureStorage".to_string()));
    }

    let mut key = [0_u8; AES_256_KEY_BYTES];
    OsRng.fill_bytes(&mut key);
    secure_storage.store_secret(
        BACKUP_SECRET_NAMESPACE,
        BACKUP_SIGNING_KEY,
        &STANDARD.encode(key),
    )?;

    Ok((key, "secureStorage".to_string()))
}

fn external_key(env_key: &str, label: &str) -> AppResult<Option<[u8; AES_256_KEY_BYTES]>> {
    let Ok(value) = std::env::var(env_key) else {
        return Ok(None);
    };
    let decoded = STANDARD.decode(value.trim()).map_err(|error| {
        AppError::Configuration(format!("{label} external key is not valid base64: {error}"))
    })?;
    let key = decoded.try_into().map_err(|_| {
        AppError::Configuration(format!(
            "{label} external key must decode to {AES_256_KEY_BYTES} bytes"
        ))
    })?;

    Ok(Some(key))
}

fn external_kms_config() -> AppResult<Option<ExternalKmsConfig>> {
    let Some(provider) = env_value(BACKUP_KMS_PROVIDER_ENV) else {
        return Ok(None);
    };
    let Some(key_id) = env_value(BACKUP_KMS_KEY_ID_ENV) else {
        return Err(AppError::Configuration(format!(
            "{BACKUP_KMS_KEY_ID_ENV} is required when {BACKUP_KMS_PROVIDER_ENV} is set"
        )));
    };
    let provider = provider.to_ascii_lowercase();

    if !matches!(provider.as_str(), "aws" | "gcp") {
        return Err(AppError::Configuration(
            "backup KMS provider must be `aws` or `gcp`".to_string(),
        ));
    }

    if key_id.chars().any(char::is_control) || key_id.trim().is_empty() {
        return Err(AppError::Validation(
            "backup KMS key id must not be empty or contain control characters".to_string(),
        ));
    }

    Ok(Some(ExternalKmsConfig { provider, key_id }))
}

fn encrypt_data_key_with_kms(
    config: &ExternalKmsConfig,
    plaintext_key: &[u8; AES_256_KEY_BYTES],
) -> AppResult<KmsEncryptedDataKey> {
    match config.provider.as_str() {
        "aws" => encrypt_data_key_with_aws_kms(config, plaintext_key),
        "gcp" => encrypt_data_key_with_gcp_kms(config, plaintext_key),
        _ => Err(AppError::Configuration(
            "unsupported backup KMS provider".to_string(),
        )),
    }
}

fn decrypt_data_key_with_kms(header: &KmsEnvelopeHeader) -> AppResult<[u8; AES_256_KEY_BYTES]> {
    match header.provider.as_str() {
        "aws" => decrypt_data_key_with_aws_kms(header),
        "gcp" => decrypt_data_key_with_gcp_kms(header),
        _ => Err(AppError::Configuration(
            "unsupported backup KMS provider".to_string(),
        )),
    }
}

fn encrypt_data_key_with_aws_kms(
    config: &ExternalKmsConfig,
    plaintext_key: &[u8; AES_256_KEY_BYTES],
) -> AppResult<KmsEncryptedDataKey> {
    let plaintext_b64 = STANDARD.encode(plaintext_key);
    let output = run_kms_command(
        "aws",
        [
            "kms".to_string(),
            "encrypt".to_string(),
            "--key-id".to_string(),
            config.key_id.clone(),
            "--plaintext".to_string(),
            plaintext_b64,
            "--output".to_string(),
            "json".to_string(),
        ],
    )?;
    ensure_kms_success("aws kms encrypt", &output)?;
    let value: serde_json::Value = serde_json::from_str(&output.stdout).map_err(|error| {
        AppError::Infrastructure(format!(
            "AWS KMS encrypt output was not valid JSON: {error}"
        ))
    })?;
    let ciphertext_b64 = value
        .get("CiphertextBlob")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            AppError::Infrastructure(
                "AWS KMS encrypt output did not include CiphertextBlob".to_string(),
            )
        })?
        .to_string();

    Ok(KmsEncryptedDataKey {
        provider: "aws".to_string(),
        key_id: config.key_id.clone(),
        ciphertext_b64,
    })
}

fn decrypt_data_key_with_aws_kms(header: &KmsEnvelopeHeader) -> AppResult<[u8; AES_256_KEY_BYTES]> {
    let output = run_kms_command(
        "aws",
        [
            "kms".to_string(),
            "decrypt".to_string(),
            "--ciphertext-blob".to_string(),
            header.encrypted_data_key.clone(),
            "--output".to_string(),
            "json".to_string(),
        ],
    )?;
    ensure_kms_success("aws kms decrypt", &output)?;
    let value: serde_json::Value = serde_json::from_str(&output.stdout).map_err(|error| {
        AppError::Infrastructure(format!(
            "AWS KMS decrypt output was not valid JSON: {error}"
        ))
    })?;
    let plaintext_b64 = value
        .get("Plaintext")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            AppError::Infrastructure("AWS KMS decrypt output did not include Plaintext".to_string())
        })?;

    decode_data_key(plaintext_b64, "AWS KMS plaintext")
}

fn encrypt_data_key_with_gcp_kms(
    config: &ExternalKmsConfig,
    plaintext_key: &[u8; AES_256_KEY_BYTES],
) -> AppResult<KmsEncryptedDataKey> {
    let kms_key = parse_gcp_kms_key_id(&config.key_id)?;
    let work_dir = std::env::temp_dir().join(format!("axiomphp-kms-{}", Uuid::new_v4()));
    fs::create_dir_all(&work_dir).map_err(|error| {
        AppError::Infrastructure(format!("failed to create KMS work directory: {error}"))
    })?;
    let plaintext_path = work_dir.join("data-key.bin");
    let ciphertext_path = work_dir.join("data-key.bin.enc");
    fs::write(&plaintext_path, plaintext_key).map_err(|error| {
        AppError::Infrastructure(format!("failed to write KMS plaintext data key: {error}"))
    })?;
    let output = run_kms_command(
        "gcloud",
        [
            "kms".to_string(),
            "encrypt".to_string(),
            "--project".to_string(),
            kms_key.project,
            "--location".to_string(),
            kms_key.location,
            "--keyring".to_string(),
            kms_key.key_ring,
            "--key".to_string(),
            kms_key.key,
            "--plaintext-file".to_string(),
            plaintext_path.to_string_lossy().into_owned(),
            "--ciphertext-file".to_string(),
            ciphertext_path.to_string_lossy().into_owned(),
        ],
    )?;
    ensure_kms_success("gcloud kms encrypt", &output)?;
    let ciphertext = fs::read(&ciphertext_path).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to read GCP KMS encrypted data key: {error}"
        ))
    })?;
    let _ = fs::remove_dir_all(&work_dir);

    Ok(KmsEncryptedDataKey {
        provider: "gcp".to_string(),
        key_id: config.key_id.clone(),
        ciphertext_b64: STANDARD.encode(ciphertext),
    })
}

fn decrypt_data_key_with_gcp_kms(header: &KmsEnvelopeHeader) -> AppResult<[u8; AES_256_KEY_BYTES]> {
    let kms_key = parse_gcp_kms_key_id(&header.key_id)?;
    let work_dir = std::env::temp_dir().join(format!("axiomphp-kms-{}", Uuid::new_v4()));
    fs::create_dir_all(&work_dir).map_err(|error| {
        AppError::Infrastructure(format!("failed to create KMS work directory: {error}"))
    })?;
    let ciphertext = STANDARD
        .decode(&header.encrypted_data_key)
        .map_err(|error| {
            AppError::Configuration(format!(
                "GCP KMS encrypted data key is invalid base64: {error}"
            ))
        })?;
    let ciphertext_path = work_dir.join("data-key.bin.enc");
    let plaintext_path = work_dir.join("data-key.bin");
    fs::write(&ciphertext_path, ciphertext).map_err(|error| {
        AppError::Infrastructure(format!("failed to write KMS ciphertext data key: {error}"))
    })?;
    let output = run_kms_command(
        "gcloud",
        [
            "kms".to_string(),
            "decrypt".to_string(),
            "--project".to_string(),
            kms_key.project,
            "--location".to_string(),
            kms_key.location,
            "--keyring".to_string(),
            kms_key.key_ring,
            "--key".to_string(),
            kms_key.key,
            "--ciphertext-file".to_string(),
            ciphertext_path.to_string_lossy().into_owned(),
            "--plaintext-file".to_string(),
            plaintext_path.to_string_lossy().into_owned(),
        ],
    )?;
    ensure_kms_success("gcloud kms decrypt", &output)?;
    let plaintext = fs::read(&plaintext_path).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to read GCP KMS plaintext data key: {error}"
        ))
    })?;
    let _ = fs::remove_dir_all(&work_dir);

    plaintext.try_into().map_err(|_| {
        AppError::Configuration("GCP KMS plaintext data key has invalid length".to_string())
    })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct GcpKmsKeyId {
    project: String,
    location: String,
    key_ring: String,
    key: String,
}

fn parse_gcp_kms_key_id(value: &str) -> AppResult<GcpKmsKeyId> {
    let parts = value.split('/').collect::<Vec<_>>();

    if parts.len() != 8
        || parts[0] != "projects"
        || parts[2] != "locations"
        || parts[4] != "keyRings"
        || parts[6] != "cryptoKeys"
        || parts.iter().any(|part| part.trim().is_empty())
    {
        return Err(AppError::Validation(
            "GCP KMS key id must be projects/<project>/locations/<location>/keyRings/<ring>/cryptoKeys/<key>".to_string(),
        ));
    }

    Ok(GcpKmsKeyId {
        project: parts[1].to_string(),
        location: parts[3].to_string(),
        key_ring: parts[5].to_string(),
        key: parts[7].to_string(),
    })
}

fn decode_data_key(value: &str, label: &str) -> AppResult<[u8; AES_256_KEY_BYTES]> {
    let decoded = STANDARD.decode(value.trim()).map_err(|error| {
        AppError::Configuration(format!("{label} is not valid base64: {error}"))
    })?;

    if decoded.len() == AES_256_KEY_BYTES {
        return decoded.try_into().map_err(|_| {
            AppError::Configuration(format!("{label} must decode to {AES_256_KEY_BYTES} bytes"))
        });
    }

    let nested = String::from_utf8(decoded).map_err(|_| {
        AppError::Configuration(format!("{label} must decode to {AES_256_KEY_BYTES} bytes"))
    })?;
    external_key_from_value(&nested, label)
}

fn external_key_from_value(value: &str, label: &str) -> AppResult<[u8; AES_256_KEY_BYTES]> {
    let decoded = STANDARD.decode(value.trim()).map_err(|error| {
        AppError::Configuration(format!("{label} nested key is not valid base64: {error}"))
    })?;

    decoded.try_into().map_err(|_| {
        AppError::Configuration(format!("{label} must decode to {AES_256_KEY_BYTES} bytes"))
    })
}

fn run_kms_command(
    program_name: &str,
    args: impl IntoIterator<Item = impl Into<String>>,
) -> AppResult<ProcessOutput> {
    let program_path = ExecutableResolver::from_env()
        .resolve(program_name)
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "{program_name} CLI executable was not found on PATH"
            ))
        })?;
    let runner = CommandRunner::new(
        CommandPolicy::deny_all()
            .allow_program_paths([program_path.clone()])
            .with_default_timeout(KMS_COMMAND_TIMEOUT)
            .with_max_output_bytes(KMS_OUTPUT_LIMIT_BYTES),
    );
    let mut command = ProcessCommand::new(program_path.to_string_lossy().into_owned())
        .args(args)
        .timeout(KMS_COMMAND_TIMEOUT);

    for key in [
        "AWS_PROFILE",
        "AWS_REGION",
        "AWS_DEFAULT_REGION",
        "GOOGLE_APPLICATION_CREDENTIALS",
        "CLOUDSDK_CONFIG",
    ] {
        if let Ok(value) = std::env::var(key) {
            command = command.env(key, value);
        }
    }
    if let Some(home) = std::env::var_os("HOME").and_then(|value| value.into_string().ok()) {
        command = command.env("HOME", home);
    }

    runner.execute(command)
}

fn ensure_kms_success(label: &str, output: &ProcessOutput) -> AppResult<()> {
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

fn write_signature(
    secure_storage: &dyn SecureStorage,
    backup_path: &Path,
    metadata_path: &Path,
) -> AppResult<PathBuf> {
    let (key, key_source) = signing_key(secure_storage)?;
    let key_fingerprint = key_fingerprint(&key);
    let signature_path = signature_path_for(backup_path);
    let signature = DatabaseBackupSignature {
        algorithm: "hmac-sha256".to_string(),
        key_source,
        key_fingerprint,
        backup_path: backup_path.to_string_lossy().into_owned(),
        metadata_path: metadata_path.to_string_lossy().into_owned(),
        signature: STANDARD.encode(sign_file(&key, backup_path)?),
        signed_at: Utc::now(),
    };
    let payload = serde_json::to_vec_pretty(&signature).map_err(|error| {
        AppError::Infrastructure(format!("failed to serialize backup signature: {error}"))
    })?;

    fs::write(&signature_path, payload).map_err(|error| {
        AppError::Infrastructure(format!("failed to write backup signature: {error}"))
    })?;

    Ok(signature_path)
}

fn verify_signature_if_present(
    secure_storage: &dyn SecureStorage,
    backup_path: &Path,
) -> AppResult<bool> {
    let Some(signature) = read_signature(backup_path)? else {
        return Ok(false);
    };
    let (key, _key_source) = signing_key(secure_storage)?;
    let expected = sign_file(&key, backup_path)?;
    let actual = STANDARD.decode(signature.signature).map_err(|error| {
        AppError::Configuration(format!("backup signature value is invalid: {error}"))
    })?;

    if expected.as_slice() != actual.as_slice() {
        let artifact_sha256 = sha256_file_hex(backup_path)?;
        let trusted_artifacts = trusted_artifact_hashes(secure_storage)?;
        let trusted_fingerprints = trusted_signing_key_fingerprints(secure_storage)?;
        let artifact_trusted = trusted_artifacts
            .iter()
            .any(|trusted| trusted == &artifact_sha256);
        let fingerprint_trusted = !signature.key_fingerprint.trim().is_empty()
            && trusted_fingerprints
                .iter()
                .any(|trusted| trusted == &signature.key_fingerprint);

        if artifact_trusted && fingerprint_trusted {
            return Ok(true);
        }

        return Err(AppError::PermissionDenied(
            "backup signature verification failed".to_string(),
        ));
    }

    let trusted_fingerprints = trusted_signing_key_fingerprints(secure_storage)?;
    if !trusted_fingerprints.is_empty() {
        let fingerprint = if signature.key_fingerprint.trim().is_empty() {
            key_fingerprint(&key)
        } else {
            signature.key_fingerprint
        };

        if !trusted_fingerprints
            .iter()
            .any(|trusted| trusted == &fingerprint)
        {
            return Err(AppError::PermissionDenied(
                "backup signature key is not enrolled as trusted on this machine".to_string(),
            ));
        }
    }

    Ok(true)
}

fn read_signature(backup_path: &Path) -> AppResult<Option<DatabaseBackupSignature>> {
    let signature_path = signature_path_for(backup_path);

    if !signature_path.exists() {
        return Ok(None);
    }

    let signature = fs::read_to_string(&signature_path).map_err(|error| {
        AppError::Infrastructure(format!("failed to read backup signature: {error}"))
    })?;
    let signature: DatabaseBackupSignature = serde_json::from_str(&signature).map_err(|error| {
        AppError::Configuration(format!("backup signature file is invalid: {error}"))
    })?;

    Ok(Some(signature))
}

fn sign_file(key: &[u8; AES_256_KEY_BYTES], path: &Path) -> AppResult<Vec<u8>> {
    let mut file = File::open(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to open backup for signing: {error}"))
    })?;
    let mut mac = <Hmac<Sha256> as HmacKeyInit>::new_from_slice(key).map_err(|error| {
        AppError::Infrastructure(format!("failed to initialize backup signer: {error}"))
    })?;
    let mut buffer = [0_u8; 8192];

    loop {
        let read_count = file.read(&mut buffer).map_err(|error| {
            AppError::Infrastructure(format!("failed to read backup for signing: {error}"))
        })?;

        if read_count == 0 {
            break;
        }

        mac.update(&buffer[..read_count]);
    }

    Ok(mac.finalize().into_bytes().to_vec())
}

fn key_source(
    secure_storage: &dyn SecureStorage,
    env_key: &str,
    storage_key: &str,
) -> AppResult<String> {
    if std::env::var(env_key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .is_some()
    {
        return Ok("environment".to_string());
    }

    if secure_storage
        .get_secret(BACKUP_SECRET_NAMESPACE, storage_key)?
        .is_some()
    {
        return Ok("secureStorage".to_string());
    }

    Ok("notInitialized".to_string())
}

fn env_value(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn source_machine_label() -> Option<String> {
    ["HOSTNAME", "COMPUTERNAME"]
        .iter()
        .find_map(|key| env_value(key))
        .map(|value| value.chars().take(128).collect())
}

fn key_fingerprint(key: &[u8; AES_256_KEY_BYTES]) -> String {
    let digest = Sha256::digest(key);

    hex_encode(&digest)
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

fn trusted_signing_key_fingerprints(secure_storage: &dyn SecureStorage) -> AppResult<Vec<String>> {
    let Some(payload) =
        secure_storage.get_secret(BACKUP_SECRET_NAMESPACE, BACKUP_TRUST_FINGERPRINTS_KEY)?
    else {
        return Ok(Vec::new());
    };

    serde_json::from_str::<Vec<String>>(&payload).map_err(|error| {
        AppError::Configuration(format!(
            "trusted backup signing fingerprint store is invalid: {error}"
        ))
    })
}

fn enroll_trusted_signing_fingerprint(
    secure_storage: &dyn SecureStorage,
    fingerprint: &str,
) -> AppResult<()> {
    let mut trusted = trusted_signing_key_fingerprints(secure_storage)?;

    if !trusted.iter().any(|trusted| trusted == fingerprint) {
        trusted.push(fingerprint.to_string());
        secure_storage.store_secret(
            BACKUP_SECRET_NAMESPACE,
            BACKUP_TRUST_FINGERPRINTS_KEY,
            &serde_json::to_string(&trusted).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to serialize trusted backup fingerprints: {error}"
                ))
            })?,
        )?;
    }

    Ok(())
}

fn trusted_artifact_hashes(secure_storage: &dyn SecureStorage) -> AppResult<Vec<String>> {
    let Some(payload) =
        secure_storage.get_secret(BACKUP_SECRET_NAMESPACE, "trusted-backup-artifact-sha256")?
    else {
        return Ok(Vec::new());
    };

    serde_json::from_str::<Vec<String>>(&payload).map_err(|error| {
        AppError::Configuration(format!(
            "trusted backup artifact hash store is invalid: {error}"
        ))
    })
}

fn enroll_trusted_artifact_hash(
    secure_storage: &dyn SecureStorage,
    artifact_sha256: &str,
) -> AppResult<()> {
    let mut trusted = trusted_artifact_hashes(secure_storage)?;

    if !trusted.iter().any(|trusted| trusted == artifact_sha256) {
        trusted.push(artifact_sha256.to_string());
        secure_storage.store_secret(
            BACKUP_SECRET_NAMESPACE,
            "trusted-backup-artifact-sha256",
            &serde_json::to_string(&trusted).map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to serialize trusted backup artifact hashes: {error}"
                ))
            })?,
        )?;
    }

    Ok(())
}

fn sha256_file_hex(path: &Path) -> AppResult<String> {
    let mut file = File::open(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to open artifact for hashing: {error}"))
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

fn validate_output_directory(path: &str) -> AppResult<PathBuf> {
    let path = Path::new(path.trim());

    if !path.is_absolute() {
        return Err(AppError::Validation(
            "backup trust export directory must be absolute".to_string(),
        ));
    }

    if path.exists() && !path.is_dir() {
        return Err(AppError::Validation(
            "backup trust export path must be a directory".to_string(),
        ));
    }

    fs::create_dir_all(path).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to create backup trust export directory: {error}"
        ))
    })?;

    Ok(path.to_path_buf())
}

fn validate_trust_bundle_path(path: &str) -> AppResult<PathBuf> {
    let path = Path::new(path.trim()).canonicalize().map_err(|error| {
        AppError::Validation(format!(
            "backup trust bundle path must exist and be readable: {error}"
        ))
    })?;

    if !path.is_file() {
        return Err(AppError::Validation(
            "backup trust bundle path must point to a file".to_string(),
        ));
    }

    Ok(path)
}

fn validate_restore_path(path: &str) -> AppResult<PathBuf> {
    let path = Path::new(path.trim());

    if !path.is_absolute() {
        return Err(AppError::Validation(
            "restore backup path must be absolute".to_string(),
        ));
    }

    let canonical = path.canonicalize().map_err(|error| {
        AppError::Validation(format!(
            "restore backup path must exist and be readable: {error}"
        ))
    })?;

    if !canonical.is_file() {
        return Err(AppError::Validation(
            "restore backup path must point to a file".to_string(),
        ));
    }

    let file_name = canonical
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    if ![".sql", ".sql.gz", ".sql.enc", ".sql.gz.enc"]
        .iter()
        .any(|suffix| file_name.ends_with(suffix))
    {
        return Err(AppError::Validation(
            "restore backup path must be .sql, .sql.gz, .sql.enc, or .sql.gz.enc".to_string(),
        ));
    }

    Ok(canonical)
}

fn prune_backups(
    profile: &ProjectDatabaseProfile,
    current_backup_path: &Path,
    retention_days: u16,
) -> AppResult<Vec<String>> {
    let backup_dir = Path::new(&profile.backup_dir);
    let cutoff = Utc::now() - Duration::days(i64::from(retention_days));
    let current_metadata_path = metadata_path_for(current_backup_path);
    let mut pruned_paths = Vec::new();

    for entry in fs::read_dir(backup_dir).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to inspect backup retention directory: {error}"
        ))
    })? {
        let path = entry
            .map_err(|error| {
                AppError::Infrastructure(format!("failed to inspect backup artifact: {error}"))
            })?
            .path();

        if path == current_backup_path
            || path == current_metadata_path
            || !is_managed_backup_path(profile, &path)
        {
            continue;
        }

        let modified_at = path
            .metadata()
            .and_then(|metadata| metadata.modified())
            .map_err(|error| {
                AppError::Infrastructure(format!(
                    "failed to inspect backup artifact metadata: {error}"
                ))
            })?;
        let modified_at = chrono::DateTime::<Utc>::from(modified_at);

        if modified_at >= cutoff {
            continue;
        }

        fs::remove_file(&path).map_err(|error| {
            AppError::Infrastructure(format!("failed to prune expired backup artifact: {error}"))
        })?;
        pruned_paths.push(path.to_string_lossy().into_owned());
    }

    Ok(pruned_paths)
}

fn write_metadata(path: &Path, metadata: &DatabaseBackupMetadata) -> AppResult<()> {
    let payload = serde_json::to_vec_pretty(metadata).map_err(|error| {
        AppError::Infrastructure(format!("failed to serialize backup metadata: {error}"))
    })?;

    fs::write(path, payload).map_err(|error| {
        AppError::Infrastructure(format!("failed to write backup metadata: {error}"))
    })
}

fn encrypted_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("backup.sql");
    path.with_file_name(format!("{file_name}.enc"))
}

fn metadata_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("backup.sql");
    path.with_file_name(format!("{file_name}.metadata.json"))
}

fn signature_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("backup.sql");
    path.with_file_name(format!("{file_name}.sig.json"))
}

fn restore_work_dir(profile: &ProjectDatabaseProfile) -> AppResult<PathBuf> {
    let backup_dir = Path::new(&profile.backup_dir)
        .canonicalize()
        .map_err(|error| {
            AppError::Validation(format!(
                "backup directory must exist and be readable: {error}"
            ))
        })?;

    Ok(backup_dir.join(".restore-work"))
}

fn file_size(path: &Path) -> AppResult<u64> {
    path.metadata()
        .map(|metadata| metadata.len())
        .map_err(|error| {
            AppError::Infrastructure(format!("failed to inspect backup file: {error}"))
        })
}

fn path_has_extension(path: &Path, extension: &str) -> bool {
    path.extension().and_then(|value| value.to_str()) == Some(extension)
}

fn file_has_gzip_magic(path: &Path) -> AppResult<bool> {
    let mut file = File::open(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to inspect restore artifact: {error}"))
    })?;
    let mut bytes = [0_u8; 2];
    let read_count = file.read(&mut bytes).map_err(|error| {
        AppError::Infrastructure(format!("failed to inspect restore artifact: {error}"))
    })?;

    Ok(read_count == 2 && bytes == [0x1f, 0x8b])
}

fn is_gzip_bytes(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x1f, 0x8b])
}

fn is_managed_backup_path(profile: &ProjectDatabaseProfile, path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    let prefix = format!(
        "{}_{}_",
        profile.project_id.0,
        profile.database_type.as_key()
    );

    file_name.starts_with(&prefix) && file_name.contains(".sql")
}

#[cfg(unix)]
fn lock_private_directory(path: &Path) -> AppResult<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to lock restore directory permissions: {error}"
        ))
    })
}

#[cfg(not(unix))]
fn lock_private_directory(_path: &Path) -> AppResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gcp_kms_key_resource() {
        let parsed =
            parse_gcp_kms_key_id("projects/demo/locations/us/keyRings/backups/cryptoKeys/axiom")
                .expect("gcp kms key");

        assert_eq!(parsed.project, "demo");
        assert_eq!(parsed.location, "us");
        assert_eq!(parsed.key_ring, "backups");
        assert_eq!(parsed.key, "axiom");
    }

    #[test]
    fn decodes_nested_aws_kms_plaintext_key() {
        let raw_key = [7_u8; AES_256_KEY_BYTES];
        let nested = STANDARD.encode(raw_key);
        let aws_plaintext = STANDARD.encode(nested.as_bytes());

        let decoded = decode_data_key(&aws_plaintext, "aws plaintext").expect("data key");

        assert_eq!(decoded, raw_key);
    }
}

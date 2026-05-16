use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use chrono::{Duration, Utc};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use uuid::Uuid;

use crate::domain::database::database_config::{
    DatabaseBackupCompression, DatabaseBackupEncryption, DatabaseBackupMetadata,
    DatabaseBackupOptions, ProjectDatabaseProfile,
};
use crate::ports::secure_storage::SecureStorage;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

const BACKUP_SECRET_NAMESPACE: &str = "database-backup";
const BACKUP_ENCRYPTION_KEY: &str = "managed-backup-key";
const ENCRYPTION_MAGIC: &[u8] = b"AXIOMDB1";
const AES_256_KEY_BYTES: usize = 32;
const AES_GCM_NONCE_BYTES: usize = 12;
const MIN_RETENTION_DAYS: u16 = 1;
const MAX_RETENTION_DAYS: u16 = 365;

#[derive(Debug, Clone)]
pub struct FinalizedBackupArtifact {
    pub backup_path: PathBuf,
    pub metadata_path: PathBuf,
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
    pub temporary_paths: Vec<PathBuf>,
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

    if options.compression == DatabaseBackupCompression::Gzip {
        current_path = compress_backup(&current_path)?;
        compressed = true;
    }

    if options.encryption == DatabaseBackupEncryption::Aes256Gcm {
        current_path = encrypt_backup(secure_storage, &current_path)?;
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
        created_at,
    };

    write_metadata(&metadata_path, &metadata)?;

    let pruned_backup_paths = prune_backups(profile, &current_path, options.retention_days)?;

    Ok(FinalizedBackupArtifact {
        backup_path: current_path,
        metadata_path,
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

fn encrypt_backup(secure_storage: &dyn SecureStorage, path: &Path) -> AppResult<PathBuf> {
    let encrypted_path = encrypted_path_for(path);
    let plaintext = fs::read(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to read backup for encryption: {error}"))
    })?;
    let ciphertext = encrypt_bytes(secure_storage, &plaintext)?;

    fs::write(&encrypted_path, ciphertext).map_err(|error| {
        AppError::Infrastructure(format!("failed to write encrypted backup: {error}"))
    })?;
    fs::remove_file(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to remove plaintext backup: {error}"))
    })?;

    Ok(encrypted_path)
}

fn decrypt_backup(secure_storage: &dyn SecureStorage, path: &Path) -> AppResult<Vec<u8>> {
    let payload = fs::read(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to read encrypted backup: {error}"))
    })?;

    decrypt_bytes(secure_storage, &payload)
}

fn encrypt_bytes(secure_storage: &dyn SecureStorage, plaintext: &[u8]) -> AppResult<Vec<u8>> {
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

    Ok(payload)
}

fn decrypt_bytes(secure_storage: &dyn SecureStorage, payload: &[u8]) -> AppResult<Vec<u8>> {
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

fn backup_key(secure_storage: &dyn SecureStorage) -> AppResult<[u8; AES_256_KEY_BYTES]> {
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

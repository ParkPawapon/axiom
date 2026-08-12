mod support;

use std::fs;

use axiomphp_lib::domain::database::database_config::{
    DatabaseBackupCompression, DatabaseBackupEncryption, DatabaseBackupOptions,
};
#[cfg(unix)]
use axiomphp_lib::domain::database::database_config::{
    DatabaseBackupRemoteDestination, DatabaseBackupRemoteDestinationProvider, DatabaseBackupResult,
};
use axiomphp_lib::domain::database::database_type::DatabaseType;
#[cfg(unix)]
use axiomphp_lib::domain::project::project_id::ProjectId;
use axiomphp_lib::infrastructure::databases::backup_artifacts::{
    enroll_backup_artifact_trust, finalize_backup_artifact, prepare_restore_artifact,
};
#[cfg(unix)]
use axiomphp_lib::infrastructure::databases::remote_backup_destination::copy_backup_to_remote_destination;
#[cfg(unix)]
use chrono::Utc;
#[cfg(unix)]
use support::PathGuard;
use support::{database_profile, env_lock, EnvVarGuard, MemorySecureStorage, TestEnvironment};

#[test]
fn managed_backup_artifact_round_trips_with_cross_machine_trust() {
    let _env = env_lock();
    let _kms_provider = EnvVarGuard::remove("AXIOM_BACKUP_KMS_PROVIDER");
    let _kms_key = EnvVarGuard::remove("AXIOM_BACKUP_KMS_KEY_ID");
    let test_env = TestEnvironment::new("backup-trust-e2e");
    let backup_dir = test_env.path("backups");
    fs::create_dir_all(&backup_dir).expect("backup dir");
    let raw_backup = backup_dir.join("demo.sql");
    fs::write(&raw_backup, "create table demo(id int);\n").expect("raw backup");
    let profile = database_profile("backup-trust-e2e", DatabaseType::Mysql, backup_dir);
    let source_storage = MemorySecureStorage::default();
    let target_storage = MemorySecureStorage::default();

    let finalized = finalize_backup_artifact(
        &profile,
        &source_storage,
        raw_backup,
        DatabaseBackupOptions {
            compression: DatabaseBackupCompression::None,
            encryption: DatabaseBackupEncryption::None,
            retention_days: 30,
        },
    )
    .expect("finalized backup");
    enroll_backup_artifact_trust(&target_storage, &finalized.backup_path.to_string_lossy())
        .expect("artifact trust enrollment");

    let prepared = prepare_restore_artifact(
        &profile,
        &target_storage,
        &finalized.backup_path.to_string_lossy(),
    )
    .expect("trusted cross-machine restore");

    assert!(prepared.signature_verified);
    assert_eq!(
        prepared.sql_path.canonicalize().expect("prepared path"),
        finalized.backup_path.canonicalize().expect("backup path")
    );
}

#[cfg(unix)]
#[test]
fn cloud_backup_destinations_record_integrity_receipts_with_provider_clis() {
    let _env = env_lock();
    let test_env = TestEnvironment::new("cloud-backup-e2e");
    let bin_dir = test_env.path("bin");
    let log_path = test_env.path("cli.log");
    let log_path = log_path.to_string_lossy().into_owned();
    fs::create_dir_all(&bin_dir).expect("fake bin");
    support::write_executable(
        &bin_dir.join("aws"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{}'\nexit 0\n",
            log_path
        ),
    );
    support::write_executable(
        &bin_dir.join("gcloud"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{}'\nexit 0\n",
            log_path
        ),
    );
    support::write_executable(
        &bin_dir.join("scp"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{}'\nexit 0\n",
            log_path
        ),
    );
    let _path = PathGuard::prepend(&bin_dir);
    let _r2_endpoint = EnvVarGuard::set("AXIOM_R2_ENDPOINT_URL", "https://r2.example.test");
    let artifact = test_env.path("backup.sql.gz.enc");
    let metadata = test_env.path("backup.sql.gz.enc.metadata.json");
    let signature = test_env.path("backup.sql.gz.enc.sig");
    fs::write(&artifact, "backup").expect("artifact");
    fs::write(&metadata, "{}").expect("metadata");
    fs::write(&signature, "{}").expect("signature");
    let result = DatabaseBackupResult {
        project_id: ProjectId("cloud-backup-e2e".to_string()),
        database_type: DatabaseType::Postgresql,
        backup_path: artifact.to_string_lossy().into_owned(),
        metadata_path: Some(metadata.to_string_lossy().into_owned()),
        signature_path: Some(signature.to_string_lossy().into_owned()),
        compression: DatabaseBackupCompression::Gzip,
        encryption: DatabaseBackupEncryption::Aes256Gcm,
        compressed: true,
        encrypted: true,
        size_bytes: 6,
        pruned_backup_paths: Vec::new(),
        remote_copy_paths: Vec::new(),
        remote_copy_receipts: Vec::new(),
        status_message: "test backup".to_string(),
    };

    for (provider, destination_path) in [
        (
            DatabaseBackupRemoteDestinationProvider::S3,
            "s3://axiom-test/backups",
        ),
        (
            DatabaseBackupRemoteDestinationProvider::R2,
            "s3://axiom-r2-test/backups",
        ),
        (
            DatabaseBackupRemoteDestinationProvider::Gcs,
            "gs://axiom-test/backups",
        ),
        (
            DatabaseBackupRemoteDestinationProvider::Sftp,
            "sftp://user@example.test/backups",
        ),
    ] {
        let receipts = copy_backup_to_remote_destination(
            &result,
            &DatabaseBackupRemoteDestination {
                project_id: result.project_id.clone(),
                database_type: result.database_type,
                provider,
                enabled: true,
                destination_path: destination_path.to_string(),
                updated_at: Utc::now(),
            },
        )
        .expect("remote copy");

        assert_eq!(receipts.len(), 3);
        assert!(receipts.iter().all(|receipt| receipt.verified));
        assert!(receipts.iter().all(|receipt| receipt.sha256.len() == 64));
    }

    let cli_log = fs::read_to_string(log_path).expect("fake CLI log");
    assert!(cli_log.contains("s3"));
    assert!(cli_log.contains("storage"));
}

#[cfg(unix)]
#[test]
fn kms_envelope_restore_round_trips_through_allowlisted_aws_cli() {
    let _env = env_lock();
    let test_env = TestEnvironment::new("kms-envelope-e2e");
    let bin_dir = test_env.path("bin");
    let backup_dir = test_env.path("backups");
    fs::create_dir_all(&bin_dir).expect("fake bin");
    fs::create_dir_all(&backup_dir).expect("backup dir");
    support::write_executable(
        &bin_dir.join("aws"),
        r#"#!/bin/sh
if [ "$1" = "kms" ] && [ "$2" = "encrypt" ]; then
  plaintext=""
  while [ "$#" -gt 0 ]; do
    if [ "$1" = "--plaintext" ]; then
      shift
      plaintext="$1"
    fi
    shift || true
  done
  printf '{"CiphertextBlob":"%s"}\n' "$plaintext"
  exit 0
fi
if [ "$1" = "kms" ] && [ "$2" = "decrypt" ]; then
  ciphertext=""
  while [ "$#" -gt 0 ]; do
    if [ "$1" = "--ciphertext-blob" ]; then
      shift
      ciphertext="$1"
    fi
    shift || true
  done
  printf '{"Plaintext":"%s"}\n' "$ciphertext"
  exit 0
fi
exit 2
"#,
    );
    let _path = PathGuard::prepend(&bin_dir);
    let _provider = EnvVarGuard::set("AXIOM_BACKUP_KMS_PROVIDER", "aws");
    let _key = EnvVarGuard::set(
        "AXIOM_BACKUP_KMS_KEY_ID",
        "arn:aws:kms:us-east-1:111122223333:key/axiom-test",
    );
    let raw_backup = backup_dir.join("demo.sql");
    fs::write(&raw_backup, "insert into demo values (1);\n").expect("raw backup");
    let storage = MemorySecureStorage::default();
    let profile = database_profile("kms-envelope-e2e", DatabaseType::Mysql, backup_dir);

    let finalized = finalize_backup_artifact(
        &profile,
        &storage,
        raw_backup,
        DatabaseBackupOptions {
            compression: DatabaseBackupCompression::Gzip,
            encryption: DatabaseBackupEncryption::Aes256Gcm,
            retention_days: 30,
        },
    )
    .expect("kms encrypted backup");

    let prepared =
        prepare_restore_artifact(&profile, &storage, &finalized.backup_path.to_string_lossy())
            .expect("kms restore");
    let sql = fs::read_to_string(prepared.sql_path).expect("prepared SQL");

    assert!(finalized.encrypted);
    assert!(prepared.decrypted);
    assert!(prepared.decompressed);
    assert!(sql.contains("insert into demo"));
}

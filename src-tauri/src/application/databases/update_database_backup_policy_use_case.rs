use chrono::{Duration, Utc};

use crate::domain::database::database_config::{
    DatabaseBackupEncryption, DatabaseBackupPolicy, DatabaseBackupPolicyUpdate,
    DatabaseBackupPolicyUpdateResult,
};
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_backup_policy_repository::DatabaseBackupPolicyRepository;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

use super::database_type_parser::parse_database_type;

const MIN_INTERVAL_MINUTES: u32 = 5;
const MAX_INTERVAL_MINUTES: u32 = 60 * 24 * 30;
const MIN_RETENTION_DAYS: u16 = 1;
const MAX_RETENTION_DAYS: u16 = 365;

pub fn update_database_backup_policy(
    backup_policy_repository: &dyn DatabaseBackupPolicyRepository,
    database_repository: &dyn DatabaseProvisioningRepository,
    project_id: &str,
    database_type: &str,
    update: DatabaseBackupPolicyUpdate,
) -> AppResult<DatabaseBackupPolicyUpdateResult> {
    validate_policy_update(&update)?;
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let database_type = parse_database_type(database_type)?;

    database_repository
        .get_profile(&project_id, database_type)?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "{} database profile was not found for project `{}`",
                database_type.as_key(),
                project_id.0
            ))
        })?;

    let existing_policy = backup_policy_repository.get_policy(&project_id, database_type)?;
    let now = Utc::now();
    let next_run_at = update
        .enabled
        .then_some(now + Duration::minutes(i64::from(update.interval_minutes)));
    let policy = DatabaseBackupPolicy {
        project_id,
        database_type,
        enabled: update.enabled,
        interval_minutes: update.interval_minutes,
        retention_days: update.retention_days,
        compression: update.compression,
        encryption: update.encryption,
        last_run_at: existing_policy.and_then(|policy| policy.last_run_at),
        next_run_at,
        updated_at: now,
    };
    let policy = backup_policy_repository.save_policy(policy)?;

    Ok(DatabaseBackupPolicyUpdateResult {
        policy,
        status_message: "Database backup policy was saved.".to_string(),
    })
}

fn validate_policy_update(update: &DatabaseBackupPolicyUpdate) -> AppResult<()> {
    if !(MIN_INTERVAL_MINUTES..=MAX_INTERVAL_MINUTES).contains(&update.interval_minutes) {
        return Err(AppError::Validation(format!(
            "backup interval must be between {MIN_INTERVAL_MINUTES} and {MAX_INTERVAL_MINUTES} minutes"
        )));
    }

    if !(MIN_RETENTION_DAYS..=MAX_RETENTION_DAYS).contains(&update.retention_days) {
        return Err(AppError::Validation(format!(
            "backup retention must be between {MIN_RETENTION_DAYS} and {MAX_RETENTION_DAYS} days"
        )));
    }

    if update.encryption == DatabaseBackupEncryption::None {
        return Err(AppError::Validation(
            "scheduled backups must use encryption".to_string(),
        ));
    }

    Ok(())
}

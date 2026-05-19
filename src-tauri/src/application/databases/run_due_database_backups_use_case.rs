use chrono::{Duration, Utc};

use crate::domain::database::database_config::{
    DatabaseBackupPolicy, ScheduledDatabaseBackupRunResult,
};
use crate::infrastructure::databases::remote_backup_destination::copy_backup_to_remote_destination;
use crate::ports::database_backup_destination_repository::DatabaseBackupDestinationRepository;
use crate::ports::database_backup_policy_repository::DatabaseBackupPolicyRepository;
use crate::ports::database_provisioner::DatabaseProvisioner;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::shared::result::app_result::AppResult;

pub fn run_due_database_backups(
    backup_policy_repository: &dyn DatabaseBackupPolicyRepository,
    backup_destination_repository: &dyn DatabaseBackupDestinationRepository,
    database_repository: &dyn DatabaseProvisioningRepository,
    database_provisioner: &dyn DatabaseProvisioner,
) -> AppResult<ScheduledDatabaseBackupRunResult> {
    let now = Utc::now();
    let policies = backup_policy_repository.list_all_policies()?;
    let checked_policies = policies.len();
    let mut backups = Vec::new();
    let mut errors = Vec::new();
    let mut skipped_backups = 0_usize;

    for policy in policies {
        if !policy.enabled
            || policy
                .next_run_at
                .is_none_or(|next_run_at| next_run_at > now)
        {
            skipped_backups += 1;
            continue;
        }

        match run_policy_backup(
            backup_policy_repository,
            backup_destination_repository,
            database_repository,
            database_provisioner,
            policy,
            now,
        ) {
            Ok(Some(backup)) => backups.push(backup),
            Ok(None) => skipped_backups += 1,
            Err(error) => {
                skipped_backups += 1;
                errors.push(error.to_string());
            }
        }
    }

    let completed_backups = backups.len();

    Ok(ScheduledDatabaseBackupRunResult {
        checked_policies,
        completed_backups,
        skipped_backups,
        backups,
        errors,
        status_message: format!(
            "Scheduled database backup check completed: {completed_backups} backup(s), {skipped_backups} skipped."
        ),
    })
}

fn run_policy_backup(
    backup_policy_repository: &dyn DatabaseBackupPolicyRepository,
    backup_destination_repository: &dyn DatabaseBackupDestinationRepository,
    database_repository: &dyn DatabaseProvisioningRepository,
    database_provisioner: &dyn DatabaseProvisioner,
    mut policy: DatabaseBackupPolicy,
    now: chrono::DateTime<Utc>,
) -> AppResult<Option<crate::domain::database::database_config::DatabaseBackupResult>> {
    let Some(profile) =
        database_repository.get_profile(&policy.project_id, policy.database_type)?
    else {
        return Ok(None);
    };
    let mut backup =
        database_provisioner.backup_project_database(&profile, policy.backup_options())?;

    if let Some(destination) =
        backup_destination_repository.get_destination(&policy.project_id, policy.database_type)?
    {
        backup.remote_copy_receipts = copy_backup_to_remote_destination(&backup, &destination)?;
        backup.remote_copy_paths = backup
            .remote_copy_receipts
            .iter()
            .map(|receipt| receipt.remote_uri.clone())
            .collect();
    }

    policy.last_run_at = Some(now);
    policy.next_run_at = Some(now + Duration::minutes(i64::from(policy.interval_minutes)));
    policy.updated_at = now;
    backup_policy_repository.save_policy(policy)?;

    Ok(Some(backup))
}

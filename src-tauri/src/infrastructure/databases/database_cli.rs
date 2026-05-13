use std::path::Path;
use std::time::Duration;

use crate::domain::database::database_config::ProjectDatabaseProfile;
use crate::domain::database::database_type::DatabaseType;
use crate::domain::security::command_policy::{CommandPolicy, ProcessCommand, ProcessOutput};
use crate::infrastructure::process::command_runner::CommandRunner;
use crate::infrastructure::services::adapters::executable_resolver::ExecutableResolver;
use crate::ports::process_manager::ProcessManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

use super::database_identifiers::{env_value, escape_sql_literal, quote_identifier};

const DATABASE_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const BACKUP_RESTORE_TIMEOUT: Duration = Duration::from_secs(120);
pub const MIGRATION_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug)]
pub enum ProvisioningAttemptError {
    Pending(String),
    Failed(String),
}

pub fn create_database_resources(
    profile: &ProjectDatabaseProfile,
    password: &str,
) -> Result<(), ProvisioningAttemptError> {
    match profile.database_type {
        DatabaseType::Mysql => create_mysql_resources(profile, password),
        DatabaseType::Postgresql => create_postgres_resources(profile, password),
    }
}

pub fn run_mysql_backup(
    profile: &ProjectDatabaseProfile,
    password: &str,
    backup_path: &Path,
) -> AppResult<ProcessOutput> {
    run_database_command(
        "mysqldump",
        ProcessCommand::new("mysqldump")
            .args([
                "--host",
                &profile.host,
                "--port",
                &profile.port.to_string(),
                "--user",
                &profile.username,
                "--single-transaction",
                "--quick",
                "--result-file",
                &backup_path.to_string_lossy(),
                &profile.database_name,
            ])
            .env("MYSQL_PWD", password),
        BACKUP_RESTORE_TIMEOUT,
    )
}

pub fn run_postgres_backup(
    profile: &ProjectDatabaseProfile,
    password: &str,
    backup_path: &Path,
) -> AppResult<ProcessOutput> {
    run_database_command(
        "pg_dump",
        ProcessCommand::new("pg_dump")
            .args([
                "--host",
                &profile.host,
                "--port",
                &profile.port.to_string(),
                "--username",
                &profile.username,
                "--format",
                "plain",
                "--no-owner",
                "--file",
                &backup_path.to_string_lossy(),
                &profile.database_name,
            ])
            .env("PGPASSWORD", password),
        BACKUP_RESTORE_TIMEOUT,
    )
}

pub fn run_mysql_restore(
    profile: &ProjectDatabaseProfile,
    password: &str,
    sql_path: &Path,
) -> AppResult<ProcessOutput> {
    run_mysql_script(profile, password, sql_path, BACKUP_RESTORE_TIMEOUT)
}

pub fn run_postgres_restore(
    profile: &ProjectDatabaseProfile,
    password: &str,
    sql_path: &Path,
) -> AppResult<ProcessOutput> {
    run_postgres_script(profile, password, sql_path, BACKUP_RESTORE_TIMEOUT)
}

pub fn run_mysql_script(
    profile: &ProjectDatabaseProfile,
    password: &str,
    sql_path: &Path,
    timeout: Duration,
) -> AppResult<ProcessOutput> {
    run_database_command(
        "mysql",
        ProcessCommand::new("mysql")
            .args([
                "--host",
                &profile.host,
                "--port",
                &profile.port.to_string(),
                "--user",
                &profile.username,
                &profile.database_name,
            ])
            .stdin_file(sql_path)
            .env("MYSQL_PWD", password),
        timeout,
    )
}

pub fn run_postgres_script(
    profile: &ProjectDatabaseProfile,
    password: &str,
    sql_path: &Path,
    timeout: Duration,
) -> AppResult<ProcessOutput> {
    run_database_command(
        "psql",
        ProcessCommand::new("psql")
            .args([
                "--host",
                &profile.host,
                "--port",
                &profile.port.to_string(),
                "--username",
                &profile.username,
                "--dbname",
                &profile.database_name,
                "--set",
                "ON_ERROR_STOP=1",
            ])
            .stdin_file(sql_path)
            .env("PGPASSWORD", password),
        timeout,
    )
}

fn create_mysql_resources(
    profile: &ProjectDatabaseProfile,
    password: &str,
) -> Result<(), ProvisioningAttemptError> {
    let Some(admin_user) = env_value("AXIOM_MYSQL_ADMIN_USER") else {
        return Err(ProvisioningAttemptError::Pending(
            "MySQL data directory and credential are ready. Set AXIOM_MYSQL_ADMIN_USER before creating the database/user automatically.".to_string(),
        ));
    };

    let sql = format!(
        "CREATE DATABASE IF NOT EXISTS `{database}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;\
         CREATE USER IF NOT EXISTS '{username}'@'127.0.0.1' IDENTIFIED BY '{password}';\
         CREATE USER IF NOT EXISTS '{username}'@'localhost' IDENTIFIED BY '{password}';\
         GRANT ALL PRIVILEGES ON `{database}`.* TO '{username}'@'127.0.0.1';\
         GRANT ALL PRIVILEGES ON `{database}`.* TO '{username}'@'localhost';\
         FLUSH PRIVILEGES;",
        database = profile.database_name,
        username = profile.username,
        password = escape_sql_literal(password),
    );
    let mut command = ProcessCommand::new("mysql").args([
        "--batch",
        "--silent",
        "--host",
        &profile.host,
        "--port",
        &profile.port.to_string(),
        "--user",
        &admin_user,
        "--execute",
        &sql,
    ]);

    if let Some(admin_password) = env_value("AXIOM_MYSQL_ADMIN_PASSWORD") {
        command = command.env("MYSQL_PWD", admin_password);
    }

    run_database_command("mysql", command, DATABASE_COMMAND_TIMEOUT)
        .map(|_| ())
        .map_err(provisioning_failure("MySQL provisioning failed"))
}

fn create_postgres_resources(
    profile: &ProjectDatabaseProfile,
    password: &str,
) -> Result<(), ProvisioningAttemptError> {
    let admin_user =
        env_value("AXIOM_POSTGRES_ADMIN_USER").unwrap_or_else(|| "postgres".to_string());

    let exists_output = run_postgres_admin_command(
        profile,
        &admin_user,
        &format!(
            "SELECT 1 FROM pg_database WHERE datname = '{}';",
            escape_sql_literal(&profile.database_name)
        ),
    )
    .map_err(provisioning_failure(
        "PostgreSQL database existence check failed",
    ))?;

    if !exists_output.stdout.lines().any(|line| line.trim() == "1") {
        run_postgres_admin_command(
            profile,
            &admin_user,
            &format!(
                "CREATE DATABASE {};",
                quote_identifier(&profile.database_name)
            ),
        )
        .map_err(provisioning_failure("PostgreSQL database creation failed"))?;
    }

    let role_sql = format!(
        "DO $$ BEGIN \
         IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '{username_literal}') THEN \
         CREATE ROLE {username_ident} LOGIN PASSWORD '{password}'; \
         ELSE ALTER ROLE {username_ident} WITH LOGIN PASSWORD '{password}'; \
         END IF; END $$;\
         GRANT ALL PRIVILEGES ON DATABASE {database_ident} TO {username_ident};",
        username_literal = escape_sql_literal(&profile.username),
        username_ident = quote_identifier(&profile.username),
        password = escape_sql_literal(password),
        database_ident = quote_identifier(&profile.database_name),
    );

    run_postgres_admin_command(profile, &admin_user, &role_sql)
        .map(|_| ())
        .map_err(provisioning_failure("PostgreSQL role provisioning failed"))
}

fn run_postgres_admin_command(
    profile: &ProjectDatabaseProfile,
    admin_user: &str,
    sql: &str,
) -> AppResult<ProcessOutput> {
    let mut command = ProcessCommand::new("psql").args([
        "--host",
        &profile.host,
        "--port",
        &profile.port.to_string(),
        "--username",
        admin_user,
        "--dbname",
        "postgres",
        "--tuples-only",
        "--no-align",
        "--set",
        "ON_ERROR_STOP=1",
        "--command",
        sql,
    ]);

    if let Some(admin_password) = env_value("AXIOM_POSTGRES_ADMIN_PASSWORD") {
        command = command.env("PGPASSWORD", admin_password);
    }

    run_database_command("psql", command, DATABASE_COMMAND_TIMEOUT)
}

fn run_database_command(
    program_name: &str,
    command: ProcessCommand,
    timeout: Duration,
) -> AppResult<ProcessOutput> {
    let Some(program_path) = ExecutableResolver::from_env().resolve(program_name) else {
        return Err(AppError::Configuration(format!(
            "{program_name} executable was not found on PATH"
        )));
    };
    let runner = CommandRunner::new(
        CommandPolicy::deny_all()
            .allow_program_paths([program_path.clone()])
            .with_default_timeout(timeout),
    );
    let command = ProcessCommand {
        program: program_path.to_string_lossy().into_owned(),
        timeout: Some(timeout),
        ..command
    };
    let output = runner.execute(command)?;

    if output.timed_out {
        return Err(AppError::Infrastructure(format!(
            "{program_name} command timed out"
        )));
    }

    if output.exit_code != Some(0) {
        return Err(AppError::Infrastructure(format!(
            "{program_name} command failed with exit code {:?}: {}",
            output.exit_code,
            output.stderr.trim()
        )));
    }

    Ok(output)
}

fn provisioning_failure(
    context: &'static str,
) -> impl FnOnce(AppError) -> ProvisioningAttemptError {
    move |error| match error {
        AppError::Configuration(message) if message.contains("executable was not found") => {
            ProvisioningAttemptError::Pending(format!("{context}: {message}"))
        }
        AppError::Configuration(message) => {
            ProvisioningAttemptError::Pending(format!("{context}: {message}"))
        }
        error => ProvisioningAttemptError::Failed(format!("{context}: {error}")),
    }
}

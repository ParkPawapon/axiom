#![allow(dead_code)]

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use axiomphp_lib::domain::database::database_config::{
    DatabaseProvisioningStatus, ProjectDatabaseProfile,
};
use axiomphp_lib::domain::database::database_type::DatabaseType;
use axiomphp_lib::domain::project::project::Project;
use axiomphp_lib::domain::project::project_id::ProjectId;
use axiomphp_lib::domain::project::project_path::ProjectPath;
use axiomphp_lib::ports::secure_storage::SecureStorage;
use axiomphp_lib::shared::result::app_result::AppResult;
use chrono::Utc;
use uuid::Uuid;

pub static ENV_LOCK: Mutex<()> = Mutex::new(());

#[derive(Default)]
pub struct MemorySecureStorage {
    secrets: Mutex<BTreeMap<(String, String), String>>,
}

impl SecureStorage for MemorySecureStorage {
    fn store_secret(&self, namespace: &str, key: &str, secret: &str) -> AppResult<()> {
        let mut secrets = self.secrets.lock().expect("memory secure storage poisoned");
        secrets.insert((namespace.to_string(), key.to_string()), secret.to_string());
        Ok(())
    }

    fn get_secret(&self, namespace: &str, key: &str) -> AppResult<Option<String>> {
        let secrets = self.secrets.lock().expect("memory secure storage poisoned");
        Ok(secrets
            .get(&(namespace.to_string(), key.to_string()))
            .cloned())
    }

    fn delete_secret(&self, namespace: &str, key: &str) -> AppResult<()> {
        let mut secrets = self.secrets.lock().expect("memory secure storage poisoned");
        secrets.remove(&(namespace.to_string(), key.to_string()));
        Ok(())
    }
}

pub struct TestEnvironment {
    root: PathBuf,
}

impl TestEnvironment {
    pub fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!("axiom-{label}-{}", Uuid::new_v4().simple()));
        fs::create_dir_all(&root).expect("test environment root");
        Self { root }
    }

    #[allow(dead_code)]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub struct EnvVarGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvVarGuard {
    pub fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        Self { key, previous }
    }

    pub fn remove(key: &'static str) -> Self {
        let previous = std::env::var_os(key);
        std::env::remove_var(key);
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(previous) = &self.previous {
            std::env::set_var(self.key, previous);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

pub struct PathGuard {
    _guard: EnvVarGuard,
}

impl PathGuard {
    pub fn prepend(path: &Path) -> Self {
        let old_path = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = vec![path.to_path_buf()];
        paths.extend(std::env::split_paths(&old_path));
        let joined = std::env::join_paths(paths).expect("join PATH");

        Self {
            _guard: EnvVarGuard::set("PATH", joined),
        }
    }
}

pub fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[allow(dead_code)]
pub fn project(id: &str, document_root: PathBuf) -> Project {
    let now = Utc::now();

    Project {
        id: ProjectId(id.to_string()),
        name: format!("Project {id}"),
        document_root: ProjectPath(document_root.to_string_lossy().into_owned()),
        created_at: now,
        updated_at: now,
    }
}

pub fn database_profile(
    project_id: &str,
    database_type: DatabaseType,
    backup_dir: PathBuf,
) -> ProjectDatabaseProfile {
    let now = Utc::now();

    ProjectDatabaseProfile {
        project_id: ProjectId(project_id.to_string()),
        database_type,
        database_name: format!("db_{}", project_id.replace('-', "_")),
        username: format!("user_{}", project_id.replace('-', "_")),
        host: "127.0.0.1".to_string(),
        port: match database_type {
            DatabaseType::Mysql => 3306,
            DatabaseType::Postgresql => 5432,
        },
        data_dir: backup_dir.join("data").to_string_lossy().into_owned(),
        backup_dir: backup_dir.to_string_lossy().into_owned(),
        migration_dir: backup_dir.join("migrations").to_string_lossy().into_owned(),
        admin_url: None,
        status: DatabaseProvisioningStatus::Ready,
        status_message: "test profile".to_string(),
        applied_migrations: Vec::new(),
        created_at: now,
        updated_at: now,
    }
}

#[cfg(unix)]
pub fn write_executable(path: &Path, contents: &str) {
    use std::os::unix::fs::PermissionsExt;

    fs::write(path, contents).expect("write fake executable");
    let mut permissions = fs::metadata(path)
        .expect("fake executable metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("fake executable permissions");
}

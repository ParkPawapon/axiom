export type DatabaseType = "mysql" | "postgresql";

export type DatabaseProvisioningStatus = "failed" | "pending" | "ready";
export type ManagedDatabaseDependencyStatus = "installed" | "pending";
export type DatabaseBackupCompression = "gzip" | "none";
export type DatabaseBackupEncryption = "aes256Gcm" | "none";

export interface ProjectDatabaseProfile {
  readonly projectId: string;
  readonly databaseType: DatabaseType;
  readonly databaseName: string;
  readonly username: string;
  readonly host: string;
  readonly port: number;
  readonly dataDir: string;
  readonly backupDir: string;
  readonly migrationDir: string;
  readonly adminUrl?: string;
  readonly status: DatabaseProvisioningStatus;
  readonly statusMessage: string;
  readonly appliedMigrations: string[];
  readonly createdAt: string;
  readonly updatedAt: string;
}

export interface DatabaseProvisioningResult {
  readonly profile: ProjectDatabaseProfile;
  readonly credentialStored: boolean;
  readonly databaseCreated: boolean;
  readonly dependencyReport?: ManagedDatabaseDependencyReport | null;
  readonly phpmyadminAccess?: PhpMyAdminAccess | null;
  readonly serviceReport?: ManagedDatabaseServiceReport | null;
  readonly statusMessage: string;
}

export interface ManagedDatabasePackage {
  readonly packageName: string;
  readonly alreadyInstalled: boolean;
  readonly installedNow: boolean;
}

export interface ManagedDatabaseDependencyReport {
  readonly databaseType: DatabaseType;
  readonly provider: string;
  readonly status: ManagedDatabaseDependencyStatus;
  readonly packages: ManagedDatabasePackage[];
  readonly diagnostics: string[];
  readonly statusMessage: string;
}

export interface ManagedDatabaseServiceReport {
  readonly serviceId: string;
  readonly started: boolean;
  readonly statusMessage: string;
}

export interface PhpMyAdminAccess {
  readonly url: string;
  readonly documentRoot: string;
  readonly configPath: string;
  readonly reverseProxyConfigPath: string;
  readonly reverseProxyStarted: boolean;
  readonly statusMessage: string;
}

export interface DatabaseBackupResult {
  readonly projectId: string;
  readonly databaseType: DatabaseType;
  readonly backupPath: string;
  readonly metadataPath?: string | null;
  readonly compression: DatabaseBackupCompression;
  readonly encryption: DatabaseBackupEncryption;
  readonly compressed: boolean;
  readonly encrypted: boolean;
  readonly sizeBytes: number;
  readonly prunedBackupPaths: string[];
  readonly statusMessage: string;
}

export interface DatabaseRestoreResult {
  readonly projectId: string;
  readonly databaseType: DatabaseType;
  readonly backupPath: string;
  readonly restoredFromPath: string;
  readonly decrypted: boolean;
  readonly decompressed: boolean;
  readonly statusMessage: string;
}

export interface DatabaseBackupOptions {
  readonly compression: DatabaseBackupCompression;
  readonly encryption: DatabaseBackupEncryption;
  readonly retentionDays: number;
}

export interface DatabaseBackupPolicy {
  readonly projectId: string;
  readonly databaseType: DatabaseType;
  readonly enabled: boolean;
  readonly intervalMinutes: number;
  readonly retentionDays: number;
  readonly compression: DatabaseBackupCompression;
  readonly encryption: DatabaseBackupEncryption;
  readonly lastRunAt?: string | null;
  readonly nextRunAt?: string | null;
  readonly updatedAt: string;
}

export interface DatabaseBackupPolicyUpdate {
  readonly enabled: boolean;
  readonly intervalMinutes: number;
  readonly retentionDays: number;
  readonly compression: DatabaseBackupCompression;
  readonly encryption: DatabaseBackupEncryption;
}

export interface DatabaseBackupPolicyUpdateResult {
  readonly policy: DatabaseBackupPolicy;
  readonly statusMessage: string;
}

export interface ScheduledDatabaseBackupRunResult {
  readonly checkedPolicies: number;
  readonly completedBackups: number;
  readonly skippedBackups: number;
  readonly backups: DatabaseBackupResult[];
  readonly errors: string[];
  readonly statusMessage: string;
}

export interface DatabaseMigrationFile {
  readonly projectId: string;
  readonly databaseType: DatabaseType;
  readonly migrationPath: string;
  readonly statusMessage: string;
}

export interface DatabaseMigrationRunResult {
  readonly projectId: string;
  readonly databaseType: DatabaseType;
  readonly appliedMigrations: string[];
  readonly statusMessage: string;
}

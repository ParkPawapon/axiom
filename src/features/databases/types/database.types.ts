export type DatabaseType = "mysql" | "postgresql";

export type DatabaseProvisioningStatus = "failed" | "pending" | "ready";

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
  readonly statusMessage: string;
}

export interface DatabaseBackupResult {
  readonly projectId: string;
  readonly databaseType: DatabaseType;
  readonly backupPath: string;
  readonly statusMessage: string;
}

export interface DatabaseRestoreResult {
  readonly projectId: string;
  readonly databaseType: DatabaseType;
  readonly backupPath: string;
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

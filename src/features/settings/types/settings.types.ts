export type AppTheme = "voiceboxLight";

export interface AppSettings {
  readonly appName: string;
  readonly theme: AppTheme;
  readonly phpBinarySearchPaths: string[];
  readonly dockerContext?: string | null;
  readonly defaultPhpPort: number;
  readonly defaultMysqlPort: number;
  readonly defaultPostgresPort: number;
  readonly auditLogEnabled: boolean;
  readonly updatedAt: string;
}

export interface AppSettingsUpdate {
  readonly theme?: AppTheme;
  readonly phpBinarySearchPaths?: string[];
  readonly dockerContext?: string | null;
  readonly defaultPhpPort?: number;
  readonly defaultMysqlPort?: number;
  readonly defaultPostgresPort?: number;
  readonly auditLogEnabled?: boolean;
}

export interface AppSettingsUpdateResult {
  readonly settings: AppSettings;
  readonly statusMessage: string;
}

export interface ProjectPlaceholder {
  readonly id: string;
  readonly name: string;
}

export type PhpVersionSupportPhase = "active" | "security";

export interface PhpVersionOption {
  readonly version: string;
  readonly label: string;
  readonly supportPhase: PhpVersionSupportPhase;
  readonly recommended: boolean;
}

export interface ProjectPhpVersionConfig {
  readonly projectId: string;
  readonly selectedPhpVersion: string;
  readonly availablePhpVersions: PhpVersionOption[];
  readonly statusMessage: string;
}

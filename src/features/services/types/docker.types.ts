export type DockerComposeProfile =
  | "mailpit"
  | "mysql"
  | "php"
  | "postgresql"
  | "redis"
  | "reverseProxy";

export interface DockerProjectImageOverride {
  readonly profile: DockerComposeProfile;
  readonly image: string;
}

export interface DockerProjectResourceLimits {
  readonly cpus: number | null;
  readonly memoryMb: number | null;
}

export interface DockerRegistryTrustMetadata {
  readonly registry: string;
  readonly repository: string;
  readonly reference: string;
  readonly digest: string;
  readonly mediaType: string;
  readonly platformCount: number;
  readonly allowedRegistry: boolean;
  readonly statusMessage: string;
}

export interface DockerImagePinResolution {
  readonly profile: DockerComposeProfile;
  readonly sourceImage: string;
  readonly pinnedImage: string;
  readonly metadata: DockerRegistryTrustMetadata;
  readonly statusMessage: string;
}

export interface DockerImagePinResolutionReport {
  readonly resolutions: DockerImagePinResolution[];
  readonly diagnostics: string[];
  readonly statusMessage: string;
}

export interface DockerImageTrustEvaluation {
  readonly profile: DockerComposeProfile;
  readonly image: string;
  readonly pinnedByDigest: boolean;
  readonly registryAllowed: boolean;
  readonly metadataVerified: boolean;
  readonly allowed: boolean;
  readonly metadata?: DockerRegistryTrustMetadata;
  readonly statusMessage: string;
}

export interface DockerProjectServicePlan {
  readonly profile: DockerComposeProfile;
  readonly serviceName: string;
  readonly image: string;
  readonly hostPort?: number;
  readonly containerPort?: number;
  readonly statusMessage: string;
}

export interface DockerProjectVolumePlan {
  readonly name: string;
  readonly serviceName: string;
  readonly mountPath: string;
  readonly created: boolean;
}

export interface DockerProjectComposePlan {
  readonly projectId: string;
  readonly projectName: string;
  readonly composeProjectName: string;
  readonly composeFilePath: string;
  readonly composeFileWritten: boolean;
  readonly envFilePath: string;
  readonly reverseProxyConfigPath?: string;
  readonly profiles: DockerComposeProfile[];
  readonly services: DockerProjectServicePlan[];
  readonly volumes: DockerProjectVolumePlan[];
  readonly imageTrust: DockerImageTrustEvaluation[];
  readonly resourceLimits: DockerProjectResourceLimits;
  readonly diagnostics: string[];
  readonly generatedAt: string;
  readonly statusMessage: string;
}

export interface DockerProjectContainerStatus {
  readonly name: string;
  readonly serviceName: string;
  readonly state: string;
  readonly status: string;
}

export interface DockerProjectRuntimeStatus {
  readonly projectId: string;
  readonly composeProjectName: string;
  readonly engineRunning: boolean;
  readonly composeFileExists: boolean;
  readonly containers: DockerProjectContainerStatus[];
  readonly volumes: DockerProjectVolumePlan[];
  readonly diagnostics: string[];
  readonly checkedAt: string;
  readonly statusMessage: string;
}

export interface DockerProjectActionResult {
  readonly projectId: string;
  readonly action: string;
  readonly plan: DockerProjectComposePlan;
  readonly runtime: DockerProjectRuntimeStatus;
  readonly statusMessage: string;
}

export interface DockerProjectLogReadResult {
  readonly projectId: string;
  readonly lines: string[];
  readonly truncated: boolean;
  readonly statusMessage: string;
}

export interface DockerProjectVolumeLifecycleResult {
  readonly projectId: string;
  readonly volumes: DockerProjectVolumePlan[];
  readonly statusMessage: string;
}

export interface DockerDiagnosticCheck {
  readonly name: string;
  readonly healthy: boolean;
  readonly statusMessage: string;
}

export interface DockerDiagnosticsReport {
  readonly cliFound: boolean;
  readonly engineRunning: boolean;
  readonly composeAvailable: boolean;
  readonly dockerContext?: string;
  readonly checks: DockerDiagnosticCheck[];
  readonly statusMessage: string;
}

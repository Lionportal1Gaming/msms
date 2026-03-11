export type ServerStatus =
  | "stopped"
  | "starting"
  | "running"
  | "stopping"
  | "error";

export interface ManagedServer {
  id: string;
  name: string;
  minecraftVersion: string;
  serverPath: string;
  jarPath: string;
  javaRuntimeId: string | null;
  status: ServerStatus;
  port: number;
  memoryMb: number;
  eulaAccepted: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface JavaRuntime {
  id: string;
  version: string;
  vendor: string;
  installPath: string;
  architecture: string;
  managedByApp: boolean;
}

export interface BackupJob {
  id: string;
  serverId: string;
  schedule: string;
  schedulePreset: BackupSchedulePreset | null;
  scheduleConfig: BackupScheduleConfig;
  retentionCount: number;
  destinationPath: string;
  nextRunAt: string | null;
  lastRunAt: string | null;
  lastStatus: BackupRunStatus;
  lastDurationMs: number | null;
  lastResult: string;
  isLegacySchedule: boolean;
}

export type BackupSchedulePreset = "hourly" | "daily" | "weekly";

export type BackupRunStatus = "idle" | "running" | "succeeded" | "failed";

export interface BackupScheduleConfig {
  intervalHours?: number;
  weekday?: number;
  hour?: number;
  minute?: number;
}

export interface BackupRunRecord {
  id: number;
  jobId: string;
  startedAt: string;
  finishedAt: string | null;
  status: BackupRunStatus;
  message: string;
}

export interface AppSettings {
  authMode: "password";
  updateChannel: UpdateChannel;
  diagnosticsOptIn: boolean;
  defaultServerDirectory: string;
  defaultBackupDirectory: string;
  defaultJavaDirectory: string;
}

export type UpdateChannel = "stable" | "beta";

export type ConsoleSource = "stdout" | "stderr" | "command" | "system";

export interface ConsoleEntry {
  serverId: string;
  source: ConsoleSource;
  message: string;
  timestamp: string;
}

export interface CommandHistoryEntry {
  serverId: string;
  command: string;
  timestamp: string;
}

export type ServerProperties = Record<string, string>;

export interface ProvisionServerRequest {
  name: string;
  minecraftVersion: string;
  targetDirectory: string;
  javaRuntimeId?: string;
  memoryMb: number;
  port: number;
  eulaAccepted: boolean;
  serverProperties: ServerProperties;
}

export interface ValidateProvisioningRequest {
  name: string;
  minecraftVersion: string;
  targetDirectory: string;
  javaRuntimeId?: string;
  memoryMb: number;
  port: number;
  serverProperties: ServerProperties;
}

export interface MinecraftVersionOption {
  id: string;
  releaseType: string;
  publishedAt: string;
  serverDownloadAvailable: boolean;
  requiredJavaMajor: number | null;
}

export interface ProvisionValidationIssue {
  field: string;
  step: string;
  message: string;
}

export interface ProvisionValidationResult {
  normalizedTargetDirectory: string;
  issues: ProvisionValidationIssue[];
}

export interface CreateBackupJobRequest {
  serverId: string;
  schedulePreset: BackupSchedulePreset;
  scheduleConfig: BackupScheduleConfig;
  retentionCount: number;
  destinationPath: string;
}

export interface SetPasswordRequest {
  password: string;
}

export interface UnlockRequest {
  password: string;
}

export interface UpdateSettingsRequest {
  updateChannel: UpdateChannel;
  diagnosticsOptIn: boolean;
  defaultServerDirectory: string;
  defaultBackupDirectory: string;
  defaultJavaDirectory: string;
}

export interface InstallJavaRuntimeRequest {
  vendor: string;
  version: string;
  downloadUrl: string;
  archiveKind: "zip" | "tar.gz";
}

export interface RestoreBackupRequest {
  archivePath: string;
  targetDirectory: string;
}

export interface ListBackupRunRecordsRequest {
  backupJobId: string;
}

export interface SendServerCommandRequest {
  serverId: string;
  command: string;
}

export interface UpdateServerPropertiesRequest {
  serverId: string;
  properties: ServerProperties;
}

export interface AvailableRelease {
  version: string;
  notes: string | null;
  publishedAt: string | null;
  downloadReady: boolean;
  installReady: boolean;
}

export type UpdateInstallState =
  | "idle"
  | "checking"
  | "ready"
  | "installing"
  | "restartRequired"
  | "error";

export interface UpdaterStatus {
  currentVersion: string;
  channel: UpdateChannel;
  lastCheckedAt: string | null;
  updateAvailable: boolean;
  availableRelease: AvailableRelease | null;
  installState: UpdateInstallState;
  error: string | null;
}

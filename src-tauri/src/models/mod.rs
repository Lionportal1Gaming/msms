use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedServer {
    pub id: String,
    pub name: String,
    pub minecraft_version: String,
    pub server_path: String,
    pub jar_path: String,
    pub java_runtime_id: Option<String>,
    pub status: ServerStatus,
    pub port: u16,
    pub memory_mb: u32,
    pub eula_accepted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaRuntime {
    pub id: String,
    pub version: String,
    pub vendor: String,
    pub install_path: String,
    pub architecture: String,
    pub managed_by_app: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupJob {
    pub id: String,
    pub server_id: String,
    pub schedule: String,
    pub schedule_preset: Option<BackupSchedulePreset>,
    pub schedule_config: BackupScheduleConfig,
    pub retention_count: u32,
    pub destination_path: String,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_status: BackupRunStatus,
    pub last_duration_ms: Option<i64>,
    pub last_result: String,
    pub is_legacy_schedule: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BackupSchedulePreset {
    Hourly,
    Daily,
    Weekly,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BackupScheduleConfig {
    pub interval_hours: Option<u32>,
    pub weekday: Option<u8>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BackupRunStatus {
    Idle,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRunRecord {
    pub id: i64,
    pub job_id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: BackupRunStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub auth_mode: String,
    pub update_channel: UpdateChannel,
    pub diagnostics_opt_in: bool,
    pub default_server_directory: String,
    pub default_backup_directory: String,
    pub default_java_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapStatus {
    pub password_configured: bool,
    pub unlocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterStatus {
    pub current_version: String,
    pub channel: UpdateChannel,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub update_available: bool,
    pub available_release: Option<AvailableRelease>,
    pub install_state: UpdateInstallState,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableRelease {
    pub version: String,
    pub notes: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub download_ready: bool,
    pub install_ready: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    Stable,
    Beta,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum UpdateInstallState {
    Idle,
    Checking,
    Ready,
    Installing,
    RestartRequired,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionServerRequest {
    pub name: String,
    pub minecraft_version: String,
    pub target_directory: String,
    pub java_runtime_id: Option<String>,
    pub memory_mb: u32,
    pub port: u16,
    pub eula_accepted: bool,
    pub server_properties: ServerProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateProvisioningRequest {
    pub name: String,
    pub minecraft_version: String,
    pub target_directory: String,
    pub java_runtime_id: Option<String>,
    pub memory_mb: u32,
    pub port: u16,
    pub server_properties: ServerProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftVersionOption {
    pub id: String,
    pub release_type: String,
    pub published_at: DateTime<Utc>,
    pub server_download_available: bool,
    pub required_java_major: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionValidationIssue {
    pub field: String,
    pub step: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionValidationResult {
    pub normalized_target_directory: String,
    pub issues: Vec<ProvisionValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsRequest {
    pub update_channel: UpdateChannel,
    pub diagnostics_opt_in: bool,
    pub default_server_directory: String,
    pub default_backup_directory: String,
    pub default_java_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackupJobRequest {
    pub server_id: String,
    pub schedule_preset: BackupSchedulePreset,
    pub schedule_config: BackupScheduleConfig,
    pub retention_count: u32,
    pub destination_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreBackupRequest {
    pub archive_path: String,
    pub target_directory: String,
}

pub type ServerProperties = BTreeMap<String, String>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleSource {
    Stdout,
    Stderr,
    Command,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleEntry {
    pub server_id: String,
    pub source: ConsoleSource,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandHistoryEntry {
    pub server_id: String,
    pub command: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateServerPropertiesRequest {
    pub server_id: String,
    pub properties: ServerProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPasswordRequest {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockRequest {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendServerCommandRequest {
    pub server_id: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallJavaRuntimeRequest {
    pub vendor: String,
    pub version: String,
    pub download_url: String,
    pub archive_kind: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("time parse error: {0}")]
    Time(#[from] chrono::ParseError),
    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("password hash error: {0}")]
    PasswordHash(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::PasswordHash(value.to_string())
    }
}

impl UpdaterStatus {
    pub fn new(current_version: impl Into<String>, channel: UpdateChannel) -> Self {
        Self {
            current_version: current_version.into(),
            channel,
            last_checked_at: None,
            update_available: false,
            available_release: None,
            install_state: UpdateInstallState::Idle,
            error: None,
        }
    }
}

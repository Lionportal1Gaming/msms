use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{BufRead, BufReader, Cursor, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::Engine;
use chrono::{DateTime, Datelike, Duration, Local, LocalResult, TimeZone, Utc};
use keyring::Entry;
use rand::rngs::OsRng;
use reqwest::{Client, Url};
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;
use walkdir::WalkDir;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::{
    db::{archive_destination, Database},
    models::{
        AppError, AppSettings, AvailableRelease, BackupJob, BackupRunRecord, BackupRunStatus,
        BackupScheduleConfig, BackupSchedulePreset, BootstrapStatus, CommandHistoryEntry,
        ConsoleEntry, ConsoleSource, CreateBackupJobRequest, InstallJavaRuntimeRequest,
        JavaRuntime, ManagedServer, MinecraftVersionOption, ProvisionServerRequest,
        ProvisionValidationIssue, ProvisionValidationResult, RestoreBackupRequest,
        SendServerCommandRequest, ServerProperties, ServerStatus, UpdateChannel,
        UpdateInstallState, UpdateServerPropertiesRequest, UpdateSettingsRequest, UpdaterStatus,
        ValidateProvisioningRequest,
    },
    state::{AppState, CachedMinecraftVersion, ManagedProcess},
};

const KEYRING_SERVICE: &str = "msms";
const KEYRING_PASSWORD_LABEL: &str = "local-admin-password-hash";
const VERSION_MANIFEST_URL: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";
const LIVE_CONSOLE_LIMIT: usize = 500;
const SCHEDULER_POLL_INTERVAL_SECS: u64 = 30;
const MIN_PROVISION_MEMORY_MB: u32 = 1024;
const MAX_PROVISION_MEMORY_MB: u32 = 32_768;
const DEFAULT_STABLE_UPDATER_ENDPOINT: &str = "https://downloads.example.com/msms/stable/latest.json";
const DEFAULT_BETA_UPDATER_ENDPOINT: &str = "https://downloads.example.com/msms/beta/latest.json";

pub struct AuthService;

impl AuthService {
    pub fn bootstrap_status(state: &AppState) -> Result<BootstrapStatus, AppError> {
        let password_configured = Self::load_password_hash(&state.database)?
            .map(|hash| !hash.is_empty())
            .unwrap_or(false);
        let unlocked = *state.unlocked.lock().expect("unlock state poisoned");
        Ok(BootstrapStatus {
            password_configured,
            unlocked,
        })
    }

    pub fn set_password(state: &AppState, password: &str) -> Result<(), AppError> {
        if password.len() < 12 {
            return Err(AppError::Message(
                "Password must be at least 12 characters long".into(),
            ));
        }

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        if Self::store_password_hash_in_keyring(&password_hash).is_err() {
            state.database.save_password_hash(&password_hash)?;
        }

        *state.unlocked.lock().expect("unlock state poisoned") = true;
        Ok(())
    }

    pub fn unlock(state: &AppState, password: &str) -> Result<(), AppError> {
        let password_hash = Self::load_password_hash(&state.database)?
            .ok_or_else(|| AppError::Message("Password has not been configured".into()))?;
        let parsed_hash = PasswordHash::new(&password_hash)?;

        Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;
        *state.unlocked.lock().expect("unlock state poisoned") = true;
        Ok(())
    }

    fn store_password_hash_in_keyring(password_hash: &str) -> Result<(), AppError> {
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_PASSWORD_LABEL)?;
        entry.set_password(password_hash)?;
        Ok(())
    }

    fn load_password_hash(database: &Database) -> Result<Option<String>, AppError> {
        if let Ok(entry) = Entry::new(KEYRING_SERVICE, KEYRING_PASSWORD_LABEL) {
            if let Ok(value) = entry.get_password() {
                if !value.is_empty() {
                    return Ok(Some(value));
                }
            }
        }
        database.load_password_hash()
    }
}

pub struct SettingsService;

impl SettingsService {
    pub fn get(database: &Database) -> Result<AppSettings, AppError> {
        database.get_settings()
    }

    pub fn update(
        state: &AppState,
        request: UpdateSettingsRequest,
    ) -> Result<AppSettings, AppError> {
        fs::create_dir_all(&request.default_server_directory)?;
        fs::create_dir_all(&request.default_backup_directory)?;
        fs::create_dir_all(&request.default_java_directory)?;

        let settings = state.database.save_settings(&AppSettings {
            auth_mode: "password".into(),
            update_channel: request.update_channel,
            diagnostics_opt_in: request.diagnostics_opt_in,
            default_server_directory: request.default_server_directory,
            default_backup_directory: request.default_backup_directory,
            default_java_directory: request.default_java_directory,
        })?;

        let mut updater_status = state
            .updater_status
            .lock()
            .expect("updater status poisoned");
        updater_status.channel = settings.update_channel;
        updater_status.last_checked_at = None;
        updater_status.update_available = false;
        updater_status.available_release = None;
        updater_status.install_state = UpdateInstallState::Idle;
        updater_status.error = None;

        Ok(settings)
    }
}

pub struct JavaService;

impl JavaService {
    pub fn discover(database: &Database) -> Result<Vec<JavaRuntime>, AppError> {
        let mut candidates = Vec::new();

        if let Ok(java_path) = which::which("java") {
            candidates.push(java_path);
        }

        let settings = database.get_settings()?;
        let default_java_dir = PathBuf::from(settings.default_java_directory);
        if default_java_dir.exists() {
            for entry in WalkDir::new(&default_java_dir)
                .min_depth(1)
                .max_depth(3)
                .into_iter()
                .filter_map(Result::ok)
            {
                if entry.file_name() == OsStr::new("java")
                    || entry.file_name() == OsStr::new("java.exe")
                {
                    candidates.push(entry.path().to_path_buf());
                }
            }
        }

        for path in deduplicate_paths(candidates) {
            if let Ok(runtime) = inspect_java_runtime(&path) {
                database.upsert_java_runtime(&runtime)?;
            }
        }

        database.list_java_runtimes()
    }

    pub async fn install(
        database: &Database,
        request: InstallJavaRuntimeRequest,
    ) -> Result<JavaRuntime, AppError> {
        let settings = database.get_settings()?;
        let install_root = Path::new(&settings.default_java_directory).join(format!(
            "{}-{}",
            request.vendor.to_lowercase(),
            request.version
        ));
        fs::create_dir_all(&install_root)?;

        let response = Client::new()
            .get(&request.download_url)
            .send()
            .await?
            .bytes()
            .await?;

        match request.archive_kind.as_str() {
            "zip" => {
                let reader = Cursor::new(response);
                let mut archive = ZipArchive::new(reader)?;
                archive.extract(&install_root)?;
            }
            "tar.gz" => {
                let reader = flate2::read::GzDecoder::new(Cursor::new(response));
                let mut archive = tar::Archive::new(reader);
                archive.unpack(&install_root)?;
            }
            value => {
                return Err(AppError::Message(format!(
                    "Unsupported archive kind: {}",
                    value
                )))
            }
        }

        let java_executable = find_java_binary(&install_root).ok_or_else(|| {
            AppError::Message("Unable to locate java executable after install".into())
        })?;
        let runtime = inspect_java_runtime(&java_executable)?.with_managed_flag(true);
        database.upsert_java_runtime(&runtime)
    }
}

pub struct ProvisioningService;

impl ProvisioningService {
    pub async fn list_versions(state: &AppState) -> Result<Vec<MinecraftVersionOption>, AppError> {
        Ok(fetch_cached_minecraft_versions(state)
            .await?
            .into_iter()
            .map(|entry| entry.option)
            .collect())
    }

    pub async fn validate(
        state: &AppState,
        request: ValidateProvisioningRequest,
    ) -> Result<ProvisionValidationResult, AppError> {
        validate_provisioning_request(state, &request).await
    }

    pub async fn provision(
        state: &AppState,
        request: ProvisionServerRequest,
    ) -> Result<ManagedServer, AppError> {
        if !request.eula_accepted {
            return Err(AppError::Message(
                "EULA must be accepted before provisioning a server".into(),
            ));
        }

        let validation = validate_provisioning_request(
            state,
            &ValidateProvisioningRequest {
                name: request.name.clone(),
                minecraft_version: request.minecraft_version.clone(),
                target_directory: request.target_directory.clone(),
                java_runtime_id: request.java_runtime_id.clone(),
                memory_mb: request.memory_mb,
                port: request.port,
                server_properties: request.server_properties.clone(),
            },
        )
        .await?;
        if let Some(issue) = validation.issues.first() {
            return Err(AppError::Message(issue.message.clone()));
        }

        let cached_versions = fetch_cached_minecraft_versions(state).await?;
        let version = cached_versions
            .into_iter()
            .find(|entry| entry.option.id == request.minecraft_version)
            .ok_or_else(|| {
                AppError::Message(format!(
                    "Minecraft version {} is not present in the Mojang manifest",
                    request.minecraft_version
                ))
            })?;
        let detail = load_version_detail(&version.detail_url).await?;
        let server_download = detail.downloads.server.ok_or_else(|| {
            AppError::Message("Selected version does not expose a server download".into())
        })?;
        let server_root = PathBuf::from(&validation.normalized_target_directory);
        fs::create_dir_all(&server_root)?;
        let jar_path = server_root.join("server.jar");
        let provision_result = async {
            let jar_bytes = Client::new()
                .get(server_download.url)
                .send()
                .await?
                .bytes()
                .await?;
            fs::write(&jar_path, jar_bytes)?;
            fs::write(server_root.join("eula.txt"), "eula=true\n")?;
            let properties = normalize_provision_server_properties(
                &request.name,
                request.port,
                &request.server_properties,
            );
            write_server_properties(&server_root.join("server.properties"), &properties)?;

            let now = Utc::now();
            let server = ManagedServer {
                id: format!("srv-{}", slugify(&request.name)),
                name: request.name,
                minecraft_version: request.minecraft_version,
                server_path: server_root.display().to_string(),
                jar_path: jar_path.display().to_string(),
                java_runtime_id: request.java_runtime_id,
                status: ServerStatus::Stopped,
                port: request.port,
                memory_mb: request.memory_mb,
                eula_accepted: request.eula_accepted,
                created_at: now,
                updated_at: now,
            };
            state.database.upsert_server(&server)
        }
        .await;
        if provision_result.is_err() {
            let _ = fs::remove_dir_all(&server_root);
        }

        provision_result
    }
}

pub struct ServerConfigurationService;

impl ServerConfigurationService {
    pub fn get_properties(
        database: &Database,
        server_id: &str,
    ) -> Result<ServerProperties, AppError> {
        let server = database.find_server(server_id)?;
        let properties_path = Path::new(&server.server_path).join("server.properties");
        read_server_properties(&properties_path)
    }

    pub fn update_properties(
        database: &Database,
        request: UpdateServerPropertiesRequest,
    ) -> Result<ServerProperties, AppError> {
        let server = database.find_server(&request.server_id)?;
        let properties_path = Path::new(&server.server_path).join("server.properties");
        write_server_properties(&properties_path, &request.properties)?;
        Ok(request.properties)
    }
}

pub struct ConsoleService;

impl ConsoleService {
    pub fn get_live_console(state: &AppState, server_id: &str) -> Vec<ConsoleEntry> {
        state
            .console_logs
            .lock()
            .expect("console log state poisoned")
            .get(server_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_console_history(
        state: &AppState,
        server_id: &str,
    ) -> Result<Vec<ConsoleEntry>, AppError> {
        let before = state
            .live_session_started_at
            .lock()
            .expect("session state poisoned")
            .get(server_id)
            .cloned();
        state.database.list_console_history(server_id, before)
    }

    pub fn get_command_history(
        database: &Database,
        server_id: &str,
    ) -> Result<Vec<CommandHistoryEntry>, AppError> {
        database.list_command_history(server_id)
    }

    pub fn send_command(
        state: &AppState,
        request: SendServerCommandRequest,
    ) -> Result<(), AppError> {
        let server = state.database.find_server(&request.server_id)?;
        if server.status != ServerStatus::Running {
            return Err(AppError::Message(
                "Server must be running before commands can be submitted".into(),
            ));
        }

        send_command_internal(state, &request.server_id, &request.command, true)
    }
}

pub struct ServerLifecycleService;

impl ServerLifecycleService {
    pub fn list_servers(database: &Database) -> Result<Vec<ManagedServer>, AppError> {
        database.list_servers()
    }

    pub fn start(state: &AppState, server_id: &str) -> Result<ManagedServer, AppError> {
        let mut server = state.database.find_server(server_id)?;
        let settings = state.database.get_settings()?;
        let java_bin = Self::resolve_java_binary(&state.database, server.java_runtime_id.as_deref())?
            .unwrap_or_else(|| "java".into());

        fs::create_dir_all(&settings.default_server_directory)?;
        let mut command = Command::new(java_bin);
        command
            .arg(format!("-Xms{}M", server.memory_mb))
            .arg(format!("-Xmx{}M", server.memory_mb))
            .arg("-jar")
            .arg(&server.jar_path)
            .arg("nogui")
            .current_dir(&server.server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::Message("Failed to capture server stdin".into()))?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let started_at = Utc::now();

        state
            .console_logs
            .lock()
            .expect("console log state poisoned")
            .insert(server_id.to_string(), Vec::new());
        state
            .live_session_started_at
            .lock()
            .expect("session state poisoned")
            .insert(server_id.to_string(), started_at);
        state
            .processes
            .lock()
            .expect("process state poisoned")
            .insert(
                server_id.to_string(),
                ManagedProcess {
                    child,
                    stdin: Arc::new(Mutex::new(stdin)),
                },
            );

        server.status = ServerStatus::Running;
        server.updated_at = Utc::now();
        state.database.upsert_server(&server)?;
        append_console_entry(
            state,
            server_id,
            ConsoleSource::System,
            "Server process started.",
        )?;

        if let Some(stdout) = stdout {
            spawn_log_reader(
                state.database.clone(),
                state.console_logs.clone(),
                server_id.to_string(),
                ConsoleSource::Stdout,
                stdout,
            );
        }
        if let Some(stderr) = stderr {
            spawn_log_reader(
                state.database.clone(),
                state.console_logs.clone(),
                server_id.to_string(),
                ConsoleSource::Stderr,
                stderr,
            );
        }

        Ok(server)
    }

    pub fn stop(state: &AppState, server_id: &str) -> Result<ManagedServer, AppError> {
        let mut server = state.database.find_server(server_id)?;
        let mut process = state
            .processes
            .lock()
            .expect("process state poisoned")
            .remove(server_id)
            .ok_or_else(|| AppError::Message("Server is not currently running".into()))?;

        server.status = ServerStatus::Stopping;
        server.updated_at = Utc::now();
        state.database.upsert_server(&server)?;
        append_console_entry(
            state,
            server_id,
            ConsoleSource::System,
            "Graceful shutdown requested.",
        )?;

        {
            let mut stdin = process.stdin.lock().expect("stdin state poisoned");
            stdin.write_all(b"stop\n")?;
            stdin.flush()?;
        }

        let _ = process.child.wait()?;
        append_console_entry(state, server_id, ConsoleSource::System, "Server stopped.")?;
        cleanup_live_session(state, server_id);

        server.status = ServerStatus::Stopped;
        server.updated_at = Utc::now();
        state.database.upsert_server(&server)
    }

    pub fn restart(state: &AppState, server_id: &str) -> Result<ManagedServer, AppError> {
        let _ = Self::stop(state, server_id)?;
        Self::start(state, server_id)
    }

    pub fn kill(state: &AppState, server_id: &str) -> Result<ManagedServer, AppError> {
        let mut server = state.database.find_server(server_id)?;
        let mut process = state
            .processes
            .lock()
            .expect("process state poisoned")
            .remove(server_id)
            .ok_or_else(|| AppError::Message("Server is not currently running".into()))?;

        append_console_entry(
            state,
            server_id,
            ConsoleSource::System,
            "Forced termination requested.",
        )?;
        let _ = process.child.kill();
        let _ = process.child.wait();
        append_console_entry(
            state,
            server_id,
            ConsoleSource::System,
            "Server process killed.",
        )?;
        cleanup_live_session(state, server_id);

        server.status = ServerStatus::Stopped;
        server.updated_at = Utc::now();
        state.database.upsert_server(&server)
    }

    fn resolve_java_binary(
        database: &Database,
        runtime_id: Option<&str>,
    ) -> Result<Option<String>, AppError> {
        if let Some(runtime_id) = runtime_id {
            let runtime = database
                .find_java_runtime(runtime_id)?
                .ok_or_else(|| AppError::Message("Pinned Java runtime is no longer available".into()))?;
            let candidate = Path::new(&runtime.install_path);
            if candidate.is_file() {
                return Ok(Some(candidate.display().to_string()));
            }
            if let Some(java_binary) = find_java_binary(candidate) {
                return Ok(Some(java_binary.display().to_string()));
            }
            return Err(AppError::Message(
                "Pinned Java runtime does not contain a usable java binary".into(),
            ));
        }

        if let Some(runtime) = database.list_java_runtimes()?.into_iter().next() {
            let candidate = Path::new(&runtime.install_path);
            if candidate.is_file() {
                return Ok(Some(candidate.display().to_string()));
            }
            if let Some(java_binary) = find_java_binary(candidate) {
                return Ok(Some(java_binary.display().to_string()));
            }
        }

        Ok(which::which("java")
            .ok()
            .map(|path| path.display().to_string()))
    }
}

pub struct BackupService;

impl BackupService {
    pub fn list_jobs(database: &Database) -> Result<Vec<BackupJob>, AppError> {
        database.list_backup_jobs()
    }

    pub fn list_run_records(
        database: &Database,
        backup_job_id: &str,
    ) -> Result<Vec<BackupRunRecord>, AppError> {
        database.list_backup_run_records(backup_job_id)
    }

    pub fn create_job(
        database: &Database,
        request: CreateBackupJobRequest,
    ) -> Result<BackupJob, AppError> {
        fs::create_dir_all(&request.destination_path)?;
        let next_run_at = compute_next_run_at(
            request.schedule_preset,
            &request.schedule_config,
            Utc::now(),
        )?;
        let backup_job = BackupJob {
            id: format!("backup-{}", uuid::Uuid::new_v4()),
            server_id: request.server_id,
            schedule: format_backup_schedule(request.schedule_preset, &request.schedule_config)?,
            schedule_preset: Some(request.schedule_preset),
            schedule_config: request.schedule_config,
            retention_count: request.retention_count,
            destination_path: request.destination_path,
            next_run_at: Some(next_run_at),
            last_run_at: None,
            last_status: BackupRunStatus::Idle,
            last_duration_ms: None,
            last_result: "Scheduled, waiting for first run.".into(),
            is_legacy_schedule: false,
        };
        database.upsert_backup_job(&backup_job)
    }

    pub fn run_job(state: &AppState, backup_job_id: &str) -> Result<String, AppError> {
        Self::execute_job(state, backup_job_id)
    }

    pub fn run_due_jobs(state: &AppState) -> Result<(), AppError> {
        let jobs = state.database.list_backup_jobs()?;
        let now = Utc::now();
        let mut last_error = None;

        for job in jobs.into_iter().filter(|job| {
            job.schedule_preset.is_some()
                && job
                    .next_run_at
                    .map(|next_run_at| next_run_at <= now)
                    .unwrap_or(false)
                && job.last_status != BackupRunStatus::Running
        }) {
            if let Err(error) = Self::execute_job(state, &job.id) {
                last_error = Some(error);
            }
        }

        if let Some(error) = last_error {
            return Err(error);
        }

        Ok(())
    }

    pub fn spawn_scheduler(state: AppState) {
        thread::spawn(move || {
            let _ = Self::run_due_jobs(&state);
            loop {
                thread::sleep(std::time::Duration::from_secs(SCHEDULER_POLL_INTERVAL_SECS));
                let _ = Self::run_due_jobs(&state);
            }
        });
    }

    pub fn restore(request: RestoreBackupRequest) -> Result<(), AppError> {
        let archive_file = File::open(request.archive_path)?;
        let mut archive = ZipArchive::new(archive_file)?;
        archive.extract(request.target_directory)?;
        Ok(())
    }

    fn execute_job(state: &AppState, backup_job_id: &str) -> Result<String, AppError> {
        let started_at = Utc::now();
        let mut backup_job = state.database.find_backup_job(backup_job_id)?;
        if backup_job.last_status == BackupRunStatus::Running {
            return Err(AppError::Message("Backup job is already running".into()));
        }
        let server = state.database.find_server(&backup_job.server_id)?;

        backup_job.last_status = BackupRunStatus::Running;
        backup_job.last_result = format!("Backup started at {}", started_at.to_rfc3339());
        backup_job.next_run_at = backup_job
            .schedule_preset
            .map(|schedule_preset| {
                compute_next_run_at(schedule_preset, &backup_job.schedule_config, started_at)
            })
            .transpose()?;
        state.database.upsert_backup_job(&backup_job)?;

        let result = Self::perform_archive(state, &backup_job, &server);
        let finished_at = Utc::now();
        let duration_ms = finished_at
            .signed_duration_since(started_at)
            .num_milliseconds();
        let message = match &result {
            Ok(archive_path) => format!("Backup created at {}", archive_path.display()),
            Err(error) => error.to_string(),
        };

        backup_job.last_run_at = Some(finished_at);
        backup_job.last_duration_ms = Some(duration_ms);
        backup_job.last_status = if result.is_ok() {
            BackupRunStatus::Succeeded
        } else {
            BackupRunStatus::Failed
        };
        backup_job.last_result = message.clone();
        state.database.upsert_backup_job(&backup_job)?;
        state.database.append_backup_run_record(&BackupRunRecord {
            id: 0,
            job_id: backup_job.id.clone(),
            started_at,
            finished_at: Some(finished_at),
            status: backup_job.last_status,
            message: message.clone(),
        })?;

        result.map(|_| message)
    }

    fn perform_archive(
        state: &AppState,
        backup_job: &BackupJob,
        server: &ManagedServer,
    ) -> Result<PathBuf, AppError> {
        fs::create_dir_all(&backup_job.destination_path)?;

        if server.status == ServerStatus::Running {
            send_command_internal(state, &server.id, "save-all", false)?;
            thread::sleep(pre_backup_delay());
        }

        let file_name = format!(
            "{}-{}.zip",
            slugify(&server.name),
            Utc::now().format("%Y%m%d%H%M%S")
        );
        let archive_path = archive_destination(&backup_job.destination_path, &file_name);
        create_zip_from_directory(Path::new(&server.server_path), &archive_path)?;
        enforce_retention(&backup_job.destination_path, backup_job.retention_count)?;
        Ok(archive_path)
    }
}

pub struct UpdateService;

impl UpdateService {
    pub fn get_status(state: &AppState) -> UpdaterStatus {
        state
            .updater_status
            .lock()
            .expect("updater status poisoned")
            .clone()
    }

    pub async fn check(
        state: &AppState,
        app_handle: &AppHandle,
    ) -> Result<UpdaterStatus, AppError> {
        let settings = state.database.get_settings()?;
        set_updater_status(
            state,
            |status| {
                status.channel = settings.update_channel;
                status.install_state = UpdateInstallState::Checking;
                status.error = None;
            },
        );

        let updater = app_handle
            .updater_builder()
            .endpoints(resolve_updater_endpoints(settings.update_channel)?)
            .map_err(|error| AppError::Message(error.to_string()))?
            .build()
            .map_err(|error| AppError::Message(error.to_string()))?;

        let checked_at = Utc::now();
        match updater.check().await {
            Ok(Some(update)) => {
                let available_release = available_release_from_metadata(
                    &update.version,
                    update.body.clone(),
                    update
                        .date
                        .and_then(|value| DateTime::<Utc>::from_timestamp(value.unix_timestamp(), value.nanosecond())),
                );
                let next_status = set_updater_status(state, |status| {
                    status.channel = settings.update_channel;
                    status.last_checked_at = Some(checked_at);
                    status.update_available = true;
                    status.available_release = Some(available_release.clone());
                    status.install_state = UpdateInstallState::Ready;
                    status.error = None;
                });
                Ok(next_status)
            }
            Ok(None) => {
                let next_status = set_updater_status(state, |status| {
                    status.channel = settings.update_channel;
                    status.last_checked_at = Some(checked_at);
                    status.update_available = false;
                    status.available_release = None;
                    status.install_state = UpdateInstallState::Idle;
                    status.error = None;
                });
                Ok(next_status)
            }
            Err(error) => {
                set_updater_status(state, |status| {
                    status.channel = settings.update_channel;
                    status.last_checked_at = Some(checked_at);
                    status.update_available = false;
                    status.available_release = None;
                    status.install_state = UpdateInstallState::Error;
                    status.error = Some(error.to_string());
                });
                Err(AppError::Message(error.to_string()))
            }
        }
    }

    pub async fn install(
        state: &AppState,
        app_handle: &AppHandle,
    ) -> Result<UpdaterStatus, AppError> {
        let settings = state.database.get_settings()?;
        let updater = app_handle
            .updater_builder()
            .endpoints(resolve_updater_endpoints(settings.update_channel)?)
            .map_err(|error| AppError::Message(error.to_string()))?
            .build()
            .map_err(|error| AppError::Message(error.to_string()))?;
        let update = updater
            .check()
            .await
            .map_err(|error| {
                set_updater_status(state, |status| {
                    status.channel = settings.update_channel;
                    status.last_checked_at = Some(Utc::now());
                    status.install_state = UpdateInstallState::Error;
                    status.error = Some(error.to_string());
                });
                AppError::Message(error.to_string())
            })?
            .ok_or_else(|| {
                set_updater_status(state, |status| {
                    status.channel = settings.update_channel;
                    status.last_checked_at = Some(Utc::now());
                    status.update_available = false;
                    status.available_release = None;
                    status.install_state = UpdateInstallState::Idle;
                    status.error = Some("No update is currently available.".into());
                });
                AppError::Message("No update is currently available.".into())
            })?;

        let available_release = available_release_from_metadata(
            &update.version,
            update.body.clone(),
            update
                .date
                .and_then(|value| DateTime::<Utc>::from_timestamp(value.unix_timestamp(), value.nanosecond())),
        );

        set_updater_status(state, |status| {
            status.channel = settings.update_channel;
            status.last_checked_at = Some(Utc::now());
            status.update_available = true;
            status.available_release = Some(available_release.clone());
            status.install_state = UpdateInstallState::Installing;
            status.error = None;
        });

        if let Err(error) = stop_running_servers_for_update(state) {
            set_updater_status(state, |status| {
                status.channel = settings.update_channel;
                status.install_state = UpdateInstallState::Error;
                status.error = Some(error.to_string());
            });
            return Err(error);
        }

        match update.download_and_install(|_, _| {}, || {}).await {
            Ok(()) => {
                let next_status = set_updater_status(state, |status| {
                    status.channel = settings.update_channel;
                    status.last_checked_at = Some(Utc::now());
                    status.update_available = false;
                    status.available_release = Some(available_release.clone());
                    status.install_state = UpdateInstallState::RestartRequired;
                    status.error = None;
                });
                Ok(next_status)
            }
            Err(error) => {
                set_updater_status(state, |status| {
                    status.channel = settings.update_channel;
                    status.last_checked_at = Some(Utc::now());
                    status.update_available = true;
                    status.available_release = Some(available_release);
                    status.install_state = UpdateInstallState::Error;
                    status.error = Some(error.to_string());
                });
                Err(AppError::Message(error.to_string()))
            }
        }
    }
}

fn resolve_updater_endpoints(channel: UpdateChannel) -> Result<Vec<Url>, AppError> {
    let endpoint = match channel {
        UpdateChannel::Stable => std::env::var("MSMS_UPDATER_STABLE_ENDPOINT")
            .unwrap_or_else(|_| DEFAULT_STABLE_UPDATER_ENDPOINT.to_string()),
        UpdateChannel::Beta => std::env::var("MSMS_UPDATER_BETA_ENDPOINT")
            .unwrap_or_else(|_| DEFAULT_BETA_UPDATER_ENDPOINT.to_string()),
    };

    Ok(vec![Url::parse(&endpoint).map_err(|error| AppError::Message(error.to_string()))?])
}

fn available_release_from_metadata(
    version: &str,
    notes: Option<String>,
    published_at: Option<DateTime<Utc>>,
) -> AvailableRelease {
    AvailableRelease {
        version: version.to_string(),
        notes,
        published_at,
        download_ready: true,
        install_ready: true,
    }
}

fn set_updater_status(
    state: &AppState,
    update: impl FnOnce(&mut UpdaterStatus),
) -> UpdaterStatus {
    let mut status = state
        .updater_status
        .lock()
        .expect("updater status poisoned");
    update(&mut status);
    status.clone()
}

fn stop_running_servers_for_update(state: &AppState) -> Result<(), AppError> {
    let running_servers = state
        .database
        .list_servers()?
        .into_iter()
        .filter(|server| server.status == ServerStatus::Running)
        .collect::<Vec<_>>();

    for server in running_servers {
        ServerLifecycleService::stop(state, &server.id).map_err(|error| {
            AppError::Message(format!(
                "Failed to stop {} before installing the update: {}",
                server.name, error
            ))
        })?;
    }

    Ok(())
}

async fn fetch_cached_minecraft_versions(
    state: &AppState,
) -> Result<Vec<CachedMinecraftVersion>, AppError> {
    {
        let cached = state
            .minecraft_versions
            .lock()
            .expect("minecraft version cache poisoned");
        if !cached.is_empty() {
            return Ok(cached.clone());
        }
    }

    let manifest: VersionManifest = Client::new()
        .get(VERSION_MANIFEST_URL)
        .send()
        .await?
        .json()
        .await?;

    let mut versions = Vec::new();
    for (index, entry) in manifest
        .versions
        .into_iter()
        .filter(|version| version.version_type == "release")
        .enumerate()
    {
        let detail = if index < 12 {
            load_version_detail(&entry.url).await.ok()
        } else {
            None
        };
        versions.push(build_cached_minecraft_version(entry, detail.as_ref()));
    }

    let mut cached = state
        .minecraft_versions
        .lock()
        .expect("minecraft version cache poisoned");
    *cached = versions.clone();
    Ok(versions)
}

async fn load_version_detail(detail_url: &str) -> Result<VersionDetail, AppError> {
    Ok(Client::new().get(detail_url).send().await?.json().await?)
}

fn build_cached_minecraft_version(
    entry: VersionManifestEntry,
    detail: Option<&VersionDetail>,
) -> CachedMinecraftVersion {
    let server_download_available = detail
        .and_then(|value| value.downloads.server.as_ref())
        .is_some();
    CachedMinecraftVersion {
        option: MinecraftVersionOption {
            id: entry.id,
            release_type: entry.version_type,
            published_at: entry.release_time,
            server_download_available: detail.is_none() || server_download_available,
            required_java_major: detail
                .and_then(|value| value.java_version.as_ref())
                .map(|value| value.major_version),
        },
        detail_url: entry.url,
    }
}

async fn validate_provisioning_request(
    state: &AppState,
    request: &ValidateProvisioningRequest,
) -> Result<ProvisionValidationResult, AppError> {
    let normalized_target_directory = normalize_target_directory(&request.target_directory)?;
    let mut issues = Vec::new();

    if request.name.trim().is_empty() || slugify(&request.name).is_empty() {
        issues.push(ProvisionValidationIssue {
            field: "name".into(),
            step: "details".into(),
            message: "Provide a server name with at least one letter or number.".into(),
        });
    }

    if request.memory_mb < MIN_PROVISION_MEMORY_MB || request.memory_mb > MAX_PROVISION_MEMORY_MB {
        issues.push(ProvisionValidationIssue {
            field: "memoryMb".into(),
            step: "details".into(),
            message: format!(
                "Memory must be between {} MB and {} MB.",
                MIN_PROVISION_MEMORY_MB, MAX_PROVISION_MEMORY_MB
            ),
        });
    }

    if request.port < 1024 {
        issues.push(ProvisionValidationIssue {
            field: "port".into(),
            step: "details".into(),
            message: "Use a TCP port between 1024 and 65535.".into(),
        });
    }

    if let Some(existing_server) = state.database.find_server_by_port(request.port)? {
        issues.push(ProvisionValidationIssue {
            field: "port".into(),
            step: "details".into(),
            message: format!(
                "Port {} is already assigned to {}.",
                request.port, existing_server.name
            ),
        });
    }

    if let Some(existing_server) = state
        .database
        .find_server_by_path(&normalized_target_directory)?
    {
        issues.push(ProvisionValidationIssue {
            field: "targetDirectory".into(),
            step: "details".into(),
            message: format!(
                "The selected directory is already managed by {}.",
                existing_server.name
            ),
        });
    }

    let target_path = PathBuf::from(&normalized_target_directory);
    if target_path.exists() {
        if target_path.is_file() {
            issues.push(ProvisionValidationIssue {
                field: "targetDirectory".into(),
                step: "details".into(),
                message: "The selected directory points to a file, not a folder.".into(),
            });
        } else if fs::read_dir(&target_path)?.next().is_some() {
            issues.push(ProvisionValidationIssue {
                field: "targetDirectory".into(),
                step: "details".into(),
                message: "The selected directory already contains files. Choose an empty folder."
                    .into(),
            });
        }
    }

    for key in request.server_properties.keys() {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            issues.push(ProvisionValidationIssue {
                field: "serverProperties".into(),
                step: "properties".into(),
                message: "Server property keys cannot be empty.".into(),
            });
            break;
        }
    }

    if let Some(port_override) = request.server_properties.get("server-port") {
        if port_override != &request.port.to_string() {
            issues.push(ProvisionValidationIssue {
                field: "serverProperties".into(),
                step: "properties".into(),
                message: "The server-port property must match the selected port.".into(),
            });
        }
    }

    let cached_versions = fetch_cached_minecraft_versions(state).await?;
    let selected = cached_versions
        .iter()
        .find(|entry| entry.option.id == request.minecraft_version)
        .cloned();
    let selected = match selected {
        Some(value) => value,
        None => {
            issues.push(ProvisionValidationIssue {
                field: "minecraftVersion".into(),
                step: "version".into(),
                message: format!(
                    "Minecraft version {} is not present in the Mojang release catalog.",
                    request.minecraft_version
                ),
            });
            return Ok(ProvisionValidationResult {
                normalized_target_directory,
                issues,
            });
        }
    };

    let detail = load_version_detail(&selected.detail_url).await?;
    let required_java_major = detail
        .java_version
        .as_ref()
        .map(|value| value.major_version);
    upsert_cached_version_detail(
        state,
        &selected.option.id,
        detail.downloads.server.is_some(),
        required_java_major,
    );

    if detail.downloads.server.is_none() {
        issues.push(ProvisionValidationIssue {
            field: "minecraftVersion".into(),
            step: "version".into(),
            message: "The selected release does not expose a Vanilla server download.".into(),
        });
    }

    match request.java_runtime_id.as_deref() {
        Some(runtime_id) => {
            let runtime = state.database.find_java_runtime(runtime_id)?.ok_or_else(|| {
                AppError::Message("Selected Java runtime could not be found.".into())
            })?;
            if let Some(required_java_major) = required_java_major {
                if let Some(runtime_major) = parse_java_major(&runtime.version) {
                    if runtime_major < required_java_major {
                        issues.push(ProvisionValidationIssue {
                            field: "javaRuntimeId".into(),
                            step: "version".into(),
                            message: format!(
                                "{} {} is too old for Minecraft {}. Java {} or newer is required.",
                                runtime.vendor, runtime.version, request.minecraft_version, required_java_major
                            ),
                        });
                    }
                }
            }
        }
        None => {
            issues.push(ProvisionValidationIssue {
                field: "javaRuntimeId".into(),
                step: "version".into(),
                message: "Choose a Java runtime before provisioning this server.".into(),
            });
        }
    }

    Ok(ProvisionValidationResult {
        normalized_target_directory,
        issues,
    })
}

fn upsert_cached_version_detail(
    state: &AppState,
    version_id: &str,
    server_download_available: bool,
    required_java_major: Option<u32>,
) {
    let mut cache = state
        .minecraft_versions
        .lock()
        .expect("minecraft version cache poisoned");
    if let Some(entry) = cache.iter_mut().find(|entry| entry.option.id == version_id) {
        entry.option.server_download_available = server_download_available;
        if required_java_major.is_some() {
            entry.option.required_java_major = required_java_major;
        }
    }
}

fn normalize_target_directory(value: &str) -> Result<String, AppError> {
    let path = PathBuf::from(value);
    let normalized = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()?.join(path)
    };
    Ok(normalized.display().to_string())
}

fn normalize_provision_server_properties(
    server_name: &str,
    port: u16,
    request_properties: &ServerProperties,
) -> ServerProperties {
    let mut properties = default_provision_server_properties(server_name, port);
    for (key, value) in request_properties {
        properties.insert(key.clone(), value.clone());
    }
    properties.insert("server-port".into(), port.to_string());
    properties
}

fn default_provision_server_properties(server_name: &str, port: u16) -> ServerProperties {
    let mut properties = ServerProperties::new();
    properties.insert("motd".into(), server_name.to_string());
    properties.insert("server-port".into(), port.to_string());
    properties.insert("difficulty".into(), "normal".into());
    properties.insert("max-players".into(), "20".into());
    properties.insert("online-mode".into(), "true".into());
    properties.insert("pvp".into(), "true".into());
    properties
}

fn parse_java_major(version: &str) -> Option<u32> {
    let normalized = version.trim();
    if let Some(remainder) = normalized.strip_prefix("1.") {
        return remainder.split(['.', '_']).next()?.parse().ok();
    }
    normalized.split(['.', '_', '-']).next()?.parse().ok()
}

trait ManagedFlag {
    fn with_managed_flag(self, managed_by_app: bool) -> Self;
}

impl ManagedFlag for JavaRuntime {
    fn with_managed_flag(mut self, managed_by_app: bool) -> Self {
        self.managed_by_app = managed_by_app;
        self
    }
}

fn send_command_internal(
    state: &AppState,
    server_id: &str,
    command: &str,
    persist_history: bool,
) -> Result<(), AppError> {
    let stdin = {
        let processes = state.processes.lock().expect("process state poisoned");
        let process = processes
            .get(server_id)
            .ok_or_else(|| AppError::Message("Server is not currently running".into()))?;
        process.stdin.clone()
    };

    {
        let mut handle = stdin.lock().expect("stdin state poisoned");
        handle.write_all(command.as_bytes())?;
        handle.write_all(b"\n")?;
        handle.flush()?;
    }

    append_console_entry(state, server_id, ConsoleSource::Command, command)?;
    if persist_history {
        state
            .database
            .append_command_history(&CommandHistoryEntry {
                server_id: server_id.to_string(),
                command: command.to_string(),
                timestamp: Utc::now(),
            })?;
    }
    Ok(())
}

fn append_console_entry(
    state: &AppState,
    server_id: &str,
    source: ConsoleSource,
    message: &str,
) -> Result<ConsoleEntry, AppError> {
    let entry = ConsoleEntry {
        server_id: server_id.to_string(),
        source,
        message: message.to_string(),
        timestamp: Utc::now(),
    };

    push_live_console_entry(&state.console_logs, &entry);
    state.database.append_console_entry(&entry)?;
    Ok(entry)
}

fn push_live_console_entry(
    console_logs: &Arc<Mutex<std::collections::HashMap<String, Vec<ConsoleEntry>>>>,
    entry: &ConsoleEntry,
) {
    let mut logs = console_logs.lock().expect("console logs poisoned");
    let server_entries = logs.entry(entry.server_id.clone()).or_default();
    server_entries.push(entry.clone());
    if server_entries.len() > LIVE_CONSOLE_LIMIT {
        let overflow = server_entries.len() - LIVE_CONSOLE_LIMIT;
        server_entries.drain(0..overflow);
    }
}

fn cleanup_live_session(state: &AppState, server_id: &str) {
    state
        .console_logs
        .lock()
        .expect("console log state poisoned")
        .remove(server_id);
    state
        .live_session_started_at
        .lock()
        .expect("session state poisoned")
        .remove(server_id);
}

fn compute_next_run_at(
    schedule_preset: BackupSchedulePreset,
    schedule_config: &BackupScheduleConfig,
    reference: DateTime<Utc>,
) -> Result<DateTime<Utc>, AppError> {
    match schedule_preset {
        BackupSchedulePreset::Hourly => {
            let interval_hours = schedule_config
                .interval_hours
                .ok_or_else(|| AppError::Message("Hourly schedules require an interval".into()))?;
            if interval_hours == 0 {
                return Err(AppError::Message(
                    "Hourly schedules require an interval of at least 1 hour".into(),
                ));
            }
            Ok(reference + Duration::hours(i64::from(interval_hours)))
        }
        BackupSchedulePreset::Daily => {
            let hour = schedule_config
                .hour
                .ok_or_else(|| AppError::Message("Daily schedules require an hour".into()))?;
            let minute = schedule_config
                .minute
                .ok_or_else(|| AppError::Message("Daily schedules require a minute".into()))?;
            let local_reference = reference.with_timezone(&Local);
            let mut candidate = build_local_datetime(
                local_reference.year(),
                local_reference.month(),
                local_reference.day(),
                hour,
                minute,
            )?;
            if candidate <= local_reference {
                let next_day = local_reference
                    .date_naive()
                    .checked_add_signed(Duration::days(1))
                    .ok_or_else(|| AppError::Message("Daily schedule overflowed".into()))?;
                candidate = build_local_datetime(
                    next_day.year(),
                    next_day.month(),
                    next_day.day(),
                    hour,
                    minute,
                )?;
            }
            Ok(candidate.with_timezone(&Utc))
        }
        BackupSchedulePreset::Weekly => {
            let weekday = schedule_config
                .weekday
                .ok_or_else(|| AppError::Message("Weekly schedules require a weekday".into()))?;
            if weekday > 6 {
                return Err(AppError::Message(
                    "Weekly schedules require a weekday between 0 and 6".into(),
                ));
            }
            let hour = schedule_config
                .hour
                .ok_or_else(|| AppError::Message("Weekly schedules require an hour".into()))?;
            let minute = schedule_config
                .minute
                .ok_or_else(|| AppError::Message("Weekly schedules require a minute".into()))?;
            let local_reference = reference.with_timezone(&Local);
            let current_weekday = local_reference.weekday().num_days_from_sunday();
            let days_until = (i64::from(weekday) - i64::from(current_weekday) + 7) % 7;
            let mut candidate_date = local_reference
                .date_naive()
                .checked_add_signed(Duration::days(days_until))
                .ok_or_else(|| AppError::Message("Weekly schedule overflowed".into()))?;
            let mut candidate = build_local_datetime(
                candidate_date.year(),
                candidate_date.month(),
                candidate_date.day(),
                hour,
                minute,
            )?;
            if candidate <= local_reference {
                candidate_date = candidate_date
                    .checked_add_signed(Duration::days(7))
                    .ok_or_else(|| AppError::Message("Weekly schedule overflowed".into()))?;
                candidate = build_local_datetime(
                    candidate_date.year(),
                    candidate_date.month(),
                    candidate_date.day(),
                    hour,
                    minute,
                )?;
            }
            Ok(candidate.with_timezone(&Utc))
        }
    }
}

fn format_backup_schedule(
    schedule_preset: BackupSchedulePreset,
    schedule_config: &BackupScheduleConfig,
) -> Result<String, AppError> {
    match schedule_preset {
        BackupSchedulePreset::Hourly => {
            let interval_hours = schedule_config
                .interval_hours
                .ok_or_else(|| AppError::Message("Hourly schedules require an interval".into()))?;
            if interval_hours == 0 {
                return Err(AppError::Message(
                    "Hourly schedules require an interval of at least 1 hour".into(),
                ));
            }
            if interval_hours == 1 {
                Ok("Every hour".into())
            } else {
                Ok(format!("Every {} hours", interval_hours))
            }
        }
        BackupSchedulePreset::Daily => Ok(format!(
            "Daily at {}",
            format_hhmm(
                schedule_config
                    .hour
                    .ok_or_else(|| AppError::Message("Daily schedules require an hour".into()))?,
                schedule_config
                    .minute
                    .ok_or_else(|| AppError::Message("Daily schedules require a minute".into()))?,
            )
        )),
        BackupSchedulePreset::Weekly => Ok(format!(
            "Weekly on {} at {}",
            weekday_name(schedule_config.weekday.ok_or_else(|| {
                AppError::Message("Weekly schedules require a weekday".into())
            })?,)?,
            format_hhmm(
                schedule_config
                    .hour
                    .ok_or_else(|| AppError::Message("Weekly schedules require an hour".into()))?,
                schedule_config
                    .minute
                    .ok_or_else(|| AppError::Message("Weekly schedules require a minute".into()))?,
            )
        )),
    }
}

fn build_local_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u8,
    minute: u8,
) -> Result<DateTime<Local>, AppError> {
    if hour > 23 || minute > 59 {
        return Err(AppError::Message(
            "Scheduled times must use a 24-hour clock between 00:00 and 23:59".into(),
        ));
    }

    match Local.with_ymd_and_hms(year, month, day, u32::from(hour), u32::from(minute), 0) {
        LocalResult::Single(value) => Ok(value),
        LocalResult::Ambiguous(first, _) => Ok(first),
        LocalResult::None => Err(AppError::Message(
            "Selected schedule time is invalid in the current timezone".into(),
        )),
    }
}

fn format_hhmm(hour: u8, minute: u8) -> String {
    format!("{:02}:{:02}", hour, minute)
}

fn weekday_name(weekday: u8) -> Result<&'static str, AppError> {
    match weekday {
        0 => Ok("Sunday"),
        1 => Ok("Monday"),
        2 => Ok("Tuesday"),
        3 => Ok("Wednesday"),
        4 => Ok("Thursday"),
        5 => Ok("Friday"),
        6 => Ok("Saturday"),
        _ => Err(AppError::Message(
            "Weekly schedules require a weekday between 0 and 6".into(),
        )),
    }
}

#[cfg(test)]
fn pre_backup_delay() -> std::time::Duration {
    std::time::Duration::from_millis(0)
}

#[cfg(not(test))]
fn pre_backup_delay() -> std::time::Duration {
    std::time::Duration::from_secs(2)
}

fn inspect_java_runtime(path: &Path) -> Result<JavaRuntime, AppError> {
    let output = Command::new(path).arg("-version").output()?;
    let raw = String::from_utf8_lossy(if output.stderr.is_empty() {
        &output.stdout
    } else {
        &output.stderr
    });
    let version = raw
        .lines()
        .next()
        .and_then(|line| line.split('"').nth(1))
        .unwrap_or("unknown")
        .to_string();
    let vendor = if raw.contains("OpenJDK") {
        "OpenJDK"
    } else if raw.contains("Temurin") {
        "Temurin"
    } else {
        "Unknown"
    };
    let architecture = std::env::consts::ARCH.to_string();
    Ok(JavaRuntime {
        id: format!(
            "java-{}",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(path.display().to_string())
        ),
        version,
        vendor: vendor.to_string(),
        install_path: path.display().to_string(),
        architecture,
        managed_by_app: false,
    })
}

fn deduplicate_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut deduped = Vec::new();
    for path in paths {
        if !deduped.iter().any(|existing| existing == &path) {
            deduped.push(path);
        }
    }
    deduped
}

fn find_java_binary(root: &Path) -> Option<PathBuf> {
    for entry in WalkDir::new(root)
        .min_depth(1)
        .max_depth(4)
        .into_iter()
        .flatten()
    {
        if entry.file_name() == OsStr::new("java") || entry.file_name() == OsStr::new("java.exe") {
            return Some(entry.path().to_path_buf());
        }
    }
    None
}

fn slugify(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn spawn_log_reader<T>(
    database: Database,
    console_logs: Arc<Mutex<std::collections::HashMap<String, Vec<ConsoleEntry>>>>,
    server_id: String,
    source: ConsoleSource,
    stream: T,
) where
    T: Read + Send + 'static,
{
    std::thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines().map_while(Result::ok) {
            let normalized = line.trim_end_matches('\r').to_string();
            let entry = ConsoleEntry {
                server_id: server_id.clone(),
                source,
                message: normalized,
                timestamp: Utc::now(),
            };
            push_live_console_entry(&console_logs, &entry);
            let _ = database.append_console_entry(&entry);
        }
    });
}

fn create_zip_from_directory(source_dir: &Path, archive_path: &Path) -> Result<(), AppError> {
    let file = File::create(archive_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(source_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let relative = path
            .strip_prefix(source_dir)
            .map_err(|error| AppError::Message(error.to_string()))?;

        if relative.as_os_str().is_empty() {
            continue;
        }

        let name = relative.to_string_lossy().replace('\\', "/");
        if entry.file_type().is_dir() {
            zip.add_directory(name, options)?;
            continue;
        }

        zip.start_file(name, options)?;
        let mut input = File::open(path)?;
        let mut buffer = Vec::new();
        input.read_to_end(&mut buffer)?;
        zip.write_all(&buffer)?;
    }

    zip.finish()?;
    Ok(())
}

fn read_server_properties(path: &Path) -> Result<ServerProperties, AppError> {
    if !path.exists() {
        return Ok(ServerProperties::new());
    }

    let content = fs::read_to_string(path)?;
    let mut properties = ServerProperties::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            properties.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    Ok(properties)
}

fn write_server_properties(path: &Path, properties: &ServerProperties) -> Result<(), AppError> {
    let mut lines = Vec::new();
    for (key, value) in properties {
        lines.push(format!("{}={}", key, value));
    }
    fs::write(path, format!("{}\n", lines.join("\n")))?;
    Ok(())
}

fn enforce_retention(destination_path: &str, retention_count: u32) -> Result<(), AppError> {
    let mut archives = fs::read_dir(destination_path)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension() == Some(OsStr::new("zip")))
        .collect::<Vec<_>>();
    archives.sort_by_key(|entry| entry.file_name());
    let archives_to_remove = archives.len().saturating_sub(retention_count as usize);
    for entry in archives.into_iter().take(archives_to_remove) {
        fs::remove_file(entry.path())?;
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct VersionManifest {
    versions: Vec<VersionManifestEntry>,
}

#[derive(serde::Deserialize)]
struct VersionManifestEntry {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    #[serde(rename = "releaseTime")]
    release_time: DateTime<Utc>,
    url: String,
}

#[derive(serde::Deserialize)]
struct VersionDetail {
    downloads: VersionDownloads,
    #[serde(rename = "javaVersion")]
    java_version: Option<VersionJavaRequirement>,
}

#[derive(serde::Deserialize)]
struct VersionDownloads {
    server: Option<DownloadAsset>,
}

#[derive(serde::Deserialize)]
struct DownloadAsset {
    url: String,
}

#[derive(serde::Deserialize)]
struct VersionJavaRequirement {
    #[serde(rename = "majorVersion")]
    major_version: u32,
}

#[cfg(test)]
mod tests {
    use std::{fs, process::Stdio, sync::Arc};

    use super::*;

    fn make_state() -> AppState {
        let temp_dir = std::env::temp_dir().join(format!("msms-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("temp dir");
        AppState {
            database: Database::initialize_at(temp_dir).expect("db"),
            processes: Arc::new(Mutex::new(std::collections::HashMap::new())),
            console_logs: Arc::new(Mutex::new(std::collections::HashMap::new())),
            live_session_started_at: Arc::new(Mutex::new(std::collections::HashMap::new())),
            minecraft_versions: Arc::new(Mutex::new(Vec::new())),
            updater_status: Arc::new(Mutex::new(UpdaterStatus::new(
                "0.1.0",
                UpdateChannel::Stable,
            ))),
            unlocked: Arc::new(Mutex::new(false)),
        }
    }

    fn insert_running_server(state: &AppState, server_id: &str) -> ManagedServer {
        let server_root = state.database.root_dir.join(server_id);
        fs::create_dir_all(&server_root).expect("server root");
        fs::write(server_root.join("server.properties"), "motd=Test\n").expect("server properties");
        let server = ManagedServer {
            id: server_id.to_string(),
            name: "Test".into(),
            minecraft_version: "1.21.4".into(),
            server_path: server_root.display().to_string(),
            jar_path: server_root.join("server.jar").display().to_string(),
            java_runtime_id: None,
            status: ServerStatus::Running,
            port: 25565,
            memory_mb: 2048,
            eula_accepted: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        state
            .database
            .upsert_server(&server)
            .expect("insert server");
        state
            .console_logs
            .lock()
            .expect("console log state poisoned")
            .insert(server_id.to_string(), Vec::new());
        state
            .live_session_started_at
            .lock()
            .expect("session state poisoned")
            .insert(server_id.to_string(), Utc::now());
        server
    }

    fn insert_backup_job(
        state: &AppState,
        server_id: &str,
        destination_path: &std::path::Path,
        next_run_at: Option<DateTime<Utc>>,
    ) -> BackupJob {
        let backup_job = BackupJob {
            id: format!("backup-{}", uuid::Uuid::new_v4()),
            server_id: server_id.to_string(),
            schedule: "Every hour".into(),
            schedule_preset: Some(BackupSchedulePreset::Hourly),
            schedule_config: BackupScheduleConfig {
                interval_hours: Some(1),
                weekday: None,
                hour: None,
                minute: None,
            },
            retention_count: 2,
            destination_path: destination_path.display().to_string(),
            next_run_at,
            last_run_at: None,
            last_status: BackupRunStatus::Idle,
            last_duration_ms: None,
            last_result: "Scheduled".into(),
            is_legacy_schedule: false,
        };
        state
            .database
            .upsert_backup_job(&backup_job)
            .expect("insert backup job")
    }

    fn attach_waiting_process(state: &AppState, server_id: &str) {
        let mut child = Command::new("python3")
            .arg("-c")
            .arg("import sys; sys.stdin.readline()")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn test process");
        let stdin = child.stdin.take().expect("stdin");
        state
            .processes
            .lock()
            .expect("process state poisoned")
            .insert(
                server_id.to_string(),
                ManagedProcess {
                    child,
                    stdin: Arc::new(Mutex::new(stdin)),
                },
            );
    }

    #[test]
    fn slugify_normalizes_server_names() {
        assert_eq!(slugify("Primary Survival!"), "primary-survival");
    }

    #[test]
    fn build_cached_version_extracts_java_requirement() {
        let cached = build_cached_minecraft_version(
            VersionManifestEntry {
                id: "1.21.4".into(),
                version_type: "release".into(),
                release_time: Utc::now(),
                url: "https://example.com/1.21.4.json".into(),
            },
            Some(&VersionDetail {
                downloads: VersionDownloads {
                    server: Some(DownloadAsset {
                        url: "https://example.com/server.jar".into(),
                    }),
                },
                java_version: Some(VersionJavaRequirement { major_version: 21 }),
            }),
        );

        assert_eq!(cached.option.id, "1.21.4");
        assert!(cached.option.server_download_available);
        assert_eq!(cached.option.required_java_major, Some(21));
    }

    #[test]
    fn resolve_java_binary_prefers_pinned_runtime() {
        let state = make_state();
        let pinned_java = state.database.root_dir.join("fake-java");
        fs::write(&pinned_java, "#!/bin/sh\n").expect("fake java");
        let pinned_runtime = JavaRuntime {
            id: "java-pinned".into(),
            version: "21.0.6".into(),
            vendor: "Temurin".into(),
            install_path: pinned_java.display().to_string(),
            architecture: "x64".into(),
            managed_by_app: true,
        };
        state
            .database
            .upsert_java_runtime(&pinned_runtime)
            .expect("insert runtime");

        let resolved = ServerLifecycleService::resolve_java_binary(
            &state.database,
            Some(&pinned_runtime.id),
        )
        .expect("resolve java");

        assert_eq!(resolved, Some(pinned_java.display().to_string()));
    }

    #[tokio::test]
    async fn validation_rejects_unknown_versions_duplicate_ports_and_invalid_memory() {
        let state = make_state();
        state
            .minecraft_versions
            .lock()
            .expect("minecraft versions cache poisoned")
            .push(CachedMinecraftVersion {
                option: MinecraftVersionOption {
                    id: "1.21.4".into(),
                    release_type: "release".into(),
                    published_at: Utc::now(),
                    server_download_available: true,
                    required_java_major: Some(21),
                },
                detail_url: "not-used".into(),
            });
        let existing = insert_running_server(&state, "srv-existing");
        let result = validate_provisioning_request(
            &state,
            &ValidateProvisioningRequest {
                name: "Primary Survival".into(),
                minecraft_version: "unknown-release".into(),
                target_directory: existing.server_path.clone(),
                java_runtime_id: None,
                memory_mb: 512,
                port: existing.port,
                server_properties: ServerProperties::new(),
            },
        )
        .await
        .expect("validate request");

        assert!(result
            .issues
            .iter()
            .any(|issue| issue.field == "minecraftVersion"));
        assert!(result.issues.iter().any(|issue| issue.field == "port"));
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.field == "memoryMb"));
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.field == "targetDirectory"));
    }

    #[tokio::test]
    async fn provisioning_failure_does_not_create_partial_server_records() {
        let state = make_state();
        state
            .minecraft_versions
            .lock()
            .expect("minecraft versions cache poisoned")
            .push(CachedMinecraftVersion {
                option: MinecraftVersionOption {
                    id: "1.21.4".into(),
                    release_type: "release".into(),
                    published_at: Utc::now(),
                    server_download_available: true,
                    required_java_major: Some(21),
                },
                detail_url: "not-a-valid-url".into(),
            });
        let runtime = JavaRuntime {
            id: "java-21".into(),
            version: "21.0.6".into(),
            vendor: "Temurin".into(),
            install_path: "/tmp/java-21".into(),
            architecture: "x64".into(),
            managed_by_app: true,
        };
        state
            .database
            .upsert_java_runtime(&runtime)
            .expect("insert runtime");
        let target_directory = state.database.root_dir.join("primary-survival");

        let result = ProvisioningService::provision(
            &state,
            ProvisionServerRequest {
                name: "Primary Survival".into(),
                minecraft_version: "1.21.4".into(),
                target_directory: target_directory.display().to_string(),
                java_runtime_id: Some(runtime.id.clone()),
                memory_mb: 4096,
                port: 25565,
                eula_accepted: true,
                server_properties: default_provision_server_properties("Primary Survival", 25565),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(state.database.list_servers().expect("servers").is_empty());
        assert!(!target_directory.exists());
    }

    #[test]
    fn send_command_fails_for_non_running_server() {
        let state = make_state();
        let server_root = state.database.root_dir.join("srv-stopped");
        fs::create_dir_all(&server_root).expect("server root");
        state
            .database
            .upsert_server(&ManagedServer {
                id: "srv-stopped".into(),
                name: "Stopped".into(),
                minecraft_version: "1.21.4".into(),
                server_path: server_root.display().to_string(),
                jar_path: server_root.join("server.jar").display().to_string(),
                java_runtime_id: None,
                status: ServerStatus::Stopped,
                port: 25565,
                memory_mb: 2048,
                eula_accepted: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .expect("insert server");

        let result = ConsoleService::send_command(
            &state,
            SendServerCommandRequest {
                server_id: "srv-stopped".into(),
                command: "list".into(),
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn send_command_writes_newline_terminated_input() {
        let state = make_state();
        insert_running_server(&state, "srv-command");
        let output_file = state.database.root_dir.join("command-output.txt");
        let mut child = Command::new("python3")
            .arg("-c")
            .arg("import pathlib, sys; pathlib.Path(sys.argv[1]).write_bytes(sys.stdin.buffer.readline())")
            .arg(&output_file)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn test process");
        let stdin = child.stdin.take().expect("stdin");
        state
            .processes
            .lock()
            .expect("process state poisoned")
            .insert(
                "srv-command".into(),
                ManagedProcess {
                    child,
                    stdin: Arc::new(Mutex::new(stdin)),
                },
            );

        ConsoleService::send_command(
            &state,
            SendServerCommandRequest {
                server_id: "srv-command".into(),
                command: "save-all".into(),
            },
        )
        .expect("send command");
        let mut child = state
            .processes
            .lock()
            .expect("process state poisoned")
            .remove("srv-command")
            .expect("process");
        let _ = child.child.wait().expect("wait");
        let command = fs::read_to_string(output_file).expect("read output");
        assert_eq!(command, "save-all\n");
    }

    #[test]
    fn stop_records_system_events() {
        let state = make_state();
        insert_running_server(&state, "srv-stop");
        let mut child = Command::new("python3")
            .arg("-c")
            .arg("import sys; sys.stdin.readline()")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn test process");
        let stdin = child.stdin.take().expect("stdin");
        state
            .processes
            .lock()
            .expect("process state poisoned")
            .insert(
                "srv-stop".into(),
                ManagedProcess {
                    child,
                    stdin: Arc::new(Mutex::new(stdin)),
                },
            );

        let server = ServerLifecycleService::stop(&state, "srv-stop").expect("stop");
        let history = state
            .database
            .list_console_history("srv-stop", None)
            .expect("history");

        assert_eq!(server.status, ServerStatus::Stopped);
        assert!(history
            .iter()
            .any(|entry| entry.message == "Graceful shutdown requested."));
        assert!(history
            .iter()
            .any(|entry| entry.message == "Server stopped."));
    }

    #[test]
    fn compute_next_run_supports_hourly_daily_and_weekly_presets() {
        let reference = Utc::now();

        let hourly = compute_next_run_at(
            BackupSchedulePreset::Hourly,
            &BackupScheduleConfig {
                interval_hours: Some(4),
                weekday: None,
                hour: None,
                minute: None,
            },
            reference,
        )
        .expect("hourly schedule");
        assert!(hourly >= reference + Duration::hours(4));

        let daily = compute_next_run_at(
            BackupSchedulePreset::Daily,
            &BackupScheduleConfig {
                interval_hours: None,
                weekday: None,
                hour: Some(2),
                minute: Some(0),
            },
            reference,
        )
        .expect("daily schedule");
        assert!(daily > reference);

        let weekly = compute_next_run_at(
            BackupSchedulePreset::Weekly,
            &BackupScheduleConfig {
                interval_hours: None,
                weekday: Some(3),
                hour: Some(4),
                minute: Some(30),
            },
            reference,
        )
        .expect("weekly schedule");
        assert!(weekly > reference);
    }

    #[test]
    fn running_backup_sends_save_all_before_archive_creation() {
        let state = make_state();
        insert_running_server(&state, "srv-backup");
        let output_file = state.database.root_dir.join("backup-command-output.txt");
        let mut child = Command::new("python3")
            .arg("-c")
            .arg("import pathlib, sys; pathlib.Path(sys.argv[1]).write_bytes(sys.stdin.buffer.readline())")
            .arg(&output_file)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn test process");
        let stdin = child.stdin.take().expect("stdin");
        state
            .processes
            .lock()
            .expect("process state poisoned")
            .insert(
                "srv-backup".into(),
                ManagedProcess {
                    child,
                    stdin: Arc::new(Mutex::new(stdin)),
                },
            );

        let backup_dir = state.database.root_dir.join("backups");
        let backup_job = insert_backup_job(
            &state,
            "srv-backup",
            &backup_dir,
            Some(Utc::now() - Duration::minutes(1)),
        );

        let result = BackupService::run_job(&state, &backup_job.id).expect("run backup");
        let mut child = state
            .processes
            .lock()
            .expect("process state poisoned")
            .remove("srv-backup")
            .expect("process");
        let _ = child.child.wait().expect("wait");
        let command = fs::read_to_string(output_file).expect("read output");

        assert_eq!(command, "save-all\n");
        assert!(result.contains("Backup created at"));
    }

    #[test]
    fn run_due_jobs_executes_due_backups_once_and_records_history() {
        let state = make_state();
        let server = insert_running_server(&state, "srv-scheduled");
        attach_waiting_process(&state, "srv-scheduled");

        let backup_dir = state.database.root_dir.join("scheduled-backups");
        let backup_job = insert_backup_job(
            &state,
            &server.id,
            &backup_dir,
            Some(Utc::now() - Duration::minutes(5)),
        );

        BackupService::run_due_jobs(&state).expect("run due backups");
        let job_after_first_run = state
            .database
            .find_backup_job(&backup_job.id)
            .expect("load backup job");
        let next_run_after_first = job_after_first_run.next_run_at.expect("next run");
        let run_records = state
            .database
            .list_backup_run_records(&backup_job.id)
            .expect("run records");

        BackupService::run_due_jobs(&state).expect("run due backups again");
        let job_after_second_scan = state
            .database
            .find_backup_job(&backup_job.id)
            .expect("load backup job after second scan");
        let run_records_after_second_scan = state
            .database
            .list_backup_run_records(&backup_job.id)
            .expect("run records after second scan");

        assert_eq!(job_after_first_run.last_status, BackupRunStatus::Succeeded);
        assert_eq!(run_records.len(), 1);
        assert_eq!(run_records_after_second_scan.len(), 1);
        assert_eq!(
            job_after_second_scan.next_run_at,
            Some(next_run_after_first)
        );
    }

    #[test]
    fn failed_backup_records_failure_and_advances_next_run() {
        let state = make_state();
        let server_root = state.database.root_dir.join("srv-failed");
        let backup_dir = state.database.root_dir.join("failed-backups");
        fs::write(&backup_dir, "not a directory").expect("seed invalid backup destination");
        let server = ManagedServer {
            id: "srv-failed".into(),
            name: "Failed".into(),
            minecraft_version: "1.21.4".into(),
            server_path: server_root.display().to_string(),
            jar_path: server_root.join("server.jar").display().to_string(),
            java_runtime_id: None,
            status: ServerStatus::Stopped,
            port: 25565,
            memory_mb: 2048,
            eula_accepted: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        state
            .database
            .upsert_server(&server)
            .expect("insert server");

        let backup_job = insert_backup_job(
            &state,
            &server.id,
            &backup_dir,
            Some(Utc::now() - Duration::minutes(1)),
        );

        let result = BackupService::run_job(&state, &backup_job.id);
        let job = state
            .database
            .find_backup_job(&backup_job.id)
            .expect("load backup job");
        let records = state
            .database
            .list_backup_run_records(&backup_job.id)
            .expect("run records");

        assert!(result.is_err());
        assert_eq!(job.last_status, BackupRunStatus::Failed);
        assert!(job.next_run_at.is_some());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].status, BackupRunStatus::Failed);
    }

    #[test]
    fn resolve_updater_endpoints_uses_separate_channel_feeds() {
        let stable = resolve_updater_endpoints(UpdateChannel::Stable).expect("stable endpoint");
        let beta = resolve_updater_endpoints(UpdateChannel::Beta).expect("beta endpoint");

        assert_ne!(stable, beta);
        assert!(stable[0].as_str().contains("/stable/"));
        assert!(beta[0].as_str().contains("/beta/"));
    }

    #[test]
    fn available_release_keeps_notes_and_publish_date() {
        let published_at = Utc::now();
        let release = available_release_from_metadata(
            "0.1.1",
            Some("Release notes".into()),
            Some(published_at),
        );

        assert_eq!(release.version, "0.1.1");
        assert_eq!(release.notes.as_deref(), Some("Release notes"));
        assert_eq!(release.published_at, Some(published_at));
        assert!(release.install_ready);
    }

    #[test]
    fn stop_running_servers_for_update_stops_managed_processes() {
        let state = make_state();
        insert_running_server(&state, "srv-update-stop");
        attach_waiting_process(&state, "srv-update-stop");

        stop_running_servers_for_update(&state).expect("stop running servers");
        let server = state
            .database
            .find_server("srv-update-stop")
            .expect("load server");

        assert_eq!(server.status, ServerStatus::Stopped);
        assert!(
            state
                .database
                .list_console_history("srv-update-stop", None)
                .expect("console history")
                .iter()
                .any(|entry| entry.message == "Server stopped.")
        );
    }

    #[test]
    fn stop_running_servers_for_update_aborts_when_stop_fails() {
        let state = make_state();
        let server_root = state.database.root_dir.join("srv-update-fail");
        fs::create_dir_all(&server_root).expect("server root");
        state
            .database
            .upsert_server(&ManagedServer {
                id: "srv-update-fail".into(),
                name: "UpdateFail".into(),
                minecraft_version: "1.21.4".into(),
                server_path: server_root.display().to_string(),
                jar_path: server_root.join("server.jar").display().to_string(),
                java_runtime_id: None,
                status: ServerStatus::Running,
                port: 25565,
                memory_mb: 2048,
                eula_accepted: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .expect("insert server");

        let result = stop_running_servers_for_update(&state);
        assert!(result.is_err());
    }
}

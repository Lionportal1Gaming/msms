use tauri::{AppHandle, State};

use crate::{
    models::{
        AppError, AppSettings, BackupJob, BackupRunRecord, BootstrapStatus, CommandHistoryEntry,
        ConsoleEntry, CreateBackupJobRequest, InstallJavaRuntimeRequest, JavaRuntime,
        ManagedServer, MinecraftVersionOption, ProvisionServerRequest, ProvisionValidationResult,
        RestoreBackupRequest, SendServerCommandRequest, ServerProperties, SetPasswordRequest,
        UnlockRequest, UpdateServerPropertiesRequest, UpdateSettingsRequest, UpdaterStatus,
        ValidateProvisioningRequest,
    },
    services::{
        AuthService, BackupService, ConsoleService, JavaService, ProvisioningService,
        ServerConfigurationService, ServerLifecycleService, SettingsService, UpdateService,
    },
    state::AppState,
};

#[tauri::command]
pub fn bootstrap_status(state: State<'_, AppState>) -> Result<BootstrapStatus, AppError> {
    AuthService::bootstrap_status(state.inner())
}

#[tauri::command]
pub fn list_servers(state: State<'_, AppState>) -> Result<Vec<ManagedServer>, AppError> {
    ServerLifecycleService::list_servers(&state.inner().database)
}

#[tauri::command]
pub async fn discover_java_runtimes(
    state: State<'_, AppState>,
) -> Result<Vec<JavaRuntime>, AppError> {
    JavaService::discover(&state.inner().database)
}

#[tauri::command]
pub async fn install_java_runtime(
    state: State<'_, AppState>,
    request: InstallJavaRuntimeRequest,
) -> Result<JavaRuntime, AppError> {
    JavaService::install(&state.inner().database, request).await
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    SettingsService::get(&state.inner().database)
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    request: UpdateSettingsRequest,
) -> Result<AppSettings, AppError> {
    SettingsService::update(state.inner(), request)
}

#[tauri::command]
pub fn set_app_password(
    state: State<'_, AppState>,
    request: SetPasswordRequest,
) -> Result<(), AppError> {
    AuthService::set_password(state.inner(), &request.password)
}

#[tauri::command]
pub fn unlock_app(state: State<'_, AppState>, request: UnlockRequest) -> Result<(), AppError> {
    AuthService::unlock(state.inner(), &request.password)
}

#[tauri::command]
pub async fn provision_server(
    state: State<'_, AppState>,
    request: ProvisionServerRequest,
) -> Result<ManagedServer, AppError> {
    ProvisioningService::provision(state.inner(), request).await
}

#[tauri::command]
pub async fn list_minecraft_versions(
    state: State<'_, AppState>,
) -> Result<Vec<MinecraftVersionOption>, AppError> {
    ProvisioningService::list_versions(state.inner()).await
}

#[tauri::command]
pub async fn validate_provisioning(
    state: State<'_, AppState>,
    request: ValidateProvisioningRequest,
) -> Result<ProvisionValidationResult, AppError> {
    ProvisioningService::validate(state.inner(), request).await
}

#[tauri::command]
pub fn start_server(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<ManagedServer, AppError> {
    ServerLifecycleService::start(state.inner(), &server_id)
}

#[tauri::command]
pub fn stop_server(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<ManagedServer, AppError> {
    ServerLifecycleService::stop(state.inner(), &server_id)
}

#[tauri::command]
pub fn restart_server(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<ManagedServer, AppError> {
    ServerLifecycleService::restart(state.inner(), &server_id)
}

#[tauri::command]
pub fn kill_server(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<ManagedServer, AppError> {
    ServerLifecycleService::kill(state.inner(), &server_id)
}

#[tauri::command]
pub fn get_server_console(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<Vec<ConsoleEntry>, AppError> {
    Ok(ConsoleService::get_live_console(state.inner(), &server_id))
}

#[tauri::command]
pub fn get_console_history(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<Vec<ConsoleEntry>, AppError> {
    ConsoleService::get_console_history(state.inner(), &server_id)
}

#[tauri::command]
pub fn get_command_history(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<Vec<CommandHistoryEntry>, AppError> {
    ConsoleService::get_command_history(&state.inner().database, &server_id)
}

#[tauri::command]
pub fn send_server_command(
    state: State<'_, AppState>,
    request: SendServerCommandRequest,
) -> Result<(), AppError> {
    ConsoleService::send_command(state.inner(), request)
}

#[tauri::command]
pub fn get_server_properties(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<ServerProperties, AppError> {
    ServerConfigurationService::get_properties(&state.inner().database, &server_id)
}

#[tauri::command]
pub fn update_server_properties(
    state: State<'_, AppState>,
    request: UpdateServerPropertiesRequest,
) -> Result<ServerProperties, AppError> {
    ServerConfigurationService::update_properties(&state.inner().database, request)
}

#[tauri::command]
pub fn list_backup_jobs(state: State<'_, AppState>) -> Result<Vec<BackupJob>, AppError> {
    BackupService::list_jobs(&state.inner().database)
}

#[tauri::command]
pub fn list_backup_run_records(
    state: State<'_, AppState>,
    backup_job_id: String,
) -> Result<Vec<BackupRunRecord>, AppError> {
    BackupService::list_run_records(&state.inner().database, &backup_job_id)
}

#[tauri::command]
pub fn create_backup_job(
    state: State<'_, AppState>,
    request: CreateBackupJobRequest,
) -> Result<BackupJob, AppError> {
    BackupService::create_job(&state.inner().database, request)
}

#[tauri::command]
pub fn run_backup_job(
    state: State<'_, AppState>,
    backup_job_id: String,
) -> Result<String, AppError> {
    BackupService::run_job(state.inner(), &backup_job_id)
}

#[tauri::command]
pub fn restore_backup(
    _state: State<'_, AppState>,
    request: RestoreBackupRequest,
) -> Result<(), AppError> {
    BackupService::restore(request)
}

#[tauri::command]
pub fn get_updater_status(state: State<'_, AppState>) -> Result<UpdaterStatus, AppError> {
    Ok(UpdateService::get_status(state.inner()))
}

#[tauri::command]
pub async fn check_for_updates(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<UpdaterStatus, AppError> {
    UpdateService::check(state.inner(), &app_handle).await
}

#[tauri::command]
pub async fn install_update(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<UpdaterStatus, AppError> {
    UpdateService::install(state.inner(), &app_handle).await
}

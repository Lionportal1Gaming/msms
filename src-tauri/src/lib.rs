mod commands;
mod db;
pub mod models;
pub mod services;
mod state;

use commands::{
    bootstrap_status, check_for_updates, create_backup_job, discover_java_runtimes,
    get_command_history, get_console_history, get_server_console, get_server_properties,
    get_settings, get_updater_status, install_java_runtime, install_update, kill_server,
    list_backup_jobs, list_backup_run_records, list_minecraft_versions, list_servers,
    provision_server, restart_server, restore_backup, run_backup_job, send_server_command,
    set_app_password, start_server, stop_server, unlock_app, update_server_properties,
    update_settings, validate_provisioning,
};
use services::BackupService;
use state::AppState;

pub fn run() {
    let app_state = AppState::initialize().expect("failed to initialize app state");
    BackupService::spawn_scheduler(app_state.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            bootstrap_status,
            list_servers,
            discover_java_runtimes,
            install_java_runtime,
            get_settings,
            update_settings,
            set_app_password,
            unlock_app,
            list_minecraft_versions,
            validate_provisioning,
            provision_server,
            start_server,
            stop_server,
            restart_server,
            kill_server,
            get_server_console,
            get_console_history,
            get_command_history,
            send_server_command,
            get_server_properties,
            update_server_properties,
            list_backup_jobs,
            list_backup_run_records,
            create_backup_job,
            run_backup_job,
            restore_backup,
            get_updater_status,
            check_for_updates,
            install_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

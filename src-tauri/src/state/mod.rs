use std::{
    collections::HashMap,
    process::{Child, ChildStdin},
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};

use crate::{
    db::Database,
    models::{AppError, ConsoleEntry, MinecraftVersionOption, UpdateChannel, UpdaterStatus},
};

pub struct ManagedProcess {
    pub child: Child,
    pub stdin: Arc<Mutex<ChildStdin>>,
}

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub processes: Arc<Mutex<HashMap<String, ManagedProcess>>>,
    pub console_logs: Arc<Mutex<HashMap<String, Vec<ConsoleEntry>>>>,
    pub live_session_started_at: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
    pub minecraft_versions: Arc<Mutex<Vec<CachedMinecraftVersion>>>,
    pub updater_status: Arc<Mutex<UpdaterStatus>>,
    pub unlocked: Arc<Mutex<bool>>,
}

#[derive(Clone)]
pub struct CachedMinecraftVersion {
    pub option: MinecraftVersionOption,
    pub detail_url: String,
}

impl AppState {
    pub fn initialize() -> Result<Self, AppError> {
        let database = Database::initialize()?;
        let settings = database.get_settings()?;
        Ok(Self {
            database,
            processes: Arc::new(Mutex::new(HashMap::new())),
            console_logs: Arc::new(Mutex::new(HashMap::new())),
            live_session_started_at: Arc::new(Mutex::new(HashMap::new())),
            minecraft_versions: Arc::new(Mutex::new(Vec::new())),
            updater_status: Arc::new(Mutex::new(UpdaterStatus::new(
                env!("CARGO_PKG_VERSION"),
                match settings.update_channel {
                    UpdateChannel::Stable => UpdateChannel::Stable,
                    UpdateChannel::Beta => UpdateChannel::Beta,
                },
            ))),
            unlocked: Arc::new(Mutex::new(false)),
        })
    }
}

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use rusqlite::{params, Connection, OptionalExtension};

use crate::models::{
    AppError, AppSettings, BackupJob, BackupRunRecord, BackupRunStatus, BackupScheduleConfig,
    BackupSchedulePreset, CommandHistoryEntry, ConsoleEntry, ConsoleSource, JavaRuntime,
    ManagedServer, ServerStatus, UpdateChannel,
};

const CONSOLE_HISTORY_LIMIT: i64 = 500;
const COMMAND_HISTORY_LIMIT: i64 = 50;

#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
    pub root_dir: PathBuf,
}

impl Database {
    pub fn initialize() -> Result<Self, AppError> {
        let project_dirs = ProjectDirs::from("com", "alextaylor", "msms")
            .ok_or_else(|| AppError::Message("Unable to resolve project directories".into()))?;
        let root_dir = project_dirs.data_dir().to_path_buf();
        Self::initialize_at(root_dir)
    }

    pub fn initialize_at(root_dir: PathBuf) -> Result<Self, AppError> {
        fs::create_dir_all(&root_dir)?;

        let db_path = root_dir.join("msms.sqlite3");
        let connection = Connection::open(db_path)?;
        let database = Self {
            connection: Arc::new(Mutex::new(connection)),
            root_dir,
        };

        database.run_migrations()?;
        database.seed_defaults()?;
        Ok(database)
    }

    fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.connection.lock().expect("database mutex poisoned")
    }

    fn run_migrations(&self) -> Result<(), AppError> {
        self.connection().execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS settings (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS auth_state (
              id INTEGER PRIMARY KEY CHECK (id = 1),
              password_hash TEXT,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS managed_servers (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              minecraft_version TEXT NOT NULL,
              server_path TEXT NOT NULL,
              jar_path TEXT NOT NULL,
              status TEXT NOT NULL,
              port INTEGER NOT NULL,
              memory_mb INTEGER NOT NULL,
              eula_accepted INTEGER NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS java_runtimes (
              id TEXT PRIMARY KEY,
              version TEXT NOT NULL,
              vendor TEXT NOT NULL,
              install_path TEXT NOT NULL,
              architecture TEXT NOT NULL,
              managed_by_app INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS backup_jobs (
              id TEXT PRIMARY KEY,
              server_id TEXT NOT NULL,
              schedule TEXT NOT NULL,
              retention_count INTEGER NOT NULL,
              destination_path TEXT NOT NULL,
              last_result TEXT NOT NULL,
              FOREIGN KEY(server_id) REFERENCES managed_servers(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS backup_run_records (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              job_id TEXT NOT NULL,
              started_at TEXT NOT NULL,
              finished_at TEXT,
              status TEXT NOT NULL,
              message TEXT NOT NULL,
              FOREIGN KEY(job_id) REFERENCES backup_jobs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS console_history (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              server_id TEXT NOT NULL,
              source TEXT NOT NULL,
              message TEXT NOT NULL,
              timestamp TEXT NOT NULL,
              FOREIGN KEY(server_id) REFERENCES managed_servers(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS command_history (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              server_id TEXT NOT NULL,
              command TEXT NOT NULL,
              timestamp TEXT NOT NULL,
              FOREIGN KEY(server_id) REFERENCES managed_servers(id) ON DELETE CASCADE
            );
        ",
        )?;

        self.ensure_column("backup_jobs", "schedule_preset", "TEXT")?;
        self.ensure_column("backup_jobs", "interval_hours", "INTEGER")?;
        self.ensure_column("backup_jobs", "weekday", "INTEGER")?;
        self.ensure_column("backup_jobs", "hour", "INTEGER")?;
        self.ensure_column("backup_jobs", "minute", "INTEGER")?;
        self.ensure_column("backup_jobs", "next_run_at", "TEXT")?;
        self.ensure_column("backup_jobs", "last_run_at", "TEXT")?;
        self.ensure_column("backup_jobs", "last_status", "TEXT NOT NULL DEFAULT 'idle'")?;
        self.ensure_column("backup_jobs", "last_duration_ms", "INTEGER")?;
        self.ensure_column("managed_servers", "java_runtime_id", "TEXT")?;

        Ok(())
    }

    fn seed_defaults(&self) -> Result<(), AppError> {
        let defaults = self.default_settings();
        self.set_setting("auth_mode", &defaults.auth_mode)?;
        self.set_setting(
            "update_channel",
            update_channel_to_str(defaults.update_channel),
        )?;
        self.set_setting(
            "diagnostics_opt_in",
            if defaults.diagnostics_opt_in {
                "1"
            } else {
                "0"
            },
        )?;
        self.set_setting(
            "default_server_directory",
            &defaults.default_server_directory,
        )?;
        self.set_setting(
            "default_backup_directory",
            &defaults.default_backup_directory,
        )?;
        self.set_setting("default_java_directory", &defaults.default_java_directory)?;
        Ok(())
    }

    pub fn default_settings(&self) -> AppSettings {
        let server_dir = self.root_dir.join("servers");
        let backup_dir = self.root_dir.join("backups");
        let java_dir = self.root_dir.join("java");

        AppSettings {
            auth_mode: "password".into(),
            update_channel: UpdateChannel::Stable,
            diagnostics_opt_in: false,
            default_server_directory: server_dir.display().to_string(),
            default_backup_directory: backup_dir.display().to_string(),
            default_java_directory: java_dir.display().to_string(),
        }
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        self.connection().execute(
            "
            INSERT INTO settings (key, value)
            VALUES (?1, ?2)
            ON CONFLICT(key) DO NOTHING
            ",
            params![key, value],
        )?;
        Ok(())
    }

    fn ensure_column(&self, table: &str, column: &str, definition: &str) -> Result<(), AppError> {
        let connection = self.connection();
        let pragma = format!("PRAGMA table_info({})", table);
        let mut stmt = connection.prepare(&pragma)?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        let mut exists = false;
        for row in rows {
            if row? == column {
                exists = true;
                break;
            }
        }
        drop(stmt);
        if !exists {
            connection.execute(
                &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition),
                [],
            )?;
        }
        Ok(())
    }

    pub fn get_settings(&self) -> Result<AppSettings, AppError> {
        let connection = self.connection();
        let mut stmt = connection.prepare("SELECT key, value FROM settings ORDER BY key ASC")?;
        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })?;

        let mut settings = self.default_settings();
        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "auth_mode" => settings.auth_mode = value,
                "update_channel" => settings.update_channel = update_channel_from_str(&value),
                "diagnostics_opt_in" => settings.diagnostics_opt_in = value == "1",
                "default_server_directory" => settings.default_server_directory = value,
                "default_backup_directory" => settings.default_backup_directory = value,
                "default_java_directory" => settings.default_java_directory = value,
                _ => {}
            }
        }
        Ok(settings)
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<AppSettings, AppError> {
        let connection = self.connection();
        let mut statement = connection.prepare(
            "
            INSERT INTO settings (key, value)
            VALUES (?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            ",
        )?;

        for (key, value) in [
            ("auth_mode", settings.auth_mode.clone()),
            (
                "update_channel",
                update_channel_to_str(settings.update_channel).to_string(),
            ),
            (
                "diagnostics_opt_in",
                if settings.diagnostics_opt_in {
                    "1"
                } else {
                    "0"
                }
                .to_string(),
            ),
            (
                "default_server_directory",
                settings.default_server_directory.clone(),
            ),
            (
                "default_backup_directory",
                settings.default_backup_directory.clone(),
            ),
            (
                "default_java_directory",
                settings.default_java_directory.clone(),
            ),
        ] {
            statement.execute(params![key, value])?;
        }

        Ok(settings.clone())
    }

    pub fn save_password_hash(&self, hash: &str) -> Result<(), AppError> {
        let timestamp = Utc::now().to_rfc3339();
        self.connection().execute(
            "
            INSERT INTO auth_state (id, password_hash, updated_at)
            VALUES (1, ?1, ?2)
            ON CONFLICT(id) DO UPDATE
            SET password_hash = excluded.password_hash, updated_at = excluded.updated_at
            ",
            params![hash, timestamp],
        )?;
        Ok(())
    }

    pub fn load_password_hash(&self) -> Result<Option<String>, AppError> {
        let hash = self
            .connection()
            .query_row(
                "SELECT password_hash FROM auth_state WHERE id = 1",
                [],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten();
        Ok(hash)
    }

    pub fn upsert_server(&self, server: &ManagedServer) -> Result<ManagedServer, AppError> {
        self.connection().execute(
            "
            INSERT INTO managed_servers (
              id, name, minecraft_version, server_path, jar_path, status, port, memory_mb,
              eula_accepted, created_at, updated_at, java_runtime_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(id) DO UPDATE SET
              name = excluded.name,
              minecraft_version = excluded.minecraft_version,
              server_path = excluded.server_path,
              jar_path = excluded.jar_path,
              status = excluded.status,
              port = excluded.port,
              memory_mb = excluded.memory_mb,
              eula_accepted = excluded.eula_accepted,
              java_runtime_id = excluded.java_runtime_id,
              updated_at = excluded.updated_at
            ",
            params![
                server.id,
                server.name,
                server.minecraft_version,
                server.server_path,
                server.jar_path,
                server_status_to_str(&server.status),
                server.port,
                server.memory_mb,
                server.eula_accepted as i32,
                server.created_at.to_rfc3339(),
                server.updated_at.to_rfc3339(),
                server.java_runtime_id
            ],
        )?;

        Ok(server.clone())
    }

    pub fn list_servers(&self) -> Result<Vec<ManagedServer>, AppError> {
        let connection = self.connection();
        let mut stmt = connection.prepare(
            "
            SELECT id, name, minecraft_version, server_path, jar_path, status, port, memory_mb,
                   eula_accepted, created_at, updated_at, java_runtime_id
            FROM managed_servers
            ORDER BY updated_at DESC
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ManagedServer {
                id: row.get(0)?,
                name: row.get(1)?,
                minecraft_version: row.get(2)?,
                server_path: row.get(3)?,
                jar_path: row.get(4)?,
                status: server_status_from_str(&row.get::<_, String>(5)?),
                port: row.get(6)?,
                memory_mb: row.get(7)?,
                eula_accepted: row.get::<_, i32>(8)? == 1,
                created_at: parse_utc(&row.get::<_, String>(9)?)?,
                updated_at: parse_utc(&row.get::<_, String>(10)?)?,
                java_runtime_id: row.get(11)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn find_server(&self, server_id: &str) -> Result<ManagedServer, AppError> {
        let connection = self.connection();
        let mut stmt = connection.prepare(
            "
            SELECT id, name, minecraft_version, server_path, jar_path, status, port, memory_mb,
                   eula_accepted, created_at, updated_at, java_runtime_id
            FROM managed_servers
            WHERE id = ?1
            ",
        )?;

        let server = stmt.query_row([server_id], |row| {
            Ok(ManagedServer {
                id: row.get(0)?,
                name: row.get(1)?,
                minecraft_version: row.get(2)?,
                server_path: row.get(3)?,
                jar_path: row.get(4)?,
                status: server_status_from_str(&row.get::<_, String>(5)?),
                port: row.get(6)?,
                memory_mb: row.get(7)?,
                eula_accepted: row.get::<_, i32>(8)? == 1,
                created_at: parse_utc(&row.get::<_, String>(9)?)?,
                updated_at: parse_utc(&row.get::<_, String>(10)?)?,
                java_runtime_id: row.get(11)?,
            })
        })?;
        Ok(server)
    }

    pub fn find_server_by_path(&self, server_path: &str) -> Result<Option<ManagedServer>, AppError> {
        let connection = self.connection();
        let mut stmt = connection.prepare(
            "
            SELECT id, name, minecraft_version, server_path, jar_path, status, port, memory_mb,
                   eula_accepted, created_at, updated_at, java_runtime_id
            FROM managed_servers
            WHERE server_path = ?1
            ",
        )?;
        let server = stmt
            .query_row([server_path], |row| {
                Ok(ManagedServer {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    minecraft_version: row.get(2)?,
                    server_path: row.get(3)?,
                    jar_path: row.get(4)?,
                    status: server_status_from_str(&row.get::<_, String>(5)?),
                    port: row.get(6)?,
                    memory_mb: row.get(7)?,
                    eula_accepted: row.get::<_, i32>(8)? == 1,
                    created_at: parse_utc(&row.get::<_, String>(9)?)?,
                    updated_at: parse_utc(&row.get::<_, String>(10)?)?,
                    java_runtime_id: row.get(11)?,
                })
            })
            .optional()?;
        Ok(server)
    }

    pub fn find_server_by_port(&self, port: u16) -> Result<Option<ManagedServer>, AppError> {
        let connection = self.connection();
        let mut stmt = connection.prepare(
            "
            SELECT id, name, minecraft_version, server_path, jar_path, status, port, memory_mb,
                   eula_accepted, created_at, updated_at, java_runtime_id
            FROM managed_servers
            WHERE port = ?1
            ",
        )?;
        let server = stmt
            .query_row([port], |row| {
                Ok(ManagedServer {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    minecraft_version: row.get(2)?,
                    server_path: row.get(3)?,
                    jar_path: row.get(4)?,
                    status: server_status_from_str(&row.get::<_, String>(5)?),
                    port: row.get(6)?,
                    memory_mb: row.get(7)?,
                    eula_accepted: row.get::<_, i32>(8)? == 1,
                    created_at: parse_utc(&row.get::<_, String>(9)?)?,
                    updated_at: parse_utc(&row.get::<_, String>(10)?)?,
                    java_runtime_id: row.get(11)?,
                })
            })
            .optional()?;
        Ok(server)
    }

    pub fn upsert_java_runtime(&self, runtime: &JavaRuntime) -> Result<JavaRuntime, AppError> {
        self.connection().execute(
            "
            INSERT INTO java_runtimes (id, version, vendor, install_path, architecture, managed_by_app)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
              version = excluded.version,
              vendor = excluded.vendor,
              install_path = excluded.install_path,
              architecture = excluded.architecture,
              managed_by_app = excluded.managed_by_app
            ",
            params![
                runtime.id,
                runtime.version,
                runtime.vendor,
                runtime.install_path,
                runtime.architecture,
                runtime.managed_by_app as i32
            ],
        )?;
        Ok(runtime.clone())
    }

    pub fn list_java_runtimes(&self) -> Result<Vec<JavaRuntime>, AppError> {
        let connection = self.connection();
        let mut stmt = connection.prepare(
            "
            SELECT id, version, vendor, install_path, architecture, managed_by_app
            FROM java_runtimes
            ORDER BY vendor ASC, version DESC
            ",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(JavaRuntime {
                id: row.get(0)?,
                version: row.get(1)?,
                vendor: row.get(2)?,
                install_path: row.get(3)?,
                architecture: row.get(4)?,
                managed_by_app: row.get::<_, i32>(5)? == 1,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn find_java_runtime(&self, runtime_id: &str) -> Result<Option<JavaRuntime>, AppError> {
        let connection = self.connection();
        let mut stmt = connection.prepare(
            "
            SELECT id, version, vendor, install_path, architecture, managed_by_app
            FROM java_runtimes
            WHERE id = ?1
            ",
        )?;
        let result = stmt
            .query_row([runtime_id], |row| {
                Ok(JavaRuntime {
                    id: row.get(0)?,
                    version: row.get(1)?,
                    vendor: row.get(2)?,
                    install_path: row.get(3)?,
                    architecture: row.get(4)?,
                    managed_by_app: row.get::<_, i32>(5)? == 1,
                })
            })
            .optional()?;
        Ok(result)
    }

    pub fn upsert_backup_job(&self, backup_job: &BackupJob) -> Result<BackupJob, AppError> {
        self.connection().execute(
            "
            INSERT INTO backup_jobs (
              id, server_id, schedule, schedule_preset, interval_hours, weekday, hour, minute,
              retention_count, destination_path, next_run_at, last_run_at, last_status,
              last_duration_ms, last_result
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(id) DO UPDATE SET
              schedule = excluded.schedule,
              schedule_preset = excluded.schedule_preset,
              interval_hours = excluded.interval_hours,
              weekday = excluded.weekday,
              hour = excluded.hour,
              minute = excluded.minute,
              retention_count = excluded.retention_count,
              destination_path = excluded.destination_path,
              next_run_at = excluded.next_run_at,
              last_run_at = excluded.last_run_at,
              last_status = excluded.last_status,
              last_duration_ms = excluded.last_duration_ms,
              last_result = excluded.last_result
            ",
            params![
                backup_job.id,
                backup_job.server_id,
                backup_job.schedule,
                backup_job
                    .schedule_preset
                    .map(backup_schedule_preset_to_str),
                backup_job.schedule_config.interval_hours,
                backup_job.schedule_config.weekday,
                backup_job.schedule_config.hour,
                backup_job.schedule_config.minute,
                backup_job.retention_count,
                backup_job.destination_path,
                backup_job.next_run_at.map(|value| value.to_rfc3339()),
                backup_job.last_run_at.map(|value| value.to_rfc3339()),
                backup_run_status_to_str(backup_job.last_status),
                backup_job.last_duration_ms,
                backup_job.last_result
            ],
        )?;
        Ok(backup_job.clone())
    }

    pub fn list_backup_jobs(&self) -> Result<Vec<BackupJob>, AppError> {
        let connection = self.connection();
        let mut stmt = connection.prepare(
            "
            SELECT id, server_id, schedule, schedule_preset, interval_hours, weekday, hour, minute,
                   retention_count, destination_path, next_run_at, last_run_at, last_status,
                   last_duration_ms, last_result
            FROM backup_jobs
            ORDER BY server_id ASC, schedule ASC
            ",
        )?;
        let rows = stmt.query_map([], |row| {
            let schedule = row.get::<_, String>(2)?;
            let stored_schedule_preset = row.get::<_, Option<String>>(3)?;
            let stored_schedule_config = BackupScheduleConfig {
                interval_hours: row.get(4)?,
                weekday: row.get(5)?,
                hour: row.get(6)?,
                minute: row.get(7)?,
            };
            let resolved_schedule = resolve_backup_schedule(
                stored_schedule_preset.as_deref(),
                &schedule,
                stored_schedule_config,
            );

            Ok(BackupJob {
                id: row.get(0)?,
                server_id: row.get(1)?,
                schedule,
                schedule_preset: resolved_schedule.schedule_preset,
                schedule_config: resolved_schedule.schedule_config,
                retention_count: row.get(8)?,
                destination_path: row.get(9)?,
                next_run_at: row
                    .get::<_, Option<String>>(10)?
                    .map(|value| parse_utc(&value))
                    .transpose()?,
                last_run_at: row
                    .get::<_, Option<String>>(11)?
                    .map(|value| parse_utc(&value))
                    .transpose()?,
                last_status: backup_run_status_from_str(&row.get::<_, String>(12)?),
                last_duration_ms: row.get(13)?,
                last_result: row.get(14)?,
                is_legacy_schedule: resolved_schedule.is_legacy_schedule,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn find_backup_job(&self, job_id: &str) -> Result<BackupJob, AppError> {
        let connection = self.connection();
        let mut stmt = connection.prepare(
            "
            SELECT id, server_id, schedule, schedule_preset, interval_hours, weekday, hour, minute,
                   retention_count, destination_path, next_run_at, last_run_at, last_status,
                   last_duration_ms, last_result
            FROM backup_jobs
            WHERE id = ?1
            ",
        )?;
        Ok(stmt.query_row([job_id], |row| {
            let schedule = row.get::<_, String>(2)?;
            let stored_schedule_preset = row.get::<_, Option<String>>(3)?;
            let stored_schedule_config = BackupScheduleConfig {
                interval_hours: row.get(4)?,
                weekday: row.get(5)?,
                hour: row.get(6)?,
                minute: row.get(7)?,
            };
            let resolved_schedule = resolve_backup_schedule(
                stored_schedule_preset.as_deref(),
                &schedule,
                stored_schedule_config,
            );

            Ok(BackupJob {
                id: row.get(0)?,
                server_id: row.get(1)?,
                schedule,
                schedule_preset: resolved_schedule.schedule_preset,
                schedule_config: resolved_schedule.schedule_config,
                retention_count: row.get(8)?,
                destination_path: row.get(9)?,
                next_run_at: row
                    .get::<_, Option<String>>(10)?
                    .map(|value| parse_utc(&value))
                    .transpose()?,
                last_run_at: row
                    .get::<_, Option<String>>(11)?
                    .map(|value| parse_utc(&value))
                    .transpose()?,
                last_status: backup_run_status_from_str(&row.get::<_, String>(12)?),
                last_duration_ms: row.get(13)?,
                last_result: row.get(14)?,
                is_legacy_schedule: resolved_schedule.is_legacy_schedule,
            })
        })?)
    }

    pub fn append_backup_run_record(&self, record: &BackupRunRecord) -> Result<(), AppError> {
        self.connection().execute(
            "
            INSERT INTO backup_run_records (job_id, started_at, finished_at, status, message)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                record.job_id,
                record.started_at.to_rfc3339(),
                record.finished_at.map(|value| value.to_rfc3339()),
                backup_run_status_to_str(record.status),
                record.message
            ],
        )?;
        Ok(())
    }

    pub fn list_backup_run_records(&self, job_id: &str) -> Result<Vec<BackupRunRecord>, AppError> {
        let connection = self.connection();
        let mut stmt = connection.prepare(
            "
            SELECT id, job_id, started_at, finished_at, status, message
            FROM backup_run_records
            WHERE job_id = ?1
            ORDER BY id DESC
            LIMIT 20
            ",
        )?;
        let rows = stmt.query_map([job_id], |row| {
            Ok(BackupRunRecord {
                id: row.get(0)?,
                job_id: row.get(1)?,
                started_at: parse_utc(&row.get::<_, String>(2)?)?,
                finished_at: row
                    .get::<_, Option<String>>(3)?
                    .map(|value| parse_utc(&value))
                    .transpose()?,
                status: backup_run_status_from_str(&row.get::<_, String>(4)?),
                message: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn append_console_entry(&self, entry: &ConsoleEntry) -> Result<(), AppError> {
        self.connection().execute(
            "
            INSERT INTO console_history (server_id, source, message, timestamp)
            VALUES (?1, ?2, ?3, ?4)
            ",
            params![
                entry.server_id,
                console_source_to_str(entry.source),
                entry.message,
                entry.timestamp.to_rfc3339()
            ],
        )?;
        self.trim_console_history(&entry.server_id)
    }

    pub fn list_console_history(
        &self,
        server_id: &str,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<ConsoleEntry>, AppError> {
        let before_value = before.map(|value| value.to_rfc3339());
        let connection = self.connection();
        let mut stmt = connection.prepare(
            "
            SELECT server_id, source, message, timestamp
            FROM console_history
            WHERE server_id = ?1
              AND (?2 IS NULL OR timestamp < ?2)
            ORDER BY id DESC
            LIMIT ?3
            ",
        )?;
        let rows = stmt.query_map(
            params![server_id, before_value, CONSOLE_HISTORY_LIMIT],
            |row| {
                Ok(ConsoleEntry {
                    server_id: row.get(0)?,
                    source: console_source_from_str(&row.get::<_, String>(1)?),
                    message: row.get(2)?,
                    timestamp: parse_utc(&row.get::<_, String>(3)?)?,
                })
            },
        )?;
        let mut entries = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::from)?;
        entries.reverse();
        Ok(entries)
    }

    pub fn append_command_history(&self, entry: &CommandHistoryEntry) -> Result<(), AppError> {
        self.connection().execute(
            "
            INSERT INTO command_history (server_id, command, timestamp)
            VALUES (?1, ?2, ?3)
            ",
            params![entry.server_id, entry.command, entry.timestamp.to_rfc3339()],
        )?;
        self.trim_command_history(&entry.server_id)
    }

    pub fn list_command_history(
        &self,
        server_id: &str,
    ) -> Result<Vec<CommandHistoryEntry>, AppError> {
        let connection = self.connection();
        let mut stmt = connection.prepare(
            "
            SELECT server_id, command, timestamp
            FROM command_history
            WHERE server_id = ?1
            ORDER BY id DESC
            LIMIT ?2
            ",
        )?;
        let rows = stmt.query_map(params![server_id, COMMAND_HISTORY_LIMIT], |row| {
            Ok(CommandHistoryEntry {
                server_id: row.get(0)?,
                command: row.get(1)?,
                timestamp: parse_utc(&row.get::<_, String>(2)?)?,
            })
        })?;
        let mut entries = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::from)?;
        entries.reverse();
        Ok(entries)
    }

    fn trim_console_history(&self, server_id: &str) -> Result<(), AppError> {
        self.connection().execute(
            "
            DELETE FROM console_history
            WHERE server_id = ?1
              AND id NOT IN (
                SELECT id
                FROM console_history
                WHERE server_id = ?1
                ORDER BY id DESC
                LIMIT ?2
              )
            ",
            params![server_id, CONSOLE_HISTORY_LIMIT],
        )?;
        Ok(())
    }

    fn trim_command_history(&self, server_id: &str) -> Result<(), AppError> {
        self.connection().execute(
            "
            DELETE FROM command_history
            WHERE server_id = ?1
              AND id NOT IN (
                SELECT id
                FROM command_history
                WHERE server_id = ?1
                ORDER BY id DESC
                LIMIT ?2
              )
            ",
            params![server_id, COMMAND_HISTORY_LIMIT],
        )?;
        Ok(())
    }
}

fn parse_utc(value: &str) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })
}

fn server_status_from_str(value: &str) -> ServerStatus {
    match value {
        "running" => ServerStatus::Running,
        "starting" => ServerStatus::Starting,
        "stopping" => ServerStatus::Stopping,
        "error" => ServerStatus::Error,
        _ => ServerStatus::Stopped,
    }
}

fn server_status_to_str(value: &ServerStatus) -> &'static str {
    match value {
        ServerStatus::Stopped => "stopped",
        ServerStatus::Starting => "starting",
        ServerStatus::Running => "running",
        ServerStatus::Stopping => "stopping",
        ServerStatus::Error => "error",
    }
}

fn console_source_from_str(value: &str) -> ConsoleSource {
    match value {
        "stderr" => ConsoleSource::Stderr,
        "command" => ConsoleSource::Command,
        "system" => ConsoleSource::System,
        _ => ConsoleSource::Stdout,
    }
}

fn console_source_to_str(value: ConsoleSource) -> &'static str {
    match value {
        ConsoleSource::Stdout => "stdout",
        ConsoleSource::Stderr => "stderr",
        ConsoleSource::Command => "command",
        ConsoleSource::System => "system",
    }
}

fn backup_schedule_preset_from_str(value: &str) -> BackupSchedulePreset {
    match value {
        "daily" => BackupSchedulePreset::Daily,
        "weekly" => BackupSchedulePreset::Weekly,
        _ => BackupSchedulePreset::Hourly,
    }
}

fn backup_schedule_preset_to_str(value: BackupSchedulePreset) -> &'static str {
    match value {
        BackupSchedulePreset::Hourly => "hourly",
        BackupSchedulePreset::Daily => "daily",
        BackupSchedulePreset::Weekly => "weekly",
    }
}

fn backup_run_status_from_str(value: &str) -> BackupRunStatus {
    match value {
        "running" => BackupRunStatus::Running,
        "succeeded" => BackupRunStatus::Succeeded,
        "failed" => BackupRunStatus::Failed,
        _ => BackupRunStatus::Idle,
    }
}

fn backup_run_status_to_str(value: BackupRunStatus) -> &'static str {
    match value {
        BackupRunStatus::Idle => "idle",
        BackupRunStatus::Running => "running",
        BackupRunStatus::Succeeded => "succeeded",
        BackupRunStatus::Failed => "failed",
    }
}

fn update_channel_from_str(value: &str) -> UpdateChannel {
    match value {
        "beta" => UpdateChannel::Beta,
        _ => UpdateChannel::Stable,
    }
}

fn update_channel_to_str(value: UpdateChannel) -> &'static str {
    match value {
        UpdateChannel::Stable => "stable",
        UpdateChannel::Beta => "beta",
    }
}

struct ResolvedBackupSchedule {
    schedule_preset: Option<BackupSchedulePreset>,
    schedule_config: BackupScheduleConfig,
    is_legacy_schedule: bool,
}

fn resolve_backup_schedule(
    stored_schedule_preset: Option<&str>,
    schedule: &str,
    stored_schedule_config: BackupScheduleConfig,
) -> ResolvedBackupSchedule {
    if let Some(preset) = stored_schedule_preset {
        return ResolvedBackupSchedule {
            schedule_preset: Some(backup_schedule_preset_from_str(preset)),
            schedule_config: stored_schedule_config,
            is_legacy_schedule: false,
        };
    }

    if let Some((schedule_preset, schedule_config)) = parse_legacy_schedule(schedule) {
        return ResolvedBackupSchedule {
            schedule_preset: Some(schedule_preset),
            schedule_config,
            is_legacy_schedule: false,
        };
    }

    ResolvedBackupSchedule {
        schedule_preset: None,
        schedule_config: stored_schedule_config,
        is_legacy_schedule: true,
    }
}

fn parse_legacy_schedule(schedule: &str) -> Option<(BackupSchedulePreset, BackupScheduleConfig)> {
    let parts = schedule.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 5 {
        return None;
    }

    let minute = parts[0].parse::<u8>().ok()?;
    let hour = parts[1];
    let day_of_month = parts[2];
    let month = parts[3];
    let day_of_week = parts[4];

    if day_of_month != "*" || month != "*" {
        return None;
    }

    if let Some(interval_hours) = hour
        .strip_prefix("*/")
        .and_then(|value| value.parse::<u32>().ok())
    {
        if minute == 0 && day_of_week == "*" && interval_hours >= 1 {
            return Some((
                BackupSchedulePreset::Hourly,
                BackupScheduleConfig {
                    interval_hours: Some(interval_hours),
                    weekday: None,
                    hour: None,
                    minute: None,
                },
            ));
        }
    }

    if let Ok(hour) = hour.parse::<u8>() {
        if day_of_week == "*" {
            return Some((
                BackupSchedulePreset::Daily,
                BackupScheduleConfig {
                    interval_hours: None,
                    weekday: None,
                    hour: Some(hour),
                    minute: Some(minute),
                },
            ));
        }

        if let Ok(raw_weekday) = day_of_week.parse::<u8>() {
            let weekday = if raw_weekday == 7 { 0 } else { raw_weekday };
            if weekday <= 6 {
                return Some((
                    BackupSchedulePreset::Weekly,
                    BackupScheduleConfig {
                        interval_hours: None,
                        weekday: Some(weekday),
                        hour: Some(hour),
                        minute: Some(minute),
                    },
                ));
            }
        }
    }

    None
}

pub fn archive_destination(base: &str, file_name: &str) -> PathBuf {
    Path::new(base).join(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn status_serialization_round_trip_helper() {
        assert_eq!(server_status_to_str(&ServerStatus::Running), "running");
        assert_eq!(server_status_from_str("stopped"), ServerStatus::Stopped);
    }

    #[test]
    fn managed_server_migration_defaults_java_runtime_to_null() {
        let temp_dir = tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("msms.sqlite3");
        let connection = Connection::open(&db_path).expect("open legacy db");
        connection
            .execute_batch(
                "
                CREATE TABLE managed_servers (
                  id TEXT PRIMARY KEY,
                  name TEXT NOT NULL,
                  minecraft_version TEXT NOT NULL,
                  server_path TEXT NOT NULL,
                  jar_path TEXT NOT NULL,
                  status TEXT NOT NULL,
                  port INTEGER NOT NULL,
                  memory_mb INTEGER NOT NULL,
                  eula_accepted INTEGER NOT NULL,
                  created_at TEXT NOT NULL,
                  updated_at TEXT NOT NULL
                );
                ",
            )
            .expect("create legacy schema");
        connection
            .execute(
                "
                INSERT INTO managed_servers (
                  id, name, minecraft_version, server_path, jar_path, status, port, memory_mb,
                  eula_accepted, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ",
                params![
                    "srv-legacy",
                    "Legacy",
                    "1.21.4",
                    temp_dir.path().join("legacy").display().to_string(),
                    temp_dir.path().join("legacy/server.jar").display().to_string(),
                    "stopped",
                    25565,
                    2048,
                    1,
                    Utc::now().to_rfc3339(),
                    Utc::now().to_rfc3339()
                ],
            )
            .expect("insert legacy row");
        drop(connection);

        let database = Database::initialize_at(temp_dir.path().to_path_buf()).expect("db");
        let server = database.find_server("srv-legacy").expect("load migrated server");

        assert_eq!(server.java_runtime_id, None);
    }

    #[test]
    fn trims_console_history_to_latest_limit() {
        let temp_dir = tempdir().expect("temp dir");
        let database = Database::initialize_at(temp_dir.path().to_path_buf()).expect("db");
        let server = ManagedServer {
            id: "srv-test".into(),
            name: "Test".into(),
            minecraft_version: "1.21.4".into(),
            server_path: temp_dir.path().join("server").display().to_string(),
            jar_path: temp_dir.path().join("server.jar").display().to_string(),
            java_runtime_id: None,
            status: ServerStatus::Stopped,
            port: 25565,
            memory_mb: 2048,
            eula_accepted: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        database.upsert_server(&server).expect("insert server");

        for index in 0..510 {
            database
                .append_console_entry(&ConsoleEntry {
                    server_id: server.id.clone(),
                    source: ConsoleSource::Stdout,
                    message: format!("line-{index}"),
                    timestamp: Utc::now(),
                })
                .expect("append console entry");
        }

        let entries = database
            .list_console_history(&server.id, None)
            .expect("list console history");
        assert_eq!(entries.len(), 500);
        assert_eq!(entries.first().expect("first").message, "line-10");
        assert_eq!(entries.last().expect("last").message, "line-509");
    }

    #[test]
    fn parses_supported_legacy_backup_schedule() {
        let parsed = parse_legacy_schedule("0 2 * * *").expect("parse legacy daily schedule");
        assert_eq!(parsed.0, BackupSchedulePreset::Daily);
        assert_eq!(parsed.1.hour, Some(2));
        assert_eq!(parsed.1.minute, Some(0));

        let weekly = parse_legacy_schedule("30 4 * * 3").expect("parse legacy weekly schedule");
        assert_eq!(weekly.0, BackupSchedulePreset::Weekly);
        assert_eq!(weekly.1.weekday, Some(3));
    }

    #[test]
    fn legacy_jobs_are_resolved_when_supported() {
        let temp_dir = tempdir().expect("temp dir");
        let database = Database::initialize_at(temp_dir.path().to_path_buf()).expect("db");
        let server = ManagedServer {
            id: "srv-test".into(),
            name: "Test".into(),
            minecraft_version: "1.21.4".into(),
            server_path: temp_dir.path().join("server").display().to_string(),
            jar_path: temp_dir.path().join("server.jar").display().to_string(),
            java_runtime_id: None,
            status: ServerStatus::Stopped,
            port: 25565,
            memory_mb: 2048,
            eula_accepted: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        database.upsert_server(&server).expect("insert server");
        database
            .connection()
            .execute(
                "
                INSERT INTO backup_jobs (id, server_id, schedule, retention_count, destination_path, last_result)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ",
                params![
                    "backup-legacy",
                    server.id,
                    "0 2 * * *",
                    7,
                    temp_dir.path().join("backups").display().to_string(),
                    "Scheduled"
                ],
            )
            .expect("insert legacy job");

        let job = database
            .find_backup_job("backup-legacy")
            .expect("load backup job");
        assert_eq!(job.schedule_preset, Some(BackupSchedulePreset::Daily));
        assert!(!job.is_legacy_schedule);
    }

    #[test]
    fn default_settings_use_stable_update_channel() {
        let temp_dir = tempdir().expect("temp dir");
        let database = Database::initialize_at(temp_dir.path().to_path_buf()).expect("db");

        let settings = database.get_settings().expect("settings");
        assert_eq!(settings.update_channel, UpdateChannel::Stable);
    }
}

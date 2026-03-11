import { useEffect, useMemo, useState, type FormEvent } from "react";
import type { BackupScheduleConfig, BackupSchedulePreset } from "../types/api";
import { useAppStore } from "../store/appStore";

const weekdays = [
  "Sunday",
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday"
];

export function BackupsPanel() {
  const { backupJobs, backupRunRecords, servers, settings, createBackupJob, runBackupJob } =
    useAppStore();
  const [serverId, setServerId] = useState(servers[0]?.id ?? "");
  const [schedulePreset, setSchedulePreset] = useState<BackupSchedulePreset>("daily");
  const [intervalHours, setIntervalHours] = useState(6);
  const [hour, setHour] = useState(2);
  const [minute, setMinute] = useState(0);
  const [weekday, setWeekday] = useState(0);
  const [retentionCount, setRetentionCount] = useState(7);
  const [archivePath, setArchivePath] = useState("");

  useEffect(() => {
    if (!serverId && servers[0]) {
      setServerId(servers[0].id);
    }
  }, [serverId, servers]);

  const selectedServer = useMemo(
    () => servers.find((entry) => entry.id === serverId),
    [serverId, servers]
  );

  const destinationPath = useMemo(() => {
    if (!selectedServer || !settings) {
      return "";
    }
    return `${settings.defaultBackupDirectory}/${selectedServer.name.toLowerCase().replace(/\s+/g, "-")}`;
  }, [selectedServer, settings]);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!serverId || !destinationPath) {
      return;
    }

    const scheduleConfig: BackupScheduleConfig =
      schedulePreset === "hourly"
        ? {
            intervalHours
          }
        : schedulePreset === "daily"
          ? {
              hour,
              minute
            }
          : {
              weekday,
              hour,
              minute
            };

    await createBackupJob({
      serverId,
      schedulePreset,
      scheduleConfig,
      retentionCount,
      destinationPath
    });
  }

  return (
    <section className="panel split-panel">
      <div>
        <div className="section-heading">
          <div>
            <p className="eyebrow">Backups</p>
            <h2>Scheduled Backup Execution</h2>
            <p className="muted">
              Automatic jobs run only while the MSMS desktop app remains open on this machine.
            </p>
          </div>
        </div>
        <form className="form-grid" onSubmit={handleSubmit}>
          <label>
            Server
            <select value={serverId} onChange={(event) => setServerId(event.target.value)}>
              {servers.map((server) => (
                <option key={server.id} value={server.id}>
                  {server.name}
                </option>
              ))}
            </select>
          </label>
          <label>
            Schedule preset
            <select
              value={schedulePreset}
              onChange={(event) => setSchedulePreset(event.target.value as BackupSchedulePreset)}
            >
              <option value="hourly">Hourly</option>
              <option value="daily">Daily</option>
              <option value="weekly">Weekly</option>
            </select>
          </label>
          {schedulePreset === "hourly" && (
            <label>
              Interval hours
              <input
                min={1}
                type="number"
                value={intervalHours}
                onChange={(event) => setIntervalHours(Number(event.target.value))}
              />
            </label>
          )}
          {schedulePreset !== "hourly" && (
            <>
              {schedulePreset === "weekly" && (
                <label>
                  Weekday
                  <select
                    value={weekday}
                    onChange={(event) => setWeekday(Number(event.target.value))}
                  >
                    {weekdays.map((label, index) => (
                      <option key={label} value={index}>
                        {label}
                      </option>
                    ))}
                  </select>
                </label>
              )}
              <label>
                Hour
                <input
                  max={23}
                  min={0}
                  type="number"
                  value={hour}
                  onChange={(event) => setHour(Number(event.target.value))}
                />
              </label>
              <label>
                Minute
                <input
                  max={59}
                  min={0}
                  type="number"
                  value={minute}
                  onChange={(event) => setMinute(Number(event.target.value))}
                />
              </label>
            </>
          )}
          <label>
            Destination
            <input disabled value={destinationPath} />
          </label>
          <label>
            Retention count
            <input
              min={1}
              type="number"
              value={retentionCount}
              onChange={(event) => setRetentionCount(Number(event.target.value))}
            />
          </label>
          <div className="form-actions">
            <button disabled={!serverId || !destinationPath} type="submit">
              Save Backup Job
            </button>
          </div>
        </form>
      </div>
      <div>
        <h3>Configured Jobs</h3>
        <div className="stack">
          {backupJobs.length === 0 && (
            <article className="list-card">
              <p className="muted">No backup jobs are configured yet.</p>
            </article>
          )}
          {backupJobs.map((job) => (
            <article key={job.id} className="list-card">
              <div className="backup-job-header">
                <div>
                  <strong>
                    {servers.find((server) => server.id === job.serverId)?.name ?? job.serverId}
                  </strong>
                  <p className="muted">{job.schedule}</p>
                </div>
                <button onClick={() => void runBackupJob(job.id)} type="button">
                  Run now
                </button>
              </div>
              {job.isLegacySchedule && (
                <p className="status-pill status-failed">
                  Legacy schedule detected. Re-save this job to enable automatic execution.
                </p>
              )}
              <div className="backup-job-meta">
                <p>
                  <strong>Next run:</strong>{" "}
                  {job.nextRunAt ? formatTimestamp(job.nextRunAt) : "Re-save required"}
                </p>
                <p>
                  <strong>Last run:</strong>{" "}
                  {job.lastRunAt ? formatTimestamp(job.lastRunAt) : "No runs recorded"}
                </p>
                <p>
                  <strong>Status:</strong> {formatStatus(job.lastStatus)}
                </p>
                <p>
                  <strong>Duration:</strong>{" "}
                  {job.lastDurationMs !== null ? formatDuration(job.lastDurationMs) : "Not available"}
                </p>
              </div>
              <p>{job.lastResult}</p>
              <div className="backup-history">
                <strong>Recent runs</strong>
                {(backupRunRecords[job.id] ?? []).length === 0 && (
                  <p className="muted">No run history recorded yet.</p>
                )}
                {(backupRunRecords[job.id] ?? []).map((record) => (
                  <div key={record.id} className="backup-history-row">
                    <span>{formatTimestamp(record.startedAt)}</span>
                    <span className={`status-pill status-${record.status}`}>{formatStatus(record.status)}</span>
                    <span>{record.message}</span>
                  </div>
                ))}
              </div>
            </article>
          ))}
        </div>
        <RestoreBackupCard
          archivePath={archivePath}
          destinationPath={destinationPath}
          serverStatus={selectedServer?.status ?? "stopped"}
          onArchivePathChange={setArchivePath}
        />
      </div>
    </section>
  );
}

function RestoreBackupCard({
  archivePath,
  destinationPath,
  serverStatus,
  onArchivePathChange
}: {
  archivePath: string;
  destinationPath: string;
  serverStatus: string;
  onArchivePathChange: (value: string) => void;
}) {
  const { restoreBackup } = useAppStore();

  async function handleRestore(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!archivePath || !destinationPath) {
      return;
    }
    const confirmed = window.confirm(
      `Restore this archive into ${destinationPath}? This can overwrite server data. ${
        serverStatus === "running" ? "Restoring while stopped is strongly recommended." : ""
      }`
    );
    if (!confirmed) {
      return;
    }
    await restoreBackup({
      archivePath,
      targetDirectory: destinationPath
    });
  }

  return (
    <article className="list-card">
      <div>
        <strong>Restore Backup</strong>
        <p className="muted">
          Provide an archive path and restore it into the selected server backup target.
        </p>
      </div>
      <form className="form-grid" onSubmit={handleRestore}>
        <label>
          Archive path
          <input
            placeholder="/path/to/archive.zip"
            value={archivePath}
            onChange={(event) => onArchivePathChange(event.target.value)}
          />
        </label>
        <label>
          Restore target
          <input disabled value={destinationPath} />
        </label>
        <div className="form-actions">
          <button type="submit">Restore Archive</button>
        </div>
      </form>
    </article>
  );
}

function formatTimestamp(value: string) {
  return new Date(value).toLocaleString();
}

function formatDuration(durationMs: number) {
  if (durationMs < 1000) {
    return `${durationMs}ms`;
  }
  return `${(durationMs / 1000).toFixed(1)}s`;
}

function formatStatus(value: string) {
  return value.charAt(0).toUpperCase() + value.slice(1);
}

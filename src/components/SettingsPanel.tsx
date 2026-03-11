import { useEffect, useState, type FormEvent } from "react";
import { useAppStore } from "../store/appStore";
import type { UpdateChannel } from "../types/api";

export function SettingsPanel() {
  const { settings, updates, servers, updateSettings, checkForUpdates, installUpdate } =
    useAppStore();
  const [diagnosticsOptIn, setDiagnosticsOptIn] = useState(settings?.diagnosticsOptIn ?? false);
  const [updateChannel, setUpdateChannel] = useState<UpdateChannel>(
    settings?.updateChannel ?? "stable"
  );
  const [defaultServerDirectory, setDefaultServerDirectory] = useState(
    settings?.defaultServerDirectory ?? ""
  );
  const [defaultBackupDirectory, setDefaultBackupDirectory] = useState(
    settings?.defaultBackupDirectory ?? ""
  );
  const [defaultJavaDirectory, setDefaultJavaDirectory] = useState(
    settings?.defaultJavaDirectory ?? ""
  );
  const [checking, setChecking] = useState(false);
  const [installing, setInstalling] = useState(false);

  useEffect(() => {
    if (!settings) {
      return;
    }
    setDiagnosticsOptIn(settings.diagnosticsOptIn);
    setUpdateChannel(settings.updateChannel);
    setDefaultServerDirectory(settings.defaultServerDirectory);
    setDefaultBackupDirectory(settings.defaultBackupDirectory);
    setDefaultJavaDirectory(settings.defaultJavaDirectory);
  }, [settings]);

  if (!settings) {
    return null;
  }

  const runningServers = servers.filter((server) => server.status === "running");

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await updateSettings({
      updateChannel,
      diagnosticsOptIn,
      defaultServerDirectory,
      defaultBackupDirectory,
      defaultJavaDirectory
    });
    setChecking(true);
    try {
      await checkForUpdates();
    } finally {
      setChecking(false);
    }
  }

  async function handleCheckForUpdates() {
    setChecking(true);
    try {
      await checkForUpdates();
    } finally {
      setChecking(false);
    }
  }

  async function handleInstallUpdate() {
    setInstalling(true);
    try {
      await installUpdate();
    } finally {
      setInstalling(false);
    }
  }

  const installBusy = installing || updates?.installState === "installing";
  const checkBusy = checking || updates?.installState === "checking";

  return (
    <section className="panel split-panel">
      <form className="form-grid" onSubmit={handleSubmit}>
        <div className="section-heading">
          <div>
            <p className="eyebrow">Settings</p>
            <h2>Application Defaults</h2>
          </div>
        </div>
        <label>
          Update channel
          <select value={updateChannel} onChange={(event) => setUpdateChannel(event.target.value as UpdateChannel)}>
            <option value="stable">Stable</option>
            <option value="beta">Beta</option>
          </select>
        </label>
        <label>
          Default server directory
          <input
            value={defaultServerDirectory}
            onChange={(event) => setDefaultServerDirectory(event.target.value)}
          />
        </label>
        <label>
          Default backup directory
          <input
            value={defaultBackupDirectory}
            onChange={(event) => setDefaultBackupDirectory(event.target.value)}
          />
        </label>
        <label>
          Default Java directory
          <input
            value={defaultJavaDirectory}
            onChange={(event) => setDefaultJavaDirectory(event.target.value)}
          />
        </label>
        <label className="checkbox">
          <input
            checked={diagnosticsOptIn}
            type="checkbox"
            onChange={(event) => setDiagnosticsOptIn(event.target.checked)}
          />
          Opt in to diagnostics and crash reporting.
        </label>
        <div className="form-actions">
          <button type="submit">Save Settings</button>
        </div>
      </form>
      <div className="panel inset-panel updater-panel">
        <p className="eyebrow">Updates</p>
        <h3>In-App Updater</h3>
        <p>
          Current version: <strong>{updates?.currentVersion ?? "0.1.0"}</strong>
        </p>
        <p>
          Selected channel: <strong>{updates?.channel ?? updateChannel}</strong>
        </p>
        <p>
          Latest available: <strong>{updates?.availableRelease?.version ?? "No update detected"}</strong>
        </p>
        <p>
          Last checked:{" "}
          <strong>
            {updates?.lastCheckedAt ? new Date(updates.lastCheckedAt).toLocaleString() : "Not checked yet"}
          </strong>
        </p>
        <p>
          {updates?.updateAvailable
            ? "A newer MSMS release is available for this channel."
            : updates?.installState === "restartRequired"
              ? "The update has been installed. Restart MSMS to finish applying it."
              : "You are on the latest release for this channel."}
        </p>
        {runningServers.length > 0 && (
          <p className="muted">
            Installing an update will gracefully stop {runningServers.length} running server
            {runningServers.length === 1 ? "" : "s"} before the installer runs.
          </p>
        )}
        {updates?.error && <p className="status-pill status-error">{updates.error}</p>}
        <div className="table-actions">
          <button disabled={checkBusy} onClick={() => void handleCheckForUpdates()} type="button">
            {checkBusy ? "Checking..." : "Check for updates"}
          </button>
          <button
            disabled={!updates?.availableRelease?.installReady || installBusy}
            onClick={() => void handleInstallUpdate()}
            type="button"
          >
            {installBusy ? "Installing..." : "Download and install"}
          </button>
        </div>
        <div className="release-notes">
          <strong>Release notes</strong>
          <div className="release-notes-body">
            {updates?.availableRelease?.notes ?? "No release notes are available for this update."}
          </div>
        </div>
      </div>
    </section>
  );
}

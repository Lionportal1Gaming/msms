import { useMemo, useState } from "react";
import { AuthGate } from "./components/AuthGate";
import { BackupsPanel } from "./components/BackupsPanel";
import { ConsolePanel } from "./components/ConsolePanel";
import { ProvisionWizard } from "./components/ProvisionWizard";
import { RuntimePanel } from "./components/RuntimePanel";
import { ServerConfigPanel } from "./components/ServerConfigPanel";
import { ServerTable } from "./components/ServerTable";
import { SettingsPanel } from "./components/SettingsPanel";
import { Sidebar } from "./components/Sidebar";
import { StatCard } from "./components/StatCard";
import { UpdateNotice } from "./components/UpdateNotice";
import { AppStoreProvider, useAppStore } from "./store/appStore";

function AppShell() {
  const [activeView, setActiveView] = useState("dashboard");
  const [selectedServerId, setSelectedServerId] = useState<string | null>(null);
  const {
    loading,
    unlocked,
    servers,
    backupJobs,
    runtimes,
    updates,
    startServer,
    stopServer,
    restartServer,
    killServer,
    loadConsole
  } = useAppStore();

  const activeServer = useMemo(() => {
    if (selectedServerId) {
      return servers.find((server) => server.id === selectedServerId) ?? servers[0];
    }
    return servers[0];
  }, [selectedServerId, servers]);

  async function handleStop(serverId: string) {
    const server = servers.find((entry) => entry.id === serverId);
    if (!server) {
      return;
    }
    if (!window.confirm(`Gracefully stop ${server.name}? This sends a normal shutdown request.`)) {
      return;
    }
    await stopServer(serverId);
  }

  async function handleKill(serverId: string) {
    const server = servers.find((entry) => entry.id === serverId);
    if (!server) {
      return;
    }
    if (!window.confirm(`Force kill ${server.name}? This terminates the process immediately.`)) {
      return;
    }
    await killServer(serverId);
  }

  if (loading) {
    return (
      <main className="loading-shell">
        <div className="panel">
          <p className="eyebrow">MSMS</p>
          <h1>Loading workspace</h1>
        </div>
      </main>
    );
  }

  if (!unlocked) {
    return <AuthGate />;
  }

  return (
    <main className="app-shell">
      <Sidebar activeView={activeView} onSelect={setActiveView} />
      <section className="content-shell">
        <header className="hero panel">
          <div>
            <p className="eyebrow">Operations Overview</p>
            <h1>Control local Minecraft infrastructure with release-grade discipline.</h1>
            <p className="muted">
              Signed updates, tracked versions, managed runtimes, and predictable server operations.
            </p>
          </div>
          <div className="stats-grid">
            <StatCard
              label="Managed servers"
              value={String(servers.length)}
              detail="Provisioned by MSMS"
            />
            <StatCard
              label="Java runtimes"
              value={String(runtimes.length)}
              detail="Detected or managed installs"
            />
            <StatCard
              label="Backup jobs"
              value={String(backupJobs.length)}
              detail="Local schedule definitions"
            />
            <StatCard
              label="Updater"
              value={
                updates?.installState === "restartRequired"
                  ? "Restart required"
                  : updates?.updateAvailable
                    ? "Update ready"
                    : "Current"
              }
              detail={`Version ${updates?.currentVersion ?? "0.1.0"}`}
            />
          </div>
        </header>

        <UpdateNotice
          updates={updates}
          onOpenSettings={() => setActiveView("settings")}
        />

        {(activeView === "dashboard" || activeView === "console") && (
          <ServerTable
            servers={servers}
            onStart={(serverId) => void startServer(serverId)}
            onStop={(serverId) => void handleStop(serverId)}
            onRestart={(serverId) => void restartServer(serverId)}
            onKill={(serverId) => void handleKill(serverId)}
            onOpenConsole={(serverId) => {
              setSelectedServerId(serverId);
              setActiveView("console");
              void loadConsole(serverId);
            }}
          />
        )}

        {activeView === "dashboard" && <ProvisionWizard />}
        {activeView === "provision" && <ProvisionWizard />}
        {activeView === "config" && <ServerConfigPanel />}
        {activeView === "runtimes" && <RuntimePanel />}
        {activeView === "backups" && <BackupsPanel />}
        {activeView === "console" && <ConsolePanel server={activeServer} />}
        {activeView === "settings" && <SettingsPanel />}
      </section>
    </main>
  );
}

export default function App() {
  return (
    <AppStoreProvider>
      <AppShell />
    </AppStoreProvider>
  );
}

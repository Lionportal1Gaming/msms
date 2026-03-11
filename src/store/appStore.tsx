import { createContext, useContext, useEffect, useState, type PropsWithChildren } from "react";
import { desktopApi } from "../lib/tauri";
import type {
  AppSettings,
  BackupJob,
  BackupRunRecord,
  CommandHistoryEntry,
  ConsoleEntry,
  CreateBackupJobRequest,
  InstallJavaRuntimeRequest,
  JavaRuntime,
  ManagedServer,
  MinecraftVersionOption,
  ProvisionServerRequest,
  ProvisionValidationResult,
  RestoreBackupRequest,
  SendServerCommandRequest,
  ServerProperties,
  SetPasswordRequest,
  UnlockRequest,
  UpdateServerPropertiesRequest,
  UpdateSettingsRequest,
  UpdaterStatus,
  ValidateProvisioningRequest
} from "../types/api";

interface AppState {
  loading: boolean;
  unlocked: boolean;
  passwordConfigured: boolean;
  servers: ManagedServer[];
  minecraftVersions: MinecraftVersionOption[];
  runtimes: JavaRuntime[];
  backupJobs: BackupJob[];
  backupRunRecords: Record<string, BackupRunRecord[]>;
  settings: AppSettings | null;
  updates: UpdaterStatus | null;
  consoleEntries: ConsoleEntry[];
  consoleHistory: ConsoleEntry[];
  commandHistory: CommandHistoryEntry[];
  serverProperties: ServerProperties | null;
  refresh: () => Promise<void>;
  setPassword: (request: SetPasswordRequest) => Promise<void>;
  unlock: (request: UnlockRequest) => Promise<void>;
  loadMinecraftVersions: () => Promise<void>;
  validateProvisioning: (
    request: ValidateProvisioningRequest
  ) => Promise<ProvisionValidationResult>;
  provisionServer: (request: ProvisionServerRequest) => Promise<void>;
  createBackupJob: (request: CreateBackupJobRequest) => Promise<void>;
  runBackupJob: (backupJobId: string) => Promise<void>;
  installJavaRuntime: (request: InstallJavaRuntimeRequest) => Promise<void>;
  restoreBackup: (request: RestoreBackupRequest) => Promise<void>;
  loadConsoleHistory: (serverId: string) => Promise<void>;
  loadCommandHistory: (serverId: string) => Promise<void>;
  sendServerCommand: (request: SendServerCommandRequest) => Promise<void>;
  loadServerProperties: (serverId: string) => Promise<void>;
  updateServerProperties: (request: UpdateServerPropertiesRequest) => Promise<void>;
  updateSettings: (request: UpdateSettingsRequest) => Promise<void>;
  checkForUpdates: () => Promise<void>;
  installUpdate: () => Promise<void>;
  startServer: (serverId: string) => Promise<void>;
  stopServer: (serverId: string) => Promise<void>;
  restartServer: (serverId: string) => Promise<void>;
  killServer: (serverId: string) => Promise<void>;
  loadConsole: (serverId: string) => Promise<void>;
}

const AppStoreContext = createContext<AppState | null>(null);

export function AppStoreProvider({ children }: PropsWithChildren) {
  const [loading, setLoading] = useState(true);
  const [unlocked, setUnlocked] = useState(false);
  const [passwordConfigured, setPasswordConfigured] = useState(false);
  const [servers, setServers] = useState<ManagedServer[]>([]);
  const [minecraftVersions, setMinecraftVersions] = useState<MinecraftVersionOption[]>([]);
  const [runtimes, setRuntimes] = useState<JavaRuntime[]>([]);
  const [backupJobs, setBackupJobs] = useState<BackupJob[]>([]);
  const [backupRunRecords, setBackupRunRecords] = useState<Record<string, BackupRunRecord[]>>({});
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [updates, setUpdates] = useState<UpdaterStatus | null>(null);
  const [consoleEntries, setConsoleEntries] = useState<ConsoleEntry[]>([]);
  const [consoleHistory, setConsoleHistory] = useState<ConsoleEntry[]>([]);
  const [commandHistory, setCommandHistory] = useState<CommandHistoryEntry[]>([]);
  const [serverProperties, setServerProperties] = useState<ServerProperties | null>(null);

  async function refresh() {
    setLoading(true);
    try {
      const [
        bootstrap,
        nextServers,
        nextMinecraftVersions,
        nextRuntimes,
        nextBackups,
        nextSettings,
        nextUpdates
      ] =
        await Promise.all([
          desktopApi.bootstrapStatus(),
          desktopApi.listServers(),
          desktopApi.listMinecraftVersions(),
          desktopApi.discoverJavaRuntimes(),
          desktopApi.listBackupJobs(),
          desktopApi.getSettings(),
          desktopApi.getUpdaterStatus()
        ]);
      const nextBackupRunRecords = Object.fromEntries(
        await Promise.all(
          nextBackups.map(async (job) => [
            job.id,
            await desktopApi.listBackupRunRecords(job.id)
          ])
        )
      ) as Record<string, BackupRunRecord[]>;

      setPasswordConfigured(bootstrap.passwordConfigured);
      setUnlocked(bootstrap.unlocked);
      setServers(nextServers);
      setMinecraftVersions(nextMinecraftVersions);
      setRuntimes(nextRuntimes);
      setBackupJobs(nextBackups);
      setBackupRunRecords(nextBackupRunRecords);
      setSettings(nextSettings);
      setUpdates(nextUpdates);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
    void (async () => {
      try {
        setUpdates(await desktopApi.checkForUpdates());
      } catch {
        setUpdates(await desktopApi.getUpdaterStatus());
      }
    })();
  }, []);

  const value: AppState = {
    loading,
    unlocked,
    passwordConfigured,
    servers,
    minecraftVersions,
    runtimes,
    backupJobs,
    backupRunRecords,
    settings,
    updates,
    consoleEntries,
    consoleHistory,
    commandHistory,
    serverProperties,
    refresh,
    setPassword: async (request) => {
      await desktopApi.setAppPassword(request);
      await refresh();
    },
    unlock: async (request) => {
      await desktopApi.unlockApp(request);
      await refresh();
    },
    loadMinecraftVersions: async () => {
      setMinecraftVersions(await desktopApi.listMinecraftVersions());
    },
    validateProvisioning: async (request) => desktopApi.validateProvisioning(request),
    provisionServer: async (request) => {
      await desktopApi.provisionServer(request);
      await refresh();
    },
    createBackupJob: async (request) => {
      await desktopApi.createBackupJob(request);
      await refresh();
    },
    runBackupJob: async (backupJobId) => {
      await desktopApi.runBackupJob(backupJobId);
      await refresh();
    },
    installJavaRuntime: async (request) => {
      await desktopApi.installJavaRuntime(request);
      await refresh();
    },
    restoreBackup: async (request) => {
      await desktopApi.restoreBackup(request);
      await refresh();
    },
    loadConsoleHistory: async (serverId) => {
      setConsoleHistory(await desktopApi.getConsoleHistory(serverId));
    },
    loadCommandHistory: async (serverId) => {
      setCommandHistory(await desktopApi.getCommandHistory(serverId));
    },
    sendServerCommand: async (request) => {
      await desktopApi.sendServerCommand(request);
      setConsoleEntries(await desktopApi.getServerConsole(request.serverId));
      setCommandHistory(await desktopApi.getCommandHistory(request.serverId));
    },
    loadServerProperties: async (serverId) => {
      setServerProperties(await desktopApi.getServerProperties(serverId));
    },
    updateServerProperties: async (request) => {
      setServerProperties(await desktopApi.updateServerProperties(request));
    },
    updateSettings: async (request) => {
      const nextSettings = await desktopApi.updateSettings(request);
      setSettings(nextSettings);
    },
    checkForUpdates: async () => {
      try {
        setUpdates(await desktopApi.checkForUpdates());
      } catch {
        setUpdates(await desktopApi.getUpdaterStatus());
      }
    },
    installUpdate: async () => {
      try {
        setUpdates(await desktopApi.installUpdate());
      } catch {
        setUpdates(await desktopApi.getUpdaterStatus());
      } finally {
        await refresh();
      }
    },
    startServer: async (serverId) => {
      await desktopApi.startServer(serverId);
      await refresh();
    },
    stopServer: async (serverId) => {
      await desktopApi.stopServer(serverId);
      await refresh();
    },
    restartServer: async (serverId) => {
      await desktopApi.restartServer(serverId);
      await refresh();
    },
    killServer: async (serverId) => {
      await desktopApi.killServer(serverId);
      await refresh();
    },
    loadConsole: async (serverId) => {
      setConsoleEntries(await desktopApi.getServerConsole(serverId));
    }
  };

  return (
    <AppStoreContext.Provider value={value}>
      {children}
    </AppStoreContext.Provider>
  );
}

export function useAppStore() {
  const context = useContext(AppStoreContext);
  if (!context) {
    throw new Error("useAppStore must be used within AppStoreProvider");
  }
  return context;
}

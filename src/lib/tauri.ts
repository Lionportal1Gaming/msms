import type {
  AvailableRelease,
  AppSettings,
  BackupJob,
  BackupRunRecord,
  BackupScheduleConfig,
  BackupSchedulePreset,
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

type CommandMap = {
  bootstrap_status: { passwordConfigured: boolean; unlocked: boolean };
  list_servers: ManagedServer[];
  list_minecraft_versions: MinecraftVersionOption[];
  discover_java_runtimes: JavaRuntime[];
  get_settings: AppSettings;
  list_backup_jobs: BackupJob[];
  list_backup_run_records: BackupRunRecord[];
  install_java_runtime: JavaRuntime;
  set_app_password: void;
  unlock_app: void;
  validate_provisioning: ProvisionValidationResult;
  provision_server: ManagedServer;
  create_backup_job: BackupJob;
  run_backup_job: string;
  restore_backup: void;
  get_console_history: ConsoleEntry[];
  get_command_history: CommandHistoryEntry[];
  send_server_command: void;
  get_server_properties: ServerProperties;
  update_server_properties: ServerProperties;
  update_settings: AppSettings;
  start_server: ManagedServer;
  stop_server: ManagedServer;
  restart_server: ManagedServer;
  kill_server: ManagedServer;
  get_server_console: ConsoleEntry[];
  get_updater_status: UpdaterStatus;
  check_for_updates: UpdaterStatus;
  install_update: UpdaterStatus;
};

const mockNow = new Date().toISOString();

const mockState: {
  passwordConfigured: boolean;
  unlocked: boolean;
  settings: AppSettings;
  minecraftVersions: MinecraftVersionOption[];
  runtimes: JavaRuntime[];
  servers: ManagedServer[];
  backups: BackupJob[];
  backupRuns: Record<string, BackupRunRecord[]>;
  updaterStatus: UpdaterStatus;
  consoleHistory: Record<string, ConsoleEntry[]>;
  liveConsole: Record<string, ConsoleEntry[]>;
  commandHistory: Record<string, CommandHistoryEntry[]>;
  serverProperties: Record<string, ServerProperties>;
} = {
  passwordConfigured: false,
  unlocked: false,
  settings: {
    authMode: "password" as const,
    updateChannel: "stable" as const,
    diagnosticsOptIn: false,
    defaultServerDirectory: "~/MSMS/servers",
    defaultBackupDirectory: "~/MSMS/backups",
    defaultJavaDirectory: "~/MSMS/java"
  },
  minecraftVersions: [
    {
      id: "1.21.4",
      releaseType: "release",
      publishedAt: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
      serverDownloadAvailable: true,
      requiredJavaMajor: 21
    },
    {
      id: "1.20.6",
      releaseType: "release",
      publishedAt: new Date(Date.now() - 21 * 24 * 60 * 60 * 1000).toISOString(),
      serverDownloadAvailable: true,
      requiredJavaMajor: 17
    },
    {
      id: "1.19.4",
      releaseType: "release",
      publishedAt: new Date(Date.now() - 365 * 24 * 60 * 60 * 1000).toISOString(),
      serverDownloadAvailable: true,
      requiredJavaMajor: 17
    }
  ],
  runtimes: [
    {
      id: "java-21",
      version: "21.0.6",
      vendor: "Temurin",
      installPath: "~/MSMS/java/jdk-21",
      architecture: "x64",
      managedByApp: true
    }
  ],
  servers: [
    {
      id: "srv-primary",
      name: "Primary Survival",
      minecraftVersion: "1.21.4",
      serverPath: "~/MSMS/servers/primary-survival",
      jarPath: "~/MSMS/servers/primary-survival/server.jar",
      javaRuntimeId: "java-21",
      status: "stopped" as const,
      port: 25565,
      memoryMb: 4096,
      eulaAccepted: true,
      createdAt: mockNow,
      updatedAt: mockNow
    }
  ],
  backups: [
    {
      id: "backup-nightly",
      serverId: "srv-primary",
      schedule: "Daily at 02:00",
      schedulePreset: "daily",
      scheduleConfig: {
        hour: 2,
        minute: 0
      },
      retentionCount: 7,
      destinationPath: "~/MSMS/backups/primary-survival",
      nextRunAt: new Date(Date.now() + 12 * 60 * 60 * 1000).toISOString(),
      lastRunAt: null,
      lastStatus: "idle",
      lastDurationMs: null,
      lastResult: "No backups have run yet.",
      isLegacySchedule: false
    }
  ],
  backupRuns: {
    "backup-nightly": []
  },
  updaterStatus: {
    currentVersion: "0.1.0",
    channel: "stable",
    lastCheckedAt: null,
    updateAvailable: false,
    availableRelease: null,
    installState: "idle",
    error: null
  },
  consoleHistory: {
    "srv-primary": [
      {
        serverId: "srv-primary",
        source: "system",
        message: "Server stopped.",
        timestamp: mockNow
      }
    ]
  },
  liveConsole: {
    "srv-primary": []
  },
  commandHistory: {
    "srv-primary": []
  },
  serverProperties: {
    "srv-primary": {
      motd: "Primary Survival",
      "server-port": "25565",
      difficulty: "normal",
      "max-players": "20",
      "online-mode": "true",
      pvp: "true"
    }
  }
};

function isTauriRuntime() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

async function invokeTyped<K extends keyof CommandMap>(
  command: K,
  args?: Record<string, unknown>
): Promise<CommandMap[K]> {
  if (!isTauriRuntime()) {
    return mockInvoke(command, args);
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return invoke(command, args) as Promise<CommandMap[K]>;
}

async function mockInvoke<K extends keyof CommandMap>(
  command: K,
  args?: Record<string, unknown>
): Promise<CommandMap[K]> {
  switch (command) {
    case "bootstrap_status":
      return {
        passwordConfigured: mockState.passwordConfigured,
        unlocked: mockState.unlocked
      } as CommandMap[K];
    case "discover_java_runtimes":
      return mockState.runtimes as CommandMap[K];
    case "list_minecraft_versions":
      return mockState.minecraftVersions as CommandMap[K];
    case "list_servers":
      return mockState.servers as CommandMap[K];
    case "get_settings":
      return mockState.settings as CommandMap[K];
    case "get_updater_status":
      return mockState.updaterStatus as CommandMap[K];
    case "list_backup_jobs":
      return mockState.backups as CommandMap[K];
    case "list_backup_run_records": {
      const backupJobId = args?.backupJobId as string;
      return (mockState.backupRuns[backupJobId] ?? []) as CommandMap[K];
    }
    case "install_java_runtime": {
      const request = args?.request as InstallJavaRuntimeRequest;
      const runtime: JavaRuntime = {
        id: `java-${request.vendor.toLowerCase()}-${request.version}`,
        version: request.version,
        vendor: request.vendor,
        installPath: `~/MSMS/java/${request.vendor.toLowerCase()}-${request.version}`,
        architecture: "x64",
        managedByApp: true
      };
      mockState.runtimes = [runtime, ...mockState.runtimes];
      return runtime as CommandMap[K];
    }
    case "set_app_password":
      mockState.passwordConfigured = true;
      mockState.unlocked = true;
      return undefined as CommandMap[K];
    case "unlock_app":
      mockState.unlocked = true;
      return undefined as CommandMap[K];
    case "validate_provisioning": {
      const request = args?.request as ValidateProvisioningRequest;
      return {
        normalizedTargetDirectory: request.targetDirectory,
        issues: validateMockProvisioning(request)
      } as CommandMap[K];
    }
    case "provision_server": {
      const request = args?.request as ProvisionServerRequest;
      const server: ManagedServer = {
        id: `srv-${request.name.toLowerCase().replace(/\s+/g, "-")}`,
        name: request.name,
        minecraftVersion: request.minecraftVersion,
        serverPath: request.targetDirectory,
        jarPath: `${request.targetDirectory}/server.jar`,
        javaRuntimeId: request.javaRuntimeId ?? null,
        status: "stopped",
        port: request.port,
        memoryMb: request.memoryMb,
        eulaAccepted: request.eulaAccepted,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString()
      };
      mockState.servers = [server, ...mockState.servers];
      mockState.consoleHistory[server.id] = [];
      mockState.liveConsole[server.id] = [];
      mockState.commandHistory[server.id] = [];
      mockState.serverProperties[server.id] = {
        ...defaultMockServerProperties(server.name, server.port),
        ...request.serverProperties,
        "server-port": String(server.port)
      };
      return server as CommandMap[K];
    }
    case "create_backup_job": {
      const request = args?.request as CreateBackupJobRequest;
      const job: BackupJob = {
        id: `backup-${request.serverId}-${Date.now()}`,
        serverId: request.serverId,
        schedule: formatMockBackupSchedule(request.schedulePreset, request.scheduleConfig),
        schedulePreset: request.schedulePreset,
        scheduleConfig: request.scheduleConfig,
        retentionCount: request.retentionCount,
        destinationPath: request.destinationPath,
        nextRunAt: computeMockNextRunAt(request.schedulePreset, request.scheduleConfig),
        lastRunAt: null,
        lastStatus: "idle",
        lastDurationMs: null,
        lastResult: "Scheduled, waiting for first run.",
        isLegacySchedule: false
      };
      mockState.backups = [job, ...mockState.backups];
      mockState.backupRuns[job.id] = [];
      return job as CommandMap[K];
    }
    case "run_backup_job": {
      const backupJobId = args?.backupJobId as string;
      const job = mockState.backups.find((entry) => entry.id === backupJobId);
      if (!job) {
        throw new Error(`Unknown backup job: ${backupJobId}`);
      }
      const timestamp = new Date().toISOString();
      const message = `Backup created at ${job.destinationPath}/${job.serverId}-${Date.now()}.zip`;
      job.lastRunAt = timestamp;
      job.lastStatus = "succeeded";
      job.lastDurationMs = 1200;
      job.lastResult = message;
      job.nextRunAt = job.schedulePreset
        ? computeMockNextRunAt(job.schedulePreset, job.scheduleConfig)
        : null;
      mockState.backupRuns[backupJobId] = [
        {
          id: Date.now(),
          jobId: backupJobId,
          startedAt: timestamp,
          finishedAt: new Date(Date.now() + 1200).toISOString(),
          status: "succeeded" as const,
          message
        },
        ...(mockState.backupRuns[backupJobId] ?? [])
      ].slice(0, 20);
      return message as CommandMap[K];
    }
    case "restore_backup":
      return undefined as CommandMap[K];
    case "get_console_history": {
      const serverId = args?.serverId as string;
      return (mockState.consoleHistory[serverId] ?? []) as CommandMap[K];
    }
    case "get_command_history": {
      const serverId = args?.serverId as string;
      return (mockState.commandHistory[serverId] ?? []) as CommandMap[K];
    }
    case "send_server_command": {
      const request = args?.request as SendServerCommandRequest;
      const server = mockState.servers.find((entry) => entry.id === request.serverId);
      if (!server || server.status !== "running") {
        throw new Error("Server must be running before commands can be submitted");
      }
      const timestamp = new Date().toISOString();
      const commandEntry: ConsoleEntry = {
        serverId: request.serverId,
        source: "command",
        message: request.command,
        timestamp
      };
      const outputEntry: ConsoleEntry = {
        serverId: request.serverId,
        source: "stdout",
        message: `Executed command: ${request.command}`,
        timestamp: new Date(Date.now() + 1).toISOString()
      };
      mockState.liveConsole[request.serverId] = [
        ...(mockState.liveConsole[request.serverId] ?? []),
        commandEntry,
        outputEntry
      ].slice(-500);
      mockState.commandHistory[request.serverId] = [
        ...(mockState.commandHistory[request.serverId] ?? []),
        {
          serverId: request.serverId,
          command: request.command,
          timestamp
        }
      ].slice(-50);
      return undefined as CommandMap[K];
    }
    case "get_server_properties": {
      const serverId = args?.serverId as string;
      return (
        mockState.serverProperties[serverId] ?? {
          motd: "Minecraft Server",
          "server-port": "25565",
          difficulty: "easy",
          "max-players": "20",
          "online-mode": "true",
          pvp: "true"
        }
      ) as CommandMap[K];
    }
    case "update_server_properties": {
      const request = args?.request as UpdateServerPropertiesRequest;
      mockState.serverProperties[request.serverId] = request.properties;
      return mockState.serverProperties[request.serverId] as CommandMap[K];
    }
    case "update_settings":
      mockState.settings = {
        ...mockState.settings,
        ...(args?.request as UpdateSettingsRequest)
      };
      mockState.updaterStatus = {
        ...mockState.updaterStatus,
        channel: mockState.settings.updateChannel,
        lastCheckedAt: null,
        updateAvailable: false,
        availableRelease: null,
        installState: "idle",
        error: null
      };
      return mockState.settings as CommandMap[K];
    case "start_server":
    case "stop_server":
    case "restart_server":
    case "kill_server": {
      const serverId = args?.serverId as string;
      const nextStatus: ManagedServer["status"] =
        command === "start_server"
          ? "running"
          : command === "stop_server"
            ? "stopped"
            : command === "restart_server"
              ? "running"
              : "stopped";
      const server = mockState.servers.find((entry) => entry.id === serverId);
      if (!server) {
        throw new Error(`Unknown server: ${serverId}`);
      }
      server.status = nextStatus;
      server.updatedAt = new Date().toISOString();
      if (command === "start_server") {
        mockState.liveConsole[serverId] = [
          {
            serverId,
            source: "system",
            message: "Server process started.",
            timestamp: new Date().toISOString()
          },
          {
            serverId,
            source: "stdout",
            message: "[Server thread/INFO]: Done (0.612s)! For help, type \"help\"",
            timestamp: new Date(Date.now() + 1).toISOString()
          }
        ];
      } else {
        const archived: ConsoleEntry[] = [
          ...(mockState.consoleHistory[serverId] ?? []),
          ...(mockState.liveConsole[serverId] ?? []),
          {
            serverId,
            source: "system",
            message:
              command === "kill_server" ? "Server process killed." : "Server stopped.",
            timestamp: new Date().toISOString()
          }
        ];
        mockState.consoleHistory[serverId] = archived.slice(-500);
        mockState.liveConsole[serverId] =
          command === "restart_server"
            ? ([
                {
                  serverId,
                  source: "system",
                  message: "Server process started.",
                  timestamp: new Date().toISOString()
                }
              ] as ConsoleEntry[])
            : [];
      }
      return server as CommandMap[K];
    }
    case "get_server_console":
      return (mockState.liveConsole[(args?.serverId as string) ?? "srv-primary"] ?? []) as CommandMap[K];
    case "check_for_updates": {
      const availableRelease = makeMockAvailableRelease(mockState.settings.updateChannel);
      mockState.updaterStatus = {
        currentVersion: "0.1.0",
        channel: mockState.settings.updateChannel,
        lastCheckedAt: new Date().toISOString(),
        updateAvailable: availableRelease ? availableRelease.version !== "0.1.0" : false,
        availableRelease,
        installState: availableRelease && availableRelease.version !== "0.1.0" ? "ready" : "idle",
        error: null
      };
      return mockState.updaterStatus as CommandMap[K];
    }
    case "install_update": {
      if (!mockState.updaterStatus.availableRelease) {
        throw new Error("No update is currently available.");
      }
      mockState.servers = mockState.servers.map((server) =>
        server.status === "running"
          ? {
              ...server,
              status: "stopped",
              updatedAt: new Date().toISOString()
            }
          : server
      );
      mockState.updaterStatus = {
        ...mockState.updaterStatus,
        updateAvailable: false,
        installState: "restartRequired",
        error: null
      };
      return mockState.updaterStatus as CommandMap[K];
    }
    default:
      throw new Error(`Unhandled mock command: ${String(command)}`);
  }
}

export const desktopApi = {
  bootstrapStatus: () => invokeTyped("bootstrap_status"),
  listServers: () => invokeTyped("list_servers"),
  listMinecraftVersions: () => invokeTyped("list_minecraft_versions"),
  discoverJavaRuntimes: () => invokeTyped("discover_java_runtimes"),
  installJavaRuntime: (request: InstallJavaRuntimeRequest) =>
    invokeTyped("install_java_runtime", { request }),
  getSettings: () => invokeTyped("get_settings"),
  getUpdaterStatus: () => invokeTyped("get_updater_status"),
  listBackupJobs: () => invokeTyped("list_backup_jobs"),
  listBackupRunRecords: (backupJobId: string) =>
    invokeTyped("list_backup_run_records", { backupJobId }),
  setAppPassword: (request: SetPasswordRequest) =>
    invokeTyped("set_app_password", { request }),
  unlockApp: (request: UnlockRequest) => invokeTyped("unlock_app", { request }),
  validateProvisioning: (request: ValidateProvisioningRequest) =>
    invokeTyped("validate_provisioning", { request }),
  provisionServer: (request: ProvisionServerRequest) =>
    invokeTyped("provision_server", { request }),
  createBackupJob: (request: CreateBackupJobRequest) =>
    invokeTyped("create_backup_job", { request }),
  runBackupJob: (backupJobId: string) =>
    invokeTyped("run_backup_job", { backupJobId }),
  restoreBackup: (request: RestoreBackupRequest) =>
    invokeTyped("restore_backup", { request }),
  getConsoleHistory: (serverId: string) =>
    invokeTyped("get_console_history", { serverId }),
  getCommandHistory: (serverId: string) =>
    invokeTyped("get_command_history", { serverId }),
  sendServerCommand: (request: SendServerCommandRequest) =>
    invokeTyped("send_server_command", { request }),
  getServerProperties: (serverId: string) =>
    invokeTyped("get_server_properties", { serverId }),
  updateServerProperties: (request: UpdateServerPropertiesRequest) =>
    invokeTyped("update_server_properties", { request }),
  updateSettings: (request: UpdateSettingsRequest) =>
    invokeTyped("update_settings", { request }),
  startServer: (serverId: string) => invokeTyped("start_server", { serverId }),
  stopServer: (serverId: string) => invokeTyped("stop_server", { serverId }),
  restartServer: (serverId: string) =>
    invokeTyped("restart_server", { serverId }),
  killServer: (serverId: string) => invokeTyped("kill_server", { serverId }),
  getServerConsole: (serverId: string) =>
    invokeTyped("get_server_console", { serverId }),
  checkForUpdates: () => invokeTyped("check_for_updates"),
  installUpdate: () => invokeTyped("install_update")
};

function defaultMockServerProperties(name: string, port: number) {
  return {
    motd: name,
    "server-port": String(port),
    difficulty: "normal",
    "max-players": "20",
    "online-mode": "true",
    pvp: "true"
  };
}

function validateMockProvisioning(request: ValidateProvisioningRequest) {
  const issues: ProvisionValidationResult["issues"] = [];
  const selectedVersion = mockState.minecraftVersions.find(
    (entry) => entry.id === request.minecraftVersion
  );

  if (!selectedVersion) {
    issues.push({
      field: "minecraftVersion",
      step: "version",
      message: `Minecraft version ${request.minecraftVersion} is not present in the Mojang release catalog.`
    });
  }

  if (!request.javaRuntimeId) {
    issues.push({
      field: "javaRuntimeId",
      step: "version",
      message: "Choose a Java runtime before provisioning this server."
    });
  } else {
    const runtime = mockState.runtimes.find((entry) => entry.id === request.javaRuntimeId);
    if (!runtime) {
      issues.push({
        field: "javaRuntimeId",
        step: "version",
        message: "Selected Java runtime could not be found."
      });
    } else if (
      selectedVersion?.requiredJavaMajor &&
      Number.parseInt(runtime.version, 10) < selectedVersion.requiredJavaMajor
    ) {
      issues.push({
        field: "javaRuntimeId",
        step: "version",
        message: `${runtime.vendor} ${runtime.version} is too old for Minecraft ${request.minecraftVersion}. Java ${selectedVersion.requiredJavaMajor} or newer is required.`
      });
    }
  }

  if (request.memoryMb < 1024 || request.memoryMb > 32768) {
    issues.push({
      field: "memoryMb",
      step: "details",
      message: "Memory must be between 1024 MB and 32768 MB."
    });
  }

  if (request.port < 1024) {
    issues.push({
      field: "port",
      step: "details",
      message: "Use a TCP port between 1024 and 65535."
    });
  }

  if (mockState.servers.some((entry) => entry.port === request.port)) {
    issues.push({
      field: "port",
      step: "details",
      message: `Port ${request.port} is already assigned to another managed server.`
    });
  }

  if (mockState.servers.some((entry) => entry.serverPath === request.targetDirectory)) {
    issues.push({
      field: "targetDirectory",
      step: "details",
      message: "The selected directory is already managed by MSMS."
    });
  }

  if (
    request.serverProperties["server-port"] &&
    request.serverProperties["server-port"] !== String(request.port)
  ) {
    issues.push({
      field: "serverProperties",
      step: "properties",
      message: "The server-port property must match the selected port."
    });
  }

  return issues;
}

function makeMockAvailableRelease(channel: AppSettings["updateChannel"]): AvailableRelease | null {
  const version = channel === "beta" ? "0.2.0-beta.1" : "0.1.1";
  return {
    version,
    notes:
      channel === "beta"
        ? "Beta release with updater UX improvements."
        : "Stable release with updater UX improvements.",
    publishedAt: new Date(Date.now() - 60 * 60 * 1000).toISOString(),
    downloadReady: true,
    installReady: true
  };
}

function formatMockBackupSchedule(
  schedulePreset: BackupSchedulePreset,
  scheduleConfig: BackupScheduleConfig
) {
  if (schedulePreset === "hourly") {
    return scheduleConfig.intervalHours === 1
      ? "Every hour"
      : `Every ${scheduleConfig.intervalHours ?? 1} hours`;
  }

  const formattedTime = `${String(scheduleConfig.hour ?? 0).padStart(2, "0")}:${String(
    scheduleConfig.minute ?? 0
  ).padStart(2, "0")}`;
  if (schedulePreset === "daily") {
    return `Daily at ${formattedTime}`;
  }

  const weekdays = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday"
  ];
  return `Weekly on ${weekdays[scheduleConfig.weekday ?? 0]} at ${formattedTime}`;
}

function computeMockNextRunAt(
  schedulePreset: BackupSchedulePreset,
  scheduleConfig: BackupScheduleConfig
) {
  const nextRun = new Date();
  if (schedulePreset === "hourly") {
    nextRun.setHours(nextRun.getHours() + (scheduleConfig.intervalHours ?? 1));
    return nextRun.toISOString();
  }

  nextRun.setSeconds(0, 0);
  nextRun.setHours(scheduleConfig.hour ?? 0, scheduleConfig.minute ?? 0, 0, 0);
  if (schedulePreset === "daily") {
    if (nextRun <= new Date()) {
      nextRun.setDate(nextRun.getDate() + 1);
    }
    return nextRun.toISOString();
  }

  const targetWeekday = scheduleConfig.weekday ?? 0;
  const currentWeekday = nextRun.getDay();
  let daysUntil = (targetWeekday - currentWeekday + 7) % 7;
  if (daysUntil === 0 && nextRun <= new Date()) {
    daysUntil = 7;
  }
  nextRun.setDate(nextRun.getDate() + daysUntil);
  return nextRun.toISOString();
}

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { BackupJob, BackupRunRecord } from "../types/api";
import { BackupsPanel } from "./BackupsPanel";

const mockStore = vi.hoisted(() => ({
  backupJobs: [] as BackupJob[],
  backupRunRecords: {} as Record<string, BackupRunRecord[]>,
  servers: [
    {
      id: "srv-primary",
      name: "Primary Survival",
      minecraftVersion: "1.21.4",
      serverPath: "/tmp/server",
      jarPath: "/tmp/server/server.jar",
      javaRuntimeId: "java-21",
      status: "stopped",
      port: 25565,
      memoryMb: 4096,
      eulaAccepted: true,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    }
  ],
  settings: {
    authMode: "password",
    updateChannel: "stable",
    diagnosticsOptIn: false,
    defaultServerDirectory: "/tmp/servers",
    defaultBackupDirectory: "/tmp/backups",
    defaultJavaDirectory: "/tmp/java"
  },
  createBackupJob: vi.fn(),
  runBackupJob: vi.fn(),
  restoreBackup: vi.fn()
}));

vi.mock("../store/appStore", () => ({
  useAppStore: () => mockStore
}));

describe("BackupsPanel", () => {
  beforeEach(() => {
    mockStore.backupJobs = [];
    mockStore.backupRunRecords = {};
    mockStore.createBackupJob.mockReset();
    mockStore.runBackupJob.mockReset();
    mockStore.restoreBackup.mockReset();
  });

  it("submits preset-based backup requests", async () => {
    render(<BackupsPanel />);

    fireEvent.change(screen.getByLabelText(/schedule preset/i), {
      target: { value: "weekly" }
    });
    fireEvent.change(screen.getByLabelText(/weekday/i), {
      target: { value: "2" }
    });
    fireEvent.change(screen.getByLabelText(/^hour$/i), {
      target: { value: "4" }
    });
    fireEvent.change(screen.getByLabelText(/minute/i), {
      target: { value: "30" }
    });
    fireEvent.change(screen.getByLabelText(/retention count/i), {
      target: { value: "5" }
    });
    fireEvent.click(screen.getByRole("button", { name: /save backup job/i }));

    await waitFor(() => {
      expect(mockStore.createBackupJob).toHaveBeenCalledWith({
        serverId: "srv-primary",
        schedulePreset: "weekly",
        scheduleConfig: {
          weekday: 2,
          hour: 4,
          minute: 30
        },
        retentionCount: 5,
        destinationPath: "/tmp/backups/primary-survival"
      });
    });
  });

  it("shows a re-save warning for legacy jobs", () => {
    mockStore.backupJobs = [
      {
        id: "backup-legacy",
        serverId: "srv-primary",
        schedule: "0 2 * * *",
        schedulePreset: null,
        scheduleConfig: {},
        retentionCount: 7,
        destinationPath: "/tmp/backups/primary-survival",
        nextRunAt: null,
        lastRunAt: null,
        lastStatus: "idle",
        lastDurationMs: null,
        lastResult: "Scheduled",
        isLegacySchedule: true
      }
    ];

    render(<BackupsPanel />);

    expect(
      screen.getByText(/legacy schedule detected\. re-save this job to enable automatic execution\./i)
    ).toBeInTheDocument();
  });

  it("requires confirmation before restore dispatch", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
    render(<BackupsPanel />);

    fireEvent.change(screen.getByPlaceholderText(/path\/to\/archive\.zip/i), {
      target: { value: "/tmp/archive.zip" }
    });
    fireEvent.click(screen.getByRole("button", { name: /restore archive/i }));

    await waitFor(() => {
      expect(confirmSpy).toHaveBeenCalled();
    });
    expect(mockStore.restoreBackup).not.toHaveBeenCalled();
    confirmSpy.mockRestore();
  });
});

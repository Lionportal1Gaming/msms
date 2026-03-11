import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AppSettings, ManagedServer, UpdaterStatus } from "../types/api";
import { SettingsPanel } from "./SettingsPanel";

const mockStore = vi.hoisted(() => ({
  settings: {
    authMode: "password",
    updateChannel: "stable",
    diagnosticsOptIn: false,
    defaultServerDirectory: "/tmp/servers",
    defaultBackupDirectory: "/tmp/backups",
    defaultJavaDirectory: "/tmp/java"
  } as AppSettings,
  updates: {
    currentVersion: "0.1.0",
    channel: "stable",
    lastCheckedAt: new Date().toISOString(),
    updateAvailable: true,
    availableRelease: {
      version: "0.1.1",
      notes: "Updater UX improvements.",
      publishedAt: new Date().toISOString(),
      downloadReady: true,
      installReady: true
    },
    installState: "ready",
    error: null
  } as UpdaterStatus,
  servers: [
    {
      id: "srv-primary",
      name: "Primary Survival",
      minecraftVersion: "1.21.4",
      serverPath: "/tmp/server",
      jarPath: "/tmp/server/server.jar",
      javaRuntimeId: "java-21",
      status: "running",
      port: 25565,
      memoryMb: 4096,
      eulaAccepted: true,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    }
  ] as ManagedServer[],
  updateSettings: vi.fn(),
  checkForUpdates: vi.fn(),
  installUpdate: vi.fn()
}));

vi.mock("../store/appStore", () => ({
  useAppStore: () => mockStore
}));

describe("SettingsPanel", () => {
  beforeEach(() => {
    mockStore.updateSettings.mockReset();
    mockStore.checkForUpdates.mockReset();
    mockStore.installUpdate.mockReset();
  });

  it("renders release notes and auto-stop warning", () => {
    render(<SettingsPanel />);

    expect(screen.getByText(/updater ux improvements\./i)).toBeInTheDocument();
    expect(
      screen.getByText(/gracefully stop 1 running server before the installer runs/i)
    ).toBeInTheDocument();
  });

  it("saves the selected channel and re-checks for updates", async () => {
    render(<SettingsPanel />);

    fireEvent.change(screen.getByLabelText(/update channel/i), {
      target: { value: "beta" }
    });
    fireEvent.click(screen.getByRole("button", { name: /save settings/i }));

    await waitFor(() => {
      expect(mockStore.updateSettings).toHaveBeenCalledWith({
        updateChannel: "beta",
        diagnosticsOptIn: false,
        defaultServerDirectory: "/tmp/servers",
        defaultBackupDirectory: "/tmp/backups",
        defaultJavaDirectory: "/tmp/java"
      });
    });
    expect(mockStore.checkForUpdates).toHaveBeenCalled();
  });

  it("runs the one-click install action", async () => {
    render(<SettingsPanel />);

    fireEvent.click(screen.getByRole("button", { name: /download and install/i }));

    await waitFor(() => {
      expect(mockStore.installUpdate).toHaveBeenCalled();
    });
  });
});

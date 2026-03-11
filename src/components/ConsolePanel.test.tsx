import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ConsolePanel } from "./ConsolePanel";

const mockStore = vi.hoisted(() => ({
  consoleEntries: [],
  consoleHistory: [],
  commandHistory: [],
  loadConsole: vi.fn(),
  loadConsoleHistory: vi.fn(),
  loadCommandHistory: vi.fn(),
  sendServerCommand: vi.fn(),
  refresh: vi.fn()
}));

vi.mock("../store/appStore", () => ({
  useAppStore: () => mockStore
}));

describe("ConsolePanel", () => {
  beforeEach(() => {
    mockStore.consoleEntries = [];
    mockStore.consoleHistory = [];
    mockStore.commandHistory = [];
    mockStore.loadConsole.mockReset();
    mockStore.loadConsoleHistory.mockReset();
    mockStore.loadCommandHistory.mockReset();
    mockStore.sendServerCommand.mockReset();
    mockStore.refresh.mockReset();
  });

  it("disables command input when the server is offline", () => {
    render(
      <ConsolePanel
        server={{
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
        }}
      />
    );

    expect(screen.getByPlaceholderText(/start the server to enable commands/i)).toBeDisabled();
  });

  it("submits a command on enter while running", async () => {
    render(
      <ConsolePanel
        server={{
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
        }}
      />
    );

    const input = screen.getByPlaceholderText(/type a minecraft server command/i);
    fireEvent.change(input, { target: { value: "save-all" } });
    fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() => {
      expect(mockStore.sendServerCommand).toHaveBeenCalledWith({
        serverId: "srv-primary",
        command: "save-all"
      });
    });
  });
});

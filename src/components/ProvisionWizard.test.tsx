import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ProvisionWizard } from "./ProvisionWizard";

const mockStore = vi.hoisted(() => ({
  settings: {
    authMode: "password",
    updateChannel: "stable",
    diagnosticsOptIn: false,
    defaultServerDirectory: "/tmp/servers",
    defaultBackupDirectory: "/tmp/backups",
    defaultJavaDirectory: "/tmp/java"
  },
  runtimes: [
    {
      id: "java-21",
      version: "21.0.6",
      vendor: "Temurin",
      installPath: "/tmp/java/jdk-21",
      architecture: "x64",
      managedByApp: true
    },
    {
      id: "java-17",
      version: "17.0.12",
      vendor: "Temurin",
      installPath: "/tmp/java/jdk-17",
      architecture: "x64",
      managedByApp: true
    }
  ],
  minecraftVersions: [
    {
      id: "1.21.4",
      releaseType: "release",
      publishedAt: new Date().toISOString(),
      serverDownloadAvailable: true,
      requiredJavaMajor: 21
    },
    {
      id: "1.20.6",
      releaseType: "release",
      publishedAt: new Date().toISOString(),
      serverDownloadAvailable: true,
      requiredJavaMajor: 17
    }
  ],
  loadMinecraftVersions: vi.fn(),
  validateProvisioning: vi.fn(),
  provisionServer: vi.fn()
}));

vi.mock("../store/appStore", () => ({
  useAppStore: () => mockStore
}));

describe("ProvisionWizard", () => {
  beforeEach(() => {
    mockStore.loadMinecraftVersions.mockReset();
    mockStore.validateProvisioning.mockReset();
    mockStore.provisionServer.mockReset();
    mockStore.validateProvisioning.mockResolvedValue({
      normalizedTargetDirectory: "/tmp/servers/primary-survival",
      issues: []
    });
  });

  it("renders the catalog-backed version selector and compatibility messaging", async () => {
    render(<ProvisionWizard />);

    expect(screen.getByText("1.21.4")).toBeInTheDocument();
    expect(screen.getByText("1.20.6")).toBeInTheDocument();
    expect(
      screen.getByText(/temurin 21\.0\.6 satisfies the java 21\+ requirement/i)
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /1\.20\.6/i }));

    await waitFor(() => {
      expect(
        screen.getByText(/temurin 21\.0\.6 satisfies the java 17\+ requirement/i)
      ).toBeInTheDocument();
    });
  });

  it("shows validation issues on the matching step", async () => {
    mockStore.validateProvisioning.mockResolvedValue({
      normalizedTargetDirectory: "/tmp/servers/primary-survival",
      issues: [
        {
          field: "port",
          step: "details",
          message: "Port 25565 is already assigned to another managed server."
        }
      ]
    });

    render(<ProvisionWizard />);
    fireEvent.click(screen.getByRole("button", { name: /details/i }));

    await waitFor(() => {
      expect(
        screen.getByText(/port 25565 is already assigned to another managed server/i)
      ).toBeInTheDocument();
    });
  });

  it("submits arbitrary server properties in the provisioning request", async () => {
    render(<ProvisionWizard />);

    fireEvent.click(screen.getByRole("button", { name: /properties/i }));
    fireEvent.click(screen.getByRole("button", { name: /add property/i }));
    const keyInputs = screen.getAllByPlaceholderText("property-key");
    const valueInputs = screen.getAllByPlaceholderText("value");
    fireEvent.change(keyInputs[keyInputs.length - 1], {
      target: { value: "spawn-protection" }
    });
    fireEvent.change(valueInputs[valueInputs.length - 1], {
      target: { value: "0" }
    });

    fireEvent.click(screen.getByRole("button", { name: /review/i }));
    fireEvent.click(screen.getByRole("button", { name: /provision server/i }));

    await waitFor(() => {
      expect(mockStore.provisionServer).toHaveBeenCalledWith(
        expect.objectContaining({
          minecraftVersion: "1.21.4",
          javaRuntimeId: "java-21",
          targetDirectory: "/tmp/servers/primary-survival",
          serverProperties: expect.objectContaining({
            motd: "Primary Survival",
            "server-port": "25565",
            "spawn-protection": "0"
          })
        })
      );
    });
  });
});

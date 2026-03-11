import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { UpdateNotice } from "./UpdateNotice";

describe("UpdateNotice", () => {
  it("renders a global update notice when a release is available", () => {
    const onOpenSettings = vi.fn();
    render(
      <UpdateNotice
        updates={{
          currentVersion: "0.1.0",
          channel: "stable",
          lastCheckedAt: new Date().toISOString(),
          updateAvailable: true,
          availableRelease: {
            version: "0.1.1",
            notes: "Release notes",
            publishedAt: new Date().toISOString(),
            downloadReady: true,
            installReady: true
          },
          installState: "ready",
          error: null
        }}
        onOpenSettings={onOpenSettings}
      />
    );

    expect(screen.getByText(/msms 0\.1\.1 is available on the stable channel/i)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /open updater/i }));
    expect(onOpenSettings).toHaveBeenCalled();
  });
});

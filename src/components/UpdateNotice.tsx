import type { UpdaterStatus } from "../types/api";

export function UpdateNotice({
  updates,
  onOpenSettings
}: {
  updates: UpdaterStatus | null;
  onOpenSettings: () => void;
}) {
  if (!updates) {
    return null;
  }

  const hasNotice =
    updates.updateAvailable ||
    updates.installState === "restartRequired" ||
    updates.installState === "error" ||
    Boolean(updates.error);

  if (!hasNotice) {
    return null;
  }

  const message = updates.installState === "restartRequired"
    ? `MSMS ${updates.availableRelease?.version ?? ""} has been installed. Restart the app to finish applying it.`
    : updates.error
      ? updates.error
      : `MSMS ${updates.availableRelease?.version ?? ""} is available on the ${updates.channel} channel.`;

  return (
    <section className="panel update-notice">
      <div>
        <p className="eyebrow">Updater</p>
        <h2>Release attention required</h2>
        <p className="muted">{message}</p>
      </div>
      <button onClick={onOpenSettings} type="button">
        Open updater
      </button>
    </section>
  );
}

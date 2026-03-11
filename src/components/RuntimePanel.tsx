import { useState, type FormEvent } from "react";
import { useAppStore } from "../store/appStore";

export function RuntimePanel() {
  const { runtimes, installJavaRuntime } = useAppStore();
  const [vendor, setVendor] = useState("Temurin");
  const [version, setVersion] = useState("21.0.6");
  const [downloadUrl, setDownloadUrl] = useState("");
  const [archiveKind, setArchiveKind] = useState<"zip" | "tar.gz">("tar.gz");

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await installJavaRuntime({
      vendor,
      version,
      downloadUrl,
      archiveKind
    });
  }

  return (
    <section className="panel split-panel">
      <div>
        <div className="section-heading">
          <div>
            <p className="eyebrow">Runtimes</p>
            <h2>Java Discovery And Installation</h2>
          </div>
        </div>
        <div className="stack">
          {runtimes.map((runtime) => (
            <article key={runtime.id} className="list-card">
              <div>
                <strong>
                  {runtime.vendor} {runtime.version}
                </strong>
                <p className="muted">{runtime.installPath}</p>
              </div>
              <p>
                {runtime.architecture} · {runtime.managedByApp ? "Managed by MSMS" : "Detected"}
              </p>
            </article>
          ))}
        </div>
      </div>
      <div>
        <div className="section-heading">
          <div>
            <p className="eyebrow">Install Runtime</p>
            <h2>Register A Managed JDK</h2>
          </div>
        </div>
        <form className="form-grid" onSubmit={handleSubmit}>
          <label>
            Vendor
            <input value={vendor} onChange={(event) => setVendor(event.target.value)} />
          </label>
          <label>
            Version
            <input value={version} onChange={(event) => setVersion(event.target.value)} />
          </label>
          <label>
            Download URL
            <input
              placeholder="https://example.com/jdk.tar.gz"
              required
              value={downloadUrl}
              onChange={(event) => setDownloadUrl(event.target.value)}
            />
          </label>
          <label>
            Archive type
            <select
              value={archiveKind}
              onChange={(event) => setArchiveKind(event.target.value as "zip" | "tar.gz")}
            >
              <option value="tar.gz">tar.gz</option>
              <option value="zip">zip</option>
            </select>
          </label>
          <div className="form-actions">
            <button type="submit">Install Runtime</button>
          </div>
        </form>
      </div>
    </section>
  );
}


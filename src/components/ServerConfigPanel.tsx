import { useEffect, useState, type FormEvent } from "react";
import { useAppStore } from "../store/appStore";

const defaults = {
  motd: "",
  "server-port": "25565",
  difficulty: "normal",
  "max-players": "20",
  "online-mode": "true",
  pvp: "true"
};

export function ServerConfigPanel() {
  const { servers, serverProperties, loadServerProperties, updateServerProperties } = useAppStore();
  const [serverId, setServerId] = useState(servers[0]?.id ?? "");
  const [formValues, setFormValues] = useState(defaults);

  useEffect(() => {
    if (!serverId && servers[0]) {
      setServerId(servers[0].id);
      return;
    }
    if (serverId) {
      void loadServerProperties(serverId);
    }
    // Store actions are recreated per render; keep this effect keyed to selection changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [serverId, servers]);

  useEffect(() => {
    if (!serverProperties) {
      return;
    }
    setFormValues({
      motd: serverProperties.motd ?? defaults.motd,
      "server-port": serverProperties["server-port"] ?? defaults["server-port"],
      difficulty: serverProperties.difficulty ?? defaults.difficulty,
      "max-players": serverProperties["max-players"] ?? defaults["max-players"],
      "online-mode": serverProperties["online-mode"] ?? defaults["online-mode"],
      pvp: serverProperties.pvp ?? defaults.pvp
    });
  }, [serverProperties]);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!serverId) {
      return;
    }
    await updateServerProperties({
      serverId,
      properties: {
        motd: formValues.motd,
        "server-port": formValues["server-port"],
        difficulty: formValues.difficulty,
        "max-players": formValues["max-players"],
        "online-mode": formValues["online-mode"],
        pvp: formValues.pvp
      }
    });
  }

  return (
    <section className="panel split-panel">
      <div>
        <div className="section-heading">
          <div>
            <p className="eyebrow">Server Config</p>
            <h2>Common `server.properties` Controls</h2>
          </div>
        </div>
        <form className="form-grid" onSubmit={handleSubmit}>
          <label>
            Server
            <select value={serverId} onChange={(event) => setServerId(event.target.value)}>
              {servers.map((server) => (
                <option key={server.id} value={server.id}>
                  {server.name}
                </option>
              ))}
            </select>
          </label>
          <label>
            MOTD
            <input
              value={formValues.motd}
              onChange={(event) =>
                setFormValues((current) => ({ ...current, motd: event.target.value }))
              }
            />
          </label>
          <label>
            Server port
            <input
              value={formValues["server-port"]}
              onChange={(event) =>
                setFormValues((current) => ({
                  ...current,
                  "server-port": event.target.value
                }))
              }
            />
          </label>
          <label>
            Difficulty
            <select
              value={formValues.difficulty}
              onChange={(event) =>
                setFormValues((current) => ({ ...current, difficulty: event.target.value }))
              }
            >
              <option value="peaceful">peaceful</option>
              <option value="easy">easy</option>
              <option value="normal">normal</option>
              <option value="hard">hard</option>
            </select>
          </label>
          <label>
            Max players
            <input
              value={formValues["max-players"]}
              onChange={(event) =>
                setFormValues((current) => ({
                  ...current,
                  "max-players": event.target.value
                }))
              }
            />
          </label>
          <label>
            Online mode
            <select
              value={formValues["online-mode"]}
              onChange={(event) =>
                setFormValues((current) => ({
                  ...current,
                  "online-mode": event.target.value
                }))
              }
            >
              <option value="true">true</option>
              <option value="false">false</option>
            </select>
          </label>
          <label>
            PvP
            <select
              value={formValues.pvp}
              onChange={(event) =>
                setFormValues((current) => ({ ...current, pvp: event.target.value }))
              }
            >
              <option value="true">true</option>
              <option value="false">false</option>
            </select>
          </label>
          <div className="form-actions">
            <button type="submit">Save Properties</button>
          </div>
        </form>
      </div>
      <div className="panel inset-panel">
        <p className="eyebrow">Preview</p>
        <h3>Current Values</h3>
        <div className="stack">
          {Object.entries(formValues).map(([key, value]) => (
            <article key={key} className="list-card">
              <strong>{key}</strong>
              <p>{value}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}

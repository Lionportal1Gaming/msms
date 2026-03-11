import type { ManagedServer } from "../types/api";

interface ServerTableProps {
  servers: ManagedServer[];
  onStart: (serverId: string) => void;
  onStop: (serverId: string) => void;
  onRestart: (serverId: string) => void;
  onKill: (serverId: string) => void;
  onOpenConsole: (serverId: string) => void;
}

export function ServerTable({
  servers,
  onStart,
  onStop,
  onRestart,
  onKill,
  onOpenConsole
}: ServerTableProps) {
  return (
    <section className="panel">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Managed Servers</p>
          <h2>Lifecycle Control</h2>
        </div>
      </div>
      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Version</th>
              <th>Status</th>
              <th>Port</th>
              <th>Memory</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {servers.map((server) => (
              <tr key={server.id}>
                <td>{server.name}</td>
                <td>{server.minecraftVersion}</td>
                <td>
                  <span className={`status-pill ${server.status}`}>{server.status}</span>
                </td>
                <td>{server.port}</td>
                <td>{server.memoryMb} MB</td>
                <td className="table-actions">
                  <button onClick={() => onStart(server.id)} type="button">Start</button>
                  <button onClick={() => onStop(server.id)} type="button">Stop</button>
                  <button onClick={() => onRestart(server.id)} type="button">Restart</button>
                  <button onClick={() => onKill(server.id)} type="button">Kill</button>
                  <button onClick={() => onOpenConsole(server.id)} type="button">Console</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}


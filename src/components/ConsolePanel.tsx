import { useEffect, useMemo, useState, type KeyboardEvent } from "react";
import { useAppStore } from "../store/appStore";
import type { ManagedServer } from "../types/api";

interface ConsolePanelProps {
  server: ManagedServer | undefined;
}

export function ConsolePanel({ server }: ConsolePanelProps) {
  const {
    consoleEntries,
    consoleHistory,
    commandHistory,
    loadConsole,
    loadConsoleHistory,
    loadCommandHistory,
    sendServerCommand,
    refresh
  } = useAppStore();
  const [commandInput, setCommandInput] = useState("");
  const [historyIndex, setHistoryIndex] = useState<number | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  useEffect(() => {
    if (!server) {
      return;
    }

    void loadConsoleHistory(server.id);
    void loadCommandHistory(server.id);
    void loadConsole(server.id);

    const interval = window.setInterval(() => {
      void loadConsole(server.id);
      void refresh();
    }, 1000);

    return () => window.clearInterval(interval);
    // Store actions are recreated per render; keep polling bound to the selected server.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [server?.id]);

  const visibleEntries = useMemo(
    () => [...consoleHistory, ...consoleEntries],
    [consoleEntries, consoleHistory]
  );
  const commandRecall = useMemo(() => [...commandHistory].reverse(), [commandHistory]);
  const isRunning = server?.status === "running";

  async function handleSubmit() {
    if (!server || !commandInput.trim() || !isRunning) {
      return;
    }

    await sendServerCommand({
      serverId: server.id,
      command: commandInput.trim()
    });
    setFeedback(`Command submitted to ${server.name}.`);
    setCommandInput("");
    setHistoryIndex(null);
  }

  function handleKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === "Enter") {
      event.preventDefault();
      void handleSubmit();
      return;
    }

    if (commandRecall.length === 0) {
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      const nextIndex =
        historyIndex === null ? 0 : Math.min(historyIndex + 1, commandRecall.length - 1);
      setHistoryIndex(nextIndex);
      setCommandInput(commandRecall[nextIndex].command);
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      if (historyIndex === null) {
        return;
      }

      const nextIndex = historyIndex - 1;
      if (nextIndex < 0) {
        setHistoryIndex(null);
        setCommandInput("");
        return;
      }

      setHistoryIndex(nextIndex);
      setCommandInput(commandRecall[nextIndex].command);
    }
  }

  return (
    <section className="panel console-panel">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Console</p>
          <h2>{server ? server.name : "Select A Server"}</h2>
          <p className="muted">
            {server ? `Status: ${server.status}` : "Select a managed server to inspect live operations."}
          </p>
        </div>
      </div>
      <div className="console-entry-list">
        {visibleEntries.length > 0 ? (
          visibleEntries.map((entry, index) => (
            <div key={`${entry.timestamp}-${index}`} className={`console-entry ${entry.source}`}>
              <span className="console-meta">
                [{new Date(entry.timestamp).toLocaleTimeString()}] {entry.source}
              </span>
              <span>{entry.message}</span>
            </div>
          ))
        ) : (
          <p className="muted">Server console output will appear here.</p>
        )}
      </div>
      <div className="form-grid">
        <label>
          Send command
          <input
            disabled={!isRunning}
            placeholder={
              isRunning
                ? "Type a Minecraft server command and press Enter"
                : "Start the server to enable commands"
            }
            value={commandInput}
            onChange={(event) => setCommandInput(event.target.value)}
            onKeyDown={handleKeyDown}
          />
        </label>
        <div className="form-actions">
          <button
            disabled={!isRunning || !commandInput.trim()}
            onClick={() => void handleSubmit()}
            type="button"
          >
            Send Command
          </button>
        </div>
        {feedback ? <p className="muted">{feedback}</p> : null}
      </div>
    </section>
  );
}

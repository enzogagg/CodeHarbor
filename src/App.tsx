import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type CommandName = "check_docker" | "start_prototype" | "stop_prototype" | "open_ide";

const actions: Array<{ command: CommandName; label: string; kind: "primary" | "secondary" }> = [
  { command: "check_docker", label: "Vérifier Docker", kind: "secondary" },
  { command: "start_prototype", label: "Démarrer", kind: "primary" },
  { command: "stop_prototype", label: "Arrêter", kind: "secondary" },
  { command: "open_ide", label: "Ouvrir IDE", kind: "secondary" },
];

function App() {
  const [busyCommand, setBusyCommand] = useState<CommandName | null>(null);
  const [message, setMessage] = useState("Prototype prêt. Vérifie Docker ou démarre le workspace.");
  const [error, setError] = useState<string | null>(null);

  async function runCommand(command: CommandName) {
    setBusyCommand(command);
    setError(null);
    setMessage("Commande en cours...");

    try {
      const response = await invoke<string>(command);
      setMessage(response);
    } catch (caught) {
      setError(String(caught));
      setMessage("Action interrompue.");
    } finally {
      setBusyCommand(null);
    }
  }

  return (
    <main className="app-shell">
      <section className="hero-panel" aria-labelledby="app-title">
        <div className="brand-mark" aria-hidden="true">
          CH
        </div>

        <div className="hero-copy">
          <p className="eyebrow">Local container harbor</p>
          <h1 id="app-title">CodeHarbor</h1>
          <p className="lede">
            Lance un environnement Ubuntu AMD64 isolé avec code-server, monté sur ton Mac et piloté depuis une app desktop.
          </p>
        </div>
      </section>

      <section className="workspace-card" aria-labelledby="workspace-title">
        <div className="card-header">
          <div>
            <p className="card-kicker">Workspace prototype</p>
            <h2 id="workspace-title">Ubuntu AMD64 Workspace</h2>
          </div>
          <span className="status-pill">Ubuntu 24.04 · x86_64</span>
        </div>

        <div className="signal-grid" aria-label="Workspace details">
          <div>
            <span>IDE</span>
            <strong>code-server</strong>
          </div>
          <div>
            <span>Port</span>
            <strong>8080</strong>
          </div>
          <div>
            <span>Mount</span>
            <strong>/workspace</strong>
          </div>
        </div>

        <div className="actions" aria-label="Workspace actions">
          {actions.map((action) => (
            <button
              className={`action-button ${action.kind}`}
              disabled={busyCommand !== null}
              key={action.command}
              onClick={() => runCommand(action.command)}
              type="button"
            >
              {busyCommand === action.command ? "En cours..." : action.label}
            </button>
          ))}
        </div>

        <div className={error ? "console-message error" : "console-message"} role="status">
          <span className="console-prompt">codeharbor</span>
          <p>{error ?? message}</p>
        </div>
      </section>
    </main>
  );
}

export default App;

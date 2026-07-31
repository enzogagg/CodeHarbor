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
    <main className="app-frame">
      <aside className="sidebar" aria-label="CodeHarbor navigation">
        <div className="sidebar-brand">
          <div className="app-icon" aria-hidden="true">
            C
          </div>
          <div>
            <h1>CodeHarbor</h1>
            <p>Docker workspaces</p>
          </div>
        </div>

        <div className="sidebar-section">
          <p className="sidebar-label">Status</p>
          <div className="status-row">
            <span className="status-dot" aria-hidden="true" />
            <span>Local Docker</span>
          </div>
        </div>

        <div className="sidebar-section">
          <p className="sidebar-label">Workspaces</p>
          <button className="workspace-nav-item active" type="button">
            <span className="workspace-glyph" aria-hidden="true">▣</span>
            <span>
              <strong>Ubuntu AMD64</strong>
              <small>code-server · 8080</small>
            </span>
          </button>
        </div>
      </aside>

      <section className="content" aria-labelledby="workspace-title">
        <header className="topbar">
          <div>
            <p className="breadcrumb">Prototype workspace</p>
            <h2 id="workspace-title">Ubuntu AMD64 Workspace</h2>
            <p className="summary">
              Environnement Ubuntu 24.04 x86_64 avec code-server, monté dans <code>/workspace</code>.
            </p>
          </div>

          <div className="topbar-actions" aria-label="Workspace actions">
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
        </header>

        <section className="workspace-panel" aria-label="Workspace details">
          <div className="panel-header">
            <div>
              <h3>Configuration</h3>
              <p>Paramètres du prototype local utilisé par CodeHarbor.</p>
            </div>
            <span className="state-badge">Ready</span>
          </div>

          <dl className="detail-list">
            <div>
              <dt>Base image</dt>
              <dd>Ubuntu 24.04</dd>
            </div>
            <div>
              <dt>Architecture</dt>
              <dd>x86_64 / linux amd64</dd>
            </div>
            <div>
              <dt>IDE URL</dt>
              <dd>http://localhost:8080</dd>
            </div>
            <div>
              <dt>Workspace mount</dt>
              <dd>/workspace</dd>
            </div>
          </dl>
        </section>

        <section className={error ? "output-panel error" : "output-panel"} aria-label="Command output">
          <div className="output-titlebar">
            <span>Output</span>
            <code>codeharbor</code>
          </div>
          <p role="status">{error ?? message}</p>
        </section>
      </section>
    </main>
  );
}

export default App;

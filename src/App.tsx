import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type EnvironmentConfig = {
  id: string;
  name: string;
  profile: string;
  host_path: string;
  container_path: string;
  ide_port: number;
  created_at: number;
};

type CommandName =
  | "start_environment"
  | "stop_environment"
  | "open_environment_ide"
  | "run_environment_build"
  | "run_environment_tests"
  | "run_environment_valgrind"
  | "run_environment_clean"
  | "check_docker";

const actions: Array<{ command: CommandName; label: string; kind: "primary" | "secondary"; needsEnvironment: boolean }> = [
  { command: "check_docker", label: "Vérifier Docker", kind: "secondary", needsEnvironment: false },
  { command: "start_environment", label: "Démarrer", kind: "primary", needsEnvironment: true },
  { command: "stop_environment", label: "Arrêter", kind: "secondary", needsEnvironment: true },
  { command: "open_environment_ide", label: "Ouvrir IDE", kind: "secondary", needsEnvironment: true },
  { command: "run_environment_build", label: "Build", kind: "secondary", needsEnvironment: true },
  { command: "run_environment_tests", label: "Tests", kind: "secondary", needsEnvironment: true },
  { command: "run_environment_valgrind", label: "Valgrind", kind: "secondary", needsEnvironment: true },
  { command: "run_environment_clean", label: "Clean", kind: "secondary", needsEnvironment: true },
];

function App() {
  const [environments, setEnvironments] = useState<EnvironmentConfig[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [busyCommand, setBusyCommand] = useState<CommandName | "create_environment" | null>(null);
  const [message, setMessage] = useState("Crée ou sélectionne un environnement pour monter un projet Mac dans Ubuntu.");
  const [error, setError] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [hostPath, setHostPath] = useState("");
  const [githubUrl, setGithubUrl] = useState("");

  const selectedEnvironment = environments.find((environment) => environment.id === selectedId) ?? environments[0] ?? null;

  async function refreshEnvironments() {
    const list = await invoke<EnvironmentConfig[]>("list_environments");
    setEnvironments(list);
    setSelectedId((current) => current ?? list[0]?.id ?? null);
  }

  useEffect(() => {
    refreshEnvironments().catch((caught) => setError(String(caught)));
  }, []);

  async function createEnvironment() {
    setBusyCommand("create_environment");
    setError(null);
    setMessage("Création de l'environnement...");

    try {
      const created = await invoke<EnvironmentConfig>("create_environment", {
        name,
        hostPath,
        githubUrl,
      });
      setName("");
      setHostPath("");
      setGithubUrl("");
      await refreshEnvironments();
      setSelectedId(created.id);
      setMessage(`${created.name} créé. Le dossier Mac est monté dans /workspace.`);
    } catch (caught) {
      setError(String(caught));
      setMessage("Création interrompue.");
    } finally {
      setBusyCommand(null);
    }
  }

  async function runCommand(command: CommandName) {
    if (command !== "check_docker" && !selectedEnvironment) {
      setError("Sélectionne ou crée un environnement avant de lancer cette action.");
      return;
    }

    setBusyCommand(command);
    setError(null);
    setMessage("Commande en cours...");

    try {
      const args = command === "check_docker" ? undefined : { environmentId: selectedEnvironment?.id };
      const response = await invoke<string>(command, args);
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
          <div className="app-icon" aria-hidden="true">C</div>
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
          {environments.length === 0 ? (
            <p className="empty-sidebar">Aucun environnement.</p>
          ) : (
            environments.map((environment) => (
              <button
                className={`workspace-nav-item ${selectedEnvironment?.id === environment.id ? "active" : ""}`}
                key={environment.id}
                onClick={() => setSelectedId(environment.id)}
                type="button"
              >
                <span className="workspace-glyph" aria-hidden="true">▣</span>
                <span>
                  <strong>{environment.name}</strong>
                  <small>{environment.profile} · {environment.ide_port}</small>
                </span>
              </button>
            ))
          )}
        </div>

        <form className="environment-form" onSubmit={(event) => { event.preventDefault(); createEnvironment(); }}>
          <p className="sidebar-label">Add environment</p>
          <label>
            Name
            <input value={name} onChange={(event) => setName(event.target.value)} placeholder="MyFTP" />
          </label>
          <label>
            Local folder path
            <input value={hostPath} onChange={(event) => setHostPath(event.target.value)} placeholder="/Users/me/Dev/myftp" />
          </label>
          <label>
            Git URL optional
            <input value={githubUrl} onChange={(event) => setGithubUrl(event.target.value)} placeholder="git@github.com:org/repo.git" />
          </label>
          <button className="create-button" disabled={busyCommand !== null || !name.trim()} type="submit">
            {busyCommand === "create_environment" ? "Création..." : "Create"}
          </button>
        </form>
      </aside>

      <section className="content" aria-labelledby="workspace-title">
        <header className="topbar">
          <div>
            <p className="breadcrumb">Evaluation profile</p>
            <h2 id="workspace-title">{selectedEnvironment?.name ?? "No environment selected"}</h2>
            <p className="summary">
              {selectedEnvironment
                ? <>Ton dossier Mac <code>{selectedEnvironment.host_path}</code> est monté dans <code>/workspace</code>.</>
                : "Crée un environnement depuis un dossier local ou une URL Git pour compiler dans Ubuntu AMD64."}
            </p>
          </div>

          <div className="topbar-actions" aria-label="Workspace actions">
            {actions.map((action) => (
              <button
                className={`action-button ${action.kind}`}
                disabled={busyCommand !== null || (action.needsEnvironment && !selectedEnvironment)}
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
              <p>Sync directe Mac → Docker par volume monté.</p>
            </div>
            <span className="state-badge">Ready</span>
          </div>

          <dl className="detail-list">
            <div><dt>Profile</dt><dd>{selectedEnvironment?.profile ?? "epitech-cpp"}</dd></div>
            <div><dt>Base image</dt><dd>Ubuntu 24.04</dd></div>
            <div><dt>Architecture</dt><dd>x86_64 / linux amd64</dd></div>
            <div><dt>IDE URL</dt><dd>{selectedEnvironment ? `http://localhost:${selectedEnvironment.ide_port}` : "Not created"}</dd></div>
            <div><dt>Host folder</dt><dd>{selectedEnvironment?.host_path ?? "Select a local project"}</dd></div>
            <div><dt>Container mount</dt><dd>/workspace</dd></div>
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

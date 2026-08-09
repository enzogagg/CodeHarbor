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

type EnvironmentRuntimeStatus = {
  environment_id: string;
  status: "running" | "stopped" | "not_created" | string;
  container_name: string;
};

type EvaluationRunRecord = {
  id: string;
  command: string;
  label: string;
  started_at: number;
  duration_ms: number;
  success: boolean;
  stdout: string;
  stderr: string;
};

type ProjectInspection = {
  has_makefile: boolean;
  make_targets: string[];
  language_counts: Record<string, number>;
  executables: string[];
  artifacts: string[];
};

type ReportFile = {
  name: string;
  path: string;
  created_at: number;
  size_bytes: number;
};

type CommandName =
  | "start_environment"
  | "stop_environment"
  | "delete_environment"
  | "open_environment_ide"
  | "open_environment_folder"
  | "run_environment_build"
  | "run_environment_tests"
  | "run_environment_valgrind"
  | "run_environment_valgrind_target"
  | "run_environment_clean"
  | "run_diagnostics"
  | "check_docker";

type Action = { command: CommandName; label: string; kind: "primary" | "secondary" | "destructive"; needsEnvironment: boolean };

const actionGroups: Array<{ title: string; actions: Action[] }> = [
  {
    title: "Lifecycle",
    actions: [
      { command: "start_environment", label: "Démarrer", kind: "primary", needsEnvironment: true },
      { command: "stop_environment", label: "Arrêter", kind: "secondary", needsEnvironment: true },
      { command: "open_environment_ide", label: "Ouvrir IDE", kind: "secondary", needsEnvironment: true },
    ],
  },
  {
    title: "Evaluation",
    actions: [
      { command: "run_environment_build", label: "Build", kind: "secondary", needsEnvironment: true },
      { command: "run_environment_tests", label: "Tests", kind: "secondary", needsEnvironment: true },
      { command: "run_environment_valgrind", label: "Valgrind", kind: "secondary", needsEnvironment: true },
      { command: "run_environment_clean", label: "Clean", kind: "secondary", needsEnvironment: true },
    ],
  },
  {
    title: "Tools",
    actions: [
      { command: "open_environment_folder", label: "Finder", kind: "secondary", needsEnvironment: true },
      { command: "check_docker", label: "Docker", kind: "secondary", needsEnvironment: false },
      { command: "run_diagnostics", label: "Diagnostics", kind: "secondary", needsEnvironment: false },
    ],
  },
  {
    title: "Danger",
    actions: [
      { command: "delete_environment", label: "Supprimer", kind: "destructive", needsEnvironment: true },
    ],
  },
];

const statusLabels: Record<string, string> = {
  running: "Running",
  stopped: "Stopped",
  not_created: "Not created",
};

function App() {
  const [environments, setEnvironments] = useState<EnvironmentConfig[]>([]);
  const [runtimeStatuses, setRuntimeStatuses] = useState<Record<string, EnvironmentRuntimeStatus>>({});
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [busyCommand, setBusyCommand] = useState<CommandName | "create_environment" | "generate_evaluation_report" | "open_report_file" | "open_report_folder" | null>(null);
  const [message, setMessage] = useState("Crée ou sélectionne un environnement pour monter un projet Mac dans Ubuntu.");
  const [error, setError] = useState<string | null>(null);
  const [pendingDeleteId, setPendingDeleteId] = useState<string | null>(null);
  const [history, setHistory] = useState<EvaluationRunRecord[]>([]);
  const [inspection, setInspection] = useState<ProjectInspection | null>(null);
  const [reports, setReports] = useState<ReportFile[]>([]);
  const [selectedValgrindTarget, setSelectedValgrindTarget] = useState("");
  const [dockerText, setDockerText] = useState("");
  const [name, setName] = useState("");
  const [hostPath, setHostPath] = useState("");
  const [githubUrl, setGithubUrl] = useState("");

  const selectedEnvironment = environments.find((environment) => environment.id === selectedId) ?? environments[0] ?? null;

  async function refreshEnvironments() {
    const list = await invoke<EnvironmentConfig[]>("list_environments");
    const statuses = await invoke<EnvironmentRuntimeStatus[]>("list_environment_statuses");
    setEnvironments(list);
    setRuntimeStatuses(Object.fromEntries(statuses.map((status) => [status.environment_id, status])));
    setSelectedId((current) => current ?? list[0]?.id ?? null);
  }

  async function refreshEvaluation(environmentId = selectedEnvironment?.id) {
    if (!environmentId) {
      setHistory([]);
      setInspection(null);
      setReports([]);
      setSelectedValgrindTarget("");
      return;
    }

    const [nextHistory, nextInspection, nextReports] = await Promise.all([
      invoke<EvaluationRunRecord[]>("list_evaluation_history", { environmentId }),
      invoke<ProjectInspection>("inspect_project", { environmentId }),
      invoke<ReportFile[]>("list_evaluation_reports", { environmentId }),
    ]);
    setHistory(nextHistory);
    setInspection(nextInspection);
    setReports(nextReports);
    setSelectedValgrindTarget((current) => current || nextInspection.executables[0] || "");
  }

  useEffect(() => {
    refreshEnvironments().catch((caught) => setError(String(caught)));
  }, []);

  useEffect(() => {
    refreshEvaluation().catch((caught) => setError(String(caught)));
  }, [selectedEnvironment?.id]);

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

  async function deleteEnvironment() {
    if (!selectedEnvironment) {
      setError("Sélectionne un environnement avant de le supprimer.");
      return;
    }

    if (pendingDeleteId !== selectedEnvironment.id) {
      setPendingDeleteId(selectedEnvironment.id);
      setError(null);
      setMessage(`Confirme la suppression de ${selectedEnvironment.name}. Les fichiers projet seront conservés.`);
      return;
    }

    const deletedId = selectedEnvironment.id;
    setBusyCommand("delete_environment");
    setError(null);
    setMessage("Suppression de l'environnement...");

    try {
      const response = await invoke<string>("delete_environment", { environmentId: deletedId });
      const list = await invoke<EnvironmentConfig[]>("list_environments");
      setEnvironments(list);
      setSelectedId(list[0]?.id ?? null);
      setPendingDeleteId(null);
      setMessage(response);
    } catch (caught) {
      setPendingDeleteId(null);
      setError(String(caught));
      setMessage("Suppression interrompue.");
    } finally {
      setBusyCommand(null);
    }
  }

  async function runCommand(command: CommandName) {
    const needsEnvironment = command !== "check_docker" && command !== "run_diagnostics";
    if (needsEnvironment && !selectedEnvironment) {
      setError("Sélectionne ou crée un environnement avant de lancer cette action.");
      return;
    }

    setBusyCommand(command);
    setError(null);
    setMessage("Commande en cours...");

    try {
      const args = needsEnvironment ? { environmentId: selectedEnvironment?.id } : undefined;
      const response = await invoke<string>(command, args);
      await refreshEnvironments();
      await refreshEvaluation();
      setMessage(response);
    } catch (caught) {
      setError(String(caught));
      setMessage("Action interrompue.");
    } finally {
      setBusyCommand(null);
    }
  }

  async function runValgrindTarget() {
    if (!selectedEnvironment || !selectedValgrindTarget) {
      setError("Sélectionne un binaire Valgrind.");
      return;
    }

    setBusyCommand("run_environment_valgrind_target");
    setError(null);
    setMessage("Valgrind en cours...");

    try {
      const response = await invoke<string>("run_environment_valgrind_target", {
        environmentId: selectedEnvironment.id,
        targetPath: selectedValgrindTarget,
      });
      await refreshEvaluation(selectedEnvironment.id);
      setMessage(response);
    } catch (caught) {
      setError(String(caught));
      setMessage("Valgrind interrompu.");
    } finally {
      setBusyCommand(null);
    }
  }

  async function loadDockerText(command: "show_environment_docker_logs" | "show_environment_compose_config") {
    if (!selectedEnvironment) {
      return;
    }
    setError(null);
    setDockerText("Chargement...");
    try {
      setDockerText(await invoke<string>(command, { environmentId: selectedEnvironment.id }));
    } catch (caught) {
      setDockerText(String(caught));
    }
  }

  async function generateReport() {
    if (!selectedEnvironment) {
      setError("Sélectionne un environnement avant de générer un rapport.");
      return;
    }

    setBusyCommand("generate_evaluation_report");
    setError(null);
    setMessage("Génération du rapport...");

    try {
      const report = await invoke<ReportFile>("generate_evaluation_report", { environmentId: selectedEnvironment.id });
      await refreshEvaluation(selectedEnvironment.id);
      setMessage(`Rapport généré: ${report.name}`);
    } catch (caught) {
      setError(String(caught));
      setMessage("Export interrompu.");
    } finally {
      setBusyCommand(null);
    }
  }

  async function openReport(reportName: string) {
    if (!selectedEnvironment) {
      return;
    }

    setBusyCommand("open_report_file");
    setError(null);
    try {
      await invoke("open_report_file", { environmentId: selectedEnvironment.id, reportName });
      setMessage(`Rapport ouvert: ${reportName}`);
    } catch (caught) {
      setError(String(caught));
    } finally {
      setBusyCommand(null);
    }
  }

  async function openReportFolder() {
    if (!selectedEnvironment) {
      return;
    }

    setBusyCommand("open_report_folder");
    setError(null);
    try {
      await invoke("open_report_folder", { environmentId: selectedEnvironment.id });
      setMessage("Dossier des rapports ouvert.");
    } catch (caught) {
      setError(String(caught));
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
                onClick={() => { setSelectedId(environment.id); setPendingDeleteId(null); }}
                type="button"
              >
                <span className="workspace-glyph" aria-hidden="true">▣</span>
                <span>
                  <strong>{environment.name}</strong>
                  <small>{environment.profile} · {environment.ide_port} · {statusLabels[runtimeStatuses[environment.id]?.status ?? "not_created"]}</small>
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
            {actionGroups.map((group) => (
              <div className="action-group" key={group.title}>
                <span className="action-group-title">{group.title}</span>
                <div className="action-row">
                  {group.actions.map((action) => (
                    <button
                      className={`action-button ${action.kind}`}
                      disabled={busyCommand !== null || (action.needsEnvironment && !selectedEnvironment)}
                      key={action.command}
                      onClick={() => action.command === "delete_environment" ? deleteEnvironment() : runCommand(action.command)}
                      type="button"
                    >
                      {busyCommand === action.command
                        ? "En cours..."
                        : action.command === "delete_environment" && pendingDeleteId === selectedEnvironment?.id
                          ? "Confirmer"
                          : action.label}
                    </button>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </header>

        <section className="workspace-panel" aria-label="Workspace details">
          <div className="panel-header">
            <div>
              <h3>Configuration</h3>
              <p>Sync directe Mac → Docker par volume monté.</p>
            </div>
            <span className={`state-badge ${runtimeStatuses[selectedEnvironment?.id ?? ""]?.status ?? "not_created"}`}>
              {statusLabels[runtimeStatuses[selectedEnvironment?.id ?? ""]?.status ?? "not_created"]}
            </span>
          </div>

          <dl className="detail-list">
            <div><dt>Profile</dt><dd>{selectedEnvironment?.profile ?? "epitech-cpp"}</dd></div>
            <div><dt>Base image</dt><dd>Ubuntu 24.04</dd></div>
            <div><dt>Architecture</dt><dd>x86_64 / linux amd64</dd></div>
            <div><dt>IDE URL</dt><dd>{selectedEnvironment ? `http://localhost:${selectedEnvironment.ide_port}` : "Not created"}</dd></div>
            <div><dt>Host folder</dt><dd>{selectedEnvironment?.host_path ?? "Select a local project"}</dd></div>
            <div><dt>Container mount</dt><dd>/workspace</dd></div>
            <div><dt>Container</dt><dd>{selectedEnvironment ? runtimeStatuses[selectedEnvironment.id]?.container_name ?? `codeharbor-${selectedEnvironment.id}` : "Not created"}</dd></div>
            <div><dt>Status</dt><dd>{statusLabels[runtimeStatuses[selectedEnvironment?.id ?? ""]?.status ?? "not_created"]}</dd></div>
          </dl>
        </section>

        <section className="evaluation-grid" aria-label="Evaluation core">
          <div className="mini-panel">
            <h3>Evaluation</h3>
            <p>Makefile: {inspection?.has_makefile ? "détecté" : "absent"}</p>
            <p>Targets: {inspection?.make_targets.join(", ") || "aucune"}</p>
            <div className="panel-actions">
              <button className="action-button secondary" disabled={!selectedEnvironment || busyCommand !== null} onClick={() => runCommand("run_environment_build")} type="button">Build</button>
              <button className="action-button secondary" disabled={!selectedEnvironment || busyCommand !== null} onClick={() => runCommand("run_environment_tests")} type="button">Tests</button>
              <button className="action-button secondary" disabled={!selectedEnvironment || busyCommand !== null} onClick={() => runCommand("run_environment_clean")} type="button">Clean</button>
            </div>
            <label className="compact-label">
              Valgrind target
              <select value={selectedValgrindTarget} onChange={(event) => setSelectedValgrindTarget(event.target.value)}>
                <option value="">Aucun binaire</option>
                {(inspection?.executables ?? []).map((binary) => <option key={binary} value={binary}>{binary}</option>)}
              </select>
            </label>
            <button className="action-button secondary" disabled={!selectedValgrindTarget || busyCommand !== null} onClick={runValgrindTarget} type="button">Run Valgrind</button>
          </div>

          <div className="mini-panel">
            <h3>History</h3>
            {history.length === 0 ? <p>Aucun run enregistré.</p> : history.slice(0, 6).map((run) => (
              <div className="history-row" key={run.id}>
                <strong>{run.command}</strong>
                <span>{run.success ? "OK" : "FAIL"} · {run.duration_ms}ms</span>
              </div>
            ))}
          </div>

          <div className="mini-panel reports-panel">
            <h3>Reports</h3>
            <p>Export Markdown local basé sur l'historique, le projet et Docker.</p>
            <div className="panel-actions">
              <button className="action-button secondary" disabled={!selectedEnvironment || busyCommand !== null} onClick={generateReport} type="button">
                {busyCommand === "generate_evaluation_report" ? "Génération..." : "Generate report"}
              </button>
              <button className="action-button secondary" disabled={!selectedEnvironment || reports.length === 0 || busyCommand !== null} onClick={() => openReport(reports[0].name)} type="button">Open latest</button>
              <button className="action-button secondary" disabled={!selectedEnvironment || busyCommand !== null} onClick={openReportFolder} type="button">Open folder</button>
            </div>
            {reports.length === 0 ? <p>Aucun rapport généré.</p> : reports.slice(0, 5).map((report) => (
              <button className="report-row" key={report.name} onClick={() => openReport(report.name)} type="button">
                <strong>{report.name}</strong>
                <span>{Math.max(1, Math.round(report.size_bytes / 1024))} KB</span>
              </button>
            ))}
          </div>

          <div className="mini-panel">
            <h3>Artifacts</h3>
            <p>Langages: {inspection ? Object.entries(inspection.language_counts).map(([name, count]) => `${name}:${count}`).join(" · ") || "aucun" : "-"}</p>
            <p>Binaires: {(inspection?.executables ?? []).join(", ") || "aucun"}</p>
            <p>Fichiers: {(inspection?.artifacts ?? []).join(", ") || "aucun"}</p>
          </div>

          <div className="mini-panel docker-panel">
            <h3>Docker</h3>
            <div className="panel-actions">
              <button className="action-button secondary" disabled={!selectedEnvironment} onClick={() => loadDockerText("show_environment_docker_logs")} type="button">Logs</button>
              <button className="action-button secondary" disabled={!selectedEnvironment} onClick={() => loadDockerText("show_environment_compose_config")} type="button">Compose config</button>
            </div>
            <pre>{dockerText || "Logs/config Docker apparaîtront ici."}</pre>
          </div>
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

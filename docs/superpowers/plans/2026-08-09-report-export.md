# Report Export Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add local Markdown evaluation report export for a CodeHarbor environment.

**Architecture:** Keep report generation backend-owned in Rust/Tauri. Reports are append-only Markdown files under `~/.codeharbor/environments/<id>/reports/`, generated from existing environment metadata, project inspection, evaluation history, Docker logs/config, and displayed in a focused React `Reports` panel.

**Tech Stack:** Rust/Tauri in `src-tauri/src/main.rs`; React/TypeScript in `src/App.tsx`; CSS in `src/App.css`; Markdown files for report persistence; verification with `cargo test`, `cargo check`, and `npm run build`.

## Global Constraints

- Report files are written under `~/.codeharbor/environments/<id>/reports/`.
- Report files are append-only and timestamped, for example `report-20260809-153000.md`.
- Report generation must fail for invalid environment IDs or missing environment metadata.
- Project inspection and Docker inspection failures must be embedded into the Markdown instead of aborting export.
- Report names accepted by `open_report_file` must reject absolute paths, path separators, and `..`.
- This batch does not add PDF export, HTML preview, grading/scoring, cloud sharing, report editing, or custom report templates.
- Large output blocks are truncated: command stdout/stderr to 4,000 characters and Docker logs/config to 8,000 characters.
- Existing verification commands must pass: `cargo test`, `cargo check`, and `npm run build`.
- Do not stage unrelated dirty worktree files.

---

## File Structure

- Modify `src-tauri/src/main.rs`: add `ReportFile`, report directory/name validation helpers, Markdown rendering helpers, report generation/list/open commands, and unit tests.
- Modify `src/App.tsx`: add `ReportFile` type, report state, refresh/generate/open handlers, and a `Reports` panel in the existing Evaluation grid.
- Modify `src/App.css`: add compact report row styles reusing existing panel design.

---

### Task 1: Report Metadata, Directory, Listing, and Name Validation

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Consumes: `environment_dir(environment_id: &str) -> Result<PathBuf, String>`, `created_at_now() -> Result<u64, String>`
- Produces: `ReportFile`, `reports_dir(environment_id: &str) -> Result<PathBuf, String>`, `report_path(environment_id: &str, report_name: &str) -> Result<PathBuf, String>`, `list_report_files(environment_id: &str) -> Result<Vec<ReportFile>, String>`

- [ ] **Step 1: Add failing Rust tests**

Add these tests inside the existing `#[cfg(test)] mod tests` block in `src-tauri/src/main.rs`:

```rust
#[test]
fn rejects_unsafe_report_names() {
    assert!(validate_report_name("report-20260809-153000.md").is_ok());
    assert!(validate_report_name("../secret.md").is_err());
    assert!(validate_report_name("nested/report.md").is_err());
    assert!(validate_report_name("/tmp/report.md").is_err());
    assert!(validate_report_name("report.txt").is_err());
}

#[test]
fn lists_report_files_newest_first() {
    let environment_id = format!("reports-list-test-{}", std::process::id());
    let env_dir = environment_dir(&environment_id).expect("resolve env dir");
    let reports = reports_dir(&environment_id).expect("resolve reports dir");
    std::fs::create_dir_all(&reports).expect("create reports dir");

    std::fs::write(reports.join("report-20260809-120000.md"), "older").expect("write older");
    std::fs::write(reports.join("report-20260809-130000.md"), "newer").expect("write newer");

    let files = list_report_files(&environment_id).expect("list reports");

    assert_eq!(files.iter().map(|file| file.name.as_str()).collect::<Vec<_>>(), vec!["report-20260809-130000.md", "report-20260809-120000.md"]);
    assert!(files.iter().all(|file| file.path.ends_with(&file.name)));
    assert!(files.iter().all(|file| file.size_bytes > 0));

    std::fs::remove_dir_all(env_dir).expect("clean env dir");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run from `CodeHarbor/src-tauri`:

```bash
cargo test report
```

Expected: FAIL because `validate_report_name`, `reports_dir`, `list_report_files`, and `ReportFile` do not exist.

- [ ] **Step 3: Implement report metadata and helpers**

Add near the existing model structs:

```rust
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ReportFile {
    name: String,
    path: String,
    created_at: u64,
    size_bytes: u64,
}
```

Add near `history_dir`:

```rust
fn reports_dir(environment_id: &str) -> Result<PathBuf, String> {
    Ok(environment_dir(environment_id)?.join("reports"))
}

fn validate_report_name(report_name: &str) -> Result<(), String> {
    let path = Path::new(report_name);
    if path.is_absolute()
        || report_name.contains("..")
        || report_name.contains('/')
        || report_name.contains('\\')
        || path.components().count() != 1
    {
        return Err("Nom de rapport invalide".into());
    }
    if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
        return Err("Le rapport doit être un fichier Markdown .md".into());
    }
    Ok(())
}

fn report_path(environment_id: &str, report_name: &str) -> Result<PathBuf, String> {
    validate_report_name(report_name)?;
    Ok(reports_dir(environment_id)?.join(report_name))
}

fn list_report_files(environment_id: &str) -> Result<Vec<ReportFile>, String> {
    let dir = reports_dir(environment_id)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut reports = Vec::new();
    for entry in fs::read_dir(&dir)
        .map_err(|error| format!("Impossible de lire {}: {error}", dir.display()))?
    {
        let entry = entry.map_err(|error| format!("Entrée de rapport invalide: {error}"))?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()).map(str::to_string) else {
            continue;
        };
        if validate_report_name(&name).is_err() {
            continue;
        }
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("Impossible de lire {}: {error}", path.display()))?;
        reports.push(ReportFile {
            name,
            path: path.to_string_lossy().into_owned(),
            created_at: metadata
                .created()
                .or_else(|_| metadata.modified())
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or(0),
            size_bytes: metadata.len(),
        });
    }

    reports.sort_by(|a, b| b.name.cmp(&a.name));
    Ok(reports)
}
```

- [ ] **Step 4: Verify targeted tests pass**

Run:

```bash
cargo test report
```

Expected: PASS.

---

### Task 2: Markdown Rendering and Report Generation

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Consumes: `EnvironmentConfig`, `ProjectInspection`, `EvaluationRunRecord`, `read_environment_config`, `detect_project`, `read_history_records`, `run_environment_compose`, `reports_dir`, `list_report_files`
- Produces: `truncate_block(input: &str, limit: usize) -> String`, `render_evaluation_report(...) -> String`, `generate_report_file(environment_id: &str) -> Result<ReportFile, String>`

- [ ] **Step 1: Add failing Rust tests for content and truncation**

Add these tests inside `#[cfg(test)] mod tests`:

```rust
#[test]
fn truncates_long_report_blocks_with_marker() {
    let input = "x".repeat(12);
    let truncated = truncate_block(&input, 5);

    assert!(truncated.starts_with("xxxxx"));
    assert!(truncated.contains("[truncated"));
}

#[test]
fn renders_required_report_sections() {
    let config = EnvironmentConfig {
        id: "report-render-test".into(),
        name: "Report Render Test".into(),
        profile: "epitech-cpp".into(),
        host_path: "/tmp/report-render-test".into(),
        container_path: "/workspace".into(),
        ide_port: 8080,
        created_at: 1,
    };
    let inspection = Ok(ProjectInspection {
        has_makefile: true,
        make_targets: vec!["all".into(), "tests_run".into()],
        language_counts: std::collections::BTreeMap::from([("c".into(), 2usize)]),
        executables: vec!["my_binary".into()],
        artifacts: vec!["coverage.gcov".into()],
    });
    let history = vec![EvaluationRunRecord {
        id: "run-1".into(),
        command: "valgrind".into(),
        label: "valgrind my_binary".into(),
        started_at: 42,
        duration_ms: 100,
        success: true,
        stdout: "ok".into(),
        stderr: "".into(),
    }];

    let markdown = render_evaluation_report(
        &config,
        100,
        inspection,
        history,
        Ok("services:\n  workspace:".into()),
        Ok("container logs".into()),
    );

    assert!(markdown.contains("# CodeHarbor Evaluation Report"));
    assert!(markdown.contains("## Summary"));
    assert!(markdown.contains("## Project"));
    assert!(markdown.contains("## Evaluation History"));
    assert!(markdown.contains("## Latest Outputs"));
    assert!(markdown.contains("## Valgrind"));
    assert!(markdown.contains("## Docker"));
    assert!(markdown.contains("## Manual Review Notes"));
    assert!(markdown.contains("not an automated grade"));
    assert!(markdown.contains("my_binary"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test truncates_long_report_blocks_with_marker renders_required_report_sections
```

Expected: FAIL because `truncate_block` and `render_evaluation_report` do not exist.

- [ ] **Step 3: Implement Markdown helpers**

Add constants and helper functions near the history/report helpers:

```rust
const COMMAND_OUTPUT_REPORT_LIMIT: usize = 4_000;
const DOCKER_REPORT_LIMIT: usize = 8_000;

fn truncate_block(input: &str, limit: usize) -> String {
    if input.chars().count() <= limit {
        return input.to_string();
    }
    let truncated = input.chars().take(limit).collect::<String>();
    format!("{truncated}\n\n[truncated: original output exceeded {limit} characters]")
}

fn markdown_list(items: &[String]) -> String {
    if items.is_empty() {
        "- none\n".into()
    } else {
        items.iter().map(|item| format!("- `{item}`\n")).collect()
    }
}

fn render_evaluation_report(
    config: &EnvironmentConfig,
    generated_at: u64,
    inspection: Result<ProjectInspection, String>,
    history: Vec<EvaluationRunRecord>,
    compose_config: Result<String, String>,
    docker_logs: Result<String, String>,
) -> String {
    let mut report = String::new();
    report.push_str("# CodeHarbor Evaluation Report\n\n");
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- Environment: {}\n", config.name));
    report.push_str(&format!("- Environment ID: `{}`\n", config.id));
    report.push_str(&format!("- Generated at: `{}`\n", generated_at));
    report.push_str(&format!("- Host path: `{}`\n", config.host_path));
    report.push_str("- Note: this report is not an automated grade; manual review is required.\n\n");

    report.push_str("## Project\n\n");
    match inspection {
        Ok(project) => {
            report.push_str(&format!("- Makefile: {}\n", if project.has_makefile { "present" } else { "missing" }));
            report.push_str("- Make targets:\n");
            report.push_str(&markdown_list(&project.make_targets));
            report.push_str("- Language counts:\n");
            if project.language_counts.is_empty() {
                report.push_str("- none\n");
            } else {
                for (language, count) in project.language_counts {
                    report.push_str(&format!("- `{language}`: {count}\n"));
                }
            }
            report.push_str("- Executables:\n");
            report.push_str(&markdown_list(&project.executables));
            report.push_str("- Artifacts:\n");
            report.push_str(&markdown_list(&project.artifacts));
        }
        Err(error) => report.push_str(&format!("Project inspection failed: `{error}`\n")),
    }
    report.push('\n');

    report.push_str("## Evaluation History\n\n");
    if history.is_empty() {
        report.push_str("No evaluation runs recorded.\n\n");
    } else {
        for run in history.iter().take(20) {
            report.push_str(&format!("- `{}` `{}`: {} in {}ms at `{}`\n", run.command, run.label, if run.success { "OK" } else { "FAIL" }, run.duration_ms, run.started_at));
        }
        report.push('\n');
    }

    report.push_str("## Latest Outputs\n\n");
    for run in history.iter().take(5) {
        report.push_str(&format!("### {} - {}\n\n", run.command, run.label));
        report.push_str("```text\n");
        report.push_str(&truncate_block(&run.stdout, COMMAND_OUTPUT_REPORT_LIMIT));
        if !run.stderr.is_empty() {
            report.push_str("\n\n[stderr]\n");
            report.push_str(&truncate_block(&run.stderr, COMMAND_OUTPUT_REPORT_LIMIT));
        }
        report.push_str("\n```\n\n");
    }
    if history.is_empty() {
        report.push_str("No command output recorded.\n\n");
    }

    report.push_str("## Valgrind\n\n");
    let valgrind_runs = history.iter().filter(|run| run.command == "valgrind").collect::<Vec<_>>();
    if valgrind_runs.is_empty() {
        report.push_str("No Valgrind runs recorded.\n\n");
    } else {
        for run in valgrind_runs.iter().take(5) {
            report.push_str(&format!("### {}\n\n", run.label));
            report.push_str(&format!("- Status: {}\n", if run.success { "OK" } else { "FAIL" }));
            report.push_str(&format!("- Duration: {}ms\n\n", run.duration_ms));
            report.push_str("```text\n");
            report.push_str(&truncate_block(&format!("{}\n{}", run.stdout, run.stderr), COMMAND_OUTPUT_REPORT_LIMIT));
            report.push_str("\n```\n\n");
        }
    }

    report.push_str("## Docker\n\n");
    report.push_str("### Compose Config\n\n```yaml\n");
    report.push_str(&truncate_block(&compose_config.unwrap_or_else(|error| format!("Docker compose config failed: {error}")), DOCKER_REPORT_LIMIT));
    report.push_str("\n```\n\n### Recent Logs\n\n```text\n");
    report.push_str(&truncate_block(&docker_logs.unwrap_or_else(|error| format!("Docker logs failed: {error}")), DOCKER_REPORT_LIMIT));
    report.push_str("\n```\n\n");

    report.push_str("## Manual Review Notes\n\n");
    report.push_str("- Check subject-specific requirements manually.\n");
    report.push_str("- Confirm generated outputs match the expected correction protocol.\n");
    report.push_str("- Treat this report as supporting evidence, not a final grade.\n");
    report
}
```

- [ ] **Step 4: Verify Markdown helper tests pass**

Run:

```bash
cargo test truncates_long_report_blocks_with_marker renders_required_report_sections
```

Expected: PASS.

- [ ] **Step 5: Add failing integration-style generation test**

Add this test:

```rust
#[test]
fn generates_markdown_report_file() {
    let environment_id = format!("report-generate-test-{}", std::process::id());
    let env_dir = environment_dir(&environment_id).expect("resolve env dir");
    let project_dir = env_dir.join("project");
    std::fs::create_dir_all(&project_dir).expect("create project dir");
    std::fs::write(project_dir.join("Makefile"), "all:\n\ttrue\n").expect("write Makefile");

    let config = EnvironmentConfig {
        id: environment_id.clone(),
        name: "Report Generate Test".into(),
        profile: "epitech-cpp".into(),
        host_path: project_dir.to_string_lossy().into_owned(),
        container_path: "/workspace".into(),
        ide_port: 8080,
        created_at: 1,
    };
    std::fs::create_dir_all(&env_dir).expect("create env dir");
    std::fs::write(
        environment_config_path(&environment_id).expect("config path"),
        serde_json::to_string_pretty(&config).expect("serialize config"),
    ).expect("write config");

    let report = generate_report_file(&environment_id).expect("generate report");
    let markdown = std::fs::read_to_string(report.path).expect("read report");

    assert!(report.name.starts_with("report-"));
    assert!(report.name.ends_with(".md"));
    assert!(markdown.contains("# CodeHarbor Evaluation Report"));
    assert!(markdown.contains("Report Generate Test"));
    assert!(markdown.contains("## Docker"));

    std::fs::remove_dir_all(env_dir).expect("clean env dir");
}
```

- [ ] **Step 6: Implement file generation**

Add:

```rust
fn report_file_name(timestamp: u64, suffix: Option<usize>) -> String {
    match suffix {
        Some(index) => format!("report-{timestamp}-{index}.md"),
        None => format!("report-{timestamp}.md"),
    }
}

fn next_report_path(environment_id: &str, timestamp: u64) -> Result<(String, PathBuf), String> {
    let dir = reports_dir(environment_id)?;
    for index in 0..1000 {
        let name = report_file_name(timestamp, if index == 0 { None } else { Some(index) });
        let path = dir.join(&name);
        if !path.exists() {
            return Ok((name, path));
        }
    }
    Err("Impossible de trouver un nom de rapport libre".into())
}

fn generate_report_file(environment_id: &str) -> Result<ReportFile, String> {
    let config = read_environment_config(environment_id)?;
    let generated_at = created_at_now()?;
    let inspection = detect_project(Path::new(&config.host_path));
    let history = read_history_records(environment_id)?;
    let compose_config = run_environment_compose(environment_id, "docker compose config", &["compose", "config"]);
    let docker_logs = run_environment_compose(environment_id, "docker compose logs", &["compose", "logs", "--tail=200"]);
    let markdown = render_evaluation_report(&config, generated_at, inspection, history, compose_config, docker_logs);

    let dir = reports_dir(environment_id)?;
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Impossible de créer {}: {error}", dir.display()))?;
    let (name, path) = next_report_path(environment_id, generated_at)?;
    fs::write(&path, markdown)
        .map_err(|error| format!("Impossible d'écrire {}: {error}", path.display()))?;

    list_report_files(environment_id)?
        .into_iter()
        .find(|report| report.name == name)
        .ok_or_else(|| "Rapport généré introuvable".to_string())
}
```

- [ ] **Step 7: Verify generation test passes**

Run:

```bash
cargo test generates_markdown_report_file
```

Expected: PASS. Docker source failures are embedded in Markdown if Docker is unavailable in the test environment.

---

### Task 3: Tauri Commands for Generate, List, and Open

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Consumes: `generate_report_file`, `list_report_files`, `report_path`, existing `run_command`
- Produces: `open_filesystem_path(path: &Path) -> Result<(), String>` and Tauri commands `generate_evaluation_report`, `list_evaluation_reports`, `open_report_file`, `open_report_folder`

- [ ] **Step 1: Add filesystem open helper and command wrappers**

Add near the other `#[tauri::command]` functions:

```rust
fn open_filesystem_path(path: &Path) -> Result<(), String> {
    let path_string = path.to_string_lossy().into_owned();

    #[cfg(target_os = "macos")]
    let result = run_command("open path", "open", &[path_string.as_str()], None);

    #[cfg(target_os = "windows")]
    let result = run_command("open path", "explorer", &[path_string.as_str()], None);

    #[cfg(all(unix, not(target_os = "macos")))]
    let result = run_command("open path", "xdg-open", &[path_string.as_str()], None);

    result.map(|_| ())
}

#[tauri::command]
async fn generate_evaluation_report(environment_id: String) -> Result<ReportFile, String> {
    run_blocking_task(move || generate_report_file(&environment_id)).await
}

#[tauri::command]
async fn list_evaluation_reports(environment_id: String) -> Result<Vec<ReportFile>, String> {
    run_blocking_task(move || list_report_files(&environment_id)).await
}

#[tauri::command]
async fn open_report_file(environment_id: String, report_name: String) -> Result<(), String> {
    run_blocking_task(move || {
        let path = report_path(&environment_id, &report_name)?;
        if !path.is_file() {
            return Err(format!("Rapport introuvable: {}", path.display()));
        }
        open_filesystem_path(&path)
    })
    .await
}

#[tauri::command]
async fn open_report_folder(environment_id: String) -> Result<(), String> {
    run_blocking_task(move || {
        let dir = reports_dir(&environment_id)?;
        fs::create_dir_all(&dir)
            .map_err(|error| format!("Impossible de créer {}: {error}", dir.display()))?;
        open_filesystem_path(&dir)
    })
    .await
}
```

- [ ] **Step 2: Register commands**

Add these names to the existing `tauri::generate_handler![...]` list:

```rust
generate_evaluation_report,
list_evaluation_reports,
open_report_file,
open_report_folder,
```

- [ ] **Step 3: Verify compile**

Run from `CodeHarbor/src-tauri`:

```bash
cargo check
```

Expected: PASS.

---

### Task 4: React Report State and UI Panel

**Files:**
- Modify: `src/App.tsx`

**Interfaces:**
- Consumes: Tauri commands `generate_evaluation_report`, `list_evaluation_reports`, `open_report_file`, `open_report_folder`
- Produces: `ReportFile` type, report state, report handlers, `Reports` UI panel

- [ ] **Step 1: Add TypeScript type and state**

Add after `ProjectInspection`:

```ts
type ReportFile = {
  name: string;
  path: string;
  created_at: number;
  size_bytes: number;
};
```

Add state near the existing evaluation state:

```ts
const [reports, setReports] = useState<ReportFile[]>([]);
```

Extend `busyCommand` to include report commands:

```ts
const [busyCommand, setBusyCommand] = useState<CommandName | "create_environment" | "generate_evaluation_report" | "open_report_file" | "open_report_folder" | null>(null);
```

- [ ] **Step 2: Refresh reports with evaluation data**

Modify `refreshEvaluation` so empty selection clears reports:

```ts
setReports([]);
```

Modify the Promise section to include reports:

```ts
const [nextHistory, nextInspection, nextReports] = await Promise.all([
  invoke<EvaluationRunRecord[]>("list_evaluation_history", { environmentId }),
  invoke<ProjectInspection>("inspect_project", { environmentId }),
  invoke<ReportFile[]>("list_evaluation_reports", { environmentId }),
]);
setHistory(nextHistory);
setInspection(nextInspection);
setReports(nextReports);
```

- [ ] **Step 3: Add report handlers**

Add below `loadDockerText`:

```ts
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
```

- [ ] **Step 4: Render the Reports panel**

Add this panel inside `<section className="evaluation-grid" ...>` after the `History` panel:

```tsx
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
```

- [ ] **Step 5: Verify frontend types and build**

Run from `CodeHarbor`:

```bash
npm run build
```

Expected: PASS.

---

### Task 5: Report Panel Styling and Final Verification

**Files:**
- Modify: `src/App.css`

**Interfaces:**
- Consumes: `.reports-panel`, `.report-row` classes from `src/App.tsx`
- Produces: compact report list styling consistent with existing mini panels

- [ ] **Step 1: Add CSS**

Add near existing `.history-row` styles:

```css
.report-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  padding: 8px 0;
  border: 0;
  border-bottom: 1px solid var(--separator);
  background: transparent;
  color: var(--muted-strong);
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.report-row:hover {
  color: var(--text);
}

.report-row:last-child {
  border-bottom: 0;
}

.report-row span {
  color: var(--muted);
  font-size: 0.78rem;
  white-space: nowrap;
}
```

- [ ] **Step 2: Run full verification**

Run from `CodeHarbor/src-tauri`:

```bash
cargo test
cargo check
```

Expected: `cargo test` reports all tests passing and `cargo check` exits 0.

Run from `CodeHarbor`:

```bash
npm run build
```

Expected: TypeScript and Vite build pass.

- [ ] **Step 3: Inspect intended diff only**

Run from `CodeHarbor`:

```bash
git diff -- src-tauri/src/main.rs src/App.tsx src/App.css docs/superpowers/plans/2026-08-09-report-export.md
git status --short
```

Expected: implementation changes are limited to `src-tauri/src/main.rs`, `src/App.tsx`, `src/App.css`, and this plan file, plus unrelated pre-existing dirty files that must not be staged.

- [ ] **Step 4: Commit only intended files when requested**

If the user explicitly asks to commit:

```bash
git add -- src-tauri/src/main.rs src/App.tsx src/App.css docs/superpowers/plans/2026-08-09-report-export.md
git commit -m "feat: add evaluation report export"
```

Expected: commit includes only report export implementation and the plan.

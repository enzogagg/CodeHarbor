use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::net::TcpListener;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct EnvironmentConfig {
    id: String,
    name: String,
    profile: String,
    host_path: String,
    container_path: String,
    ide_port: u16,
    created_at: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct EnvironmentRuntimeStatus {
    environment_id: String,
    status: String,
    container_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct EvaluationRunRecord {
    id: String,
    command: String,
    label: String,
    started_at: u64,
    duration_ms: u128,
    success: bool,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ProjectInspection {
    has_makefile: bool,
    make_targets: Vec<String>,
    language_counts: BTreeMap<String, usize>,
    executables: Vec<String>,
    artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ReportFile {
    name: String,
    path: String,
    created_at: u64,
    size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FullEvaluationStep {
    name: String,
    success: bool,
    details: String,
    stops_pipeline: bool,
}

fn should_stop_full_evaluation_after_step(step: &FullEvaluationStep) -> bool {
    step.stops_pipeline && !step.success
}

fn format_full_evaluation_summary(
    steps: &[FullEvaluationStep],
    report: Result<&ReportFile, &String>,
) -> String {
    let mut summary = String::from("Full evaluation summary\n");
    for step in steps {
        summary.push_str(&format!(
            "- {}: {} - {}\n",
            step.name,
            if step.success { "OK" } else { "FAIL" },
            step.details.lines().next().unwrap_or("")
        ));
    }
    match report {
        Ok(report) => summary.push_str(&format!("- Report: {}\n", report.name)),
        Err(error) => summary.push_str(&format!("- Report: FAIL - {error}\n")),
    }
    summary
}

fn prototype_dir_from_root(root: &Path) -> Result<PathBuf, String> {
    let prototype_dir = root.join("prototype").join("docker-workspace");

    if prototype_dir.is_dir() {
        Ok(prototype_dir)
    } else {
        Err(format!(
            "Prototype Docker introuvable: {}",
            prototype_dir.display()
        ))
    }
}

fn repo_root() -> Result<PathBuf, String> {
    repo_root_from_current_dir(
        &std::env::current_dir()
            .map_err(|error| format!("Impossible de lire le dossier courant: {error}"))?,
    )
}

fn repo_root_from_current_dir(current_dir: &Path) -> Result<PathBuf, String> {
    current_dir
        .ancestors()
        .find(|candidate| {
            candidate
                .join("prototype")
                .join("docker-workspace")
                .is_dir()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            format!(
                "Impossible de résoudre la racine du projet depuis {}",
                current_dir.display()
            )
        })
}

fn codeharbor_home() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "Variable HOME introuvable".to_string())?;
    Ok(PathBuf::from(home).join(".codeharbor"))
}

fn environments_dir() -> Result<PathBuf, String> {
    Ok(codeharbor_home()?.join("environments"))
}

fn projects_dir() -> Result<PathBuf, String> {
    Ok(codeharbor_home()?.join("projects"))
}

fn sanitize_environment_id(name: &str) -> String {
    let mut id = String::new();
    let mut last_dash = false;

    for character in name.trim().to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            id.push(character);
            last_dash = false;
        } else if !last_dash && !id.is_empty() {
            id.push('-');
            last_dash = true;
        }
    }

    id.trim_matches('-').to_string()
}

fn created_at_now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| format!("Horloge système invalide: {error}"))
}

fn environment_dir(environment_id: &str) -> Result<PathBuf, String> {
    Ok(environments_dir()?.join(environment_id))
}

fn environment_config_path(environment_id: &str) -> Result<PathBuf, String> {
    Ok(environment_dir(environment_id)?.join("config.json"))
}

fn environment_is_configured(environment_id: &str) -> Result<bool, String> {
    Ok(environment_config_path(environment_id)?.is_file())
}

fn history_dir(environment_id: &str) -> Result<PathBuf, String> {
    Ok(environment_dir(environment_id)?.join("history"))
}

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
        let Some(name) = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
        else {
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

const COMMAND_OUTPUT_REPORT_LIMIT: usize = 4_000;
const DOCKER_REPORT_LIMIT: usize = 8_000;
const WORKSPACE_DOCKERFILE: &str = include_str!("../../prototype/docker-workspace/Dockerfile");

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
    report.push_str(&format!("- Generated at: `{generated_at}`\n"));
    report.push_str(&format!("- Host path: `{}`\n", config.host_path));
    report
        .push_str("- Note: this report is not an automated grade; manual review is required.\n\n");

    report.push_str("## Project\n\n");
    match inspection {
        Ok(project) => {
            report.push_str(&format!(
                "- Makefile: {}\n",
                if project.has_makefile {
                    "present"
                } else {
                    "missing"
                }
            ));
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
            report.push_str(&format!(
                "- `{}` `{}`: {} in {}ms at `{}`\n",
                run.command,
                run.label,
                if run.success { "OK" } else { "FAIL" },
                run.duration_ms,
                run.started_at
            ));
        }
        report.push('\n');
    }

    report.push_str("## Latest Outputs\n\n");
    if history.is_empty() {
        report.push_str("No command output recorded.\n\n");
    } else {
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
    }

    report.push_str("## Valgrind\n\n");
    let valgrind_runs = history
        .iter()
        .filter(|run| run.command == "valgrind")
        .collect::<Vec<_>>();
    if valgrind_runs.is_empty() {
        report.push_str("No Valgrind runs recorded.\n\n");
    } else {
        for run in valgrind_runs.iter().take(5) {
            report.push_str(&format!("### {}\n\n", run.label));
            report.push_str(&format!(
                "- Status: {}\n",
                if run.success { "OK" } else { "FAIL" }
            ));
            report.push_str(&format!("- Duration: {}ms\n\n", run.duration_ms));
            report.push_str("```text\n");
            report.push_str(&truncate_block(
                &format!("{}\n{}", run.stdout, run.stderr),
                COMMAND_OUTPUT_REPORT_LIMIT,
            ));
            report.push_str("\n```\n\n");
        }
    }

    report.push_str("## Docker\n\n");
    report.push_str("### Compose Config\n\n```yaml\n");
    report.push_str(&truncate_block(
        &compose_config.unwrap_or_else(|error| format!("Docker compose config failed: {error}")),
        DOCKER_REPORT_LIMIT,
    ));
    report.push_str("\n```\n\n### Recent Logs\n\n```text\n");
    report.push_str(&truncate_block(
        &docker_logs.unwrap_or_else(|error| format!("Docker logs failed: {error}")),
        DOCKER_REPORT_LIMIT,
    ));
    report.push_str("\n```\n\n");

    report.push_str("## Manual Review Notes\n\n");
    report.push_str("- Check subject-specific requirements manually.\n");
    report.push_str("- Confirm generated outputs match the expected correction protocol.\n");
    report.push_str("- Treat this report as supporting evidence, not a final grade.\n");
    report
}

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
    let compose_config = run_environment_compose(
        environment_id,
        "docker compose config",
        &["compose", "config"],
    );
    let docker_logs = run_environment_compose(
        environment_id,
        "docker compose logs",
        &["compose", "logs", "--tail=200"],
    );
    let markdown = render_evaluation_report(
        &config,
        generated_at,
        inspection,
        history,
        compose_config,
        docker_logs,
    );

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

fn write_history_record(environment_id: &str, record: &EvaluationRunRecord) -> Result<(), String> {
    let dir = history_dir(environment_id)?;
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Impossible de créer {}: {error}", dir.display()))?;
    let path = dir.join(format!("{}.json", record.id));
    let json = serde_json::to_string_pretty(record)
        .map_err(|error| format!("Impossible de sérialiser l'historique: {error}"))?;
    fs::write(&path, json)
        .map_err(|error| format!("Impossible d'écrire {}: {error}", path.display()))
}

fn read_history_records(environment_id: &str) -> Result<Vec<EvaluationRunRecord>, String> {
    let dir = history_dir(environment_id)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    for entry in fs::read_dir(&dir)
        .map_err(|error| format!("Impossible de lire {}: {error}", dir.display()))?
    {
        let entry = entry.map_err(|error| format!("Entrée d'historique invalide: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            let data = fs::read_to_string(&path)
                .map_err(|error| format!("Impossible de lire {}: {error}", path.display()))?;
            records.push(
                serde_json::from_str::<EvaluationRunRecord>(&data)
                    .map_err(|error| format!("Historique invalide {}: {error}", path.display()))?,
            );
        }
    }

    records.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Ok(records)
}

fn container_name(environment_id: &str) -> String {
    format!("codeharbor-{environment_id}")
}

fn environment_status_from_docker_status(status: &str) -> String {
    let status = status.trim();
    if status.is_empty() {
        "not_created".into()
    } else if status.starts_with("Up ") {
        "running".into()
    } else {
        "stopped".into()
    }
}

fn select_available_port(
    start_port: u16,
    used_ports: &[u16],
    is_available: impl Fn(u16) -> bool,
) -> Result<u16, String> {
    (start_port..9000)
        .find(|port| !used_ports.contains(port) && is_available(*port))
        .ok_or_else(|| "Aucun port IDE libre entre 8080 et 8999".into())
}

fn is_local_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn next_available_ide_port(exclude_environment_id: Option<&str>) -> Result<u16, String> {
    let used_ports = list_environment_configs()?
        .into_iter()
        .filter(|config| Some(config.id.as_str()) != exclude_environment_id)
        .map(|config| config.ide_port)
        .collect::<Vec<_>>();

    select_available_port(8080, &used_ports, is_local_port_available)
}

fn write_environment_config(config: &EnvironmentConfig) -> Result<(), String> {
    let path = environment_config_path(&config.id)?;
    let json = serde_json::to_string_pretty(config)
        .map_err(|error| format!("Impossible de sérialiser config.json: {error}"))?;
    fs::write(&path, json)
        .map_err(|error| format!("Impossible d'écrire {}: {error}", path.display()))
}

fn compose_yaml(config: &EnvironmentConfig) -> String {
    format!(
        r#"services:
  workspace:
    platform: linux/amd64

    build:
      context: .
      platforms:
        - linux/amd64

    container_name: codeharbor-{id}

    ports:
      - "127.0.0.1:{ide_port}:8080"

    volumes:
      - "{host_path}:/workspace"
      - vscode-data:/home/dev/.local/share/code-server
      - vscode-config:/home/dev/.config/code-server
      - shell-history:/command-history

    environment:
      PASSWORD: dev
      DEFAULT_WORKSPACE: /workspace
      HISTFILE: /command-history/.bash_history

    working_dir: /workspace
    init: true
    tty: true

volumes:
  vscode-data:
  vscode-config:
  shell-history:
"#,
        id = config.id,
        ide_port = config.ide_port,
        host_path = config.host_path.replace('"', "\\\"")
    )
}

fn read_environment_config(environment_id: &str) -> Result<EnvironmentConfig, String> {
    let path = environment_config_path(environment_id)?;
    let data = fs::read_to_string(&path)
        .map_err(|error| format!("Impossible de lire {}: {error}", path.display()))?;

    serde_json::from_str(&data)
        .map_err(|error| format!("Configuration invalide {}: {error}", path.display()))
}

fn write_environment_files(config: &EnvironmentConfig) -> Result<(), String> {
    let env_dir = environment_dir(&config.id)?;
    fs::create_dir_all(&env_dir)
        .map_err(|error| format!("Impossible de créer {}: {error}", env_dir.display()))?;

    fs::write(env_dir.join("Dockerfile"), WORKSPACE_DOCKERFILE)
        .map_err(|error| format!("Impossible d'écrire Dockerfile: {error}"))?;

    fs::write(env_dir.join("compose.yaml"), compose_yaml(config))
        .map_err(|error| format!("Impossible d'écrire compose.yaml: {error}"))?;

    write_environment_config(config)?;

    Ok(())
}

fn format_command_result(
    command_name: &str,
    success: bool,
    stdout: &str,
    stderr: &str,
) -> Result<String, String> {
    let stdout = stdout.trim();
    let stderr = stderr.trim();

    if success {
        if stdout.is_empty() {
            Ok(format!("{command_name} terminé avec succès."))
        } else {
            Ok(stdout.to_string())
        }
    } else if stderr.is_empty() {
        Err(format!("{command_name} a échoué."))
    } else {
        Err(format!("{command_name} a échoué: {stderr}"))
    }
}

fn run_command(
    command_name: &str,
    program: &str,
    args: &[&str],
    cwd: Option<&Path>,
) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(args);

    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    let output = command
        .output()
        .map_err(|error| format!("Impossible d'exécuter {command_name}: {error}"))?;

    format_command_result(
        command_name,
        output.status.success(),
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )
}

fn build_script() -> &'static str {
    "cd /workspace && make"
}

fn tests_script() -> &'static str {
    "cd /workspace && make tests_run"
}

fn clean_script() -> &'static str {
    "cd /workspace && (make fclean || true) && (make clean || true)"
}

fn valgrind_script() -> &'static str {
    r#"cd /workspace && \
executables=$(find . -maxdepth 2 -type f -perm -111 \
  ! -path './.git/*' \
  ! -path './node_modules/*' \
  ! -name '*.so' \
  ! -name '*.a' | sort) && \
count=$(printf '%s\n' "$executables" | sed '/^$/d' | wc -l | tr -d ' ') && \
if [ "$count" = "0" ]; then \
  printf 'Valgrind: aucun exécutable trouvé dans /workspace. Lance Build ou indique le binaire manuellement dans le terminal.\n'; \
elif [ "$count" = "1" ]; then \
  target=$(printf '%s\n' "$executables" | sed '/^$/d' | head -n 1); \
  printf 'Valgrind target: %s\n' "$target"; \
  timeout 15s xvfb-run -a valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes "$target"; \
  status=$?; \
  if [ "$status" = "124" ]; then \
    printf 'Valgrind: timeout après 15s. Le programme semble interactif ou ne se termine pas seul.\n' >&2; \
  fi; \
  exit "$status"; \
else \
  printf 'Valgrind: plusieurs exécutables possibles. Ouvre le terminal et lance valgrind sur le bon binaire:\n%s\n' "$executables"; \
fi"#
}

fn valgrind_target_script(target_path: &str) -> String {
    format!(
        "cd /workspace && timeout 15s xvfb-run -a valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes ./{}; status=$?; if [ \"$status\" = \"124\" ]; then printf 'Valgrind: timeout après 15s. Le programme semble interactif ou ne se termine pas seul.\\n' >&2; fi; exit \"$status\"",
        target_path.replace('"', "\\\"")
    )
}

fn validate_workspace_relative_path(path: &str) -> Result<(), String> {
    if !path.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '/' | '.' | '_' | '-')
    }) {
        return Err("Chemin de binaire invalide".into());
    }

    let path = Path::new(path);
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err("Chemin de binaire invalide".into());
    }

    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err("Chemin de binaire invalide".into());
    }

    Ok(())
}

fn relative_display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_plausible_executable_path(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if matches!(
        file_name.as_str(),
        "makefile" | "readme" | "readme.md" | "dockerfile"
    ) {
        return false;
    }

    let Some(extension) = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
    else {
        return true;
    };

    !matches!(
        extension.as_str(),
        "a"
            | "c"
            | "cc"
            | "cpp"
            | "cxx"
            | "gcda"
            | "gcno"
            | "gcov"
            | "h"
            | "hpp"
            | "jpeg"
            | "jpg"
            | "js"
            | "json"
            | "log"
            | "md"
            | "o"
            | "ogg"
            | "png"
            | "py"
            | "rs"
            | "so"
            | "toml"
            | "ttf"
            | "ts"
            | "txt"
            | "wav"
            | "yaml"
            | "yml"
    )
}

fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    if !is_plausible_executable_path(path) {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        return fs::metadata(path)
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false);
    }

    #[cfg(not(unix))]
    {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("exe"))
            .unwrap_or(false)
    }
}

fn collect_project_files(
    root: &Path,
    current: &Path,
    depth: usize,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if depth > 4 {
        return Ok(());
    }

    for entry in fs::read_dir(current)
        .map_err(|error| format!("Impossible de lire {}: {error}", current.display()))?
    {
        let entry = entry.map_err(|error| format!("Entrée projet invalide: {error}"))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".git" || name == "node_modules" || name == "target" {
            continue;
        }

        if path.is_dir() {
            collect_project_files(root, &path, depth + 1, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }

    let _ = root;
    Ok(())
}

fn make_targets(makefile: &Path) -> Vec<String> {
    let Ok(content) = fs::read_to_string(makefile) else {
        return Vec::new();
    };

    let mut targets = Vec::new();
    for line in content.lines() {
        if line.starts_with('\t') || line.starts_with(' ') || line.starts_with('#') {
            continue;
        }
        if let Some((target, _)) = line.split_once(':') {
            let target = target.trim();
            if !target.is_empty()
                && target.chars().all(|character| {
                    character.is_ascii_alphanumeric() || character == '_' || character == '-'
                })
            {
                targets.push(target.to_string());
            }
        }
    }
    targets.sort();
    targets.dedup();
    targets
}

fn detect_project(root: &Path) -> Result<ProjectInspection, String> {
    if !root.is_dir() {
        return Err(format!("Dossier projet introuvable: {}", root.display()));
    }

    let makefile = root.join("Makefile");
    let mut files = Vec::new();
    collect_project_files(root, root, 0, &mut files)?;

    let mut language_counts = BTreeMap::new();
    let mut executables = Vec::new();
    let mut artifacts = Vec::new();

    for file in files {
        let relative = relative_display_path(root, &file);
        if let Some(extension) = file
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_lowercase)
        {
            match extension.as_str() {
                "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "js" | "ts" | "py" | "rs" => {
                    *language_counts.entry(extension.clone()).or_insert(0) += 1;
                }
                "gcov" | "gcda" | "gcno" | "log" => artifacts.push(relative.clone()),
                _ => {}
            }
        }

        if is_executable_file(&file) {
            executables.push(relative);
        }
    }

    executables.sort();
    artifacts.sort();

    Ok(ProjectInspection {
        has_makefile: makefile.is_file(),
        make_targets: if makefile.is_file() {
            make_targets(&makefile)
        } else {
            Vec::new()
        },
        language_counts,
        executables,
        artifacts,
    })
}

fn run_workspace_script(command_name: &str, script: &str) -> Result<String, String> {
    let root = repo_root()?;
    let prototype_dir = prototype_dir_from_root(&root)?;

    run_command(
        command_name,
        "docker",
        &["compose", "exec", "-T", "workspace", "bash", "-lc", script],
        Some(&prototype_dir),
    )
}

fn run_environment_compose(
    environment_id: &str,
    command_name: &str,
    args: &[&str],
) -> Result<String, String> {
    let config = read_environment_config(environment_id)?;
    let env_dir = environment_dir(&config.id)?;

    run_command(command_name, "docker", args, Some(&env_dir))
}

fn environment_runtime_status(config: &EnvironmentConfig) -> EnvironmentRuntimeStatus {
    let name = container_name(&config.id);
    let docker_status = run_command(
        "docker ps",
        "docker",
        &[
            "ps",
            "-a",
            "--filter",
            &format!("name=^/{name}$"),
            "--format",
            "{{.Status}}",
        ],
        None,
    )
    .unwrap_or_default();

    EnvironmentRuntimeStatus {
        environment_id: config.id.clone(),
        status: environment_status_from_docker_status(&docker_status),
        container_name: name,
    }
}

fn delete_environment_files(
    environment_id: &str,
    run_docker_cleanup: bool,
) -> Result<String, String> {
    if environment_id.is_empty() || sanitize_environment_id(environment_id) != environment_id {
        return Err("Identifiant d'environnement invalide".into());
    }

    let env_dir = environment_dir(environment_id)?;
    if !env_dir.exists() {
        return Ok(format!("Environnement {environment_id} supprimé."));
    }

    let compose_path = env_dir.join("compose.yaml");
    if run_docker_cleanup && compose_path.is_file() {
        run_command(
            "docker compose down",
            "docker",
            &["compose", "down"],
            Some(&env_dir),
        )?;
    }

    fs::remove_dir_all(&env_dir)
        .map_err(|error| format!("Impossible de supprimer {}: {error}", env_dir.display()))?;

    Ok(format!(
        "Environnement {environment_id} supprimé. Les fichiers projet sont conservés."
    ))
}

fn run_environment_script(
    environment_id: &str,
    command_name: &str,
    script: &str,
) -> Result<String, String> {
    run_environment_compose(
        environment_id,
        command_name,
        &["compose", "exec", "-T", "workspace", "bash", "-lc", script],
    )
}

fn run_recorded_environment_script(
    environment_id: &str,
    command: &str,
    label: &str,
    script: &str,
) -> Result<String, String> {
    let started_at = created_at_now()?;
    let start = Instant::now();
    let result = run_environment_script(environment_id, label, script);
    let duration_ms = start.elapsed().as_millis();

    let record = match &result {
        Ok(stdout) => EvaluationRunRecord {
            id: format!("{started_at}-{command}-{}", std::process::id()),
            command: command.into(),
            label: label.into(),
            started_at,
            duration_ms,
            success: true,
            stdout: stdout.clone(),
            stderr: String::new(),
        },
        Err(error) => EvaluationRunRecord {
            id: format!("{started_at}-{command}-{}", std::process::id()),
            command: command.into(),
            label: label.into(),
            started_at,
            duration_ms,
            success: false,
            stdout: String::new(),
            stderr: error.clone(),
        },
    };
    write_history_record(environment_id, &record)?;

    result
}

fn recorded_step(
    environment_id: &str,
    name: &str,
    command: &str,
    label: &str,
    script: &str,
    stops_pipeline: bool,
) -> FullEvaluationStep {
    match run_recorded_environment_script(environment_id, command, label, script) {
        Ok(output) => FullEvaluationStep {
            name: name.into(),
            success: true,
            details: output,
            stops_pipeline,
        },
        Err(error) => FullEvaluationStep {
            name: name.into(),
            success: false,
            details: error,
            stops_pipeline,
        },
    }
}

fn run_full_evaluation_inner(
    environment_id: &str,
    target_path: Option<String>,
) -> Result<String, String> {
    if let Some(target) = target_path
        .as_ref()
        .filter(|target| !target.trim().is_empty())
    {
        validate_workspace_relative_path(target)?;
    }

    let mut steps = Vec::new();

    let clean = recorded_step(
        environment_id,
        "Clean",
        "clean",
        "make clean",
        clean_script(),
        false,
    );
    steps.push(clean);

    let build = recorded_step(
        environment_id,
        "Build",
        "build",
        "make",
        build_script(),
        true,
    );
    let stop_after_build = should_stop_full_evaluation_after_step(&build);
    steps.push(build);

    if !stop_after_build {
        let tests = recorded_step(
            environment_id,
            "Tests",
            "tests",
            "make tests_run",
            tests_script(),
            false,
        );
        steps.push(tests);

        match target_path.filter(|target| !target.trim().is_empty()) {
            Some(target) => {
                let valgrind = recorded_step(
                    environment_id,
                    "Valgrind",
                    "valgrind",
                    &format!("valgrind {target}"),
                    &valgrind_target_script(&target),
                    false,
                );
                steps.push(valgrind);
            }
            None => steps.push(FullEvaluationStep {
                name: "Valgrind".into(),
                success: true,
                details: "skipped: no target selected".into(),
                stops_pipeline: false,
            }),
        }
    }

    let report = generate_report_file(environment_id);
    match &report {
        Ok(report) => Ok(format_full_evaluation_summary(&steps, Ok(report))),
        Err(error) => Ok(format_full_evaluation_summary(&steps, Err(error))),
    }
}

async fn run_blocking_task<T>(
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String>
where
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|error| format!("La tâche a été interrompue: {error}"))?
}

#[tauri::command]
async fn check_docker() -> Result<String, String> {
    let version =
        run_blocking_task(|| run_command("docker --version", "docker", &["--version"], None))
            .await?;

    Ok(format!("Docker disponible: {version}"))
}

#[tauri::command]
async fn list_environments() -> Result<Vec<EnvironmentConfig>, String> {
    run_blocking_task(|| {
        let dir = environments_dir()?;
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut environments = Vec::new();
        for entry in fs::read_dir(&dir)
            .map_err(|error| format!("Impossible de lire {}: {error}", dir.display()))?
        {
            let entry = entry.map_err(|error| format!("Entrée invalide: {error}"))?;
            let config_path = entry.path().join("config.json");
            if config_path.is_file() {
                let data = fs::read_to_string(&config_path).map_err(|error| {
                    format!("Impossible de lire {}: {error}", config_path.display())
                })?;
                let config = serde_json::from_str::<EnvironmentConfig>(&data).map_err(|error| {
                    format!("Configuration invalide {}: {error}", config_path.display())
                })?;
                environments.push(config);
            }
        }

        environments.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(environments)
    })
    .await
}

#[tauri::command]
async fn list_environment_statuses() -> Result<Vec<EnvironmentRuntimeStatus>, String> {
    run_blocking_task(|| {
        Ok(list_environment_configs()?
            .iter()
            .map(environment_runtime_status)
            .collect())
    })
    .await
}

#[tauri::command]
async fn run_diagnostics() -> Result<String, String> {
    run_blocking_task(|| {
        let docker = run_command("docker --version", "docker", &["--version"], None)
            .unwrap_or_else(|error| error);
        let environments = list_environment_configs()?.len();
        let port_1420 = run_command(
            "port 1420",
            "lsof",
            &["-nP", "-iTCP:1420", "-sTCP:LISTEN"],
            None,
        )
        .unwrap_or_else(|_| "Port dev 1420 libre.".into());

        Ok(format!(
            "Docker: {docker}\nEnvironnements: {environments}\nDev server: {port_1420}"
        ))
    })
    .await
}

#[tauri::command]
async fn inspect_project(environment_id: String) -> Result<ProjectInspection, String> {
    run_blocking_task(move || {
        let config = read_environment_config(&environment_id)?;
        detect_project(Path::new(&config.host_path))
    })
    .await
}

#[tauri::command]
async fn show_environment_docker_logs(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || {
        run_environment_compose(
            &environment_id,
            "docker compose logs",
            &["compose", "logs", "--tail=200"],
        )
    })
    .await
}

#[tauri::command]
async fn show_environment_compose_config(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || {
        run_environment_compose(
            &environment_id,
            "docker compose config",
            &["compose", "config"],
        )
    })
    .await
}

#[tauri::command]
async fn create_environment(
    name: String,
    host_path: String,
    github_url: String,
) -> Result<EnvironmentConfig, String> {
    run_blocking_task(move || {
        let id = sanitize_environment_id(&name);
        if id.is_empty() {
            return Err("Nom d'environnement invalide".into());
        }

        if environment_is_configured(&id)? {
            return Err(format!("Un environnement existe déjà: {id}"));
        }

        let github_url = github_url.trim().to_string();
        let host_path = host_path.trim().to_string();
        let final_host_path = if github_url.is_empty() {
            if host_path.is_empty() {
                return Err("Renseigne un dossier local ou une URL Git".into());
            }
            let path = PathBuf::from(&host_path);
            if !path.is_dir() {
                return Err(format!("Dossier local introuvable: {}", path.display()));
            }
            path
        } else {
            let clone_path = if host_path.is_empty() {
                projects_dir()?.join(&id)
            } else {
                PathBuf::from(&host_path)
            };

            if clone_path.exists() {
                return Err(format!(
                    "Le dossier de clone existe déjà: {}",
                    clone_path.display()
                ));
            }

            if let Some(parent) = clone_path.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!("Impossible de créer {}: {error}", parent.display())
                })?;
            }

            run_command(
                "git clone",
                "git",
                &[
                    "clone",
                    github_url.as_str(),
                    clone_path.to_string_lossy().as_ref(),
                ],
                None,
            )?;

            clone_path
        };

        let config = EnvironmentConfig {
            id,
            name: name.trim().to_string(),
            profile: "epitech-cpp".into(),
            host_path: final_host_path.to_string_lossy().to_string(),
            container_path: "/workspace".into(),
            ide_port: next_available_ide_port(None)?,
            created_at: created_at_now()?,
        };

        write_environment_files(&config)?;
        Ok(config)
    })
    .await
}

#[tauri::command]
async fn start_prototype() -> Result<String, String> {
    run_blocking_task(|| {
        let root = repo_root()?;
        let prototype_dir = prototype_dir_from_root(&root)?;

        run_command(
            "docker compose up",
            "docker",
            &["compose", "up", "--build", "-d"],
            Some(&prototype_dir),
        )
    })
    .await?;

    Ok("Workspace Ubuntu AMD64 démarré. Ouvre l'IDE sur http://localhost:8080.".into())
}

#[tauri::command]
async fn stop_prototype() -> Result<String, String> {
    run_blocking_task(|| {
        let root = repo_root()?;
        let prototype_dir = prototype_dir_from_root(&root)?;

        run_command(
            "docker compose down",
            "docker",
            &["compose", "down"],
            Some(&prototype_dir),
        )
    })
    .await?;

    Ok("Workspace Ubuntu AMD64 arrêté.".into())
}

#[tauri::command]
async fn open_ide() -> Result<String, String> {
    run_blocking_task(|| {
        let url = "http://localhost:8080";

        #[cfg(target_os = "macos")]
        let result = run_command("open IDE", "open", &[url], None);

        #[cfg(target_os = "windows")]
        let result = run_command("open IDE", "cmd", &["/C", "start", url], None);

        #[cfg(all(unix, not(target_os = "macos")))]
        let result = run_command("open IDE", "xdg-open", &[url], None);

        result.map(|_| "IDE ouvert dans le navigateur.".into())
    })
    .await
}

#[tauri::command]
async fn run_build() -> Result<String, String> {
    run_blocking_task(|| run_workspace_script("make", build_script())).await
}

#[tauri::command]
async fn run_tests() -> Result<String, String> {
    run_blocking_task(|| run_workspace_script("make tests_run", tests_script())).await
}

#[tauri::command]
async fn run_clean() -> Result<String, String> {
    run_blocking_task(|| run_workspace_script("make clean", clean_script())).await
}

#[tauri::command]
async fn run_valgrind() -> Result<String, String> {
    run_blocking_task(|| run_workspace_script("valgrind", valgrind_script())).await
}

fn list_environment_configs() -> Result<Vec<EnvironmentConfig>, String> {
    let dir = environments_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut environments = Vec::new();
    for entry in fs::read_dir(&dir)
        .map_err(|error| format!("Impossible de lire {}: {error}", dir.display()))?
    {
        let entry = entry.map_err(|error| format!("Entrée invalide: {error}"))?;
        let config_path = entry.path().join("config.json");
        if config_path.is_file() {
            let data = fs::read_to_string(&config_path).map_err(|error| {
                format!("Impossible de lire {}: {error}", config_path.display())
            })?;
            environments.push(serde_json::from_str::<EnvironmentConfig>(&data).map_err(
                |error| format!("Configuration invalide {}: {error}", config_path.display()),
            )?);
        }
    }

    Ok(environments)
}

#[tauri::command]
async fn start_environment(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || {
        let mut config = read_environment_config(&environment_id)?;
        if !is_local_port_available(config.ide_port) {
            config.ide_port = next_available_ide_port(Some(&environment_id))?;
            write_environment_files(&config)?;
        }

        run_environment_compose(
            &environment_id,
            "docker compose up",
            &["compose", "up", "--build", "-d"],
        )?;
        let config = read_environment_config(&environment_id)?;
        Ok(format!(
            "{} démarré. IDE: http://localhost:{}",
            config.name, config.ide_port
        ))
    })
    .await
}

#[tauri::command]
async fn stop_environment(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || {
        run_environment_compose(&environment_id, "docker compose down", &["compose", "down"])?;
        let config = read_environment_config(&environment_id)?;
        Ok(format!("{} arrêté.", config.name))
    })
    .await
}

#[tauri::command]
async fn delete_environment(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || delete_environment_files(&environment_id, true)).await
}

#[tauri::command]
async fn list_evaluation_history(
    environment_id: String,
) -> Result<Vec<EvaluationRunRecord>, String> {
    run_blocking_task(move || read_history_records(&environment_id)).await
}

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

#[tauri::command]
async fn open_environment_ide(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || {
        let config = read_environment_config(&environment_id)?;
        let url = format!("http://localhost:{}", config.ide_port);

        #[cfg(target_os = "macos")]
        let result = run_command("open IDE", "open", &[url.as_str()], None);

        #[cfg(target_os = "windows")]
        let result = run_command("open IDE", "cmd", &["/C", "start", url.as_str()], None);

        #[cfg(all(unix, not(target_os = "macos")))]
        let result = run_command("open IDE", "xdg-open", &[url.as_str()], None);

        result.map(|_| format!("IDE ouvert: {url}"))
    })
    .await
}

#[tauri::command]
async fn open_environment_folder(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || {
        let config = read_environment_config(&environment_id)?;

        #[cfg(target_os = "macos")]
        let result = run_command("open folder", "open", &[config.host_path.as_str()], None);

        #[cfg(target_os = "windows")]
        let result = run_command(
            "open folder",
            "explorer",
            &[config.host_path.as_str()],
            None,
        );

        #[cfg(all(unix, not(target_os = "macos")))]
        let result = run_command(
            "open folder",
            "xdg-open",
            &[config.host_path.as_str()],
            None,
        );

        result.map(|_| format!("Dossier ouvert: {}", config.host_path))
    })
    .await
}

#[tauri::command]
async fn run_environment_build(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || {
        run_recorded_environment_script(&environment_id, "build", "make", build_script())
    })
    .await
}

#[tauri::command]
async fn run_environment_tests(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || {
        run_recorded_environment_script(&environment_id, "tests", "make tests_run", tests_script())
    })
    .await
}

#[tauri::command]
async fn run_environment_clean(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || {
        run_recorded_environment_script(&environment_id, "clean", "make clean", clean_script())
    })
    .await
}

#[tauri::command]
async fn run_environment_valgrind(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || {
        run_recorded_environment_script(&environment_id, "valgrind", "valgrind", valgrind_script())
    })
    .await
}

#[tauri::command]
async fn run_environment_valgrind_target(
    environment_id: String,
    target_path: String,
) -> Result<String, String> {
    run_blocking_task(move || {
        validate_workspace_relative_path(&target_path)?;
        run_recorded_environment_script(
            &environment_id,
            "valgrind",
            &format!("valgrind {target_path}"),
            &valgrind_target_script(&target_path),
        )
    })
    .await
}

#[tauri::command]
async fn run_full_evaluation(
    environment_id: String,
    target_path: Option<String>,
) -> Result<String, String> {
    run_blocking_task(move || run_full_evaluation_inner(&environment_id, target_path)).await
}

#[cfg(test)]
mod tests {
    use super::{
        compose_yaml, container_name, delete_environment_files, detect_project,
        environment_config_path, environment_dir, environment_is_configured,
        environment_status_from_docker_status, format_command_result,
        format_full_evaluation_summary, generate_report_file,
        list_report_files, prototype_dir_from_root, read_history_records, render_evaluation_report,
        repo_root_from_current_dir, reports_dir, run_full_evaluation_inner,
        sanitize_environment_id, select_available_port, should_stop_full_evaluation_after_step,
        truncate_block, validate_report_name, validate_workspace_relative_path,
        write_environment_files, write_history_record, EnvironmentConfig, EvaluationRunRecord,
        FullEvaluationStep, ProjectInspection, ReportFile,
    };

    #[test]
    fn resolves_existing_prototype_directory() {
        let root = std::env::temp_dir().join(format!("codeharbor-test-{}", std::process::id()));
        let prototype = root.join("prototype").join("docker-workspace");

        std::fs::create_dir_all(&prototype).expect("create prototype dir");

        let resolved = prototype_dir_from_root(&root).expect("prototype path should resolve");

        assert_eq!(resolved, prototype);

        std::fs::remove_dir_all(root).expect("clean temp dir");
    }

    #[test]
    fn reports_missing_prototype_directory() {
        let root =
            std::env::temp_dir().join(format!("codeharbor-missing-test-{}", std::process::id()));

        let error = prototype_dir_from_root(&root).expect_err("missing prototype should fail");

        assert!(error.contains("Prototype Docker introuvable"));
    }

    #[test]
    fn formats_failed_command_with_stderr() {
        let message = format_command_result("docker compose", false, "", "daemon unavailable");

        assert_eq!(
            message,
            Err("docker compose a échoué: daemon unavailable".into())
        );
    }

    #[test]
    fn sanitizes_environment_name_for_file_system_and_container_names() {
        assert_eq!(
            sanitize_environment_id("My FTP / Student #42"),
            "my-ftp-student-42"
        );
    }

    #[test]
    fn orphan_environment_directory_does_not_count_as_existing_environment() {
        let environment_id = format!("orphan-env-{}", std::process::id());
        let env_dir = environment_dir(&environment_id).expect("resolve environment dir");

        std::fs::create_dir_all(&env_dir).expect("create orphan environment dir");

        assert!(!environment_is_configured(&environment_id).expect("check environment"));

        std::fs::remove_dir_all(env_dir).expect("clean env dir");
    }

    #[test]
    fn builds_stable_container_name_from_environment_id() {
        assert_eq!(container_name("my-ftp"), "codeharbor-my-ftp");
    }

    #[test]
    fn maps_docker_status_to_environment_status() {
        assert_eq!(
            environment_status_from_docker_status("Up 3 minutes"),
            "running"
        );
        assert_eq!(
            environment_status_from_docker_status("Exited (0) 1 minute ago"),
            "stopped"
        );
        assert_eq!(environment_status_from_docker_status(""), "not_created");
    }

    #[test]
    fn selects_first_free_ide_port_after_used_and_unavailable_ports() {
        let selected = select_available_port(8080, &[8081], |port| port != 8080)
            .expect("port should be selected");

        assert_eq!(selected, 8082);
    }

    #[test]
    fn reports_when_no_ide_port_is_available() {
        let error =
            select_available_port(8999, &[], |_| false).expect_err("no port should be selected");

        assert_eq!(error, "Aucun port IDE libre entre 8080 et 8999");
    }

    #[test]
    fn writes_and_reads_history_records_newest_first() {
        let environment_id = format!("history-test-{}", std::process::id());
        let env_dir = environment_dir(&environment_id).expect("resolve env dir");
        std::fs::create_dir_all(&env_dir).expect("create env dir");

        let older = EvaluationRunRecord {
            id: "older".into(),
            command: "build".into(),
            label: "make".into(),
            started_at: 10,
            duration_ms: 5,
            success: true,
            stdout: "ok".into(),
            stderr: "".into(),
        };
        let newer = EvaluationRunRecord {
            id: "newer".into(),
            started_at: 20,
            ..older.clone()
        };

        write_history_record(&environment_id, &older).expect("write older");
        write_history_record(&environment_id, &newer).expect("write newer");

        let records = read_history_records(&environment_id).expect("read records");

        assert_eq!(
            records
                .iter()
                .map(|record| record.id.as_str())
                .collect::<Vec<_>>(),
            vec!["newer", "older"]
        );

        std::fs::remove_dir_all(env_dir).expect("clean env dir");
    }

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

        assert_eq!(
            files
                .iter()
                .map(|file| file.name.as_str())
                .collect::<Vec<_>>(),
            vec!["report-20260809-130000.md", "report-20260809-120000.md"]
        );
        assert!(files.iter().all(|file| file.path.ends_with(&file.name)));
        assert!(files.iter().all(|file| file.size_bytes > 0));

        std::fs::remove_dir_all(env_dir).expect("clean env dir");
    }

    #[test]
    fn truncates_long_report_blocks_with_marker() {
        let input = "x".repeat(12);
        let truncated = truncate_block(&input, 5);

        assert!(truncated.starts_with("xxxxx"));
        assert!(truncated.contains("[truncated"));
    }

    #[test]
    fn full_evaluation_summary_includes_step_statuses_and_report() {
        let steps = vec![
            FullEvaluationStep {
                name: "Clean".into(),
                success: true,
                details: "clean ok".into(),
                stops_pipeline: false,
            },
            FullEvaluationStep {
                name: "Build".into(),
                success: true,
                details: "build ok".into(),
                stops_pipeline: true,
            },
            FullEvaluationStep {
                name: "Tests".into(),
                success: false,
                details: "tests failed".into(),
                stops_pipeline: false,
            },
        ];
        let report = ReportFile {
            name: "report-1.md".into(),
            path: "/tmp/report-1.md".into(),
            created_at: 1,
            size_bytes: 10,
        };

        let summary = format_full_evaluation_summary(&steps, Ok(&report));

        assert!(summary.contains("Full evaluation summary"));
        assert!(summary.contains("Clean: OK"));
        assert!(summary.contains("Tests: FAIL"));
        assert!(summary.contains("Report: report-1.md"));
    }

    #[test]
    fn full_evaluation_stops_only_after_failed_stop_step() {
        let build_failure = FullEvaluationStep {
            name: "Build".into(),
            success: false,
            details: "build failed".into(),
            stops_pipeline: true,
        };
        let tests_failure = FullEvaluationStep {
            name: "Tests".into(),
            success: false,
            details: "tests failed".into(),
            stops_pipeline: false,
        };

        assert!(should_stop_full_evaluation_after_step(&build_failure));
        assert!(!should_stop_full_evaluation_after_step(&tests_failure));
    }

    #[test]
    fn full_evaluation_inner_exposes_required_orchestration_signature() {
        let _: fn(&str, Option<String>) -> Result<String, String> = run_full_evaluation_inner;
    }

    #[test]
    fn full_evaluation_rejects_invalid_target_before_orchestration() {
        let environment_id = format!("full-evaluation-invalid-target-{}", std::process::id());
        let env_dir = environment_dir(&environment_id).expect("resolve env dir");

        let result = run_full_evaluation_inner(&environment_id, Some("../secret".into()));

        assert_eq!(result, Err("Chemin de binaire invalide".into()));
        assert!(
            !env_dir.exists(),
            "invalid target must be rejected before creating environment artifacts"
        );
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
        )
        .expect("write config");

        let report = generate_report_file(&environment_id).expect("generate report");
        let markdown = std::fs::read_to_string(report.path).expect("read report");

        assert!(report.name.starts_with("report-"));
        assert!(report.name.ends_with(".md"));
        assert!(markdown.contains("# CodeHarbor Evaluation Report"));
        assert!(markdown.contains("Report Generate Test"));
        assert!(markdown.contains("## Docker"));

        std::fs::remove_dir_all(env_dir).expect("clean env dir");
    }

    #[test]
    fn write_environment_files_uses_embedded_workspace_dockerfile() {
        let environment_id = format!("embedded-dockerfile-test-{}", std::process::id());
        let env_dir = environment_dir(&environment_id).expect("resolve env dir");
        let workspace = std::env::temp_dir().join(format!(
            "codeharbor-embedded-workspace-{}",
            std::process::id()
        ));

        std::fs::create_dir_all(&workspace).expect("create workspace dir");

        let config = EnvironmentConfig {
            id: environment_id.clone(),
            name: "Embedded Dockerfile Test".into(),
            profile: "epitech-cpp".into(),
            host_path: workspace.to_string_lossy().into_owned(),
            container_path: "/workspace".into(),
            ide_port: 8080,
            created_at: 1,
        };

        let original_dir = std::env::current_dir().expect("read current dir");
        std::env::set_current_dir("/").expect("simulate installed app cwd");
        let result = write_environment_files(&config);
        std::env::set_current_dir(original_dir).expect("restore current dir");

        result.expect("write environment files from installed app cwd");
        let dockerfile = std::fs::read_to_string(env_dir.join("Dockerfile")).expect("read Dockerfile");

        assert!(dockerfile.contains("FROM ubuntu:24.04"));
        assert!(dockerfile.contains("libsfml-dev"));

        std::fs::remove_dir_all(env_dir).expect("clean env dir");
        std::fs::remove_dir_all(workspace).expect("clean workspace dir");
    }

    #[test]
    fn rejects_unsafe_container_relative_paths() {
        assert!(validate_workspace_relative_path("bin/my_binary").is_ok());
        assert!(validate_workspace_relative_path("codeharbor_sample").is_ok());
        assert!(validate_workspace_relative_path("../secret").is_err());
        assert!(validate_workspace_relative_path("/etc/passwd").is_err());
        assert!(validate_workspace_relative_path("bin/foo;touch pwned").is_err());
    }

    #[test]
    fn detects_project_shape_and_artifacts() {
        let root =
            std::env::temp_dir().join(format!("codeharbor-project-detect-{}", std::process::id()));
        let binary = root.join("my_binary");
        let makefile = root.join("Makefile");
        let asset_dir = root.join("da");
        let asset = asset_dir.join("Back.png");

        std::fs::create_dir_all(&root).expect("create project dir");
        std::fs::create_dir_all(&asset_dir).expect("create asset dir");
        std::fs::write(
            &makefile,
            "all:\n\tcc main.c\nclean:\n\trm -f my_binary\ntests_run:\n\ttrue\n",
        )
        .expect("write Makefile");
        std::fs::write(root.join("main.c"), "int main(void) { return 0; }\n")
            .expect("write C source");
        std::fs::write(root.join("main.cpp"), "int main() { return 0; }\n")
            .expect("write C++ source");
        std::fs::write(&asset, "png\n").expect("write asset");
        std::fs::write(root.join("main.gcov"), "coverage\n").expect("write artifact");
        std::fs::write(&binary, "binary\n").expect("write binary");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&binary).expect("metadata").permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&binary, permissions).expect("set executable");

            let mut permissions = std::fs::metadata(&makefile).expect("metadata").permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&makefile, permissions).expect("chmod Makefile");

            let mut permissions = std::fs::metadata(&asset).expect("metadata").permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&asset, permissions).expect("chmod asset");
        }

        let inspection = detect_project(&root).expect("detect project");

        assert!(inspection.has_makefile);
        assert!(inspection.make_targets.contains(&"all".into()));
        assert!(inspection.make_targets.contains(&"clean".into()));
        assert!(inspection.make_targets.contains(&"tests_run".into()));
        assert_eq!(inspection.language_counts.get("c"), Some(&1));
        assert_eq!(inspection.language_counts.get("cpp"), Some(&1));
        assert!(inspection.executables.contains(&"my_binary".into()));
        assert!(!inspection.executables.contains(&"Makefile".into()));
        assert!(!inspection.executables.contains(&"da/Back.png".into()));
        assert!(inspection.artifacts.contains(&"main.gcov".into()));

        std::fs::remove_dir_all(root).expect("clean project dir");
    }

    #[test]
    fn generated_compose_mounts_host_project_to_workspace() {
        let config = EnvironmentConfig {
            id: "my-ftp".into(),
            name: "My FTP".into(),
            profile: "epitech-cpp".into(),
            host_path: "/Users/me/projects/myftp".into(),
            container_path: "/workspace".into(),
            ide_port: 8083,
            created_at: 1,
        };

        let compose = compose_yaml(&config);

        assert!(compose.contains("container_name: codeharbor-my-ftp"));
        assert!(compose.contains("127.0.0.1:8083:8080"));
        assert!(compose.contains("\"/Users/me/projects/myftp:/workspace\""));
    }

    #[test]
    fn deleting_environment_files_removes_generated_env_and_keeps_workspace() {
        let environment_id = format!("delete-test-{}", std::process::id());
        let workspace =
            std::env::temp_dir().join(format!("codeharbor-workspace-{}", std::process::id()));
        let env_dir = environment_dir(&environment_id).expect("resolve environment dir");

        std::fs::create_dir_all(&workspace).expect("create workspace dir");
        std::fs::create_dir_all(&env_dir).expect("create environment dir");
        std::fs::write(env_dir.join("compose.yaml"), "services: {}\n").expect("write compose file");
        std::fs::write(workspace.join("main.c"), "int main(void) { return 0; }\n")
            .expect("write workspace file");

        let result = delete_environment_files(&environment_id, false).expect("delete environment");

        assert!(result.contains(&environment_id));
        assert!(!env_dir.exists());
        assert!(workspace.exists());
        assert!(workspace.join("main.c").exists());

        std::fs::remove_dir_all(workspace).expect("clean workspace dir");
    }

    #[test]
    fn deleting_environment_files_rejects_invalid_environment_id() {
        let result = delete_environment_files("../outside", false);

        assert_eq!(result, Err("Identifiant d'environnement invalide".into()));
    }

    #[test]
    fn resolves_repo_root_from_tauri_working_directory() {
        let root =
            std::env::temp_dir().join(format!("codeharbor-root-test-{}", std::process::id()));
        let src_tauri = root.join("src-tauri");
        let prototype = root.join("prototype").join("docker-workspace");

        std::fs::create_dir_all(&src_tauri).expect("create src-tauri dir");
        std::fs::create_dir_all(&prototype).expect("create prototype dir");

        let resolved = repo_root_from_current_dir(&src_tauri).expect("repo root should resolve");

        assert_eq!(resolved, root);

        std::fs::remove_dir_all(resolved).expect("clean temp dir");
    }

    mod evaluation_command_tests {
        use super::super::{
            build_script, clean_script, tests_script, valgrind_script, valgrind_target_script,
        };

        #[test]
        fn build_script_runs_make_in_workspace() {
            assert_eq!(build_script(), "cd /workspace && make");
        }

        #[test]
        fn tests_script_runs_tests_run_target() {
            assert_eq!(tests_script(), "cd /workspace && make tests_run");
        }

        #[test]
        fn clean_script_runs_fclean_then_clean_without_failing() {
            assert_eq!(
                clean_script(),
                "cd /workspace && (make fclean || true) && (make clean || true)"
            );
        }

        #[test]
        fn valgrind_script_lists_executables_when_target_is_ambiguous() {
            assert!(valgrind_script().contains("find . -maxdepth 2 -type f -perm -111"));
            assert!(valgrind_script().contains("Valgrind: plusieurs exécutables possibles"));
        }

        #[test]
        fn valgrind_scripts_run_under_virtual_x_server() {
            assert!(valgrind_script().contains("xvfb-run -a"));
            assert!(valgrind_target_script("my_hunter").contains("xvfb-run -a"));
        }

        #[test]
        fn valgrind_scripts_have_runtime_timeout_for_interactive_programs() {
            assert!(valgrind_script().contains("timeout 15s"));
            assert!(valgrind_target_script("my_hunter").contains("timeout 15s"));
        }

        #[test]
        fn valgrind_scripts_explain_timeout_failures() {
            assert!(valgrind_script().contains("Valgrind: timeout après 15s"));
            assert!(valgrind_target_script("my_hunter").contains("Valgrind: timeout après 15s"));
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            check_docker,
            start_prototype,
            stop_prototype,
            open_ide,
            run_build,
            run_tests,
            run_clean,
            run_valgrind,
            list_environments,
            list_environment_statuses,
            run_diagnostics,
            inspect_project,
            show_environment_docker_logs,
            show_environment_compose_config,
            create_environment,
            start_environment,
            stop_environment,
            delete_environment,
            list_evaluation_history,
            generate_evaluation_report,
            list_evaluation_reports,
            open_report_file,
            open_report_folder,
            open_environment_ide,
            open_environment_folder,
            run_environment_build,
            run_environment_tests,
            run_environment_clean,
            run_environment_valgrind,
            run_environment_valgrind_target,
            run_full_evaluation
        ])
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                window.app_handle().exit(0);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running CodeHarbor");
}

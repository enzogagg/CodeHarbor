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
    repo_root_from_current_dir(&std::env::current_dir()
        .map_err(|error| format!("Impossible de lire le dossier courant: {error}"))?
    )
}

fn repo_root_from_current_dir(current_dir: &Path) -> Result<PathBuf, String> {
    current_dir
        .ancestors()
        .find(|candidate| candidate.join("prototype").join("docker-workspace").is_dir())
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

fn history_dir(environment_id: &str) -> Result<PathBuf, String> {
    Ok(environment_dir(environment_id)?.join("history"))
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
            records.push(serde_json::from_str::<EvaluationRunRecord>(&data).map_err(|error| {
                format!("Historique invalide {}: {error}", path.display())
            })?);
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
    fs::write(&path, json).map_err(|error| format!("Impossible d'écrire {}: {error}", path.display()))
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

    let root = repo_root()?;
    let source_dockerfile = prototype_dir_from_root(&root)?.join("Dockerfile");
    fs::copy(&source_dockerfile, env_dir.join("Dockerfile")).map_err(|error| {
        format!(
            "Impossible de copier {}: {error}",
            source_dockerfile.display()
        )
    })?;

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

fn run_command(command_name: &str, program: &str, args: &[&str], cwd: Option<&Path>) -> Result<String, String> {
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
  valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes "$target"; \
else \
  printf 'Valgrind: plusieurs exécutables possibles. Ouvre le terminal et lance valgrind sur le bon binaire:\n%s\n' "$executables"; \
fi"#
}

fn valgrind_target_script(target_path: &str) -> String {
    format!(
        "cd /workspace && valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes ./{}",
        target_path.replace('"', "\\\"")
    )
}

fn validate_workspace_relative_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err("Chemin de binaire invalide".into());
    }

    if path.components().any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))) {
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

fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
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

fn collect_project_files(root: &Path, current: &Path, depth: usize, files: &mut Vec<PathBuf>) -> Result<(), String> {
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
            if !target.is_empty() && target.chars().all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '-') {
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
        if let Some(extension) = file.extension().and_then(|extension| extension.to_str()).map(str::to_lowercase) {
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
        make_targets: if makefile.is_file() { make_targets(&makefile) } else { Vec::new() },
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

fn run_environment_compose(environment_id: &str, command_name: &str, args: &[&str]) -> Result<String, String> {
    let config = read_environment_config(environment_id)?;
    let env_dir = environment_dir(&config.id)?;

    run_command(command_name, "docker", args, Some(&env_dir))
}

fn environment_runtime_status(config: &EnvironmentConfig) -> EnvironmentRuntimeStatus {
    let name = container_name(&config.id);
    let docker_status = run_command(
        "docker ps",
        "docker",
        &["ps", "-a", "--filter", &format!("name=^/{name}$"), "--format", "{{.Status}}"],
        None,
    )
    .unwrap_or_default();

    EnvironmentRuntimeStatus {
        environment_id: config.id.clone(),
        status: environment_status_from_docker_status(&docker_status),
        container_name: name,
    }
}

fn delete_environment_files(environment_id: &str, run_docker_cleanup: bool) -> Result<String, String> {
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

    Ok(format!("Environnement {environment_id} supprimé. Les fichiers projet sont conservés."))
}

fn run_environment_script(environment_id: &str, command_name: &str, script: &str) -> Result<String, String> {
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

async fn run_blocking_task<T>(task: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String>
where
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|error| format!("La tâche a été interrompue: {error}"))?
}

#[tauri::command]
async fn check_docker() -> Result<String, String> {
    let version = run_blocking_task(|| run_command("docker --version", "docker", &["--version"], None)).await?;

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

        let env_dir = environment_dir(&id)?;
        if env_dir.exists() {
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
                &["clone", github_url.as_str(), clone_path.to_string_lossy().as_ref()],
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
            let data = fs::read_to_string(&config_path)
                .map_err(|error| format!("Impossible de lire {}: {error}", config_path.display()))?;
            environments.push(serde_json::from_str::<EnvironmentConfig>(&data).map_err(|error| {
                format!("Configuration invalide {}: {error}", config_path.display())
            })?);
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
async fn list_evaluation_history(environment_id: String) -> Result<Vec<EvaluationRunRecord>, String> {
    run_blocking_task(move || read_history_records(&environment_id)).await
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
        let result = run_command("open folder", "explorer", &[config.host_path.as_str()], None);

        #[cfg(all(unix, not(target_os = "macos")))]
        let result = run_command("open folder", "xdg-open", &[config.host_path.as_str()], None);

        result.map(|_| format!("Dossier ouvert: {}", config.host_path))
    })
    .await
}

#[tauri::command]
async fn run_environment_build(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || run_recorded_environment_script(&environment_id, "build", "make", build_script())).await
}

#[tauri::command]
async fn run_environment_tests(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || run_recorded_environment_script(&environment_id, "tests", "make tests_run", tests_script())).await
}

#[tauri::command]
async fn run_environment_clean(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || run_recorded_environment_script(&environment_id, "clean", "make clean", clean_script())).await
}

#[tauri::command]
async fn run_environment_valgrind(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || run_recorded_environment_script(&environment_id, "valgrind", "valgrind", valgrind_script())).await
}

#[tauri::command]
async fn run_environment_valgrind_target(environment_id: String, target_path: String) -> Result<String, String> {
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

#[cfg(test)]
mod tests {
    use super::{compose_yaml, container_name, delete_environment_files, detect_project, environment_dir, environment_status_from_docker_status, format_command_result, prototype_dir_from_root, read_history_records, repo_root_from_current_dir, sanitize_environment_id, select_available_port, validate_workspace_relative_path, write_history_record, EnvironmentConfig, EvaluationRunRecord};

    #[test]
    fn resolves_existing_prototype_directory() {
        let root = std::env::temp_dir().join(format!(
            "codeharbor-test-{}",
            std::process::id()
        ));
        let prototype = root.join("prototype").join("docker-workspace");

        std::fs::create_dir_all(&prototype).expect("create prototype dir");

        let resolved = prototype_dir_from_root(&root).expect("prototype path should resolve");

        assert_eq!(resolved, prototype);

        std::fs::remove_dir_all(root).expect("clean temp dir");
    }

    #[test]
    fn reports_missing_prototype_directory() {
        let root = std::env::temp_dir().join(format!(
            "codeharbor-missing-test-{}",
            std::process::id()
        ));

        let error = prototype_dir_from_root(&root).expect_err("missing prototype should fail");

        assert!(error.contains("Prototype Docker introuvable"));
    }

    #[test]
    fn formats_failed_command_with_stderr() {
        let message = format_command_result("docker compose", false, "", "daemon unavailable");

        assert_eq!(message, Err("docker compose a échoué: daemon unavailable".into()));
    }

    #[test]
    fn sanitizes_environment_name_for_file_system_and_container_names() {
        assert_eq!(sanitize_environment_id("My FTP / Student #42"), "my-ftp-student-42");
    }

    #[test]
    fn builds_stable_container_name_from_environment_id() {
        assert_eq!(container_name("my-ftp"), "codeharbor-my-ftp");
    }

    #[test]
    fn maps_docker_status_to_environment_status() {
        assert_eq!(environment_status_from_docker_status("Up 3 minutes"), "running");
        assert_eq!(environment_status_from_docker_status("Exited (0) 1 minute ago"), "stopped");
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
        let error = select_available_port(8999, &[], |_| false)
            .expect_err("no port should be selected");

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
        let newer = EvaluationRunRecord { id: "newer".into(), started_at: 20, ..older.clone() };

        write_history_record(&environment_id, &older).expect("write older");
        write_history_record(&environment_id, &newer).expect("write newer");

        let records = read_history_records(&environment_id).expect("read records");

        assert_eq!(records.iter().map(|record| record.id.as_str()).collect::<Vec<_>>(), vec!["newer", "older"]);

        std::fs::remove_dir_all(env_dir).expect("clean env dir");
    }

    #[test]
    fn rejects_unsafe_container_relative_paths() {
        assert!(validate_workspace_relative_path("bin/my_binary").is_ok());
        assert!(validate_workspace_relative_path("../secret").is_err());
        assert!(validate_workspace_relative_path("/etc/passwd").is_err());
    }

    #[test]
    fn detects_project_shape_and_artifacts() {
        let root = std::env::temp_dir().join(format!("codeharbor-project-detect-{}", std::process::id()));
        let binary = root.join("my_binary");

        std::fs::create_dir_all(&root).expect("create project dir");
        std::fs::write(root.join("Makefile"), "all:\n\tcc main.c\nclean:\n\trm -f my_binary\ntests_run:\n\ttrue\n").expect("write Makefile");
        std::fs::write(root.join("main.c"), "int main(void) { return 0; }\n").expect("write C source");
        std::fs::write(root.join("main.cpp"), "int main() { return 0; }\n").expect("write C++ source");
        std::fs::write(root.join("main.gcov"), "coverage\n").expect("write artifact");
        std::fs::write(&binary, "binary\n").expect("write binary");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&binary).expect("metadata").permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&binary, permissions).expect("set executable");
        }

        let inspection = detect_project(&root).expect("detect project");

        assert!(inspection.has_makefile);
        assert!(inspection.make_targets.contains(&"all".into()));
        assert!(inspection.make_targets.contains(&"clean".into()));
        assert!(inspection.make_targets.contains(&"tests_run".into()));
        assert_eq!(inspection.language_counts.get("c"), Some(&1));
        assert_eq!(inspection.language_counts.get("cpp"), Some(&1));
        assert!(inspection.executables.contains(&"my_binary".into()));
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
        let workspace = std::env::temp_dir().join(format!(
            "codeharbor-workspace-{}",
            std::process::id()
        ));
        let env_dir = environment_dir(&environment_id).expect("resolve environment dir");

        std::fs::create_dir_all(&workspace).expect("create workspace dir");
        std::fs::create_dir_all(&env_dir).expect("create environment dir");
        std::fs::write(env_dir.join("compose.yaml"), "services: {}\n").expect("write compose file");
        std::fs::write(workspace.join("main.c"), "int main(void) { return 0; }\n").expect("write workspace file");

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
        let root = std::env::temp_dir().join(format!(
            "codeharbor-root-test-{}",
            std::process::id()
        ));
        let src_tauri = root.join("src-tauri");
        let prototype = root.join("prototype").join("docker-workspace");

        std::fs::create_dir_all(&src_tauri).expect("create src-tauri dir");
        std::fs::create_dir_all(&prototype).expect("create prototype dir");

        let resolved = repo_root_from_current_dir(&src_tauri).expect("repo root should resolve");

        assert_eq!(resolved, root);

        std::fs::remove_dir_all(resolved).expect("clean temp dir");
    }

    mod evaluation_command_tests {
        use super::super::{build_script, clean_script, tests_script, valgrind_script};

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
            assert_eq!(clean_script(), "cd /workspace && (make fclean || true) && (make clean || true)");
        }

        #[test]
        fn valgrind_script_lists_executables_when_target_is_ambiguous() {
            assert!(valgrind_script().contains("find . -maxdepth 2 -type f -perm -111"));
            assert!(valgrind_script().contains("Valgrind: plusieurs exécutables possibles"));
        }
    }
}

fn main() {
    tauri::Builder::default()
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
            open_environment_ide,
            open_environment_folder,
            run_environment_build,
            run_environment_tests,
            run_environment_clean,
            run_environment_valgrind,
            run_environment_valgrind_target
        ])
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                window.app_handle().exit(0);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running CodeHarbor");
}

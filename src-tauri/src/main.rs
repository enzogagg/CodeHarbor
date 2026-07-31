use std::path::{Path, PathBuf};
use std::process::Command;

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
    std::env::current_dir()
        .map_err(|error| format!("Impossible de lire le dossier courant: {error}"))?
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "Impossible de résoudre la racine du projet".to_string())
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

#[tauri::command]
fn check_docker() -> Result<String, String> {
    let version = run_command("docker --version", "docker", &["--version"], None)?;

    Ok(format!("Docker disponible: {version}"))
}

#[tauri::command]
fn start_prototype() -> Result<String, String> {
    let root = repo_root()?;
    let prototype_dir = prototype_dir_from_root(&root)?;

    run_command(
        "docker compose up",
        "docker",
        &["compose", "up", "--build", "-d"],
        Some(&prototype_dir),
    )?;

    Ok("Workspace Ubuntu AMD64 démarré. Ouvre l'IDE sur http://localhost:8080.".into())
}

#[tauri::command]
fn stop_prototype() -> Result<String, String> {
    let root = repo_root()?;
    let prototype_dir = prototype_dir_from_root(&root)?;

    run_command(
        "docker compose down",
        "docker",
        &["compose", "down"],
        Some(&prototype_dir),
    )?;

    Ok("Workspace Ubuntu AMD64 arrêté.".into())
}

#[tauri::command]
fn open_ide() -> Result<String, String> {
    let url = "http://localhost:8080";

    #[cfg(target_os = "macos")]
    let result = run_command("open IDE", "open", &[url], None);

    #[cfg(target_os = "windows")]
    let result = run_command("open IDE", "cmd", &["/C", "start", url], None);

    #[cfg(all(unix, not(target_os = "macos")))]
    let result = run_command("open IDE", "xdg-open", &[url], None);

    result.map(|_| "IDE ouvert dans le navigateur.".into())
}

#[cfg(test)]
mod tests {
    use super::{format_command_result, prototype_dir_from_root};

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
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            check_docker,
            start_prototype,
            stop_prototype,
            open_ide
        ])
        .run(tauri::generate_context!())
        .expect("error while running CodeHarbor");
}

# Environment Deletion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe environment deletion that removes generated CodeHarbor environment files without deleting source projects.

**Architecture:** The backend owns destructive filesystem and Docker cleanup through a new Tauri command. The frontend exposes a confirmed `Supprimer` action, invokes the command, refreshes the environment list, and clears the deleted selection.

**Tech Stack:** Rust/Tauri commands in `src-tauri/src/main.rs`; React/TypeScript UI in `src/App.tsx`; CSS in `src/App.css`; verification with `cargo test`, `cargo check`, and `npm run build`.

## Global Constraints

- Deleting an environment removes only `~/.codeharbor/environments/<id>`.
- Deleting an environment must not remove the mounted project folder or Git clone under `~/.codeharbor/projects/<id>`.
- The backend should run `docker compose down` when `compose.yaml` exists.
- Docker cleanup failure should be shown as an error, not hidden.
- Missing environment directories should be safe to delete from the UI by refreshing state.
- Invalid environment IDs must be rejected before resolving filesystem paths.
- Do not introduce OAuth, full project deletion, or a custom modal in this task.
- Do not commit unless the user explicitly asks for a commit.

---

## File Structure

- Modify `src-tauri/src/main.rs`: add pure deletion helper, Tauri command, test, and register command in `generate_handler!`.
- Modify `src/App.tsx`: add delete command typing, busy state, confirmation flow, invoke backend, refresh list, and clear selected environment.
- Modify `src/App.css`: add a destructive button variant using the existing action-button styling.

---

### Task 1: Backend Safe Delete Helper and Tauri Command

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Consumes: `environment_dir(environment_id: &str) -> Result<PathBuf, String>`, `run_command(command_name: &str, program: &str, args: &[&str], cwd: Option<&Path>) -> Result<String, String>`
- Produces: `fn delete_environment_files(environment_id: &str, run_docker_cleanup: bool) -> Result<String, String>` and `async fn delete_environment(environment_id: String) -> Result<String, String>`

- [ ] **Step 1: Write the failing test**

Add `delete_environment_files` and `environment_dir` to the existing test import, then add this test inside `#[cfg(test)] mod tests` in `src-tauri/src/main.rs`:

```rust
use super::{compose_yaml, delete_environment_files, environment_dir, format_command_result, prototype_dir_from_root, repo_root_from_current_dir, sanitize_environment_id, EnvironmentConfig};

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
```

- [ ] **Step 2: Run test to verify it fails**

Run from `CodeHarbor/src-tauri`:

```bash
cargo test deleting_environment_files_removes_generated_env_and_keeps_workspace
```

Expected: FAIL because `delete_environment_files` does not exist.

- [ ] **Step 3: Write minimal implementation**

Add this helper after `run_environment_compose` in `src-tauri/src/main.rs`:

```rust
fn delete_environment_files(environment_id: &str, run_docker_cleanup: bool) -> Result<String, String> {
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
```

Add this guard test after `deleting_environment_files_removes_generated_env_and_keeps_workspace`:

```rust
#[test]
fn deleting_environment_files_rejects_invalid_environment_id() {
    let result = delete_environment_files("../outside", false);

    assert_eq!(result, Err("Identifiant d'environnement invalide".into()));
}
```

Add this validation at the start of `delete_environment_files`:

```rust
if environment_id.is_empty() || sanitize_environment_id(environment_id) != environment_id {
    return Err("Identifiant d'environnement invalide".into());
}
```

Add this Tauri command after `stop_environment`:

```rust
#[tauri::command]
async fn delete_environment(environment_id: String) -> Result<String, String> {
    run_blocking_task(move || delete_environment_files(&environment_id, true)).await
}
```

Register it in `tauri::generate_handler!` after `stop_environment`:

```rust
delete_environment,
```

- [ ] **Step 4: Run test to verify it passes**

Run from `CodeHarbor/src-tauri`:

```bash
cargo test deleting_environment_files_removes_generated_env_and_keeps_workspace
```

Expected: PASS.

- [ ] **Step 5: Run backend test suite**

Run from `CodeHarbor/src-tauri`:

```bash
cargo test
```

Expected: all tests pass.

---

### Task 2: Frontend Delete Action

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.css`

**Interfaces:**
- Consumes: Tauri command `delete_environment` with `{ environmentId: string }` arguments.
- Produces: `deleteEnvironment()` UI flow and `destructive` action button styling.

- [ ] **Step 1: Add TypeScript command and busy-state types**

In `src/App.tsx`, add `"delete_environment"` to `CommandName`:

```ts
type CommandName =
  | "start_environment"
  | "stop_environment"
  | "delete_environment"
  | "open_environment_ide"
  | "run_environment_build"
  | "run_environment_tests"
  | "run_environment_valgrind"
  | "run_environment_clean"
  | "check_docker";
```

Change the action type to allow destructive actions:

```ts
const actions: Array<{ command: CommandName; label: string; kind: "primary" | "secondary" | "destructive"; needsEnvironment: boolean }> = [
```

Add the delete action after `Arrêter`:

```ts
  { command: "delete_environment", label: "Supprimer", kind: "destructive", needsEnvironment: true },
```

Update `busyCommand` so existing create state still works:

```ts
const [busyCommand, setBusyCommand] = useState<CommandName | "create_environment" | null>(null);
```

- [ ] **Step 2: Add the delete flow**

Add this function after `createEnvironment()` in `src/App.tsx`:

```ts
async function deleteEnvironment() {
  if (!selectedEnvironment) {
    setError("Sélectionne un environnement avant de le supprimer.");
    return;
  }

  const confirmed = window.confirm(
    `Supprimer l'environnement ${selectedEnvironment.name} ?\n\nCodeHarbor supprimera seulement les fichiers générés de l'environnement. Le dossier projet sera conservé.`
  );

  if (!confirmed) {
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
    setMessage(response);
  } catch (caught) {
    setError(String(caught));
    setMessage("Suppression interrompue.");
  } finally {
    setBusyCommand(null);
  }
}
```

- [ ] **Step 3: Route delete action separately from generic command execution**

In the actions button `onClick`, replace this line:

```tsx
onClick={() => runCommand(action.command)}
```

with:

```tsx
onClick={() => action.command === "delete_environment" ? deleteEnvironment() : runCommand(action.command)}
```

- [ ] **Step 4: Add destructive styling**

In `src/App.css`, add this block after `.action-button.secondary`:

```css
.action-button.destructive {
  border-color: rgba(255, 69, 58, 0.38);
  background: rgba(255, 69, 58, 0.12);
  color: #ffb4ad;
}
```

Add this block after the primary hover block:

```css
.action-button.destructive:hover:not(:disabled),
.action-button.destructive:focus-visible:not(:disabled) {
  border-color: rgba(255, 69, 58, 0.7);
  background: rgba(255, 69, 58, 0.2);
}
```

- [ ] **Step 5: Run frontend build**

Run from `CodeHarbor`:

```bash
npm run build
```

Expected: TypeScript and Vite build complete successfully.

---

### Task 3: Final Verification

**Files:**
- No code changes expected.

**Interfaces:**
- Consumes: all changes from Tasks 1 and 2.
- Produces: verification evidence for completion.

- [ ] **Step 1: Run Rust tests**

Run from `CodeHarbor/src-tauri`:

```bash
cargo test
```

Expected: all tests pass, including `deleting_environment_files_removes_generated_env_and_keeps_workspace`.

- [ ] **Step 2: Run Rust check**

Run from `CodeHarbor/src-tauri`:

```bash
cargo check
```

Expected: check completes without errors.

- [ ] **Step 3: Run frontend build**

Run from `CodeHarbor`:

```bash
npm run build
```

Expected: TypeScript and Vite build complete successfully.

- [ ] **Step 4: Inspect changed files**

Run from `CodeHarbor`:

```bash
git diff -- src-tauri/src/main.rs src/App.tsx src/App.css docs/superpowers/specs/2026-07-31-environment-deletion-design.md docs/superpowers/plans/2026-07-31-environment-deletion.md
```

Expected: diff contains only the safe deletion feature, spec, and plan.

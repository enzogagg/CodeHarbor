# Evaluation Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CodeHarbor's first evaluation core: run history, Docker inspection, project detection, Valgrind target selection, and artifact listing.

**Architecture:** Keep persistence backend-owned under `~/.codeharbor/environments/<id>/history/`. Rust/Tauri exposes typed inspection and execution commands; React renders focused `Evaluation`, `History`, `Docker`, and `Artifacts` panels without adding more header buttons.

**Tech Stack:** Rust/Tauri in `src-tauri/src/main.rs`; React/TypeScript in `src/App.tsx`; CSS in `src/App.css`; JSON files for history persistence; verification with `cargo test`, `cargo check`, and `npm run build`.

## Global Constraints

- Evaluation actions write one JSON history entry under `~/.codeharbor/environments/<id>/history/` for Build, Tests, Clean, and Valgrind.
- History entries store ID, command kind, command label, start timestamp, duration milliseconds, success flag, stdout, and stderr/error text.
- Project inspection reports Makefile presence, common Make targets, language counts, executable candidates, and basic artifacts.
- Docker inspection exposes `docker compose logs --tail=200` and `docker compose config`.
- Valgrind execution accepts only a selected executable path relative to `/workspace`.
- Backend commands reject absolute paths and paths containing `..` for container execution.
- No full Markdown/PDF report, full Epitech compliance scoring, guided evaluator mode, profiles, templates, snapshots, or terminal emulator in this batch.
- Do not commit unless the user explicitly asks for a commit.

---

## File Structure

- Modify `src-tauri/src/main.rs`: add history model, project inspection model, artifact discovery, Valgrind target validation, Docker inspection commands, and register new Tauri commands.
- Modify `src/App.tsx`: add UI state and panels for Evaluation, History, Docker, and Artifacts; replace generic Valgrind action with target selection.
- Modify `src/App.css`: add panel layouts for evaluator sections, run history, path lists, and Docker output blocks.

---

### Task 1: History Model and Persistence

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `EvaluationRunRecord`, `history_dir(environment_id: &str) -> Result<PathBuf, String>`, `write_history_record(environment_id: &str, record: &EvaluationRunRecord) -> Result<(), String>`, `read_history_records(environment_id: &str) -> Result<Vec<EvaluationRunRecord>, String>`
- Consumes: existing `environment_dir`, `created_at_now`

- [ ] **Step 1: Write failing Rust tests**

Add tests for JSON persistence and sorting:

```rust
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
```

- [ ] **Step 2: Run failing test**

Run from `CodeHarbor/src-tauri`:

```bash
cargo test writes_and_reads_history_records_newest_first
```

Expected: FAIL because `EvaluationRunRecord`, `write_history_record`, and `read_history_records` do not exist.

- [ ] **Step 3: Implement minimal history persistence**

Add serializable record type, create `history/`, write one JSON file per run, read JSON files, sort by `started_at` descending.

- [ ] **Step 4: Verify targeted test passes**

Run:

```bash
cargo test writes_and_reads_history_records_newest_first
```

Expected: PASS.

---

### Task 2: Recorded Evaluation Commands

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: recorded Build, Tests, Clean, and Valgrind commands; `list_evaluation_history(environment_id: String)` Tauri command.
- Consumes: Task 1 history helpers and existing evaluation scripts.

- [ ] **Step 1: Add failing test for path validation**

```rust
#[test]
fn rejects_unsafe_container_relative_paths() {
    assert!(validate_workspace_relative_path("bin/my_binary").is_ok());
    assert!(validate_workspace_relative_path("../secret").is_err());
    assert!(validate_workspace_relative_path("/etc/passwd").is_err());
}
```

- [ ] **Step 2: Run failing test**

Run:

```bash
cargo test rejects_unsafe_container_relative_paths
```

Expected: FAIL because `validate_workspace_relative_path` does not exist.

- [ ] **Step 3: Implement command recording and safe Valgrind target execution**

Wrap `run_environment_script` calls in a helper that measures duration and writes a history record. Add `run_environment_valgrind_target(environment_id, target_path)` using `valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes "$target"` after validation.

- [ ] **Step 4: Register new Tauri commands**

Register `list_evaluation_history` and `run_environment_valgrind_target` in `tauri::generate_handler!`.

- [ ] **Step 5: Verify backend tests**

Run:

```bash
cargo test
```

Expected: all tests pass.

---

### Task 3: Project Detection and Artifacts

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `ProjectInspection`, `inspect_project(environment_id: String)` Tauri command.
- Consumes: `EnvironmentConfig.host_path`.

- [ ] **Step 1: Write failing fixture test**

Create a temporary project with `Makefile`, `.c`, `.cpp`, executable file, and `.gcov`, then assert inspection detects them.

- [ ] **Step 2: Run failing test**

Run:

```bash
cargo test detects_project_shape_and_artifacts
```

Expected: FAIL because inspection does not exist.

- [ ] **Step 3: Implement inspection**

Scan the host path recursively with a shallow limit, count extensions, detect common Make targets, list executable files, and list `.gcov`, `.gcda`, `.gcno`, `.log` artifacts.

- [ ] **Step 4: Verify targeted test passes**

Run:

```bash
cargo test detects_project_shape_and_artifacts
```

Expected: PASS.

---

### Task 4: Docker Inspection Commands

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `show_environment_docker_logs(environment_id: String) -> Result<String, String>`, `show_environment_compose_config(environment_id: String) -> Result<String, String>`.
- Consumes: existing `run_environment_compose`.

- [ ] **Step 1: Implement commands**

Add commands that run `docker compose logs --tail=200` and `docker compose config` in the environment directory.

- [ ] **Step 2: Register commands**

Register both in `tauri::generate_handler!`.

- [ ] **Step 3: Verify compile**

Run:

```bash
cargo check
```

Expected: PASS.

---

### Task 5: Evaluation UI Panels

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.css`

**Interfaces:**
- Consumes: `inspect_project`, `list_evaluation_history`, `run_environment_valgrind_target`, `show_environment_docker_logs`, `show_environment_compose_config`.
- Produces: focused sections `Evaluation`, `History`, `Docker`, and `Artifacts`.

- [ ] **Step 1: Add frontend types and state**

Add TypeScript types for `EvaluationRunRecord` and `ProjectInspection`, plus state for inspection, history, selected Valgrind target, and active Docker text.

- [ ] **Step 2: Refresh inspection/history with environments**

After environment load and after any evaluation command, call backend commands and update UI state.

- [ ] **Step 3: Move evaluation controls into panel**

Keep header focused on lifecycle. Add Evaluation panel with Build, Tests, Clean, Valgrind selector, Run Valgrind.

- [ ] **Step 4: Add History/Docker/Artifacts panels**

Render recent history entries, Docker logs/config buttons, executable candidates, artifacts, Makefile state, Make targets, and language counts.

- [ ] **Step 5: Add CSS**

Add compact sections, list rows, status chips, and scrollable output blocks.

- [ ] **Step 6: Verify frontend build**

Run:

```bash
npm run build
```

Expected: PASS.

---

### Task 6: Final Verification

**Files:**
- No code changes expected.

**Interfaces:**
- Consumes all tasks.

- [ ] **Step 1: Run Rust tests**

Run:

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 2: Run Rust check**

Run:

```bash
cargo check
```

Expected: PASS.

- [ ] **Step 3: Run frontend build**

Run:

```bash
npm run build
```

Expected: PASS.

- [ ] **Step 4: Inspect diff**

Run:

```bash
git diff -- src-tauri/src/main.rs src/App.tsx src/App.css docs/superpowers/specs/2026-08-05-evaluation-core-design.md docs/superpowers/plans/2026-08-05-evaluation-core.md
```

Expected: diff contains Evaluation Core only.

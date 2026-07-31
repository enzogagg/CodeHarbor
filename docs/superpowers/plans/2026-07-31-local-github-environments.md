# Local And GitHub Environments Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add selectable environments that mount Mac project folders into Ubuntu AMD64 Docker workspaces, with optional Git clone import.

**Architecture:** Rust owns environment persistence and generated Compose files under `~/.codeharbor`. React lists environments, creates new ones from local path or Git URL, and invokes lifecycle/evaluation commands against the selected environment.

**Tech Stack:** Tauri, Rust, serde JSON, React, TypeScript, Docker Compose, Git.

## Global Constraints

- Primary sync model is Docker bind mount from Mac folder to `/workspace`.
- Keep profile fixed to `epitech-cpp` for this milestone.
- Do not add GitHub OAuth, branch management, pull/push buttons, file picker dialogs, reports, or integrated terminal.
- Preserve existing prototype commands for compatibility.
- Verify with `cargo test`, `cargo check`, `npm run build`, and Docker Compose config validation.

---

## Task 1: Environment Persistence And Generation

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `EnvironmentConfig` serializable struct.
- Produces: `list_environments()`.
- Produces: `create_environment(name, host_path, github_url)`.

- [ ] Add serde dependencies.
- [ ] Add tests for environment id sanitization and generated Compose mount.
- [ ] Implement config storage under `~/.codeharbor/environments/<id>/`.
- [ ] Implement optional `git clone` into `~/.codeharbor/projects/<id>/`.

## Task 2: Selected Environment Commands

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `start_environment`, `stop_environment`, `open_environment_ide`.
- Produces: `run_environment_build`, `run_environment_tests`, `run_environment_clean`, `run_environment_valgrind`.

- [ ] Reuse evaluation scripts for generated environments.
- [ ] Execute Docker Compose in the selected environment directory.
- [ ] Keep commands on `spawn_blocking`.

## Task 3: React Environment UI

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.css`

**Interfaces:**
- Consumes: environment list/create/command APIs.
- Produces: sidebar list, add environment form, selected environment actions.

- [ ] Load environments on app startup.
- [ ] Add form fields for name, local folder path and optional Git URL.
- [ ] Wire lifecycle and evaluation buttons to selected environment.
- [ ] Display selected environment details.

## Task 4: Docs And Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/development.md`

- [ ] Document local folder and Git URL flows.
- [ ] Run `cargo test && cargo check` in `src-tauri`.
- [ ] Run `npm run build`.
- [ ] Run `docker compose -f prototype/docker-workspace/compose.yaml config >/dev/null`.

---

## Self-Review

- Spec coverage: persistence, local path, Git clone, generated Compose, UI selection and command routing are covered.
- Placeholder scan: no unresolved placeholders.
- Type consistency: command names match the design.

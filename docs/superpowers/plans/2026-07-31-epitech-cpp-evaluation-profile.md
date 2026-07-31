# Epitech C/C++ Evaluation Profile Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade the current Ubuntu AMD64 prototype into a usable Epitech C/C++ evaluation workspace.

**Architecture:** Keep the single prototype workspace. Extend the Docker image with C/C++ evaluation tools, add Rust Tauri commands that execute Makefile-oriented evaluation commands inside the running container, and expose those commands through the existing React UI.

**Tech Stack:** Docker, Ubuntu 24.04, Tauri, Rust, React, TypeScript.

## Global Constraints

- Keep the current single prototype workspace.
- Do not introduce multi-environment generation, file picker storage, SQLite, reports, or an integrated terminal.
- Preserve existing commands: `check_docker`, `start_prototype`, `stop_prototype`, and `open_ide`.
- Add commands: `run_build`, `run_tests`, `run_clean`, and `run_valgrind`.
- Return full command output to the UI.
- Verify with `npm run build`, `cargo test`, `cargo check`, and Docker Compose config validation.

---

## File Structure

- `prototype/docker-workspace/Dockerfile`: add Epitech C/C++ evaluation packages.
- `src-tauri/src/main.rs`: add command runner helpers and Tauri commands for build/tests/clean/valgrind.
- `src/App.tsx`: rename the workspace and add evaluation buttons.
- `docs/development.md`: document Epitech evaluation workflow.
- `README.md`: mention C/C++ evaluation profile.

---

### Task 1: Extend Docker Evaluation Toolchain

**Files:**
- Modify: `prototype/docker-workspace/Dockerfile`

**Interfaces:**
- Produces: an Ubuntu AMD64 image with C/C++ evaluation tools available in `/workspace`.

- [ ] **Step 1: Add apt packages**

Add `cmake`, `file`, `g++`, `gcc`, `gdb`, `gcovr`, `lcov`, `clang`, `clang-format`, `pkg-config`, `strace`, `tree`, and `valgrind` to the existing apt install list.

- [ ] **Step 2: Validate Dockerfile through Compose config**

Run: `docker compose -f prototype/docker-workspace/compose.yaml config >/dev/null`

Expected: command exits with status `0`.

---

### Task 2: Add Rust Evaluation Commands With Tests

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `run_build() -> Result<String, String>`.
- Produces: `run_tests() -> Result<String, String>`.
- Produces: `run_clean() -> Result<String, String>`.
- Produces: `run_valgrind() -> Result<String, String>`.

- [ ] **Step 1: Add failing tests for command construction**

Add unit tests for helper functions that build shell commands for `make`, `make tests_run`, clean, and valgrind.

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test evaluation_command_tests`

Working directory: `src-tauri`

Expected: tests fail because helper functions do not exist yet.

- [ ] **Step 3: Implement helper functions and Tauri commands**

Add helper functions that execute `docker compose exec -T workspace bash -lc <script>` in the prototype directory.

- [ ] **Step 4: Verify Rust tests pass**

Run: `cargo test`

Working directory: `src-tauri`

Expected: all tests pass.

---

### Task 3: Add Evaluation Actions To UI

**Files:**
- Modify: `src/App.tsx`

**Interfaces:**
- Consumes: Tauri commands `run_build`, `run_tests`, `run_clean`, and `run_valgrind`.
- Produces: buttons `Build`, `Tests`, `Valgrind`, and `Clean`.

- [ ] **Step 1: Extend command type and actions**

Add the four command names to `CommandName` and the UI action list.

- [ ] **Step 2: Rename workspace content**

Change visible title/sidebar text to `Epitech C/C++ Evaluation` and mention Makefile evaluation.

- [ ] **Step 3: Verify frontend build**

Run: `npm run build`

Expected: command exits with status `0`.

---

### Task 4: Update Docs And Final Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/development.md`

**Interfaces:**
- Produces: updated instructions for using CodeHarbor to evaluate Epitech C/C++ projects.

- [ ] **Step 1: Document evaluation flow**

Document rebuild/start, mounting `/workspace`, and using `Build`, `Tests`, `Valgrind`, and `Clean`.

- [ ] **Step 2: Run final checks**

Run: `npm run build`

Expected: command exits with status `0`.

Run: `cargo test && cargo check`

Working directory: `src-tauri`

Expected: command exits with status `0`.

Run: `docker compose -f prototype/docker-workspace/compose.yaml config >/dev/null`

Expected: command exits with status `0`.

---

## Self-Review

- Spec coverage: the plan covers Docker tools, Rust commands, UI actions, docs, and verification.
- Placeholder scan: no unresolved placeholders.
- Type consistency: command names match the spec and frontend consumers.

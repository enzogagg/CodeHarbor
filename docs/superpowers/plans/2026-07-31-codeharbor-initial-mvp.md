# CodeHarbor Initial MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first CodeHarbor repository baseline with a runnable Docker AMD64 code-server prototype and a lightweight Tauri product skeleton.

**Architecture:** The repository separates the immediately runnable Docker prototype from the future desktop application skeleton. `prototype/docker-workspace/` is executable today; `src/`, `src-tauri/`, `templates/`, and `docs/` document and reserve the future Tauri architecture without over-building it.

**Tech Stack:** Docker, Docker Compose, Ubuntu 24.04, code-server, Tauri, React, TypeScript, Rust.

## Global Constraints

- Do not overwrite the existing `LICENSE` file.
- Use Apache License 2.0 as the project license.
- Keep the initial app skeleton lightweight; do not scaffold a full Tauri app yet.
- The Docker prototype must force `linux/amd64`.
- The Docker prototype must mount `./workspace` to `/workspace`.
- The default code-server password is `dev`.
- Do not create a git commit unless the user explicitly asks for it.

---

## File Structure

- `README.md`: project overview, goals, stack, prototype launch instructions, license.
- `.gitignore`: ignore Node, Rust/Tauri, Docker local overrides, databases, logs, macOS, IDE, and temp files.
- `.env.example`: reserve future local app configuration.
- `CONTRIBUTING.md`: lightweight contribution rules and commit convention.
- `SECURITY.md`: private reporting guidance for early development.
- `CODE_OF_CONDUCT.md`: concise community expectations.
- `package.json`: lightweight project metadata and placeholder scripts for future Tauri work.
- `src/README.md`: future React frontend boundary.
- `src-tauri/README.md`: future Rust/Tauri backend boundary.
- `templates/ubuntu/README.md`: base Ubuntu workspace template notes.
- `templates/node/README.md`: Node.js workspace template notes.
- `templates/python/README.md`: Python workspace template notes.
- `templates/custom/README.md`: custom workspace template notes.
- `docs/architecture.md`: product architecture summary.
- `docs/roadmap.md`: versioned MVP roadmap.
- `docs/development.md`: local development workflow.
- `prototype/docker-workspace/compose.yaml`: runnable Compose configuration.
- `prototype/docker-workspace/Dockerfile`: Ubuntu AMD64 code-server image.
- `prototype/docker-workspace/workspace/.gitkeep`: keep the mount directory in git.

---

### Task 1: Repository Baseline Documentation

**Files:**
- Create: `README.md`
- Create: `.gitignore`
- Create: `.env.example`
- Create: `CONTRIBUTING.md`
- Create: `SECURITY.md`
- Create: `CODE_OF_CONDUCT.md`

**Interfaces:**
- Consumes: existing `LICENSE`.
- Produces: repository-level documentation and ignore rules used by every later task.

- [ ] **Step 1: Create repository documentation files**

Create the files listed above with CodeHarbor metadata, launch instructions, contribution rules, and local ignore rules.

- [ ] **Step 2: Verify documentation files exist**

Run: `test -f README.md && test -f .gitignore && test -f .env.example && test -f CONTRIBUTING.md && test -f SECURITY.md && test -f CODE_OF_CONDUCT.md`

Expected: command exits with status `0` and no output.

---

### Task 2: Docker Workspace Prototype

**Files:**
- Create: `prototype/docker-workspace/compose.yaml`
- Create: `prototype/docker-workspace/Dockerfile`
- Create: `prototype/docker-workspace/workspace/.gitkeep`

**Interfaces:**
- Consumes: project description from `README.md`.
- Produces: runnable Docker workspace launched with `docker compose up --build -d`.

- [ ] **Step 1: Create Docker prototype files**

Create a Compose service named `workspace` with `platform: linux/amd64`, local ports `8080`, `3000`, `5173`, and `8000`, persistent code-server volumes, and a mounted `./workspace:/workspace` directory.

- [ ] **Step 2: Validate Compose syntax**

Run: `docker compose config`

Working directory: `prototype/docker-workspace`

Expected: command exits with status `0` and prints normalized Compose configuration.

---

### Task 3: Product Skeleton And Templates

**Files:**
- Create: `package.json`
- Create: `src/README.md`
- Create: `src-tauri/README.md`
- Create: `templates/ubuntu/README.md`
- Create: `templates/node/README.md`
- Create: `templates/python/README.md`
- Create: `templates/custom/README.md`
- Create: `docs/architecture.md`
- Create: `docs/roadmap.md`
- Create: `docs/development.md`

**Interfaces:**
- Consumes: Docker prototype paths from Task 2.
- Produces: future application boundaries and roadmap for Tauri implementation.

- [ ] **Step 1: Create product skeleton files**

Create lightweight placeholder documentation that defines each directory's responsibility without pretending the full Tauri app exists yet.

- [ ] **Step 2: Verify skeleton files exist**

Run: `test -f package.json && test -f src/README.md && test -f src-tauri/README.md && test -f templates/ubuntu/README.md && test -f templates/node/README.md && test -f templates/python/README.md && test -f templates/custom/README.md && test -f docs/architecture.md && test -f docs/roadmap.md && test -f docs/development.md`

Expected: command exits with status `0` and no output.

---

### Task 4: Final Verification

**Files:**
- Read: `README.md`
- Read: `prototype/docker-workspace/compose.yaml`
- Read: `prototype/docker-workspace/Dockerfile`

**Interfaces:**
- Consumes: all files created by Tasks 1-3.
- Produces: verified repository baseline.

- [ ] **Step 1: Check git status**

Run: `git status --short`

Expected: newly created files are shown as untracked or modified; `LICENSE` is not modified.

- [ ] **Step 2: Validate Compose file**

Run: `docker compose config`

Working directory: `prototype/docker-workspace`

Expected: command exits with status `0`.

- [ ] **Step 3: Build Docker image if Docker is available**

Run: `docker compose build`

Working directory: `prototype/docker-workspace`

Expected: command exits with status `0`, or reports that Docker Desktop/daemon is unavailable.

---

## Self-Review

- Spec coverage: the plan covers the Docker prototype, project skeleton, documentation, ignore rules, and verification commands.
- Placeholder scan: no task uses `TBD`, `TODO`, or undefined future behavior as an implementation requirement.
- Type consistency: no code APIs are introduced in this MVP; file paths are consistent across tasks.

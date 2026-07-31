# CodeHarbor Initial MVP Design

## Purpose

CodeHarbor starts as a local Docker-based development environment manager. The first milestone combines an immediately testable Docker workspace prototype with a lightweight Tauri project skeleton for the future desktop application.

## Scope

The initial MVP includes two layers:

- A runnable Docker prototype under `prototype/docker-workspace/`.
- A minimal product repository structure for the future Tauri, React, TypeScript, and Rust application.

The MVP does not implement the full desktop UI, Docker orchestration backend, integrated terminal, or SQLite storage yet. Those belong to later milestones.

## Docker Prototype

The Docker prototype provides one Ubuntu 24.04 AMD64 workspace running `code-server` in the browser.

It includes:

- Ubuntu x86_64 via `platform: linux/amd64`.
- `code-server` exposed on `127.0.0.1:8080`.
- Common development tools: Git, curl, build tools, Python, sudo, SSH client, nano, unzip, wget.
- A local mounted `workspace/` directory at `/workspace`.
- Persistent code-server data, configuration, and shell history volumes.
- Additional forwarded ports for common dev servers: `3000`, `5173`, and `8000`.

The expected validation command inside the integrated terminal is `uname -m`, which should return `x86_64`.

## Product Skeleton

The repository includes documentation and placeholders that define the intended future application without over-building it.

Planned structure:

- `src/` for the future React frontend.
- `src-tauri/` for the future Rust/Tauri backend.
- `templates/` for reusable environment templates.
- `docs/` for architecture, roadmap, and development notes.
- Project governance files: `CONTRIBUTING.md`, `SECURITY.md`, and `CODE_OF_CONDUCT.md`.

The initial `package.json` documents the planned frontend stack and scripts, but the repository remains lightweight until the official Tauri scaffold is initialized.

## Future Architecture

The desktop application will use:

- Tauri for the native desktop shell.
- React and TypeScript for the UI.
- Rust commands for local Docker CLI orchestration.
- Docker Compose files generated per environment.
- JSON or SQLite storage for environment metadata.
- `code-server` for the browser-based IDE.

## MVP User Flow

For the first prototype, the user manually runs:

```bash
cd prototype/docker-workspace
docker compose up --build -d
```

Then opens:

```text
http://localhost:8080
```

The default password is:

```text
dev
```

## Error Handling

The prototype relies on Docker Compose errors for missing Docker Desktop, image build failures, port conflicts, and volume permission problems. The README documents the most likely checks.

Future application error handling will detect Docker availability, compose failures, port conflicts, missing paths, and unsupported architectures before launching an environment.

## Testing And Verification

Initial verification is manual and command-based:

- `docker compose config` validates the Compose file.
- `docker compose build` validates the Dockerfile.
- `docker compose up -d` starts the workspace.
- `uname -m` inside code-server terminal confirms `x86_64`.

Later milestones should add unit tests for environment configuration generation and integration tests around Docker command execution.

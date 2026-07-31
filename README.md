# CodeHarbor

CodeHarbor is a local development environment manager powered by Docker.

It helps developers create, configure and run isolated development workspaces through a simple desktop application. The first prototype is a Docker Compose workspace running Ubuntu AMD64 with code-server.

## Tagline

Your local harbor for containerized development environments.

## Goals

- Create reproducible Docker-based development environments
- Support AMD64 and ARM64 workspaces
- Mount local project directories
- Manage ports, volumes and environment variables
- Start, stop and delete development environments
- Open a browser-based IDE or an interactive terminal
- Generate reusable Docker Compose configurations

## Prototype

The current prototype lives in `prototype/docker-workspace/`.

It provides:

- Ubuntu 24.04
- AMD64 execution through Docker Desktop emulation on Apple Silicon
- code-server exposed on `http://localhost:8080`
- Git, build tools, Python, sudo, SSH client and common shell utilities
- A local `workspace/` directory mounted to `/workspace`
- Persistent code-server data and configuration volumes

### Run The Prototype

```bash
cd prototype/docker-workspace
docker compose up --build -d
```

Open:

```text
http://localhost:8080
```

Password:

```text
dev
```

Verify the architecture inside the integrated terminal:

```bash
uname -m
```

Expected result:

```text
x86_64
```

### Stop The Prototype

```bash
cd prototype/docker-workspace
docker compose down
```

## Planned Stack

- Tauri
- React
- TypeScript
- Rust
- Docker
- Docker Compose
- xterm.js
- SQLite or JSON storage

## Project Status

CodeHarbor is in early development. The Docker prototype is the first runnable milestone; the desktop application skeleton is intentionally lightweight.

## License

Licensed under the Apache License 2.0.

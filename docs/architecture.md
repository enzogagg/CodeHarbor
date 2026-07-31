# Architecture

CodeHarbor is designed as a small desktop manager around Docker and Docker Compose.

## Layers

- Desktop shell: Tauri
- Interface: React and TypeScript
- Local backend: Rust Tauri commands
- Runtime: Docker and Docker Compose
- IDE: code-server inside each workspace container
- Storage: SQLite or JSON metadata in a local CodeHarbor directory

## Environment Model

Each environment maps to a generated directory:

```text
~/.codeharbor/environments/<environment-id>/
├── compose.yaml
├── Dockerfile
├── config.json
└── data/
```

The first prototype keeps this simpler and runs from `prototype/docker-workspace/`.

## Boundaries

CodeHarbor should not reimplement Docker, VS Code or a terminal emulator from scratch. It should generate configuration, call local tools, expose status clearly and open the right URLs.

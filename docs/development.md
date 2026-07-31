# Development

## Requirements

- macOS
- Docker Desktop
- Docker Compose v2
- Node.js for future frontend development
- Rust for future Tauri backend development

## Run The Docker Prototype

```bash
cd prototype/docker-workspace
docker compose up --build -d
```

Open `http://localhost:8080` and sign in with password `dev`.

## Validate Compose

```bash
cd prototype/docker-workspace
docker compose config
```

## Stop The Docker Prototype

```bash
cd prototype/docker-workspace
docker compose down
```

## Future App Development

The Tauri application is not scaffolded yet. When the project is ready for the desktop app, initialize Tauri with React and TypeScript, then wire Rust commands to Docker CLI operations.

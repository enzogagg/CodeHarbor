# Development

## Requirements

- macOS
- Docker Desktop
- Docker Compose v2
- Node.js
- Rust

## Install Dependencies

```bash
npm install
```

## Run The Desktop App

```bash
npm run tauri:dev
```

The app can verify Docker, start the prototype workspace, stop it and open code-server at `http://localhost:8080`.

## Build Checks

```bash
npm run build
```

```bash
cd src-tauri
cargo check
```

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

## Verify The Workspace Architecture

After starting the workspace from the app or with Docker Compose, open `http://localhost:8080` and run:

```bash
uname -m
```

Expected result:

```text
x86_64
```

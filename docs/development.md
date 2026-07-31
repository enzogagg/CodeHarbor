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

The app can verify Docker, start the Epitech C/C++ prototype workspace, stop it, open code-server at `http://localhost:8080`, and run evaluation commands.

## Epitech Evaluation Flow

1. Create an environment from a local student project folder or a Git URL.
2. Start the desktop app with `npm run tauri:dev`.
3. Click `Démarrer` to build and start the Ubuntu AMD64 container.
4. Click `Build` to run `make` in `/workspace`.
5. Click `Tests` to run `make tests_run` in `/workspace`.
6. Click `Valgrind` to run Valgrind when a single executable can be detected.
7. Click `Clean` to run `make fclean` and `make clean` when available.

The output panel shows the full command result returned by Docker Compose.

## Local Folder Sync

The main workflow is direct Docker volume mounting:

```text
/Users/me/Dev/student-project:/workspace
```

Code on macOS with your usual editor. Compile and execute in Ubuntu through CodeHarbor.

## Git Import

When a Git URL is provided, CodeHarbor runs `git clone` and stores the project in:

```text
~/.codeharbor/projects/<environment-id>/
```

It then generates environment files in:

```text
~/.codeharbor/environments/<environment-id>/
```

This first version uses your existing local Git authentication. There is no GitHub OAuth yet.

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

## Verify Evaluation Tools

Inside code-server or with `docker compose exec`, verify:

```bash
gcc --version
g++ --version
make --version
valgrind --version
```

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

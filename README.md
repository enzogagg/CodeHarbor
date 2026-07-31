# CodeHarbor

CodeHarbor is a local development environment manager powered by Docker.

It helps developers create, configure and run isolated development workspaces through a simple desktop application. The first profile targets Epitech C/C++ evaluation on Ubuntu AMD64 with code-server.

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

## Desktop App

CodeHarbor now includes a Tauri desktop MVP that controls the Docker prototype.

### Install Dependencies

```bash
npm install
```

### Run Frontend Build Check

```bash
npm run build
```

### Run The Desktop App

```bash
npm run tauri:dev
```

Use the app buttons to:

- Verify Docker availability
- Start the Epitech C/C++ Ubuntu AMD64 workspace
- Stop the workspace
- Open the browser IDE
- Run `make`
- Run `make tests_run`
- Run Valgrind against a detected executable
- Clean generated files through Makefile targets

### Add An Environment

CodeHarbor can create an environment from either a local folder or a Git URL.

Local folder flow:

```text
Name: MyFTP
Local folder path: /Users/me/Dev/students/myftp
Git URL optional: empty
```

Git flow:

```text
Name: MyFTP
Local folder path: empty
Git URL optional: git@github.com:org/myftp.git
```

Generated environments are stored in:

```text
~/.codeharbor/environments/<environment-id>/
```

Git clones are stored in:

```text
~/.codeharbor/projects/<environment-id>/
```

The selected project folder is mounted directly into the container as `/workspace`, so edits made on macOS are immediately visible inside Ubuntu.

## Epitech C/C++ Evaluation

The prototype workspace is designed to reduce macOS/Linux compatibility bias when evaluating student projects.

It includes:

- `gcc` and `g++`
- `make` and `cmake`
- `gdb`, `valgrind` and `strace`
- `clang` and `clang-format`
- `gcovr` and `lcov`
- `tree`, `file` and `pkg-config`

Create an environment from a student project folder, start the workspace, then use the app actions:

- `Build` runs `make`
- `Tests` runs `make tests_run`
- `Valgrind` tries to detect an executable and run Valgrind
- `Clean` runs `make fclean` and `make clean` when available

## Docker Prototype

The current prototype lives in `prototype/docker-workspace/`.

It provides:

- Ubuntu 24.04
- AMD64 execution through Docker Desktop emulation on Apple Silicon
- code-server exposed on `http://localhost:8080`
- Git, C/C++ build tools, debug tools, Python, sudo, SSH client and common shell utilities
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

CodeHarbor is in early development. The Docker prototype is runnable, and the Tauri desktop MVP can start, stop, open and run basic Epitech C/C++ evaluation commands inside the workspace.

## License

Licensed under the Apache License 2.0.

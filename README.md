# CodeHarbor

CodeHarbor is a macOS desktop app for creating and running Ubuntu AMD64 Docker workspaces from local folders or Git repositories. Its first workflow focuses on fair Epitech C/C++ project evaluation from an Apple Silicon or Intel Mac.

## What It Does

- Creates one isolated Docker Compose environment per project.
- Mounts the selected macOS project folder into the container as `/workspace`.
- Forces the workspace platform to `linux/amd64` for reproducible Ubuntu evaluation.
- Opens code-server in the browser for in-container editing and terminal access.
- Runs common evaluation commands: Build, Tests, Clean, and Valgrind.
- Records evaluation history and generates Markdown evaluation reports.
- Keeps student/project files when deleting generated CodeHarbor environments.

## Requirements

- macOS
- Docker Desktop or another Docker engine with Docker Compose v2
- Node.js
- Rust and Cargo

## Install Dependencies

```bash
npm install
```

## Run In Development

```bash
npm run tauri:dev
```

The dev command cleans stale CodeHarbor/Tauri/Vite dev processes before starting the app.

## Install As A macOS App

```bash
npm run mac:install
```

This builds the Tauri app and installs it to:

```text
~/Applications/CodeHarbor.app
```

After installation, launch CodeHarbor from Finder, Spotlight, the Dock, or:

```bash
open ~/Applications/CodeHarbor.app
```

To remove only the installed macOS app bundle:

```bash
npm run mac:uninstall
```

Uninstalling the app does not delete `~/.codeharbor` projects, environments, Docker containers, or Docker volumes.

## Create An Environment

Use a local project folder:

```text
Name: MyFTP
Local folder path: /Users/me/Dev/students/myftp
Git URL optional: empty
```

Or clone from Git:

```text
Name: MyFTP
Local folder path: empty
Git URL optional: git@github.com:org/myftp.git
```

Generated environment files live in:

```text
~/.codeharbor/environments/<environment-id>/
```

Git clones live in:

```text
~/.codeharbor/projects/<environment-id>/
```

## Environment Actions

- `Démarrer`: builds and starts the Docker workspace.
- `Arrêter`: stops the workspace.
- `Ouvrir IDE`: opens code-server in the browser.
- `Finder`: opens the project folder on macOS.
- `Docker`: checks Docker availability.
- `Diagnostics`: shows local diagnostics including Docker and dev port state.
- `Supprimer`: deletes generated environment files after confirmation.

Deleting an environment removes `~/.codeharbor/environments/<environment-id>/` but keeps the project folder or Git clone.

## Evaluation Actions

- `Build`: runs `make` in `/workspace`.
- `Tests`: runs `make tests_run` in `/workspace`.
- `Clean`: runs `make fclean` and `make clean` when available.
- `Run Valgrind`: runs Valgrind against the selected detected executable.

The Evaluation panels show:

- `History`: recent recorded evaluation runs.
- `Artifacts`: detected executables, coverage files, logs, and language counts.
- `Docker`: recent Docker logs and Compose config.
- `Reports`: local Markdown report generation and opening actions.

## Reports

Generate reports from the `Reports` panel. Reports are written to:

```text
~/.codeharbor/environments/<environment-id>/reports/
```

Reports include environment metadata, project inspection, evaluation history, command outputs, Valgrind entries, Docker config/logs, and manual review notes. They are supporting evidence, not an automated grade.

## Validate The Project

```bash
npm run test:all
```

This runs the frontend build, Rust unit tests, and Rust compile check.

## License

Licensed under the Apache License 2.0.

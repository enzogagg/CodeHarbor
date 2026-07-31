# Tauri Desktop MVP Design

## Purpose

This milestone turns CodeHarbor from a Docker prototype repository into a runnable desktop application. The app should provide a simple interface around the existing Ubuntu AMD64 code-server prototype without introducing multi-environment management yet.

## Scope

The MVP includes:

- A real Tauri application scaffold using npm, Vite, React and TypeScript.
- A Rust Tauri backend with commands for Docker checks and prototype lifecycle actions.
- A single-screen UI for the existing `prototype/docker-workspace/` environment.
- Scripts for development, frontend build and Tauri launch.

The MVP does not include SQLite, user-created environments, template editing, integrated terminal, account management or packaged release automation.

## User Interface

The initial app shows one workspace card:

- Product title: `CodeHarbor`.
- Workspace name: `Ubuntu AMD64 Workspace`.
- Description: Ubuntu 24.04, AMD64, code-server.
- Docker/prototype status message.
- Actions: `Démarrer`, `Arrêter`, `Ouvrir IDE`.
- Feedback area for command output or errors.

The UI should be clear and minimal. It should not pretend to support multiple projects before the backend can generate and track them.

## Frontend Architecture

The frontend uses Vite, React and TypeScript in `src/`.

Suggested files:

- `src/main.tsx` bootstraps React.
- `src/App.tsx` owns the MVP screen state.
- `src/App.css` contains the app styling.
- `src/vite-env.d.ts` contains Vite typings.

Frontend state remains local to `App.tsx` for this milestone.

## Backend Architecture

The backend uses Tauri Rust commands in `src-tauri/`.

Commands:

- `check_docker() -> Result<String, String>` runs `docker --version` and returns a user-readable message.
- `start_prototype() -> Result<String, String>` runs `docker compose up --build -d` in `prototype/docker-workspace/`.
- `stop_prototype() -> Result<String, String>` runs `docker compose down` in `prototype/docker-workspace/`.
- `open_ide() -> Result<String, String>` opens `http://localhost:8080` in the system browser.

The backend should compute the prototype path relative to the repository root during development. If the path cannot be found, commands return a clear error.

## Data Flow

The React UI invokes Tauri commands using `@tauri-apps/api/core`.

Each command updates a shared message area:

- Success messages explain what happened.
- Failure messages include the command failure reason where possible.
- Buttons are disabled while a command is running to avoid overlapping Docker operations.

## Error Handling

The backend handles:

- Docker CLI missing.
- Docker daemon unavailable.
- Docker Compose failures.
- Missing prototype directory.
- Browser open failures.

The frontend displays errors as text and keeps the app usable.

## Testing And Verification

Verification commands:

- `npm install` installs frontend and Tauri dependencies.
- `npm run build` verifies the React/TypeScript frontend build.
- `cargo check` in `src-tauri/` verifies the Rust backend.
- `npm run tauri dev` launches the desktop application.
- From the app, `Démarrer` starts the prototype and `Ouvrir IDE` opens code-server.
- Inside code-server, `uname -m` should return `x86_64`.

## Commit Strategy

The design document is committed separately. Implementation changes are committed after build checks pass.

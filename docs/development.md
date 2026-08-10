# Development Guide

This guide explains how to develop, test, and manually verify CodeHarbor.

## Requirements

- macOS
- Docker Desktop or a compatible Docker engine
- Docker Compose v2
- Node.js
- Rust and Cargo

## Install Dependencies

```bash
npm install
```

## Repository Structure

- `src/`: React and TypeScript frontend.
- `src-tauri/`: Rust/Tauri backend, commands, app configuration, and unit tests.
- `prototype/docker-workspace/`: Ubuntu AMD64 Docker workspace prototype.
- `scripts/`: local development, testing, and macOS install scripts.
- `docs/superpowers/specs/`: design specs.
- `docs/superpowers/plans/`: implementation plans.

## Development Commands

```bash
npm run tauri:dev
```

Starts the app in development mode. The command first runs `scripts/clean-dev.mjs` to stop stale CodeHarbor/Tauri/Vite dev processes that can keep port `1420` busy.

```bash
npm run mac:install
```

Builds and installs `CodeHarbor.app` into `~/Applications` for normal macOS launching.

```bash
npm run mac:uninstall
```

Removes only `~/Applications/CodeHarbor.app`. It leaves `~/.codeharbor`, Docker containers, and Docker volumes untouched.

## Automated Verification

Run the complete local validation suite:

```bash
npm run test:all
```

It runs, in order:

1. `npm run build`
2. `cargo test` in `src-tauri`
3. `cargo check` in `src-tauri`

Run individual checks when iterating:

```bash
npm run build
npm run test:scripts
cd src-tauri && cargo test
cd src-tauri && cargo check
```

## Automated Test Coverage

Rust unit tests cover backend behavior including:

- environment ID sanitization
- Docker Compose generation
- environment deletion safety
- environment status mapping
- IDE port selection
- evaluation command scripts
- history persistence
- project inspection and artifact detection
- safe Valgrind target validation
- report file generation and report name validation

Node script tests cover:

- command runner success/failure behavior
- macOS app bundle path resolution
- `~/Applications/CodeHarbor.app` install target resolution
- `~/Applications/CodeHarbor.app` uninstall target and safe removal behavior

Frontend coverage in this batch is TypeScript and Vite build validation. There is no frontend unit test framework yet.

## Manual Verification Checklist

Use this checklist when validating features that require Docker or macOS UI integration:

1. Run `npm run tauri:dev`.
2. Create an environment from a local folder.
3. Create an environment from a Git URL if Git credentials are available.
4. Start the environment and confirm Docker shows a `codeharbor-<id>` container.
5. Open the IDE and confirm code-server loads.
6. Run `Build`, `Tests`, `Clean`, and `Run Valgrind` on a sample Makefile project.
7. Confirm `History`, `Artifacts`, and `Docker` panels update.
8. Generate a Markdown report and open it from the app.
9. Delete the environment and confirm the project folder remains.
10. Run `npm run mac:install`, then launch `~/Applications/CodeHarbor.app` and confirm the app icon appears in the Dock and app switcher.

## Troubleshooting

### Port 1420 Is Already In Use

Run:

```bash
npm run dev:clean
```

Then restart:

```bash
npm run tauri:dev
```

### Docker Port Conflicts

CodeHarbor assigns IDE ports from `8080` upward and skips ports already in use. If Docker reports a bind conflict, stop the container using that port or create a new environment so CodeHarbor selects another free port.

### Docker Is Not Available

Start Docker Desktop or your Docker engine, then use the app's Docker or Diagnostics action.

### macOS App Build Fails

Run:

```bash
npm run tauri -- info
```

Confirm Tauri prerequisites are installed, then retry:

```bash
npm run mac:install
```

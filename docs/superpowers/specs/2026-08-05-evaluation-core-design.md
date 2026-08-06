# Evaluation Core Design

## Goal

CodeHarbor should make local project evaluation repeatable and inspectable without forcing the user to jump between Docker, Finder, and terminal commands.

## Scope

This batch adds the first evaluation core: run history, Docker logs/config, automatic project detection, Valgrind binary selection, and basic artifact listing. It does not add full Markdown/PDF reports, full Epitech compliance scoring, guided evaluator mode, profiles, templates, snapshots, or a terminal emulator.

## Backend Behavior

Evaluation actions must write one JSON history entry under `~/.codeharbor/environments/<id>/history/` for each Build, Tests, Clean, and Valgrind run. Each entry stores an ID, command kind, command line or script label, start timestamp, duration in milliseconds, success flag, stdout, and stderr/error text.

Project inspection reads the mounted host folder and reports whether a `Makefile` exists, which common Make targets are present, language counts by extension, executable candidates, and basic artifacts such as `.gcov`, `.gcda`, `.gcno`, and generated log files.

Docker inspection exposes `docker compose logs --tail=200` and `docker compose config` for the selected environment.

Valgrind execution accepts a selected executable path relative to `/workspace` and runs Valgrind against that exact target.

## UI Behavior

The header keeps only lifecycle actions. Evaluation-related controls move into focused sections:

- `Evaluation`: Build, Tests, Clean, Valgrind target selector, Run Valgrind.
- `History`: recent run entries with status, duration, and selected output.
- `Docker`: Compose config and recent Docker logs.
- `Artifacts`: detected executables, coverage files, and generated logs.

The existing output panel remains the immediate feedback area for the latest command.

## Data Model

History entries are append-only JSON files. The UI reads them through a backend command rather than directly reading files. This keeps persisted data private to the backend and avoids frontend filesystem permissions.

Executable and artifact paths must be relative to the host project root or `/workspace` display path. Backend commands must reject paths containing `..` or absolute paths for container execution.

## Error Handling

If inspection fails because the host folder is missing, the UI should display the error in the output panel and keep the environment selectable. If Docker commands fail, the returned stderr must be visible and recorded in history for evaluation commands.

## Testing

Add Rust unit tests for history serialization, project detection on a temporary fixture, safe Valgrind target validation, and artifact/executable discovery. Existing build checks must continue to pass.

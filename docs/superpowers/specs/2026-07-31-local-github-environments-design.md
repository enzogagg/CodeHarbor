# Local And GitHub Environments Design

## Purpose

CodeHarbor should let the evaluator code on macOS while compiling and running projects inside Ubuntu AMD64 Docker environments. The primary synchronization model is a direct Docker bind mount from a Mac folder to `/workspace`.

## Scope

This milestone adds environment creation and selection while keeping the existing Epitech C/C++ profile.

It includes:

- Create an environment from an existing local folder.
- Create an environment from a Git URL by cloning it locally first.
- Generate a dedicated environment directory under `~/.codeharbor/environments/<id>/`.
- Generate `compose.yaml`, `Dockerfile`, and `config.json` for each environment.
- List environments in the app sidebar.
- Run start, stop, open IDE, build, tests, clean and valgrind against the selected environment.

It does not include GitHub OAuth, repository browsing, branch management, pull/push buttons, file picker dialogs, reports, or multiple profiles.

## Environment Model

Each environment stores:

- `id`: sanitized stable identifier.
- `name`: display name.
- `profile`: `epitech-cpp`.
- `host_path`: local Mac project folder.
- `container_path`: `/workspace`.
- `ide_port`: local code-server port.
- `created_at`: creation timestamp as seconds since Unix epoch.

Generated files:

```text
~/.codeharbor/environments/<id>/
├── compose.yaml
├── Dockerfile
└── config.json
```

Git clones are stored by default in:

```text
~/.codeharbor/projects/<id>/
```

## Data Flow

Local folder flow:

```text
User enters name + host path
→ CodeHarbor validates the folder exists
→ generates environment files
→ lists it in sidebar
→ Docker mounts host_path:/workspace
```

Git URL flow:

```text
User enters name + Git URL
→ CodeHarbor runs git clone into ~/.codeharbor/projects/<id>
→ generated environment mounts cloned project into /workspace
```

## Commands

New commands:

- `list_environments() -> Result<Vec<EnvironmentConfig>, String>`.
- `create_environment(name, host_path, github_url) -> Result<EnvironmentConfig, String>`.
- `start_environment(environment_id) -> Result<String, String>`.
- `stop_environment(environment_id) -> Result<String, String>`.
- `open_environment_ide(environment_id) -> Result<String, String>`.
- `run_environment_build(environment_id) -> Result<String, String>`.
- `run_environment_tests(environment_id) -> Result<String, String>`.
- `run_environment_clean(environment_id) -> Result<String, String>`.
- `run_environment_valgrind(environment_id) -> Result<String, String>`.

Existing prototype commands may remain for backward compatibility but the UI should use selected-environment commands.

## UI

The app shows:

- Sidebar environment list.
- Add environment form with `Name`, `Local folder path`, and optional `Git URL`.
- Selected environment details.
- Existing lifecycle and evaluation actions operating on the selected environment.
- Output panel with latest command output.

## Verification

- `npm run build`.
- `cargo test`.
- `cargo check`.
- `docker compose -f prototype/docker-workspace/compose.yaml config >/dev/null`.
- Create a local-folder environment manually from the app and confirm its generated Compose file mounts the Mac folder into `/workspace`.

# Epitech C/C++ Evaluation Profile Design

## Purpose

CodeHarbor should help evaluate Epitech student projects from macOS in a reproducible Ubuntu AMD64 environment. This milestone upgrades the existing prototype into a practical C/C++ evaluation workspace.

## Scope

This milestone keeps the current single prototype workspace and does not introduce multi-environment generation, file picker storage, SQLite, reports, or an integrated terminal.

It adds:

- An Epitech C/C++ toolchain to the Ubuntu AMD64 Docker image.
- Evaluation commands exposed through Tauri.
- UI buttons for common evaluation actions.
- Full command output displayed in the existing output panel.

## Docker Image

The existing `prototype/docker-workspace/Dockerfile` remains the source of the workspace image.

It should include:

- Existing tools: code-server, Git, Python, curl, sudo, SSH client and shell utilities.
- C/C++ evaluation tools: `gcc`, `g++`, `make`, `cmake`, `gdb`, `valgrind`, `strace`, `clang`, `clang-format`, `gcovr`, `lcov`, `tree`, `file`, and `pkg-config`.

The container still runs as user `dev`, mounts `/workspace`, exposes code-server on port `8080`, and forces `linux/amd64` through Docker Compose.

## Tauri Commands

Add commands:

- `run_build() -> Result<String, String>` runs `docker compose exec -T workspace bash -lc "cd /workspace && make"`.
- `run_tests() -> Result<String, String>` runs `docker compose exec -T workspace bash -lc "cd /workspace && make tests_run"`.
- `run_clean() -> Result<String, String>` runs `docker compose exec -T workspace bash -lc "cd /workspace && (make fclean || true) && (make clean || true)"`.
- `run_valgrind() -> Result<String, String>` runs a safe helper inside `/workspace` that looks for executable files and asks the evaluator to choose a binary if it cannot infer one.

All commands use the existing prototype directory resolution and return stdout/stderr text to the UI. If the container is not running, Docker Compose should return a clear error.

## UI

Rename the workspace presentation to `Epitech C/C++ Evaluation`.

Keep existing actions:

- `Vérifier Docker`
- `Démarrer`
- `Arrêter`
- `Ouvrir IDE`

Add evaluation actions:

- `Build`
- `Tests`
- `Valgrind`
- `Clean`

The output panel displays the full result from the latest command.

## Testing

Verification commands:

- `npm run build`
- `cargo test`
- `cargo check`
- `docker compose -f prototype/docker-workspace/compose.yaml config >/dev/null`

Manual verification:

- Rebuild and start the Docker prototype.
- Open code-server.
- Confirm `uname -m` returns `x86_64`.
- Confirm `gcc --version`, `g++ --version`, `make --version`, and `valgrind --version` are available in the container.

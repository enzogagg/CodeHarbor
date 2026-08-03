# Environment Deletion Design

## Goal

CodeHarbor must let users delete an environment when it is no longer needed without risking source code loss.

## Scope

Deleting an environment removes only CodeHarbor's generated environment directory at `~/.codeharbor/environments/<id>`. It does not remove the mounted project directory or Git clone under `~/.codeharbor/projects/<id>`.

## Backend Behavior

Add a Tauri command `delete_environment(environment_id)`.

The command resolves the generated environment directory, runs `docker compose down` when a `compose.yaml` exists, removes the generated environment directory, and returns a status message. Missing Docker cleanup should surface as an error rather than silently claiming deletion succeeded.

## UI Behavior

Add a `Supprimer` action to the selected environment panel. Before invoking the backend command, show a confirmation that explains project files are kept. After successful deletion, refresh the environment list and clear the selected environment if it was deleted.

## Error Handling

If the environment directory is missing, deletion should complete idempotently by refreshing the UI. If Docker cleanup fails, show the command output in the app's output panel. Invalid environment IDs must be rejected before resolving filesystem paths.

## Testing

Add Rust unit tests proving that deletion removes the generated environment directory, keeps the workspace directory, and rejects invalid environment IDs.

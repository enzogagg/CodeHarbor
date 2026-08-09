# Documentation, Test Harness, and macOS App Launch Design

## Goal

CodeHarbor should be understandable, testable, and launchable as a normal macOS app without requiring contributors or users to remember low-level terminal commands.

## Scope

This batch adds English user/developer documentation, one command that validates the current app, and a simpler macOS launch path. It also configures the Tauri app bundle to use the existing CodeHarbor icon so the app presents correctly in the Dock and app switcher.

This batch does not add a full frontend test framework, Playwright end-to-end automation, code signing/notarization, automatic updates, a custom installer DMG, or product behavior changes.

## Documentation

`README.md` becomes the primary English user guide. It must explain:

- what CodeHarbor does
- system requirements
- how to install dependencies
- how to run the app in development mode
- how to create an environment from a local folder or Git URL
- environment lifecycle actions: start, stop, delete
- safety behavior: deleting an environment keeps project files
- opening the IDE and project folder
- evaluation actions: Build, Tests, Clean, Valgrind target selection
- inspection panels: History, Artifacts, Docker
- report export: generate reports, open latest report, open report folder
- macOS app build/install flow

`docs/development.md` becomes the English developer and testing guide. It must explain:

- dependency setup
- repository structure
- development commands
- `npm run test:all`
- individual verification commands
- what automated tests cover
- which Docker/macOS behaviors require manual checks
- common troubleshooting for port `1420`, Docker port conflicts, and stale dev processes

## Test Harness

Add a Node script `scripts/test-all.mjs` and expose it through `npm run test:all`.

The script runs these commands in order and exits on the first failure:

1. `npm run build`
2. `cargo test` in `src-tauri`
3. `cargo check` in `src-tauri`

The script prints a clear label before each step and a concise success message after all steps pass. It does not hide command output.

Existing Rust tests remain the primary automated feature coverage for backend behavior. Frontend build/typechecking remains the automated coverage for React/Tauri command wiring in this batch.

## macOS App Launch

Enable Tauri bundling and configure bundle icons from `src-tauri/icons/icon.png`. The generated app must use the CodeHarbor product name and app icon so it appears correctly in Finder, Dock, and the app switcher.

Add a script `scripts/install-mac-app.mjs` and expose it through `npm run mac:install`.

The script must:

- run `npm run tauri:build`
- locate the generated `CodeHarbor.app` under `src-tauri/target/release/bundle/macos/`
- create `~/Applications` if missing
- replace `~/Applications/CodeHarbor.app` with the newly built app
- print the installed path and a launch hint

The script installs to `~/Applications` instead of `/Applications` to avoid requiring administrator privileges.

## Error Handling

`test-all` stops on the first failed command and returns that command's exit code.

`mac:install` stops if the Tauri build fails or if the generated `.app` bundle is missing. It should print the expected bundle path in the error message.

## Testing

Add lightweight Node script tests for the command-runner logic used by `scripts/test-all.mjs` and the generated `.app` path resolution used by `scripts/install-mac-app.mjs` where practical without building the app inside unit tests.

Final verification for this batch is:

- `npm run test:all`
- `npm run mac:install` if local Tauri bundling dependencies are available

If `npm run mac:install` fails because of local bundling prerequisites, document the exact failure and keep `npm run test:all` passing.

# Documentation, Test Harness, and macOS App Launch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make CodeHarbor documented in English, testable through one validation command, and installable/launchable as a normal macOS app.

**Architecture:** Add small Node ESM scripts under `scripts/` for reusable command execution, full validation, and macOS app installation. Keep product behavior unchanged; configure Tauri bundling and update English docs to explain current features and manual verification.

**Tech Stack:** Node.js ESM scripts using built-in modules only; Node's built-in `node:test`; npm scripts; Tauri configuration in `src-tauri/tauri.conf.json`; Markdown documentation in `README.md` and `docs/development.md`.

## Global Constraints

- This batch adds English user/developer documentation, one command that validates the current app, and a simpler macOS launch path.
- This batch configures the Tauri app bundle to use the existing CodeHarbor icon so the app presents correctly in the Dock and app switcher.
- This batch does not add a full frontend test framework, Playwright end-to-end automation, code signing/notarization, automatic updates, a custom installer DMG, or product behavior changes.
- `scripts/test-all.mjs` runs `npm run build`, `cargo test` in `src-tauri`, and `cargo check` in `src-tauri` in that order.
- `scripts/test-all.mjs` stops on the first failed command and returns that command's exit code.
- `scripts/install-mac-app.mjs` installs to `~/Applications` instead of `/Applications`.
- Do not stage unrelated dirty worktree files.

---

## File Structure

- Create `scripts/command-runner.mjs`: shared `runStep` helper for child process execution with labels and inherited output.
- Create `scripts/command-runner.test.mjs`: Node unit tests for `runStep` success/failure behavior.
- Create `scripts/test-all.mjs`: sequential full validation command.
- Create `scripts/install-mac-app.mjs`: builds the Tauri app and installs `CodeHarbor.app` into `~/Applications`.
- Create `scripts/install-mac-app.test.mjs`: Node unit tests for app path and install path helpers.
- Modify `package.json`: add `test:scripts`, `test:all`, and `mac:install` scripts.
- Modify `src-tauri/tauri.conf.json`: enable bundling and configure icons from `icons/icon.png`.
- Modify `README.md`: English user guide for features and macOS install/launch.
- Modify `docs/development.md`: English developer/testing guide.

---

### Task 1: Shared Script Runner and Node Unit Tests

**Files:**
- Create: `scripts/command-runner.mjs`
- Create: `scripts/command-runner.test.mjs`
- Modify: `package.json`

**Interfaces:**
- Produces: `runStep(step: { label: string, command: string, args?: string[], cwd?: string, env?: Record<string, string> }) -> Promise<void>`
- Produces npm script: `test:scripts`

- [ ] **Step 1: Write failing Node tests**

Create `scripts/command-runner.test.mjs`:

```js
import assert from "node:assert/strict";
import { test } from "node:test";
import { runStep } from "./command-runner.mjs";

test("runStep resolves when the command exits with zero", async () => {
  await runStep({
    label: "success fixture",
    command: process.execPath,
    args: ["-e", "process.exit(0)"],
  });
});

test("runStep rejects with the exit code when the command fails", async () => {
  await assert.rejects(
    runStep({
      label: "failure fixture",
      command: process.execPath,
      args: ["-e", "process.exit(7)"],
    }),
    /failure fixture failed with exit code 7/,
  );
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run from `CodeHarbor`:

```bash
node --test scripts/command-runner.test.mjs
```

Expected: FAIL because `scripts/command-runner.mjs` does not exist.

- [ ] **Step 3: Implement the shared runner**

Create `scripts/command-runner.mjs`:

```js
import { spawn } from "node:child_process";

export function runStep({ label, command, args = [], cwd = process.cwd(), env = process.env }) {
  console.log(`\n==> ${label}`);
  console.log(`$ ${[command, ...args].join(" ")}`);

  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd,
      env,
      stdio: "inherit",
      shell: false,
    });

    child.on("error", (error) => {
      reject(new Error(`${label} failed to start: ${error.message}`));
    });

    child.on("close", (code) => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(new Error(`${label} failed with exit code ${code}`));
    });
  });
}
```

- [ ] **Step 4: Add npm script for script tests**

Modify `package.json` scripts:

```json
"test:scripts": "node --test scripts/*.test.mjs"
```

- [ ] **Step 5: Verify Node script tests pass**

Run:

```bash
npm run test:scripts
```

Expected: PASS with 2 tests passing.

---

### Task 2: Full Test Harness

**Files:**
- Create: `scripts/test-all.mjs`
- Modify: `package.json`

**Interfaces:**
- Consumes: `runStep` from `scripts/command-runner.mjs`
- Produces npm script: `test:all`

- [ ] **Step 1: Create full validation script**

Create `scripts/test-all.mjs`:

```js
#!/usr/bin/env node
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { runStep } from "./command-runner.mjs";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const tauriDir = join(root, "src-tauri");

const steps = [
  { label: "Frontend TypeScript and Vite build", command: "npm", args: ["run", "build"], cwd: root },
  { label: "Rust unit tests", command: "cargo", args: ["test"], cwd: tauriDir },
  { label: "Rust compile check", command: "cargo", args: ["check"], cwd: tauriDir },
];

try {
  for (const step of steps) {
    await runStep(step);
  }
  console.log("\nAll CodeHarbor validation steps passed.");
} catch (error) {
  console.error(`\n${error.message}`);
  process.exit(1);
}
```

- [ ] **Step 2: Add npm script**

Modify `package.json` scripts:

```json
"test:all": "node scripts/test-all.mjs"
```

- [ ] **Step 3: Verify full test harness**

Run from `CodeHarbor`:

```bash
npm run test:all
```

Expected: `npm run build`, `cargo test`, and `cargo check` all pass, with a final `All CodeHarbor validation steps passed.` message.

---

### Task 3: macOS Bundle Configuration and Install Script

**Files:**
- Create: `scripts/install-mac-app.mjs`
- Create: `scripts/install-mac-app.test.mjs`
- Modify: `package.json`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Consumes: `runStep` from `scripts/command-runner.mjs`
- Produces: `releaseBundlePath(root: string) -> string`, `userApplicationsDir(home?: string) -> string`, `installTargetPath(home?: string) -> string`
- Produces npm script: `mac:install`

- [ ] **Step 1: Write failing install script tests**

Create `scripts/install-mac-app.test.mjs`:

```js
import assert from "node:assert/strict";
import { test } from "node:test";
import { join } from "node:path";
import { installTargetPath, releaseBundlePath, userApplicationsDir } from "./install-mac-app.mjs";

test("releaseBundlePath resolves the Tauri macOS app bundle path", () => {
  assert.equal(
    releaseBundlePath("/repo"),
    join("/repo", "src-tauri", "target", "release", "bundle", "macos", "CodeHarbor.app"),
  );
});

test("installTargetPath installs into the user's Applications directory", () => {
  assert.equal(userApplicationsDir("/Users/dev"), join("/Users/dev", "Applications"));
  assert.equal(installTargetPath("/Users/dev"), join("/Users/dev", "Applications", "CodeHarbor.app"));
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
node --test scripts/install-mac-app.test.mjs
```

Expected: FAIL because `scripts/install-mac-app.mjs` does not exist.

- [ ] **Step 3: Implement macOS install script**

Create `scripts/install-mac-app.mjs`:

```js
#!/usr/bin/env node
import { existsSync, rmSync, cpSync, mkdirSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { runStep } from "./command-runner.mjs";

const root = dirname(dirname(fileURLToPath(import.meta.url)));

export function releaseBundlePath(projectRoot) {
  return join(projectRoot, "src-tauri", "target", "release", "bundle", "macos", "CodeHarbor.app");
}

export function userApplicationsDir(home = homedir()) {
  return join(home, "Applications");
}

export function installTargetPath(home = homedir()) {
  return join(userApplicationsDir(home), "CodeHarbor.app");
}

export async function installMacApp(projectRoot = root) {
  await runStep({ label: "Build CodeHarbor macOS app", command: "npm", args: ["run", "tauri:build"], cwd: projectRoot });

  const source = releaseBundlePath(projectRoot);
  if (!existsSync(source)) {
    throw new Error(`Built app bundle not found at ${source}`);
  }

  const applicationsDir = userApplicationsDir();
  const target = installTargetPath();
  mkdirSync(applicationsDir, { recursive: true });
  rmSync(target, { recursive: true, force: true });
  cpSync(source, target, { recursive: true });

  console.log(`\nInstalled CodeHarbor at ${target}`);
  console.log("Launch it from Finder, Spotlight, Dock, or with: open ~/Applications/CodeHarbor.app");
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  installMacApp().catch((error) => {
    console.error(`\n${error.message}`);
    process.exit(1);
  });
}
```

- [ ] **Step 4: Add npm script**

Modify `package.json` scripts:

```json
"mac:install": "node scripts/install-mac-app.mjs"
```

- [ ] **Step 5: Enable Tauri bundle and icon**

Modify `src-tauri/tauri.conf.json` bundle section:

```json
"bundle": {
  "active": true,
  "targets": ["app"],
  "icon": ["icons/icon.png"]
}
```

- [ ] **Step 6: Verify script tests**

Run:

```bash
npm run test:scripts
```

Expected: PASS with command-runner and install path tests passing.

- [ ] **Step 7: Verify Tauri config compiles**

Run:

```bash
npm run tauri -- info
```

Expected: command exits 0 and Tauri accepts the config. If it fails because icon formats are incomplete, run `npm run tauri -- icon src-tauri/icons/icon.png`, then update `src-tauri/tauri.conf.json` to reference the generated icon files that Tauri reports.

---

### Task 4: English User and Developer Documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/development.md`

**Interfaces:**
- Consumes: current app features and scripts from Tasks 1-3
- Produces: English user guide and developer/test guide

- [ ] **Step 1: Rewrite README as the user guide**

Replace `README.md` content with:

```markdown
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
```

- [ ] **Step 2: Rewrite development guide**

Replace `docs/development.md` content with:

```markdown
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
```

- [ ] **Step 3: Verify docs mention required commands and features**

Run:

```bash
git diff -- README.md docs/development.md
```

Expected: README includes user features and `npm run mac:install`; development guide includes `npm run test:all`, manual checklist, and troubleshooting.

---

### Task 5: Final Verification and Commit Preparation

**Files:**
- Verify: all files modified in Tasks 1-4

**Interfaces:**
- Consumes: all previous tasks
- Produces: verified working tree ready for commit

- [ ] **Step 1: Run script tests**

Run:

```bash
npm run test:scripts
```

Expected: PASS.

- [ ] **Step 2: Run full validation**

Run:

```bash
npm run test:all
```

Expected: PASS, including frontend build, Rust tests, and Rust check.

- [ ] **Step 3: Try macOS install**

Run:

```bash
npm run mac:install
```

Expected: PASS and prints `Installed CodeHarbor at .../Applications/CodeHarbor.app`. If local Tauri bundling prerequisites fail, capture the exact error and report it; do not claim install success.

- [ ] **Step 4: Inspect intended diff**

Run:

```bash
git diff -- README.md docs/development.md package.json scripts/command-runner.mjs scripts/command-runner.test.mjs scripts/test-all.mjs scripts/install-mac-app.mjs scripts/install-mac-app.test.mjs src-tauri/tauri.conf.json docs/superpowers/plans/2026-08-09-docs-tests-mac-app.md
git status --short
```

Expected: intended files are changed; unrelated pre-existing dirty files are not staged.

- [ ] **Step 5: Commit only intended files when requested**

If the user explicitly asks to commit:

```bash
git add -- README.md docs/development.md package.json scripts/command-runner.mjs scripts/command-runner.test.mjs scripts/test-all.mjs scripts/install-mac-app.mjs scripts/install-mac-app.test.mjs src-tauri/tauri.conf.json docs/superpowers/plans/2026-08-09-docs-tests-mac-app.md
git commit -m "docs: add test harness and mac app install guide"
```

Expected: commit contains only documentation, scripts, package scripts, Tauri bundle config, and this plan.

#!/usr/bin/env node
import { cpSync, existsSync, mkdirSync, rmSync } from "node:fs";
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

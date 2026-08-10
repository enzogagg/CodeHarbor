#!/usr/bin/env node
import { existsSync, rmSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

export function uninstallTargetPath(home = homedir()) {
  return join(home, "Applications", "CodeHarbor.app");
}

export function uninstallMacApp(home = homedir()) {
  const target = uninstallTargetPath(home);
  if (!existsSync(target)) {
    return false;
  }

  rmSync(target, { recursive: true, force: true });
  return true;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const target = uninstallTargetPath();
  const removed = uninstallMacApp();

  if (removed) {
    console.log(`Removed CodeHarbor from ${target}`);
  } else {
    console.log(`CodeHarbor is not installed at ${target}`);
  }
  console.log("Project data under ~/.codeharbor was left untouched.");
}

#!/usr/bin/env node
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
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

import { execFileSync } from "node:child_process";
import process from "node:process";

function output(command, args) {
  try {
    return execFileSync(command, args, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim();
  } catch {
    return "";
  }
}

function killPids(pids) {
  for (const pid of new Set(pids.filter(Boolean))) {
    if (pid === String(process.pid)) {
      continue;
    }

    const command = output("ps", ["-p", pid, "-o", "command="]);
    if (!command.includes("CodeHarbor") && !command.includes("codeharbor") && !command.includes("vite --host 127.0.0.1 --port 1420")) {
      continue;
    }

    try {
      process.kill(Number(pid), "SIGTERM");
    } catch {
      // Process already exited.
    }
  }
}

const portPids = output("lsof", ["-tiTCP:1420", "-sTCP:LISTEN"]).split(/\s+/);
const appPids = output("pgrep", ["-f", "target/debug/codeharbor"]).split(/\s+/);
const tauriPids = output("pgrep", ["-f", "CodeHarbor/node_modules/.bin/tauri dev"]).split(/\s+/);

killPids([...portPids, ...appPids, ...tauriPids]);

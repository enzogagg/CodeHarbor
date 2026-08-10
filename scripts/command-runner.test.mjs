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

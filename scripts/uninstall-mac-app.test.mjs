import assert from "node:assert/strict";
import { mkdtempSync, existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { uninstallMacApp, uninstallTargetPath } from "./uninstall-mac-app.mjs";

test("uninstallTargetPath points to the user's CodeHarbor app bundle", () => {
  assert.equal(uninstallTargetPath("/Users/dev"), join("/Users/dev", "Applications", "CodeHarbor.app"));
});

test("uninstallMacApp removes the app bundle and leaves sibling data alone", () => {
  const home = mkdtempSync(join(tmpdir(), "codeharbor-uninstall-"));
  const app = uninstallTargetPath(home);
  const sibling = join(home, "Applications", "Other.app");
  const data = join(home, ".codeharbor", "environments", "keep.txt");

  mkdirSync(app, { recursive: true });
  mkdirSync(sibling, { recursive: true });
  mkdirSync(join(home, ".codeharbor", "environments"), { recursive: true });
  writeFileSync(join(app, "marker"), "remove");
  writeFileSync(join(sibling, "marker"), "keep");
  writeFileSync(data, "keep");

  const removed = uninstallMacApp(home);

  assert.equal(removed, true);
  assert.equal(existsSync(app), false);
  assert.equal(existsSync(sibling), true);
  assert.equal(existsSync(data), true);

  rmSync(home, { recursive: true, force: true });
});

test("uninstallMacApp is a no-op when the app is not installed", () => {
  const home = mkdtempSync(join(tmpdir(), "codeharbor-uninstall-missing-"));

  const removed = uninstallMacApp(home);

  assert.equal(removed, false);
  rmSync(home, { recursive: true, force: true });
});

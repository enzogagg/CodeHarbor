import assert from "node:assert/strict";
import { join } from "node:path";
import { test } from "node:test";
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

import { readFile } from "node:fs/promises";
import { test } from "node:test";
import assert from "node:assert/strict";

test("workspace navigation shows a readable status badge", async () => {
  const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");
  const css = await readFile(new URL("../src/App.css", import.meta.url), "utf8");

  assert.match(app, /className=\{`workspace-status-badge/);
  assert.match(app, /const environmentStatus = runtimeStatuses\[environment\.id\]\?\.status \?\? "not_created"/);
  assert.match(app, /statusLabels\[environmentStatus\]/);
  assert.doesNotMatch(app, /className="workspace-glyph"/);
  assert.match(css, /\.workspace-status-badge/);
});

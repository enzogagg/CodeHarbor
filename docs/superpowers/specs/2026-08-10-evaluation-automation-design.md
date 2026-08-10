# Evaluation Automation Design

## Goal

CodeHarbor should provide a repeatable demo/evaluation path that exercises the current evaluation features without requiring a real student project or a sequence of manual clicks.

## Scope

This batch adds a sample Epitech-style C project and a one-click full evaluation action. It does not add scoring, profiles, snapshots, terminal integration, or a full automated Docker end-to-end test suite.

## Sample Project

Add `fixtures/epitech-c-sample/` with:

- `Makefile` targets: `all`, `clean`, `fclean`, `tests_run`
- a small C source file that builds an executable
- a simple test script or Make target that exits successfully
- a fixture README explaining how to create a CodeHarbor environment from the fixture and which UI features it should exercise

The fixture must be intentionally small, deterministic, and safe to run repeatedly.

## Full Evaluation Backend

Add a Tauri command:

```rust
run_full_evaluation(environment_id: String, target_path: Option<String>) -> Result<String, String>
```

The command orchestrates existing backend behavior:

1. Clean
2. Build
3. Tests
4. Valgrind when `target_path` is present and valid
5. Markdown report generation

Build failure stops the sequence after recording the build result. Tests or Valgrind failure do not prevent report generation. Report generation is attempted at the end and its success/failure is included in the summary.

The command returns a concise multiline summary including each step status and the generated report name when available.

## Full Evaluation UI

Add `Run full evaluation` to the existing Evaluation panel.

The button:

- is disabled when no environment is selected or another command is busy
- passes the selected Valgrind target when one is selected
- runs without Valgrind when no target is selected
- refreshes history, project inspection, and reports after completion
- writes the backend summary to the existing output panel

## Error Handling

Unsafe Valgrind target paths must be rejected by the existing path validator. Build failures should return a successful orchestration response only if they were captured as a normal evaluation failure; unexpected orchestration errors still return `Err`.

If report generation fails, the summary must say so clearly while preserving the step results that ran before it.

## Testing

Add Rust unit tests for the orchestration summary/status helper without requiring Docker. Existing command-level tests must continue to pass.

Manual smoke testing uses the fixture:

1. create an environment from `fixtures/epitech-c-sample/`
2. start the environment
3. run full evaluation
4. confirm History, Artifacts, Docker, and Reports update

Final automated verification remains `npm run test:all`.

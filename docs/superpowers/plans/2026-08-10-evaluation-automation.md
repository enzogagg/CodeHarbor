# Evaluation Automation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a repeatable sample project and a one-click full evaluation action that runs the current evaluation pipeline and generates a report.

**Architecture:** Reuse the existing Rust/Tauri command helpers for Clean, Build, Tests, Valgrind, and report generation. Add a small orchestration summary model that is unit-tested without Docker, then expose `run_full_evaluation` to the React UI and document a manual fixture smoke test.

**Tech Stack:** Rust/Tauri in `src-tauri/src/main.rs`; React/TypeScript in `src/App.tsx`; CSS in `src/App.css`; sample C fixture in `fixtures/epitech-c-sample/`; Markdown docs in `README.md` and `docs/development.md`; verification with `npm run test:all`.

## Global Constraints

- This batch adds a sample Epitech-style C project and a one-click full evaluation action.
- This batch does not add scoring, profiles, snapshots, terminal integration, or a full automated Docker end-to-end test suite.
- `run_full_evaluation(environment_id: String, target_path: Option<String>) -> Result<String, String>` orchestrates Clean, Build, Tests, optional Valgrind, and Markdown report generation.
- Build failure stops the sequence after recording the build result.
- Tests or Valgrind failure do not prevent report generation.
- Unsafe Valgrind target paths must be rejected by the existing path validator.
- Final automated verification remains `npm run test:all`.
- Do not stage unrelated dirty worktree files.

---

## File Structure

- Create `fixtures/epitech-c-sample/Makefile`: deterministic C sample build/test/clean targets.
- Create `fixtures/epitech-c-sample/src/main.c`: tiny executable used by Build and Valgrind.
- Create `fixtures/epitech-c-sample/README.md`: manual smoke test instructions.
- Modify `src-tauri/src/main.rs`: add full-evaluation step summary helpers, orchestration function, Tauri command, registration, and unit tests.
- Modify `src/App.tsx`: add `run_full_evaluation` command state/handler/button.
- Modify `README.md` and `docs/development.md`: document fixture and full evaluation flow.

---

### Task 1: Fixture Project

**Files:**
- Create: `fixtures/epitech-c-sample/Makefile`
- Create: `fixtures/epitech-c-sample/src/main.c`
- Create: `fixtures/epitech-c-sample/README.md`

**Interfaces:**
- Produces a project with executable `codeharbor_sample` and Make targets `all`, `clean`, `fclean`, `tests_run`.

- [ ] **Step 1: Add fixture files**

Create `fixtures/epitech-c-sample/Makefile`:

```makefile
NAME = codeharbor_sample
CC = gcc
CFLAGS = -Wall -Wextra -Werror -g
SRC = src/main.c

all: $(NAME)

$(NAME): $(SRC)
	$(CC) $(CFLAGS) -o $(NAME) $(SRC)

tests_run: $(NAME)
	./$(NAME) --self-test

clean:
	rm -f *.gcda *.gcno *.gcov *.log

fclean: clean
	rm -f $(NAME)

re: fclean all

.PHONY: all tests_run clean fclean re
```

Create `fixtures/epitech-c-sample/src/main.c`:

```c
#include <stdio.h>
#include <string.h>

static int add(int left, int right)
{
    return left + right;
}

int main(int argc, char **argv)
{
    if (argc == 2 && strcmp(argv[1], "--self-test") == 0) {
        if (add(20, 22) != 42) {
            fprintf(stderr, "self-test failed\n");
            return 84;
        }
        puts("self-test passed");
        return 0;
    }

    puts("CodeHarbor sample ready");
    return 0;
}
```

Create `fixtures/epitech-c-sample/README.md`:

```markdown
# CodeHarbor Epitech C Sample

This fixture is a tiny deterministic C project for manually smoke-testing CodeHarbor.

## Use It In CodeHarbor

1. Create an environment.
2. Set `Name` to `CodeHarbor Sample`.
3. Set `Local folder path` to this fixture directory.
4. Start the environment.
5. Run `Run full evaluation`.

Expected results:

- `Build` creates `codeharbor_sample`.
- `Tests` prints `self-test passed`.
- `Valgrind` can run against `codeharbor_sample` after it is detected.
- `History`, `Artifacts`, `Docker`, and `Reports` update in the app.
```

- [ ] **Step 2: Verify fixture locally without Docker**

Run from `CodeHarbor/fixtures/epitech-c-sample`:

```bash
make fclean && make && make tests_run && make fclean
```

Expected: compile succeeds, `self-test passed` is printed, and cleanup removes `codeharbor_sample`.

---

### Task 2: Full Evaluation Summary Helpers

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `FullEvaluationStep`, `format_full_evaluation_summary(steps: &[FullEvaluationStep], report: Result<&ReportFile, &String>) -> String`, `should_stop_full_evaluation_after_step(step: &FullEvaluationStep) -> bool`

- [ ] **Step 1: Add failing Rust tests**

Add these tests inside the existing `#[cfg(test)] mod tests` block:

```rust
#[test]
fn full_evaluation_summary_includes_step_statuses_and_report() {
    let steps = vec![
        FullEvaluationStep { name: "Clean".into(), success: true, details: "clean ok".into(), stops_pipeline: false },
        FullEvaluationStep { name: "Build".into(), success: true, details: "build ok".into(), stops_pipeline: true },
        FullEvaluationStep { name: "Tests".into(), success: false, details: "tests failed".into(), stops_pipeline: false },
    ];
    let report = ReportFile { name: "report-1.md".into(), path: "/tmp/report-1.md".into(), created_at: 1, size_bytes: 10 };

    let summary = format_full_evaluation_summary(&steps, Ok(&report));

    assert!(summary.contains("Full evaluation summary"));
    assert!(summary.contains("Clean: OK"));
    assert!(summary.contains("Tests: FAIL"));
    assert!(summary.contains("Report: report-1.md"));
}

#[test]
fn full_evaluation_stops_only_after_failed_stop_step() {
    let build_failure = FullEvaluationStep { name: "Build".into(), success: false, details: "build failed".into(), stops_pipeline: true };
    let tests_failure = FullEvaluationStep { name: "Tests".into(), success: false, details: "tests failed".into(), stops_pipeline: false };

    assert!(should_stop_full_evaluation_after_step(&build_failure));
    assert!(!should_stop_full_evaluation_after_step(&tests_failure));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run from `CodeHarbor/src-tauri`:

```bash
cargo test full_evaluation
```

Expected: FAIL because `FullEvaluationStep`, `format_full_evaluation_summary`, and `should_stop_full_evaluation_after_step` do not exist.

- [ ] **Step 3: Implement summary helpers**

Add near the report helpers:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
struct FullEvaluationStep {
    name: String,
    success: bool,
    details: String,
    stops_pipeline: bool,
}

fn should_stop_full_evaluation_after_step(step: &FullEvaluationStep) -> bool {
    step.stops_pipeline && !step.success
}

fn format_full_evaluation_summary(
    steps: &[FullEvaluationStep],
    report: Result<&ReportFile, &String>,
) -> String {
    let mut summary = String::from("Full evaluation summary\n");
    for step in steps {
        summary.push_str(&format!(
            "- {}: {} - {}\n",
            step.name,
            if step.success { "OK" } else { "FAIL" },
            step.details.lines().next().unwrap_or("")
        ));
    }
    match report {
        Ok(report) => summary.push_str(&format!("- Report: {}\n", report.name)),
        Err(error) => summary.push_str(&format!("- Report: FAIL - {error}\n")),
    }
    summary
}
```

- [ ] **Step 4: Verify targeted tests pass**

Run:

```bash
cargo test full_evaluation
```

Expected: PASS.

---

### Task 3: Backend Full Evaluation Command

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Consumes: `run_recorded_environment_script`, `clean_script`, `build_script`, `tests_script`, `validate_workspace_relative_path`, `valgrind_target_script`, `generate_report_file`, summary helpers from Task 2
- Produces Tauri command: `run_full_evaluation(environment_id: String, target_path: Option<String>) -> Result<String, String>`

- [ ] **Step 1: Implement orchestration helper**

Add:

```rust
fn recorded_step(
    environment_id: &str,
    name: &str,
    command: &str,
    label: &str,
    script: &str,
    stops_pipeline: bool,
) -> FullEvaluationStep {
    match run_recorded_environment_script(environment_id, command, label, script) {
        Ok(output) => FullEvaluationStep { name: name.into(), success: true, details: output, stops_pipeline },
        Err(error) => FullEvaluationStep { name: name.into(), success: false, details: error, stops_pipeline },
    }
}

fn run_full_evaluation_inner(environment_id: &str, target_path: Option<String>) -> Result<String, String> {
    let mut steps = Vec::new();

    let clean = recorded_step(environment_id, "Clean", "clean", "make clean", clean_script(), false);
    steps.push(clean);

    let build = recorded_step(environment_id, "Build", "build", "make", build_script(), true);
    let stop_after_build = should_stop_full_evaluation_after_step(&build);
    steps.push(build);

    if !stop_after_build {
        let tests = recorded_step(environment_id, "Tests", "tests", "make tests_run", tests_script(), false);
        steps.push(tests);

        match target_path.filter(|target| !target.trim().is_empty()) {
            Some(target) => {
                validate_workspace_relative_path(&target)?;
                let valgrind = recorded_step(
                    environment_id,
                    "Valgrind",
                    "valgrind",
                    &format!("valgrind {target}"),
                    &valgrind_target_script(&target),
                    false,
                );
                steps.push(valgrind);
            }
            None => steps.push(FullEvaluationStep {
                name: "Valgrind".into(),
                success: true,
                details: "skipped: no target selected".into(),
                stops_pipeline: false,
            }),
        }
    }

    let report = generate_report_file(environment_id);
    match &report {
        Ok(report) => Ok(format_full_evaluation_summary(&steps, Ok(report))),
        Err(error) => Ok(format_full_evaluation_summary(&steps, Err(error))),
    }
}
```

- [ ] **Step 2: Add Tauri command and register it**

Add near other evaluation commands:

```rust
#[tauri::command]
async fn run_full_evaluation(environment_id: String, target_path: Option<String>) -> Result<String, String> {
    run_blocking_task(move || run_full_evaluation_inner(&environment_id, target_path)).await
}
```

Add `run_full_evaluation,` to `tauri::generate_handler![...]`.

- [ ] **Step 3: Verify backend compile/tests**

Run from `CodeHarbor/src-tauri`:

```bash
cargo test full_evaluation
cargo check
```

Expected: PASS.

---

### Task 4: React UI Full Evaluation Action

**Files:**
- Modify: `src/App.tsx`

**Interfaces:**
- Consumes Tauri command `run_full_evaluation`
- Produces `runFullEvaluation()` handler and `Run full evaluation` button

- [ ] **Step 1: Update command state type**

Add `"run_full_evaluation"` to `CommandName`.

- [ ] **Step 2: Add handler**

Add below `runValgrindTarget`:

```tsx
async function runFullEvaluation() {
  if (!selectedEnvironment) {
    setError("Sélectionne un environnement avant de lancer l'évaluation complète.");
    return;
  }

  setBusyCommand("run_full_evaluation");
  setError(null);
  setMessage("Évaluation complète en cours...");

  try {
    const response = await invoke<string>("run_full_evaluation", {
      environmentId: selectedEnvironment.id,
      targetPath: selectedValgrindTarget || null,
    });
    await refreshEvaluation(selectedEnvironment.id);
    setMessage(response);
  } catch (caught) {
    setError(String(caught));
    setMessage("Évaluation complète interrompue.");
  } finally {
    setBusyCommand(null);
  }
}
```

- [ ] **Step 3: Add button to Evaluation panel**

Add in the Evaluation panel action area after Clean:

```tsx
<button className="action-button primary" disabled={!selectedEnvironment || busyCommand !== null} onClick={runFullEvaluation} type="button">
  {busyCommand === "run_full_evaluation" ? "Évaluation..." : "Run full evaluation"}
</button>
```

- [ ] **Step 4: Verify frontend build**

Run from `CodeHarbor`:

```bash
npm run build
```

Expected: PASS.

---

### Task 5: Documentation and Final Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/development.md`

**Interfaces:**
- Consumes fixture and UI/backend behavior from Tasks 1-4

- [ ] **Step 1: Update docs**

Add to `README.md` under Evaluation Actions:

```markdown
- `Run full evaluation`: runs Clean, Build, Tests, optional Valgrind, then generates a Markdown report.
```

Add a new `Sample Project` section:

```markdown
## Sample Project

Use `fixtures/epitech-c-sample/` to smoke-test CodeHarbor without a student project. Create an environment from that folder, start it, then run `Run full evaluation`.
```

Add to `docs/development.md` manual checklist before the existing real-project checklist:

```markdown
To smoke-test quickly, create an environment from `fixtures/epitech-c-sample/`, start it, run `Run full evaluation`, and confirm a report appears.
```

- [ ] **Step 2: Run full verification**

Run from `CodeHarbor`:

```bash
npm run test:all
```

Expected: PASS.

- [ ] **Step 3: Inspect intended diff**

Run:

```bash
git diff -- fixtures/epitech-c-sample src-tauri/src/main.rs src/App.tsx README.md docs/development.md docs/superpowers/plans/2026-08-10-evaluation-automation.md
git status --short
```

Expected: intended files are changed; unrelated dirty files remain unstaged.

- [ ] **Step 4: Commit only intended files when requested**

If the user explicitly asks to commit:

```bash
git add -- fixtures/epitech-c-sample src-tauri/src/main.rs src/App.tsx README.md docs/development.md docs/superpowers/plans/2026-08-10-evaluation-automation.md
git commit -m "feat: add full evaluation automation"
```

Expected: commit contains only fixture, full evaluation backend/UI, docs, and this plan.

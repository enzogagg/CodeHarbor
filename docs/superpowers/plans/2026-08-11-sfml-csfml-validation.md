# SFML and CSFML Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish CodeHarbor for Ubuntu AMD64 Epitech-style projects that depend on SFML or CSFML.

**Architecture:** Extend the existing Docker workspace image with SFML, CSFML, and headless display packages. Add two minimal fixtures that compile and launch through `xvfb-run` using the existing Build, Tests, Clean, Valgrind, and Run full evaluation pipeline. Keep the app UI unchanged.

**Tech Stack:** Docker Ubuntu 24.04 AMD64, `libsfml-dev`, `libcsfml-dev`, `xvfb`, `mesa-utils`, C, C++, Make, Tauri/React docs only.

## Global Constraints

- This is a validation and completion pass, not a new product feature pass.
- The app UI stays unchanged unless a small documentation label is required.
- The existing `prototype/docker-workspace/Dockerfile` remains the single source of the workspace image.
- Add `libsfml-dev`, `libcsfml-dev`, `xvfb`, and `mesa-utils` to the workspace image.
- The existing `linux/amd64` Docker Compose platform constraint remains unchanged.
- Add `fixtures/sfml-cpp-sample/` with `Makefile`, `src/main.cpp`, and `README.md`.
- Add `fixtures/csfml-c-sample/` with `Makefile`, `src/main.c`, and `README.md`.
- Fixture `tests_run` targets launch the executable under `xvfb-run` with `--self-test`.
- Fixture failure exits with `84`.
- No scoring, grading logic, interactive graphics streaming, or new UI features.
- `npm run test:all` must pass.
- `npm run mac:install` must succeed before final completion.
- Do not stage unrelated dirty worktree files.

---

## File Structure

- Modify `prototype/docker-workspace/Dockerfile`: install SFML, CSFML, and headless display packages.
- Create `fixtures/sfml-cpp-sample/Makefile`: deterministic C++ SFML build/test/clean targets.
- Create `fixtures/sfml-cpp-sample/src/main.cpp`: minimal SFML window self-test.
- Create `fixtures/sfml-cpp-sample/README.md`: manual CodeHarbor smoke-test instructions.
- Create `fixtures/csfml-c-sample/Makefile`: deterministic C CSFML build/test/clean targets.
- Create `fixtures/csfml-c-sample/src/main.c`: minimal CSFML render-window self-test.
- Create `fixtures/csfml-c-sample/README.md`: manual CodeHarbor smoke-test instructions.
- Modify `README.md`: document SFML/CSFML support and smoke-test fixtures.
- Modify `docs/development.md`: add final validation checklist for Epitech C, SFML, and CSFML.

---

### Task 1: Docker Workspace SFML/CSFML Packages

**Files:**
- Modify: `prototype/docker-workspace/Dockerfile`

**Interfaces:**
- Produces: workspace image with `libsfml-dev`, `libcsfml-dev`, `xvfb`, and `mesa-utils` installed.
- Consumes: existing Docker Compose `linux/amd64` platform constraint in `prototype/docker-workspace/compose.yaml`.

- [ ] **Step 1: Add package names to Dockerfile**

In `prototype/docker-workspace/Dockerfile`, add these packages to the existing `apt-get install -y` list, preserving alphabetical-ish grouping with the current list:

```dockerfile
    libcsfml-dev \
    libsfml-dev \
    mesa-utils \
    xvfb \
```

The resulting package block should include these names exactly once.

- [ ] **Step 2: Verify package names are present**

Run from repository root:

```bash
grep -E "libcsfml-dev|libsfml-dev|mesa-utils|xvfb" prototype/docker-workspace/Dockerfile
```

Expected: all four package names are printed.

- [ ] **Step 3: Verify Compose still forces AMD64**

Run:

```bash
grep -n "platform: linux/amd64" prototype/docker-workspace/compose.yaml
grep -n "linux/amd64" prototype/docker-workspace/compose.yaml
```

Expected: the existing `platform: linux/amd64` and build platform entry remain present.

---

### Task 2: C++ SFML Fixture

**Files:**
- Create: `fixtures/sfml-cpp-sample/Makefile`
- Create: `fixtures/sfml-cpp-sample/src/main.cpp`
- Create: `fixtures/sfml-cpp-sample/README.md`

**Interfaces:**
- Consumes: Docker image packages from Task 1.
- Produces: executable `codeharbor_sfml_sample` and Make targets `all`, `clean`, `fclean`, `tests_run`, `re`.

- [ ] **Step 1: Add SFML Makefile**

Create `fixtures/sfml-cpp-sample/Makefile`:

```makefile
NAME = codeharbor_sfml_sample
CXX = g++
CXXFLAGS = -Wall -Wextra -Werror -g
LDLIBS = -lsfml-graphics -lsfml-window -lsfml-system
SRC = src/main.cpp

all: $(NAME)

$(NAME): $(SRC)
	$(CXX) $(CXXFLAGS) -o $(NAME) $(SRC) $(LDLIBS)

tests_run: $(NAME)
	xvfb-run --auto-servernum ./$(NAME) --self-test

clean:
	rm -f *.gcda *.gcno *.gcov *.log

fclean: clean
	rm -f $(NAME)
	rm -rf $(NAME).dSYM

re: fclean all

.PHONY: all tests_run clean fclean re
```

- [ ] **Step 2: Add SFML source**

Create `fixtures/sfml-cpp-sample/src/main.cpp`:

```cpp
#include <SFML/Graphics.hpp>
#include <iostream>
#include <string>

static int run_self_test()
{
    sf::RenderWindow window(sf::VideoMode(64, 64), "CodeHarbor SFML sample", sf::Style::Close);
    if (!window.isOpen()) {
        std::cerr << "SFML self-test failed: window did not open\n";
        return 84;
    }

    window.clear(sf::Color::Black);
    window.display();
    window.close();
    std::cout << "SFML self-test passed\n";
    return 0;
}

int main(int argc, char **argv)
{
    if (argc == 2 && std::string(argv[1]) == "--self-test") {
        return run_self_test();
    }

    std::cout << "CodeHarbor SFML sample ready\n";
    return 0;
}
```

- [ ] **Step 3: Add SFML fixture README**

Create `fixtures/sfml-cpp-sample/README.md`:

```markdown
# CodeHarbor SFML C++ Sample

This fixture is a tiny deterministic C++ SFML project for smoke-testing CodeHarbor's Ubuntu AMD64 workspace.

## Use It In CodeHarbor

1. Rebuild or recreate the Docker workspace image after the Dockerfile package changes.
2. Create an environment.
3. Set `Name` to `CodeHarbor SFML Sample`.
4. Set `Local folder path` to this fixture directory.
5. Start the environment.
6. Run `Run full evaluation`.

Expected results:

- `Build` creates `codeharbor_sfml_sample`.
- `Tests` runs `xvfb-run --auto-servernum ./codeharbor_sfml_sample --self-test`.
- `Tests` prints `SFML self-test passed`.
- A Markdown report appears in the `Reports` panel.
```

- [ ] **Step 4: Verify source formatting and file list**

Run:

```bash
ls fixtures/sfml-cpp-sample fixtures/sfml-cpp-sample/src
```

Expected: `Makefile`, `README.md`, and `src/main.cpp` exist.

---

### Task 3: C CSFML Fixture

**Files:**
- Create: `fixtures/csfml-c-sample/Makefile`
- Create: `fixtures/csfml-c-sample/src/main.c`
- Create: `fixtures/csfml-c-sample/README.md`

**Interfaces:**
- Consumes: Docker image packages from Task 1.
- Produces: executable `codeharbor_csfml_sample` and Make targets `all`, `clean`, `fclean`, `tests_run`, `re`.

- [ ] **Step 1: Add CSFML Makefile**

Create `fixtures/csfml-c-sample/Makefile`:

```makefile
NAME = codeharbor_csfml_sample
CC = gcc
CFLAGS = -Wall -Wextra -Werror -g
LDLIBS = -lcsfml-graphics -lcsfml-window -lcsfml-system
SRC = src/main.c

all: $(NAME)

$(NAME): $(SRC)
	$(CC) $(CFLAGS) -o $(NAME) $(SRC) $(LDLIBS)

tests_run: $(NAME)
	xvfb-run --auto-servernum ./$(NAME) --self-test

clean:
	rm -f *.gcda *.gcno *.gcov *.log

fclean: clean
	rm -f $(NAME)
	rm -rf $(NAME).dSYM

re: fclean all

.PHONY: all tests_run clean fclean re
```

- [ ] **Step 2: Add CSFML source**

Create `fixtures/csfml-c-sample/src/main.c`:

```c
#include <SFML/Graphics.h>
#include <stdio.h>
#include <string.h>

static int run_self_test(void)
{
    sfVideoMode mode = {64, 64, 32};
    sfRenderWindow *window = sfRenderWindow_create(mode, "CodeHarbor CSFML sample", sfClose, NULL);

    if (window == NULL) {
        fprintf(stderr, "CSFML self-test failed: window did not open\n");
        return 84;
    }

    sfRenderWindow_clear(window, sfBlack);
    sfRenderWindow_display(window);
    sfRenderWindow_close(window);
    sfRenderWindow_destroy(window);
    puts("CSFML self-test passed");
    return 0;
}

int main(int argc, char **argv)
{
    if (argc == 2 && strcmp(argv[1], "--self-test") == 0) {
        return run_self_test();
    }

    puts("CodeHarbor CSFML sample ready");
    return 0;
}
```

- [ ] **Step 3: Add CSFML fixture README**

Create `fixtures/csfml-c-sample/README.md`:

```markdown
# CodeHarbor CSFML C Sample

This fixture is a tiny deterministic C CSFML project for smoke-testing CodeHarbor's Ubuntu AMD64 workspace.

## Use It In CodeHarbor

1. Rebuild or recreate the Docker workspace image after the Dockerfile package changes.
2. Create an environment.
3. Set `Name` to `CodeHarbor CSFML Sample`.
4. Set `Local folder path` to this fixture directory.
5. Start the environment.
6. Run `Run full evaluation`.

Expected results:

- `Build` creates `codeharbor_csfml_sample`.
- `Tests` runs `xvfb-run --auto-servernum ./codeharbor_csfml_sample --self-test`.
- `Tests` prints `CSFML self-test passed`.
- A Markdown report appears in the `Reports` panel.
```

- [ ] **Step 4: Verify source formatting and file list**

Run:

```bash
ls fixtures/csfml-c-sample fixtures/csfml-c-sample/src
```

Expected: `Makefile`, `README.md`, and `src/main.c` exist.

---

### Task 4: Documentation Updates

**Files:**
- Modify: `README.md`
- Modify: `docs/development.md`

**Interfaces:**
- Consumes: Docker packages and fixtures from Tasks 1-3.
- Produces: user-facing SFML/CSFML smoke-test instructions and final validation checklist.

- [ ] **Step 1: Update README sample section**

In `README.md`, extend the existing `Sample Project` section so it mentions all three fixtures:

```markdown
## Sample Projects

Use these fixtures to smoke-test CodeHarbor without a student project:

- `fixtures/epitech-c-sample/`: basic Epitech-style C project.
- `fixtures/sfml-cpp-sample/`: C++ SFML project that launches under `xvfb-run`.
- `fixtures/csfml-c-sample/`: C CSFML project that launches under `xvfb-run`.

Create an environment from a fixture folder, start it, then run `Run full evaluation`. The SFML and CSFML fixtures validate compilation and headless launch, not full interactive graphics behavior.
```

- [ ] **Step 2: Add Docker support note to README**

Under the Docker/workspace description in `README.md`, add:

```markdown
The workspace image includes SFML and CSFML development packages for Ubuntu AMD64 projects, plus `xvfb` for headless smoke tests.
```

- [ ] **Step 3: Update development manual checklist**

In `docs/development.md`, add this checklist near the manual verification section:

```markdown
For SFML/CSFML completion validation:

1. Rebuild or recreate the workspace image after editing `prototype/docker-workspace/Dockerfile`.
2. Create an environment from `fixtures/sfml-cpp-sample/`, start it, run `Run full evaluation`, and confirm a report appears.
3. Create an environment from `fixtures/csfml-c-sample/`, start it, run `Run full evaluation`, and confirm a report appears.
4. Treat these as headless launch smoke tests, not full interactive graphics tests.
```

- [ ] **Step 4: Verify docs mention both stacks**

Run:

```bash
grep -R "SFML\|CSFML\|sfml-cpp-sample\|csfml-c-sample" README.md docs/development.md
```

Expected: both docs mention SFML and CSFML fixtures.

---

### Task 5: Validation and Final Commit Prep

**Files:**
- Verify only: `prototype/docker-workspace/Dockerfile`, `fixtures/sfml-cpp-sample/`, `fixtures/csfml-c-sample/`, `README.md`, `docs/development.md`, `docs/superpowers/plans/2026-08-11-sfml-csfml-validation.md`

**Interfaces:**
- Consumes: all earlier tasks.
- Produces: verified final state ready to commit/push.

- [ ] **Step 1: Run automated project validation**

Run from repository root:

```bash
npm run test:all
```

Expected: frontend build passes, Rust tests pass, and `cargo check` passes.

- [ ] **Step 2: Build Docker workspace image**

Run from repository root:

```bash
docker compose -f prototype/docker-workspace/compose.yaml build
```

Expected: image build succeeds and apt installs `libsfml-dev`, `libcsfml-dev`, `xvfb`, and `mesa-utils`.

- [ ] **Step 3: Validate SFML fixture inside Docker image**

Run from repository root:

```bash
docker run --rm --platform linux/amd64 -v "$PWD/fixtures/sfml-cpp-sample:/workspace" -w /workspace codeharbor-docker-workspace-workspace make fclean all tests_run fclean
```

Expected: compile succeeds and output contains `SFML self-test passed`.

If the image name differs locally, run `docker images` and use the image produced by Step 2.

- [ ] **Step 4: Validate CSFML fixture inside Docker image**

Run from repository root:

```bash
docker run --rm --platform linux/amd64 -v "$PWD/fixtures/csfml-c-sample:/workspace" -w /workspace codeharbor-docker-workspace-workspace make fclean all tests_run fclean
```

Expected: compile succeeds and output contains `CSFML self-test passed`.

If the image name differs locally, run `docker images` and use the image produced by Step 2.

- [ ] **Step 5: Install macOS app**

Run from repository root:

```bash
npm run mac:install
```

Expected: `~/Applications/CodeHarbor.app` is installed and LaunchServices/icon cache refresh commands complete.

- [ ] **Step 6: Inspect intended diff only**

Run:

```bash
git diff -- prototype/docker-workspace/Dockerfile fixtures/sfml-cpp-sample fixtures/csfml-c-sample README.md docs/development.md docs/superpowers/plans/2026-08-11-sfml-csfml-validation.md
git status --short prototype/docker-workspace/Dockerfile fixtures/sfml-cpp-sample fixtures/csfml-c-sample README.md docs/development.md docs/superpowers/plans/2026-08-11-sfml-csfml-validation.md
```

Expected: intended files are changed or untracked. Unrelated dirty files remain unstaged.

- [ ] **Step 7: Commit only intended files when requested**

When committing, stage exactly:

```bash
git add -- prototype/docker-workspace/Dockerfile fixtures/sfml-cpp-sample fixtures/csfml-c-sample README.md docs/development.md docs/superpowers/plans/2026-08-11-sfml-csfml-validation.md
git commit -m "chore: validate sfml csfml workspace support"
```

Expected: commit includes only Docker package support, SFML/CSFML fixtures, docs, and this plan.

# SFML and CSFML Validation Design

## Goal

Finish CodeHarbor for the intended real-world use case: running and evaluating Epitech-style projects that depend on SFML or CSFML inside the Ubuntu AMD64 Docker workspace.

This is a validation and completion pass, not a new product feature pass. The app UI stays unchanged unless a small documentation label is required. The work proves that the Docker evaluation environment can compile and launch minimal SFML and CSFML programs through the existing evaluation flow.

## Scope

In scope:

- Install SFML and CSFML development/runtime packages in the Ubuntu AMD64 workspace image.
- Install minimal headless graphics support so smoke tests can launch graphical programs in Docker.
- Add one C++ SFML fixture and one C CSFML fixture.
- Make both fixtures compatible with the existing CodeHarbor evaluation actions: Build, Tests, Clean, Valgrind, and Run full evaluation.
- Document the SFML/CSFML smoke-test workflow.
- Run the final validation sequence before considering the project complete.

Out of scope:

- No new UI features.
- No scoring or grading logic.
- No interactive graphics streaming from the container to macOS.
- No support matrix beyond Ubuntu AMD64 packages available through `apt`.
- No new project templates unless they are needed as minimal fixtures.

## Docker Environment

The existing `prototype/docker-workspace/Dockerfile` remains the single source of the workspace image. It will add packages for both stacks:

- `libsfml-dev` for C++ SFML projects.
- `libcsfml-dev` for C CSFML projects.
- `xvfb` for headless test execution.
- `mesa-utils` for basic OpenGL runtime diagnostics.

The existing `linux/amd64` Docker Compose platform constraint remains unchanged. This is important because CodeHarbor is meant to reproduce AMD64 Linux behavior from macOS.

## Fixtures

Two fixtures will be added under `fixtures/`.

### C++ SFML Fixture

Path: `fixtures/sfml-cpp-sample/`

Expected files:

- `Makefile`
- `src/main.cpp`
- `README.md`

The fixture builds `codeharbor_sfml_sample`. Its `tests_run` target launches the executable under `xvfb-run` with a `--self-test` argument. The program creates a tiny SFML window, clears once, closes, prints a success message, and exits with `0`. Failure exits with `84`.

### C CSFML Fixture

Path: `fixtures/csfml-c-sample/`

Expected files:

- `Makefile`
- `src/main.c`
- `README.md`

The fixture builds `codeharbor_csfml_sample`. Its `tests_run` target launches the executable under `xvfb-run` with a `--self-test` argument. The program creates a tiny CSFML render window, clears once, closes, prints a success message, and exits with `0`. Failure exits with `84`.

Both fixtures should include `fclean` targets that remove their executable and common debug artifacts, including macOS `.dSYM` bundles when compiled locally.

## Evaluation Flow

No new backend command is required. The current `Run full evaluation` command should work unchanged:

1. Clean.
2. Build.
3. Tests if Build succeeds.
4. Optional Valgrind if a detected executable is selected.
5. Markdown report generation.

The SFML and CSFML fixtures should be detected as normal projects through the existing project inspection logic. Their binaries should appear as selectable Valgrind targets after Build.

## Documentation

Documentation should explain that CodeHarbor supports SFML and CSFML projects through the Ubuntu AMD64 workspace image. It should also include a concise smoke-test checklist:

1. Rebuild or recreate the workspace image after Dockerfile changes.
2. Create an environment from `fixtures/sfml-cpp-sample/`.
3. Start it and run `Run full evaluation`.
4. Repeat with `fixtures/csfml-c-sample/`.
5. Confirm reports are generated for both.

The docs should make clear that these fixtures validate compilation and headless launch, not full interactive graphics behavior.

## Validation

Automated validation:

- `npm run test:all` must pass.
- Fixture-local Makefile checks should pass where host dependencies are available. If macOS lacks SFML/CSFML libraries, fixture validation should be performed inside Docker instead.

Manual validation before calling the project complete:

1. Build/install the macOS app with `npm run mac:install`.
2. Create and start an environment from `fixtures/epitech-c-sample/`.
3. Run `Run full evaluation` and confirm a report appears.
4. Create and start an environment from `fixtures/sfml-cpp-sample/`.
5. Run `Run full evaluation` and confirm Build, Tests, and report generation.
6. Create and start an environment from `fixtures/csfml-c-sample/`.
7. Run `Run full evaluation` and confirm Build, Tests, and report generation.
8. Verify safe environment deletion still removes only generated environment files.

## Completion Criteria

The project can be considered complete for the current goal when:

- The Docker image includes both SFML and CSFML support.
- Both SFML and CSFML fixtures build and pass their headless tests in the CodeHarbor workspace.
- `npm run test:all` passes.
- `npm run mac:install` succeeds.
- The manual smoke-test checklist passes.
- No unrelated dirty files are included in the final commit.

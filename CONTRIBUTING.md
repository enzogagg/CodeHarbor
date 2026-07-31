# Contributing To CodeHarbor

CodeHarbor is in early development. Keep contributions focused, small and easy to review.

## Development Principles

- Prefer simple Docker and Tauri primitives over custom abstractions.
- Keep the Docker prototype runnable at all times.
- Document user-facing commands in `README.md` or `docs/development.md`.
- Do not commit secrets, local volumes or generated application data.

## Commit Convention

Use Conventional Commits:

```text
feat: add environment creation form
fix: handle Docker daemon unavailable
docs: add local installation guide
refactor: extract Docker command service
test: add workspace configuration tests
chore: initialize project structure
```

## Branch Convention

- `main`
- `develop`
- `feature/<name>`
- `fix/<name>`
- `docs/<name>`
- `refactor/<name>`

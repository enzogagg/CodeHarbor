# Finder Folder Picker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let users select the local project folder with the native macOS Finder dialog instead of typing absolute paths manually.

**Architecture:** Add a small Tauri command that opens a native directory picker and returns the selected path. Add a `Choose...` button beside the existing local folder field that fills the editable text input. Keep environment creation semantics unchanged.

**Tech Stack:** Tauri v2 Rust backend, React/TypeScript frontend, macOS native dialog via `tauri::WebviewWindow::dialog`.

## Global Constraints

- Keep the local folder text field editable.
- Add only a folder picker, not a new environment creation flow.
- Do not change Git URL behavior.
- Do not stage unrelated dirty worktree files.
- Final verification: `npm run test:all`, `npm run mac:install`.

---

### Task 1: Backend Folder Picker Command

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces Tauri command: `pick_local_folder(window: tauri::WebviewWindow) -> Result<Option<String>, String>`.

- [ ] Add `use tauri::Manager;` only if needed by the chosen API.
- [ ] Add a Tauri command that opens a blocking folder dialog and returns `Some(path)` or `None` when cancelled.
- [ ] Register `pick_local_folder` in `tauri::generate_handler!`.
- [ ] Run `cargo check` from `src-tauri`.

### Task 2: Frontend Button

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.css` only if layout needs a tiny utility class.

**Interfaces:**
- Consumes Tauri command `pick_local_folder`.

- [ ] Add `pickLocalFolder()` handler.
- [ ] Add `Choose...` button next to `Local folder path`.
- [ ] On selected path, set `hostPath` and clear previous error.
- [ ] Keep manual input unchanged.
- [ ] Run `npm run build`.

### Task 3: Verification And Commit

**Files:**
- Verify intended diff only.

- [ ] Run `npm run test:all`.
- [ ] Run `npm run mac:install`.
- [ ] Commit only intended files with message `feat: add finder folder picker`.
- [ ] Push to `main`.

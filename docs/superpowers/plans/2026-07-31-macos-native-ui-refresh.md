# macOS Native UI Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the decorative CodeHarbor MVP screen with a restrained macOS-style utility interface.

**Architecture:** Keep the existing Tauri command wiring in `src/App.tsx`. Replace only the React layout and CSS so the same commands render inside a sidebar + main content + output structure.

**Tech Stack:** React, TypeScript, CSS, Tauri command invocation.

## Global Constraints

- Do not modify Rust/Tauri command behavior.
- Keep commands `check_docker`, `start_prototype`, `stop_prototype`, and `open_ide` wired to the same buttons.
- Use Apple system fonts and a restrained dark macOS visual system.
- Remove decorative grid, giant cropped title, circular ornament and oversized `CH` badge.
- Verify with `npm run build` before commit.

---

## Task 1: Replace React Layout

**Files:**
- Modify: `src/App.tsx`

**Interfaces:**
- Consumes: existing `runCommand(command: CommandName)` behavior.
- Produces: sidebar, main toolbar, workspace detail panel, output panel.

- [ ] **Step 1: Replace JSX structure**

Use the existing state and commands, but render a macOS utility layout with `app-frame`, `sidebar`, `content`, `topbar`, `workspace-panel`, `detail-list`, and `output-panel` classes.

- [ ] **Step 2: Keep all command actions available**

Ensure the four buttons still call `runCommand` with the same command names.

---

## Task 2: Replace CSS Visual System

**Files:**
- Modify: `src/App.css`

**Interfaces:**
- Consumes: class names from Task 1.
- Produces: macOS-style dark utility app styling.

- [ ] **Step 1: Replace decorative styling**

Remove grid, huge hero, ornament and old card styling.

- [ ] **Step 2: Add macOS utility styling**

Add sidebar, toolbar, flat cards, subtle borders, system-blue primary button, grey secondary buttons and compact output panel.

---

## Task 3: Verify And Commit

**Files:**
- Read: `src/App.tsx`
- Read: `src/App.css`

**Interfaces:**
- Consumes: completed UI refresh.
- Produces: committed and pushable UI change.

- [ ] **Step 1: Build frontend**

Run: `npm run build`

Expected: command exits with status `0`.

- [ ] **Step 2: Check git diff**

Run: `git status --short && git diff --stat`

Expected: only UI refresh files and plan file are modified/untracked.

- [ ] **Step 3: Commit**

Commit message: `style: refresh desktop UI with macOS layout`.

---

## Self-Review

- Spec coverage: plan covers macOS utility layout, restrained palette, command preservation and build verification.
- Placeholder scan: no unresolved placeholders.
- Type consistency: command names remain unchanged.

# macOS Native UI Refresh Design

## Purpose

Refresh the CodeHarbor desktop MVP so it feels like a credible macOS utility instead of a decorative landing page. The existing Tauri commands and Docker prototype behavior stay unchanged.

## Direction

The UI should use a native macOS utility layout:

- Compact sidebar for app identity, Docker status and the selected workspace.
- Main content panel for workspace title, state, actions and details.
- Output panel at the bottom for command feedback.

## Visual System

- Use Apple system fonts throughout.
- Use a restrained dark macOS palette: near-black app background, layered charcoal surfaces, subtle borders and macOS blue accent.
- Use green only for available/running status and red only for errors.
- Use monospace only for technical values such as ports, paths and command output.
- Remove decorative grid, giant cropped title, circular ornament and oversized `CH` badge.

## Interaction

The existing actions remain:

- `Vérifier Docker`
- `Démarrer`
- `Arrêter`
- `Ouvrir IDE`

Buttons should be smaller, aligned with the workspace header and disabled while a command is running. The primary action is `Démarrer`; the other actions are secondary.

## Content Layout

The app should show:

- App title: `CodeHarbor`.
- Sidebar subtitle: `Docker workspaces`.
- Selected workspace: `Ubuntu AMD64`.
- Main title: `Ubuntu AMD64 Workspace`.
- Description: short explanation of the Ubuntu, code-server and `/workspace` mount.
- Technical details: image/base, architecture, IDE URL, mount path.
- Output panel: latest command response or error.

## Testing

Verification remains:

- `npm run build` for TypeScript and Vite.
- `cargo test` and `cargo check` for Rust backend safety.
- Manual app check with `npm run tauri:dev`.

# Changelog

All notable changes to the **Valter ERP** project will be documented in this file.

## [Unreleased] - 2026-01-06

### 🚀 Major Improvements (Architecture)
- **Strict Dev/Prod Separation:** 
    - Implemented robust logic in `main.rs` to distinguish between Development Mode (Monorepo) and Production Mode (System Install).
    - `just dev` now correctly sets the CWD to the repo root, ensuring relative paths in `valter.dev.config` work as expected.
    - Production binary now strictly respects `~/.valter` paths.
- **Dynamic Island Support:**
    - Refactored `processor.rs` to stop assuming all Islands are the first type defined in config.
    - The processor now dynamically matches files to their correct `IslandDefinition` based on `meta_file` name and directory path.

### 🐛 Bug Fixes
- **Ghost Data Fix (The "Zombie Project" Bug):**
    - Fixed a critical issue where changing the `root_path` in config did not remove old projects from the SQLite database.
    - Implemented `purge_islands()` which clears the SQL table before scanning, ensuring the database is always a 1:1 reflection of the filesystem.
- **Notification Reset:**
    - Fixed an issue where "Ignored" actions disappeared forever.
    - Implemented `reset_pending_actions()` which clears the notification history on startup/rescan, allowing users to re-evaluate conflicts.
- **Path Resolution:**
    - Fixed `watcher.rs` to handle multiple watch paths correctly.
    - Added canonicalization for relative paths in Dev mode to prevent "File not found" errors in the Watcher.

### ✨ New Features
- **"RESCAN SYSTEM" Button:**
    - Added a manual trigger in the Dashboard Header.
    - **Function:** Clears the Database Cache, Resets Notifications, and forces a deep filesystem scan. Useful when changing configs or debugging.
- **GraphQL Mutations:**
    - Added `rescanIslands` mutation to the API.

### 🛠 Internal
- **Justfile:** Updated `dev` command to use `cargo run --manifest-path` instead of `cd core`, preserving the correct environment context.
- **Config Templates:** Updated `valter.config.example` with clear instructions for Production usage.
- **Code Cleanup:** Removed hardcoded path hacks (`../..`) that were previously used as workarounds.


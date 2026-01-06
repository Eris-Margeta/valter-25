---
title: ROADMAP
---

# STRATA PROJECT ROADMAP

## Phase 1: Prototype (COMPLETED - Dec 2025)
**Goal:** Prove the concept of Filesystem-as-Database.
*   [x] **Core Engine:** Rust Daemon with `notify` watcher.
*   [x] **Basic Config:** Simple `strata.config` parser.
*   [x] **Cloud Layer:** Basic SQLite integration with "Implicit Creation".
*   [x] **API:** Simple GraphQL Server (Axum).
*   [x] **Oracle:** Basic LLM integration (Gemini).
*   [x] **Frontend:** Static React Dashboard.

---

## Phase 2: The Data Operating System (COMPLETED - Jan 2026)
**Goal:** Create a robust, whitelabel, bi-directional engine safe for business use.
*   [x] **Whitelabel Core:** 
    *   Removed hardcoded business logic from Rust.
    *   System now fully adapts to `strata.config` (Global settings, Cloud schemas, Island definitions).
*   [x] **Deep Aggregation (The Calculator):** 
    *   Implemented `Aggregator` module.
    *   Recursive scanning of sub-folders (e.g., `INTERNAL/Finances`) to calculate metrics (Sum, Count, Average) defined in config.
*   [x] **Safety Valve (Conflict Resolution):** 
    *   Implemented `pending_actions` table in SQLite.
    *   Added Fuzzy Matching (`strsim`) to detect typos (e.g., "Mircosoft" vs "Microsoft").
    *   Implemented **Action Center** in Frontend for user approval/rejection.
*   [x] **Bi-Directional Sync:** 
    *   Implemented `FsWriter` for safe atomic writes to YAML files.
    *   Enabled **Inline Editing** in Dashboard tables that updates files on disk.
    *   Implemented **Smart Merge** logic to fix typos in files directly from the UI.
*   [x] **Dynamic Frontend:** 
    *   UI now renders tables and sidebars dynamically based on Backend configuration.

---

## Phase 3: The Semantic Layer (Q2 2026)
**Goal:** Enable the system to understand the *content* of files, not just metadata.

*   [ ] **Vector Database Integration:**
    *   Embed `LanceDB` or `Qdrant` directly into the Daemon.
    *   Automatically generate embeddings for `README.md`, Source Code, and PDF documents in Islands.
    *   *Use Case:* "Find all contracts related to AI safety."
*   [ ] **RAG Pipeline (Retrieval Augmented Generation):**
    *   Upgrade Oracle to perform semantic search on file contents before answering.
    *   *Query:* "Show me all projects that use React and have pending invoices over €5000."

## Phase 4: True Graph & Connectivity (Q3 2026)
**Goal:** Move beyond simple lists to complex relationship mapping.

*   [ ] **Graph Engine:**
    *   Implement an internal Graph structure (Node/Edge) to track relationships (e.g., `Alice` -> `COMMITTED_TO` -> `Project X` -> `DEPENDS_ON` -> `Project Y`).
*   [ ] **Cross-Island Dependency:**
    *   Allow projects to reference data from *other* projects (e.g., Sub-projects).

## Phase 5: Enterprise Features (Q4 2026)
**Goal:** Production readiness for distributed teams.

*   [ ] **Multi-User Sync:** 
    *   CRDT-based syncing for teams sharing a Dropbox/Google Drive folder without file lock conflicts.
*   [ ] **Plugin System:** 
    *   Wasm-based plugins to allow users to define custom "Processors" for specific file types (e.g., parsing `.psd` metadata or CAD files).
*   [ ] **Native GUI:** 
    *   Migrate frontend to Tauri for a native desktop app experience (MacOS/Windows/Linux).

---

## Long Term Vision
To create a computing environment where the operating system is indistinguishable from the database, and the AI serves as the universal interpreter between the user's intent and the system's data.

# STRATA PROJECT ROADMAP

## Phase 1: Prototype (COMPLETED)
*   [x] **Core Engine:** Rust Daemon with `notify` watcher.
*   [x] **Configuration:** `strata.config` parser.
*   [x] **Cloud Layer:** SQLite integration with "Implicit Creation" (Upsert).
*   [x] **Context Engine:** Basic file walker and XML context generation.
*   [x] **API:** GraphQL Server (Axum).
*   [x] **Oracle:** Basic LLM integration (Gemini) with schema awareness.
*   [x] **Frontend:** React Dashboard with Project Creation and Data Visualization.

---

## Phase 2: The Semantic Layer (Q2 2026)
**Goal:** Enable the system to understand the *content* of files, not just metadata.

*   [ ] **Vector Database Integration:**
    *   Embed `LanceDB` or `Qdrant` directly into the Daemon.
    *   Automatically generate embeddings for `README.md`, Source Code, and PDF documents in Islands.
*   [ ] **RAG Pipeline:**
    *   Upgrade Oracle to perform Retrieval Augmented Generation on file contents.
    *   Query: "Show me all projects that use React and have pending invoices."

## Phase 3: True Graph & Connectivity (Q3 2026)
**Goal:** Move beyond simple lists to complex relationship mapping.

*   [ ] **Graph Engine:**
    *   Implement an internal Graph structure (Node/Edge) to track relationships (e.g., `Alice` -> `COMMITTED_TO` -> `Project X`).
*   [ ] **Bidirectional Sync:**
    *   Allow editing data in the Frontend (e.g., renaming a Client) to *write back* to the `meta.yaml` files on disk.

## Phase 4: Enterprise Features (Q4 2026)
**Goal:** Production readiness for teams.

*   [ ] **Multi-User Sync:** CRDT-based syncing for teams sharing a Dropbox/Google Drive folder.
*   [ ] **Plugin System:** Wasm-based plugins to allow users to define custom "Processors" for specific file types (e.g., parsing `.psd` metadata).
*   [ ] **Native GUI:** Migrate frontend to Tauri for a native desktop app experience.

---

## Long Term Vision
To create a computing environment where the operating system is indistinguishable from the database, and the AI serves as the universal interpreter between the user's intent and the system's data.

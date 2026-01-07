# VALTER DEVELOPER GUIDE (v3.0)

## 1. System Architecture
**VALTER ERP** is a Hyper-Converged Data Operating System. It creates a seamless bridge between the unstructured Filesystem (where you work) and structured SQL/AI layers (where the system thinks).

### The "Loop" (Data Flow)
The system relies on a continuous feedback loop to maintain consistency:

1.  **Input (File or UI):**
    *   User edits `meta.yaml` on disk.
    *   OR User clicks "Edit" in the Desktop App (triggers GraphQL mutation).
2.  **Detection (`watcher.rs`):**
    *   OS-level events (`inotify`/`FSEvents`) detect the change in the watched directory.
    *   Events are debounced and sent to the `EventProcessor`.
3.  **Processing (`processor.rs`):**
    *   Parses the YAML content.
    *   **Deep Scan:** Calls `Aggregator` to scan sub-folders defined in `valter.config` (e.g., summing invoices defined in specific sub-directories).
    *   **Relationship Check:** Checks linked entities (Clients, Operators) against the SQL Cloud.
4.  **Validation (The Safety Valve):**
    *   If a relation (e.g., "Mircosoft") is not found in the DB, `SqliteManager` performs a **Fuzzy Match**.
    *   If ambiguous, it creates a `PendingAction` and pauses. **No data is committed to the Island table.**
5.  **Commit (`cloud.rs`):**
    *   If valid, data is upserted into SQLite (Islands table).
6.  **Presentation (`api.rs`):**
    *   GraphQL API serves the fresh data to the Frontend.

---

## 2. Configuration (`valter.config`)

### 2.1 Configuration Strategy

Valter supports two modes of operation, distinguished by configuration file naming:

### A. Development Mode (Monorepo)
*   **Config File:** `valter.dev.config` (Located in repo root).
*   **Git Status:** Committed to the repository.
*   **Purpose:** Points to the local `./dev-projects-folder` so that any developer cloning the repo has immediate access to demo data.
*   **Trigger:** Running `just dev` automatically prefers this file.

### B. System Mode (Production)
*   **Config File:** `valter.config` (Located in `~/.valter/`).
*   **Git Status:** Ignored (User specific).
*   **Purpose:** Defines your real-world business data.
*   **Trigger:** Running the installed binary or app.

---

## 3. The Unified Architecture (Tauri + Core)

As of v3.0, Valter has transitioned to a unified **Native Application** architecture using Tauri v2.

### A. Core Library (`core/`)
The backend logic has been refactored from a binary into a library crate (`valter_core`).
*   **Responsibility:** Watcher, SQLite management, GraphQL Server, AI Oracle.
*   **Portability:** Can be embedded into the Tauri app OR run as a standalone headless daemon.

### B. The Application (`app/`)
*   **Tauri Host (`app/src-tauri/`):** A lightweight Rust shell that initializes `valter_core` in a background thread. It handles native OS integration (Menu bars, Window management).
*   **Frontend (`app/src/`):** A React 19 application running inside the native WebView. It communicates with the Core via the local GraphQL server.

---

## 4. Frontend Architecture
*   **Technology:** React 19 + TypeScript + Vite + Tailwind + React Router.
*   **Dynamic Routing:** The UI is dynamically generated based on the Config.
    *   `/list/:type/:name`: Generic table for Clouds/Islands.
    *   `/entity/:type/:name/:id`: Generic detail form.
*   **Universal Form:** A `<EntityForm />` component that renders inputs based on field definitions in `valter.config`.
*   **Action Center:**
    *   Polls `GET_PENDING_ACTIONS`.
    *   Allows users to **Merge** (fix file typo via `UPDATE_ISLAND_FIELD`) or **Create New** (approve via `RESOLVE_ACTION`).

---

## 5. CLI Commands (JUST)

Valter uses `just` as a command runner to manage the entire lifecycle.

### Development

*   **`just dev`** (Primary)
    *   Starts the **Full Desktop App** (Tauri) in development mode.
    *   Hot-reloading enabled for both Rust and React.
*   **`just dev-core`**
    *   Runs the **Headless Backend** only. Useful for debugging logic without the UI.
*   **`just dev-web`**
    *   Runs the **Web Frontend** only (in browser mode).

### Build & Release

*   **`just build-app`**
    *   Builds the production **Desktop Application** (`.dmg`, `.exe`, `.deb`).
*   **`just build-server`**
    *   Builds the standalone **Headless Binary** (Legacy/Server mode) with the frontend assets embedded.

### System Utilities

*   **`just install`**
    *   Installs the **Headless Daemon** to `~/.local/bin/valter`.
    *   Sets up `~/.valter` config directory.
*   **`just update`**
    *   Updates the headless binary without touching user data/config.

---

## 6. Schema Migrations

Valter uses a **Dynamic Schema** approach.

*   **Source of Truth:** `valter.config`.
*   **Mechanism:** On startup, `SqliteManager` compares the fields defined in Config against the SQLite `PRAGMA table_info`.
*   **Action:** If a field exists in Config but not in DB, it runs `ALTER TABLE ADD COLUMN`.
*   **Limitations:**
    *   **Deletions:** Removing fields from Config does *not* delete columns in DB (safety first).
    *   **Renaming:** Renaming a field is treated as adding a new column.

---

## 7. Extending Valter

### Scenario: Adding a "Department" Cloud
1.  Open `valter.config`.
2.  Add to `CLOUDS`:
    ```yaml
    - name: "Department"
      icon: "users"
      fields:
        - key: "title"
          type: "string"
    ```
3.  Restart Valter.
4.  **Result:** The SQL table is created, and the Dashboard automatically shows "Department" in the sidebar.
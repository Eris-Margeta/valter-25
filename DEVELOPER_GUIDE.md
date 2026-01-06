Evo kompletno ažuriranog **VALTER DEVELOPER GUIDE** dokumenta.

Ažurirao sam sve reference na **Valter**, dodao novu sekciju za **Just** komande, objasnio logiku **Schema Migracija**, te prilagodio putanje novoj **Monorepo** strukturi.

***

# VALTER DEVELOPER GUIDE (v3.0)

## 1. System Architecture
**VALTER ERP** is a Hyper-Converged Data Operating System. It creates a seamless bridge between the unstructured Filesystem (where you work) and structured SQL/AI layers (where the system thinks).

### The "Loop" (Data Flow)
The system relies on a continuous feedback loop to maintain consistency:

1.  **Input (File or UI):**
    *   User edits `meta.yaml` on disk.
    *   OR User clicks "Edit" in the Dashboard (triggers `FsWriter`).
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
    *   GraphQL API serves the fresh data to the Dashboard and AI Oracle.

---

## 2. Configuration (`valter.config`)

## 2.1 Configuration Strategy

Valter supports two modes of operation, distinguished by configuration file naming:

### A. Development Mode (Monorepo)
*   **Config File:** `valter.dev.config` (Located in repo root).
*   **Git Status:** Committed to the repository.
*   **Purpose:** Points to the local `./dev-projects-folder` so that any developer cloning the repo has immediate access to demo data without setup.
*   **Trigger:** Running `just dev` or `cargo run` in the repo automatically prefers this file.

### B. System Mode (Production)
*   **Config File:** `valter.config` (Located in `~/.valter/`).
*   **Git Status:** Ignored (User specific).
*   **Purpose:** Defines your real-world business data.
*   **Trigger:** Running the installed binary `valter` (which sets `VALTER_HOME=~/.valter`).

## 2.2 configuration file fields (valter.cofig)
*   **GLOBAL:** Company settings (Currency, Name, Locale).
*   **CLOUDS:** Defines SQL Tables (e.g., `Client`, `Operator`).
    *   `fields`: Defines columns and data types dynamically.
*   **ISLANDS:** Defines Project structures and filesystem scanning rules.
    *   `relations`: Which YAML fields link to which Cloud.
    *   `aggregations`: Rules for scanning sub-folders (Path, Logic, Target Field).

---

## 3. Core Modules

### A. Cloud Layer (`core/src/cloud.rs`)
*   **Dynamic Schema & Auto-Migration:** On startup, Valter compares `valter.config` against the SQLite database. If new fields are added to the config, Valter automatically executes `ALTER TABLE ADD COLUMN` to migrate the schema without data loss.
*   **Implicit Creation:** Can create new entities on the fly if exact matches are found during file scanning.
*   **Conflict Resolution:**
    *   `check_or_create_pending`: The guard logic detecting typos or new entities.
    *   `approve_pending_creation`: Promotes a pending string to a real Entity UUID.

### B. Processor & Aggregator (`core/src/processor.rs`)
*   **Stateless:** The processor doesn't remember state; it recalculates the "Truth" from files every time they change.
*   **Aggregator:** Uses `glob` patterns to find files (e.g., `VALTER-INTERNAL/WorkOrders/*.md`) and math logic (`sum`, `count`, `average`) to produce virtual columns in the database.

### C. File System Writer (`core/src/fs_writer.rs`)
*   **Atomic Writes:** Writes to a `.tmp` file and renames it to ensure data integrity during power failures.
*   **Preservation:** Used by the Frontend to "fix" data on disk (Inline Editing) while attempting to preserve YAML structure.

---

## 4. Frontend Architecture (`dashboard/`)
*   **Technology:** React + TypeScript + Vite + Tailwind.
*   **Dynamic Rendering:** The UI is not hardcoded.
    *   `App.tsx` fetches `GET_CONFIG`.
    *   It generates Sidebar links and Routes based on `CLOUDS` and `ISLANDS` arrays defined in the backend config.
*   **Action Center:**
    *   Polls `GET_PENDING_ACTIONS`.
    *   Allows users to **Merge** (fix file typo via `UPDATE_ISLAND_FIELD` mutation) or **Create New** (approve via `RESOLVE_ACTION` mutation).

---

## 5. CLI Commands (JUST)

Valter uses `just` as a command runner to manage the Monorepo and Installation lifecycles.

### `just dev`
Runs the system in **Monorepo Mode** for development.
*   **Backend:** Runs `cargo run` in `core/`, watching the repository root.
*   **Frontend:** Runs `pnpm dev` in `dashboard/` on port 5173.
*   **Config:** Uses `./valter.dev.config` from the repo root.
*   **Data Source:** Scans `./dev-projects-folder` inside the repo.
*   **Database:** Uses `./valter.db` (ephemeral/local).

### `just install`
Installs Valter to your user system for production use.
*   **Binary:** Compiles release mode and copies to `~/.local/bin/valter`.
*   **Config:** Copies `valter.config.example` to `~/.valter/valter.config` (if not exists).
*   **Database:** Initializes `~/.valter/valter.db`.
*   **WARNING:** This is a "hard" install. It ensures the environment is set up correctly in `~/.valter`.
*   **WARNING:** This does NOT copy the dev config. It sets you up for a fresh production environment.


### `just update`
Updates the binary *without* touching your data.
*   **Binary:** Recompiles and updates `~/.local/bin/valter`.
*   **Data Safety:** Does NOT touch `~/.valter/valter.db` or your config.
*   **Migration:** Upon the next run, the new binary will detect any changes you made to your config and migrate the SQL schema automatically.

---

## 6. Schema Migrations

Valter uses a **Dynamic Schema** approach which eliminates manual SQL migrations files.

*   **Source of Truth:** `valter.config`.
*   **Mechanism:** On startup, `SqliteManager` compares the fields defined in Config against the SQLite `PRAGMA table_info`.
*   **Action:** If a field exists in Config but not in DB, it runs `ALTER TABLE ADD COLUMN`.
*   **Limitations:**
    *   **Deletions:** Removing fields from Config does *not* delete columns in DB (to prevent accidental data loss).
    *   **Renaming:** Renaming a field in Config is treated as adding a new column. Old data remains in the old column. Manual SQL intervention is required to move data if needed.

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
          required: true
    ```
3.  Restart Valter (`just dev` or restart binary).
4.  **Result:** The SQL table is automatically created/migrated, the API endpoint `cloudData(name: "Department")` becomes active, and the Dashboard sidebar shows "Department".

### Scenario: Tracking "Bugs" in Projects
1.  Open `valter.config`.
2.  Add to `ISLANDS` -> `aggregations`:
    ```yaml
    - name: "bug_count"
      path: "INTERNAL/Issues/*.yaml"
      target_field: "id"
      logic: "count"
    ```
3.  **Result:** Every project now has a `bug_count` column in the database that updates automatically whenever files are added to that folder.

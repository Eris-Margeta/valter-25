# STRATA DEVELOPER GUIDE (v2.1)

## 1. System Architecture
Strata is a **Hyper-Converged Data Operating System**. It creates a bridge between the unstructured Filesystem and structured SQL/AI layers.

### The "Loop" (Data Flow)
The system relies on a continuous feedback loop to maintain consistency:

1.  **Input (File or UI):**
    *   User edits `meta.yaml` on disk.
    *   OR User clicks "Edit" in the Dashboard (triggers `FsWriter`).
2.  **Detection (`watcher.rs`):**
    *   OS-level events (`inotify`/`FSEvents`) detect the change.
    *   Events are debounced and sent to the `EventProcessor`.
3.  **Processing (`processor.rs`):**
    *   Parses the YAML content.
    *   **Deep Scan:** Calls `Aggregator` to scan sub-folders defined in `strata.config` (e.g., summing invoices).
    *   **Relationship Check:** Checks linked entities (Clients, Operators) against the SQL Cloud.
4.  **Validation (The Safety Valve):**
    *   If a relation (e.g., "Mircosoft") is not found in the DB, `SqliteManager` performs a **Fuzzy Match**.
    *   If ambiguous, it creates a `PendingAction` and pauses. **No data is committed to the Island table.**
5.  **Commit (`cloud.rs`):**
    *   If valid, data is upserted into SQLite (Islands table).
6.  **Presentation (`api.rs`):**
    *   GraphQL API serves the fresh data to the Frontend/AI.

---

## 2. Configuration (`strata.config`)
This file is the **Source of Truth**. Do not modify Rust code to change business logic.

*   **GLOBAL:** Company settings (Currency, Name).
*   **CLOUDS:** Defines SQL Tables (e.g., `Client`, `Operator`).
    *   `fields`: Defines columns and data types dynamically.
*   **ISLANDS:** Defines Project structures.
    *   `relations`: Which YAML fields link to which Cloud.
    *   `aggregations`: Rules for scanning sub-folders (Path, Logic, Target Field).

---

## 3. Core Modules

### A. Cloud Layer (`src/cloud.rs`)
*   **Dynamic Schema:** Creates tables based on config on startup (`init_schema`).
*   **Implicit Creation:** Can create new entities on the fly if exact matches are found.
*   **Conflict Resolution:**
    *   `check_or_create_pending`: The guard logic.
    *   `approve_pending_creation`: Promotes a pending string to a real Entity UUID.
    *   `reject_pending_action`: Ignores the unknown string (leaves relation as NULL).

### B. Processor & Aggregator (`src/processor.rs`, `src/aggregator.rs`)
*   **Stateless:** The processor doesn't remember state; it recalculates the "Truth" from files every time they change.
*   **Aggregator:** Uses `glob` patterns to find files (e.g., `INTERNAL/Finances/*.yaml`) and math logic (`sum`, `count`, `average`) to produce virtual columns.

### C. File System Writer (`src/fs_writer.rs`)
*   **Atomic Writes:** Writes to a `.tmp` file and renames it to ensure data integrity.
*   **Preservation:** Attempts to keep YAML structure valid. Used by the Frontend to "fix" data on disk.

---

## 4. Frontend Architecture (`/web`)
*   **Technology:** React + TypeScript + Vite + Tailwind.
*   **Dynamic Rendering:** The UI is not hardcoded.
    *   `App.tsx` fetches `GET_CONFIG`.
    *   It generates Sidebar links and Routes based on `CLOUDS` and `ISLANDS` arrays.
*   **Action Center:**
    *   Polls `GET_PENDING_ACTIONS`.
    *   Allows users to **Merge** (fix file typo via `UPDATE_ISLAND_FIELD`) or **Create New** (approve via `RESOLVE_ACTION`).

---

## 5. Extending Strata

### Scenario: Adding a "Department" Cloud
1.  Open `strata.config`.
2.  Add to `CLOUDS`:
    ```yaml
    - name: "Department"
      icon: "users"
      fields:
        - key: "title"
          type: "string"
          required: true
    ```
3.  Restart `./run.sh`.
4.  **Result:** The SQL table is created, the API endpoint `cloudData(name: "Department")` is active, and the Frontend sidebar shows "Department".

### Scenario: Tracking "Bugs" in Projects
1.  Open `strata.config`.
2.  Add to `ISLANDS` -> `aggregations`:
    ```yaml
    - name: "bug_count"
      path: "INTERNAL/Issues/*.yaml"
      target_field: "id"
      logic: "count"
    ```
3.  **Result:** Every project now has a `bug_count` column in the dashboard that updates automatically when files are added to that folder.


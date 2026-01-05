# STRATA DEVELOPER GUIDE (V2.0)

## Architecture Overview
Strata is a **Hyper-Converged Data Operating System**. It unifies the Filesystem (Source of Truth), SQLite (Performance Cache), and AI (Intelligence).

### 1. The Core Loop (Bi-Directional)
The system ensures data integrity through a strict loop:

1.  **Change:** User edits a file OR Frontend calls `updateIslandField`.
2.  **Detection:** `Watcher` detects FS event -> `Processor` parses YAML.
3.  **Aggregation:** `Aggregator` recursively scans sub-folders (deep scan) defined in `strata.config` to calculate totals (e.g., Revenue).
4.  **Validation (The Safety Valve):**
    *   New entities (Clients/Operators) are checked against the DB via `strsim` (Fuzzy Match).
    *   **Ambiguity:** If a name is similar to an existing one (e.g., "Mircosoft"), a `PendingAction` is created. **No data is committed.**
    *   **Resolution:** User resolves via Frontend Action Center -> System fixes the file on disk -> Loop restarts.
5.  **Commit:** Clean data is upserted to SQLite tables.
6.  **View:** GraphQL API serves clean, aggregated JSON to Frontend/AI.

### 2. Configuration (`strata.config`)
This file drives the entire engine.
*   **GLOBAL:** Company details.
*   **CLOUDS:** SQL Tables definition (Clients, Operators).
*   **ISLANDS:** Project definitions & Aggregation Rules.

### 3. Extending the System
**Do not modify Rust code for business logic.**
*   To add a new field (e.g., "Phone Number" to Client): Add it to `strata.config`. The backend adapts automatically.
*   To add a new metric (e.g., "Total Bugs"): Add an `aggregation` rule in `strata.config` pointing to your bug tracker files.

### 4. Frontend Development
*   **Dynamic:** Never hardcode table columns. Use `GET_CONFIG` query to render UI.
*   **Action Center:** Always check `GET_PENDING_ACTIONS`. This is how users handle data conflicts.

### 5. Troubleshooting
*   **Logs:** Check `strata.log` (Backend) and `vite.log` (Frontend).
*   **Data missing?** Check if `meta.yaml` is valid YAML. Check if a Pending Action is blocking the update.


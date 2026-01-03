# STRATA DEVELOPER GUIDE

## Architecture Overview

Strata is an **Event-Driven Architecture**. It does not poll; it reacts.

### Data Flow Pipeline
1.  **Event:** User saves `DEV/Project_X/meta.yaml`.
2.  **Watcher (`src/watcher.rs`):** `notify` crate detects the OS signal. Debounces the event (prevents duplicate triggers). Sends event to MPSC Channel.
3.  **Loop (`src/main.rs`):** The main Tokio loop receives the message.
4.  **Processor (`src/processor.rs`):**
    *   Reads the file content.
    *   Parses YAML.
    *   Extracts "Relational Fields" (Client, Operator) based on hardcoded logic (Prototype limitation) or `strata.config` (Goal).
5.  **Cloud (`src/cloud.rs`):**
    *   **Upsert Logic:** Checks if the entity (e.g., "Acme Corp") exists in the SQL table.
    *   If NO: Generates UUID v4 -> INSERT -> Returns UUID.
    *   If YES: SELECT id -> Returns UUID.
6.  **Sky (`src/api.rs`):** The GraphQL API now has fresh data to serve.

---

## extending the System

### How to Add a New Entity (e.g., "Department")

1.  **Update Configuration:**
    Edit `strata.config`:
    ```yaml
    - CLOUD: Department
      fields: [name, head_of_dept]
    ```

2.  **Update Processor Logic:**
    Edit `src/processor.rs` to look for the new field in `meta.yaml`:
    ```rust
    if let Some(dept_val) = yaml.get("department") {
        if let Some(dept_name) = dept_val.as_str() {
            self.cloud.upsert_entity("Department", "name", dept_name)?;
        }
    }
    ```

3.  **Update API Schema:**
    Edit `src/api.rs` to expose the new table:
    ```rust
    async fn departments(&self, ctx: &Context<'_>) -> Vec<Entity> {
        // ... fetch from cloud ...
    }
    ```

### Frontend Development
The frontend is a standard Vite+React app.
*   **Location:** `/web`
*   **State Management:** Local React State (for prototype). Recommend moving to TanStack Query for production.
*   **Styling:** Tailwind CSS (v4 configuration).

### Debugging
*   **Backend Logs:** `cargo run` prints structured logs via `tracing`. Look for `INFO` messages regarding "Upsert" and "Linked Project".
*   **Frontend Logs:** Check Browser Console (F12).

---

## Known Limitations (Prototype)

1.  **Schema Rigidity:** `src/processor.rs` currently has hardcoded field mapping (Client, Operator, Project). It does not yet fully dynamically map *every* field in `strata.config` to SQL columns automatically.
2.  **Filesystem Scan:** Currently only scans `meta.yaml` in direct subdirectories of `./DEV`. Nested islands logic needs expansion.
3.  **Graph API:** The "Graph" is simulated via SQL Foreign Keys. A true GraphDB layer (like Petgraph or creating an adjacency matrix) is planned for Phase 2.

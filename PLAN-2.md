# STRATA DEVELOPMENT PLAN - PHASE 2: PRODUCTION ARCHITECTURE

**Objective:** Transition Strata from a Proof-of-Concept to a Whitelabel, Local-First, Hyper-Converged Database Engine.
**Core Philosophy:** The Filesystem is the Master DB. The UI is a reactive layer. No data ambiguity is allowed without user confirmation.

---

## 1. THE CONFIGURATION ENGINE (The Brain)
**Goal:** Remove all hardcoded logic from the Rust backend. The system must behave entirely based on `strata.config`.

### 1.1. Redefine `strata.config` Schema
We need a more expressive YAML structure to handle aggregations and generic definitions.
*   **Task:** Define `CloudDefinition` structure in `config.rs`.
    *   Attributes: `name`, `fields` (type, validation regex), `icon` (for UI).
*   **Task:** Define `IslandDefinition` structure.
    *   Attributes: `root_path` (e.g., "./DEV"), `meta_file` (e.g., "meta.yaml").
    *   **New Attribute:** `aggregations`. This defines how to process sub-folders.
        *   Example: `path: "INTERNAL/Finances/*.yaml"`, `type: "sum"`, `target_field: "total_revenue"`, `source_field: "amount"`.
*   **Task:** Define `GlobalSettings`.
    *   Attributes: `company_name`, `ui_theme`, `currency`.

### 1.2. Generic Config Parser
*   **Task:** Refactor `src/config.rs` to parse this advanced structure using `serde`.
*   **Validation:** Ensure config is valid on startup (panic if Aggregation target field doesn't exist).

---

## 2. THE GENERIC PROCESSOR & DEEP SCAN (The Muscle)
**Goal:** The processor must walk the entire directory tree of an Island, not just the root file, and calculate totals based on the Config.

### 2.1. Refactor Event Processor
*   **Task:** Delete hardcoded fields (Client, Operator) in `src/processor.rs`.
*   **Task:** Implement `DynamicEntityUpsert`. Iterate through `config.clouds`. For each defined Cloud (e.g., "Client"), look for that key in the Island's `meta.yaml`.

### 2.2. Implement Deep Scanner (Aggregator)
*   **Task:** Create `src/aggregator.rs`.
*   **Logic:**
    1.  When an Island is processed, check `config.islands.aggregations`.
    2.  Use `glob` crate to find matching files (e.g., `INTERNAL/Finances/*.yaml`).
    3.  Parse each sub-file.
    4.  Perform the math (Sum, Count, Average).
    5.  Return a `HashMap<String, f64>` (e.g., `{"total_revenue": 5000.00}`).
*   **Output:** This calculated data is injected into the Island's in-memory / SQL representation, acting as "Virtual Columns".

---

## 3. THE CONFLICT RESOLUTION SYSTEM (The Safety Valve)
**Goal:** Prevent "Dirty Data" entry. The system halts implicit creation if ambiguity exists.

### 3.1. The Pending State
*   **Task:** Create a new internal SQL table: `SystemEvents` or `PendingActions`.
    *   Columns: `id`, `type` (e.g., "CreateNewEntity"), `payload` (JSON), `status` (Pending/Resolved).
*   **Logic Change in `cloud.rs`:**
    *   When `upsert_entity` is called for a new value (e.g., Client: "Microsft"):
    *   **Step 1:** Fuzzy match against existing DB (Levenshtein distance).
    *   **Step 2:** If exact match -> Link.
    *   **Step 3:** If NO match -> **DO NOT INSERT**. Instead, create a `PendingAction`.
    *   **Step 4:** Return a temporary "Unresolved" status to the UI.

### 3.2. Frontend Notification Center
*   **Task:** Create a WebSocket/Subscription channel for `SystemEvents`.
*   **UI:** Create a "Notification Bell" or "Action Required" modal.
*   **UX:**
    *   "We found a new client 'Microsft' in Project Phoenix. Did you mean 'Microsoft' (Existing) or Create New?"
    *   If 'Microsoft': Update the `meta.yaml` file on disk automatically.
    *   If 'Create New': Commit to Cloud DB.

---

## 4. BI-DIRECTIONAL SYNC (The Nervous System)
**Goal:** Editing in the Dashboard writes to the Filesystem. Local-First compliance.

### 4.1. File System Writer
*   **Task:** Create `src/fs_writer.rs`.
*   **Function:** `update_yaml_field(path: PathBuf, field: String, value: Value)`.
    *   Must use a YAML parser that preserves comments (e.g., `serde_yaml` with careful handling or `yaml-rust` purely for editing).
*   **Safety:** Atomic writes (write to temp file, then rename) to prevent corruption.

### 4.2. GraphQL Mutations
*   **Task:** Implement generic mutations based on Config.
    *   `updateIslandField(island_id: ID, field: String, value: String)`
    *   `createIsland(name: String, template_type: String)`
*   **Flow:** Mutation -> Calls `fs_writer` -> Updates File -> OS triggers Watcher -> Processor updates DB -> UI updates via Subscription.

---

## 5. DYNAMIC FRONTEND (The Skin)
**Goal:** The UI renders itself based on the Config.

### 5.1. Dynamic Routes
*   **Task:** React Router setup.
    *   Route: `/dashboard/islands` (Projects)
    *   Route: `/dashboard/clouds/:cloudName` (e.g., /dashboard/clouds/clients)
*   **Logic:** Fetch `strata.config` via GraphQL on app start. Generate sidebar links from `DEFINITIONS`.

### 5.2. Generic Data Tables
*   **Task:** Create a `<DynamicTable cloud={cloudName} />` component.
    *   It asks GraphQL for the fields defined in config for that cloud.
    *   Renders columns dynamically.

---

## 6. EXTENDED CLOUD LOGIC (The Insight)
**Goal:** Clouds are not just lists; they are analytical views derived from Islands.

### 6.1. Reverse Indexing
*   **Task:** When processing Islands, maintain a "Reverse Index".
    *   If Project A has Client X and Revenue 100.
    *   Update Client X's cached stats: `total_revenue += 100`.
*   **Implementation:** Complex SQL Views or on-the-fly aggregation in Rust.

### 6.2. Cloud Metadata
*   **Task:** Allow Clouds to have their own "Home Folders" (Optional).
    *   Example: A `CLIENTS` folder where each client has a `client_meta.yaml` for storing data that doesn't belong in a specific project (e.g., Annual Contracts).

---

## EXECUTION ORDER
1.  **Refactor Config:** Define the new strict schema.
2.  **Writer Module:** Enable Rust to safely edit YAML.
3.  **Refactor Processor:** Make it generic and recursive (Deep Scan).
4.  **Conflict System:** Implement the "Pending" table and logic.
5.  **Frontend Core:** Dynamic Routing and Action Center.


# INSTRUCTIONS FOR UPDATING DEVELOPER_GUIDE.md

The current Developer Guide is outdated due to the shift to Phase 2 Architecture. Please update `DEVELOPER_GUIDE.md` with the following sections.

## 1. Updated Architecture Diagram
Replace the simple data flow with the **Bi-Directional Confirmation Loop**:
1.  **UI/User** changes data (File Edit or GUI Mutation).
2.  **Watcher** detects change.
3.  **Processor** parses strict YAML.
4.  **Deep Scanner** aggregates sub-folder data (Sum/Count).
5.  **Conflict Guard:**
    *   Checks if new entities (Clients/Operators) are ambiguous.
    *   If Ambiguous: Creates `PendingAction`.
    *   If Clear: Commits to SQL `Cloud` Tables.
6.  **Resolution:**
    *   If `PendingAction`: UI prompts user.
    *   User decision triggers **FS Writer** to correct the file OR commits the new entity.

## 2. The Configuration Protocol
Explain that `src/main.rs` and `src/processor.rs` NO LONGER contain business logic.
*   **Rule:** If you need a new entity type, you edit `strata.config`, NOT the Rust code.
*   **Rule:** Every Island must have a `meta.yaml`.
*   **Rule:** Sub-folder aggregation rules are defined in `strata.config` under `aggregations`.

## 3. The "Local-First" Law
Add a specific section on Data Integrity:
*   **The Filesystem is the Single Source of Truth.**
*   The Database (SQLite) is merely a cached index for performance.
*   We NEVER write to SQLite "Islands" table directly. We write to the YAML file, and let the Watcher update SQLite.
*   Exception: `Cloud` tables (like specific settings for a Client that don't live in a project) can be modified, but ideally should also be backed by a file in a `CLOUDS/` directory.

## 4. Frontend Dynamic Rendering
Explain to frontend developers:
*   Do not hardcode routes like `/clients`.
*   Fetch the `Config` query on startup.
*   Iterate through `config.definitions` to build the Sidebar and Routes.
*   Implement the `NotificationCenter` for the Conflict Resolution system.

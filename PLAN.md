# STRATA DEVELOPMENT PLAN

## PHASE 1: The Core & Watcher

### 1.1: Setup Workspace & Config
- [x] Initialize Rust project.
- [x] Configure `Cargo.toml` dependencies.
- [x] Create `strata.config` (Source of Truth).
- [x] Define `Config` struct in `src/config.rs`.
- [x] Scaffold `src/main.rs` to load config.

### 1.2: Implement Watcher
- [x] Implement `Watcher` service using `notify`.
- [x] Handle debounce logic.

### 1.3: Testing
- [x] Test file events and config reloading.

## PHASE 2: Context Engine
- [x] Implement `ContextWalker`.
- [x] Implement `ContentProcessor` (Binary checks, XML format).
- [x] Test on dummy folder.

## PHASE 3: The Cloud (SQL Bridge)
- [x] Implement `SqliteManager`.
- [x] Auto-generate `CREATE TABLE`.
- [x] Implement `Upsert` Logic.
- [x] Test with manual upsert.

## PHASE 4: The Sky & API
- [x] Integrate `DuckDB` (Simulated via SQLite Views).
- [x] Build the `Axum` server with GraphQL.
- [x] Test GraphQL query.

## PHASE 5: The Oracle (AI Agents)
- [x] Implement `ToolGenerator`.
- [x] Test JSON Schema output.

## PHASE 6: Integration (The Wires)
- [x] Implement `EventProcessor`.
- [x] Connect Watcher -> Processor -> Cloud.
- [x] Test End-to-End: Edit `meta.yaml` -> Query API.

## PHASE 7: The Interface (Frontend)
- [x] Scaffold React/Vite Project.
- [x] Implement Dashboard UI (Stats, Lists, Oracle).
- [x] Connect Frontend to GraphQL API.
- [x] Create Unified Startup Script (`run.sh`).

## COMPLETION
- Project successfully built and tested.
- Daemon runs, watches files, provides GraphQL API, and generates AI Tools.
- Template Repository "Project Phoenix" created and active.
- Full Stack System live via `./run.sh`.

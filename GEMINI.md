# PROJECT VALTER: The Hyper-Converged Data Operating System
## MASTER ARCHITECTURAL PLAN & DEVELOPMENT PROTOCOL

**TARGET SYSTEM:** "VALTER" - A Local-First, AI-Native, Filesystem-Based ERP/Database.
**ROLE:** You are the **Lead Systems Architect and Senior Rust Engineer**.
**OBJECTIVE:** Build a production-grade, open-source database daemon from scratch that unifies the filesystem, SQL, Graph, and AI.

---

## 1. THE CORE PHILOSOPHY
We are rejecting the traditional database model. VALTER operates on the premise that **The Filesystem is the User Interface**.
1.  **The Daemon:** A background service that watches file changes.
2.  **The Source of Truth:** A single configuration file (`valter.config`) that defines the logic.
3.  **The Goal:** To allow a user to manage a business simply by creating folders and editing YAML files, while the Daemon automatically structures, indexes, and enables AI queries on that data.

---

## 2. DETAILED TECHNICAL SPECIFICATIONS

### A. The "Context Engine" (File Content Logic)
*Requirement:* The database must be able to read, understand, and vector-embed the *contents* of the project folders it manages.
*Algorithm:* You must implement a module called `ContextEngine` with the following specific logic:
1.  **Traversal:** Recursively walk directory trees starting from a defined "Island" root.
2.  **Ignore Rules:** It must utilize standard `.gitignore` parsing (via the `ignore` crate) to skip system files, build artifacts, and hidden directories (`.git`).
3.  **Binary Detection:** Before reading a file, read the first 1024 bytes. If null bytes (`0x00`) are detected or if the file signature suggests a binary format (images, compiled executables), SKIP the file.
4.  **Token Estimation:** Calculate rough token counts (using simple whitespace/punctuation splitting) for each file.
5.  **Output Format:** When queried, the engine must return a concatenated string of the directory context in this exact format:
    ```xml
    <file path="src/main.rs">
    [FILE CONTENT HERE]
    </file>
    <file path="README.md">
    [FILE CONTENT HERE]
    </file>
    ```

### B. The 4 Layers of Architecture

#### 1. ISLANDS (The Physical / Document Layer)
*   **Definition:** Specific folders (e.g., `/DEV/*`) are "Islands."
*   **Structure:** Each Island has a root `meta.yaml`.
*   **Sub-Documents:** Specific subfolders (e.g., `/DEV/Project A/Finances/`) contain list-item documents (e.g., `invoice_001.yaml`).
*   **Behavior:** The Daemon monitors these paths using OS-level events (`inotify`/`FSEvents`).

#### 2. CLOUDS (The Relational / SQL Layer)
*   **Technology:** Embedded `SQLite` (via `rusqlite`).
*   **The "Implicit Creation" Rule:** This is a critical feature, now enhanced with a **Safety Valve**.
    *   *Scenario:* User edits `/DEV/Project A/metadata.yaml` and adds `Client: "Omega Corp"`.
    *   *System Action:* The Daemon parses this, checks the SQL Table `Clients`, and if the entity is unknown or ambiguous (e.g., a typo), it creates a `PendingAction` for user review in the UI instead of polluting the database.

#### 3. SKY (The Graph & Analytics Layer)
*   **Technology:** GraphQL API with dynamic resolvers.
*   **Graph Logic:** The system must map relationships.
    *   *Query:* "Who worked on the most projects?" -> The system queries the relationship edges between `Operator` (SQL) and `Project` (Island).
*   **Vector Logic (Future):** Using the **Context Engine**, the system will embed the project's codebase and notes.
    *   *Query:* "Which project uses React?" -> The system will perform a semantic search on the vector index.

#### 4. ORACLE (The AI / API Layer)
*   **Technology:** `Axum` (Web Server) + `Async-GraphQL`.
*   **Auto-Building API:** As the Schema changes, the GraphQL resolvers are reconfigured dynamically. Real-time updates are handled via the frontend polling mechanism.
*   **Agentic Functions:** The system must expose an endpoint that returns a JSON Schema of **Tools** for an external AI Agent.

---

## 3. THE "SINGLE SOURCE OF TRUTH" FILE
The entire system logic is defined in a generic file (e.g., `valter.config`).
**Required Parsing Logic:**
```yaml
# Example Structure
GLOBAL:
  # ... global settings
CLOUDS:
  - name: Operator
    fields:
      - key: name
        type: string
ISLANDS:
  - name: Project
    root_path: "./dev-projects-folder/*"
    meta_file: "meta.yaml"
    relations:
      - field: operator
        target_cloud: Operator
    aggregations:
      - name: task_count
        path: "INTERNAL/*.md"
        logic: count
```

---

## 4. DEVELOPMENT ROADMAP & STATUS

**Constraint:** You will act as an iterative builder. You will not write all code at once. You will follow this cycle for every single step:
1.  **Specification:** Define the Structs/Traits.
2.  **Test (TDD):** Write a *failing* test that asserts the desired functionality.
3.  **Implementation:** Write the Rust code to pass the test.
4.  **Refactor:** Optimize.

### PHASE 1: The Core & Watcher [COMPLETED]
*   [x] **1.1:** Setup Rust workspace. Define the `Config` struct.
*   [x] **1.2:** Implement the `Watcher` service using `notify`.
*   [x] **1.3:** Test file events and config reloading.

### PHASE 2: The Data Engine [COMPLETED]
*   [x] **2.1:** Implement `SqliteManager` with dynamic table generation.
*   [x] **2.2:** Implement `EventProcessor` with "Implicit Creation" and the "Safety Valve" (Pending Actions).
*   [x] **2.3:** Implement `Aggregator` for deep scanning of sub-folders.

### PHASE 3: The API & Interface [COMPLETED]
*   [x] **3.1:** Build the `Axum` server with `Async-GraphQL`.
*   [x] **3.2:** Build the dynamic React frontend that renders based on the config.
*   [x] **3.3:** Implement CI/CD pipeline for testing and releases.

### PHASE 4: The Semantic Layer (Next)
*   [ ] **4.1:** Integrate a vector database (e.g., `LanceDB`).
*   [ ] **4.2:** Implement the `ContextEngine` to embed file contents.
*   [ ] **4.3:** Upgrade the Oracle to use a RAG pipeline for queries.

---

## 5. DEVELOPMENT WORKFLOW & PROTOCOL

This protocol is mandatory to ensure code quality and a clean project history.

### A. Git Workflow
1.  All new work must be done on a feature branch created from `main`.
    *   Branch naming: `feat/add-new-feature` or `fix/resolve-bug-123`.
2.  All work must be merged back into `main` via a Pull Request (PR).
3.  PRs must pass all CI checks before they can be merged.

### B. Commit Messages
We use the **Conventional Commits** specification. This is not optional, as it drives automated changelog generation.
*   **`feat:`** for a new feature.
*   **`fix:`** for a bug fix.
*   **`docs:`** for changes to documentation (`.md` files).
*   **`style:`** for code formatting changes (no logic change).
*   **`refactor:`** for code changes that neither fix a bug nor add a feature.
*   **`test:`** for adding or correcting tests.
*   **`chore:`** for build process or tooling changes (e.g., updating CI workflows).

*Example:* `git commit -m "feat(api): Add mutation for creating new islands"`

### C. Changelog Management (`CHANGELOG.md`)
*   **DO NOT** edit `CHANGELOG.md` manually.
*   The changelog will be automatically generated and updated by our release workflow based on the conventional commit messages since the last tag.

### D. Correcting Files & The Development Cycle
1.  **Modify:** Make your code changes locally.
2.  **Run:** Use `just dev` to run the entire stack and test your changes in the integrated environment.
3.  **Test:** Run `cargo test` and any other relevant checks.
4.  **Commit:** Use the conventional commit format.
5.  **Push & PR:** Push your branch and create a Pull Request. Let the CI system do the final validation.

---

## 6. INITIATION
**Current Status:** Phase 4 planning. Core system is operational.
**Instructions:**
1.  Acknowledge this updated architecture.
2.  Adhere strictly to the Development Workflow & Protocol.
3.  Begin work on tasks for **Phase 4**.

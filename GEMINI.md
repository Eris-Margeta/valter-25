# PROJECT STRATA: The Hyper-Converged Database System
## MASTER ARCHITECTURAL PLAN & DEVELOPMENT PROTOCOL

**TARGET SYSTEM:** "Strata" - A Local-First, AI-Native, Filesystem-Based ERP/Database.
**ROLE:** You are the **Lead Systems Architect and Senior Rust Engineer**.
**OBJECTIVE:** Build a production-grade, open-source database daemon from scratch that unifies the filesystem, SQL, Graph, and AI.

---

## 1. THE CORE PHILOSOPHY
We are rejecting the traditional database model. Strata operates on the premise that **The Filesystem is the User Interface**.
1.  **The Daemon:** A background service that watches file changes.
2.  **The Source of Truth:** A single schema file (`strata.def`) that defines the logic.
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
*   **Structure:** Each Island has a root `metadata.yaml`.
*   **Sub-Documents:** Specific subfolders (e.g., `/DEV/Project A/Finances/`) contain list-item documents (e.g., `invoice_001.yaml`).
*   **Behavior:** The Daemon monitors these paths using OS-level events (`inotify`/`FSEvents`).

#### 2. CLOUDS (The Relational / SQL Layer)
*   **Technology:** Embedded `SQLite` (via `rusqlite`).
*   **The "Implicit Creation" Rule:** This is the most critical feature.
    *   *Scenario:* User edits `/DEV/Project A/metadata.yaml` and adds `Client: "Omega Corp"`.
    *   *System Action:* The Daemon parses this. It checks the SQL Table `Clients`.
    *   *Branch A:* If "Omega Corp" exists, it retrieves the UUID and links the Foreign Key in the internal graph.
    *   *Branch B:* If "Omega Corp" does *not* exist, the Daemon **automatically INSERTs** a new row into the `Clients` table with a new UUID, then links it.
    *   *Constraint:* We do not need to pre-seed the SQL tables. They grow organically based on the files in the Islands.

#### 3. SKY (The Graph & Analytics Layer)
*   **Technology:** Embedded `DuckDB` (for OLAP) + `LanceDB` (for Vector Search).
*   **Graph Logic:** The system must map relationships.
    *   *Query:* "Who worked on the most projects?" -> The system queries the relationship edges between `Operator` (SQL) and `Project` (Island).
*   **Vector Logic:** Using the **Context Engine**, the system embeds the project's codebase and notes.
    *   *Query:* "Which project uses React?" -> The system performs a semantic search on the vector index generated from the file contents.

#### 4. ORACLE (The AI / API Layer)
*   **Technology:** `Axum` (Web Server) + `Async-GraphQL`.
*   **Auto-Building API:**
    *   As the Schema changes, the GraphQL resolvers must be regenerated/reconfigured dynamically.
    *   Real-time updates via WebSockets/Subscriptions (Frontend does not refresh).
*   **Agentic Functions:**
    *   The system must expose an endpoint that returns a JSON Schema of **Tools** for an external AI Agent.
    *   *Example Tool:* `{"name": "get_client_revenue", "parameters": {"client_name": "string"}}`.
    *   The Oracle (AI) uses these tools to query the data structure.

---

## 3. THE "SINGLE SOURCE OF TRUTH" FILE
The entire system logic is defined in a generic file (e.g., `strata.config`).
**Required Parsing Logic:**
```yaml
# Example Structure
DEFINITIONS:
  - CLOUD: Operator
    fields: [name (unique), hourly_rate]

  - CLOUD: Client
    fields: [name (unique), email]

  - ISLAND: Project
    path: "./DEV/*"
    meta_file: "meta.yaml"
    relations:
      - operator -> CLOUD.Operator(name)  # The Implicit Link
      - client -> CLOUD.Client(name)

VIEWS:
  - GRAPH: "Productivity"
    logic: "SUM(Project.tasks.hours) GROUP BY operator"
```

---

## 4. DEVELOPMENT ROADMAP & TESTING PROTOCOL

**Constraint:** You will act as an iterative builder. You will not write all code at once. You will follow this cycle for every single step:
1.  **Specification:** Define the Structs/Traits.
2.  **Test (TDD):** Write a *failing* test that asserts the desired functionality.
3.  **Implementation:** Write the Rust code to pass the test.
4.  **Refactor:** Optimize.

### PHASE 1: The Core & Watcher
*   **1.1:** Setup Rust workspace. Define the `Config` struct to parse the "Source of Truth" file.
*   **1.2:** Implement the `Watcher` service (using `notify` crate) that debounces events.
*   **1.3:** Test: Create a temp directory, modify a file, ensure the Event Bus receives the correct signal.

### PHASE 2: The Context Engine (File Logic)
*   **2.1:** Implement `ContextWalker`. Logic: Recursive read using `ignore` crate.
*   **2.2:** Implement `ContentProcessor`. Logic: Binary detection (first 1024 bytes check), Text concatenation in XML format.
*   **2.3:** Test: Run on a dummy folder with mixed binary/text files. Assert output string matches expected format.

### PHASE 3: The Cloud (SQL Bridge)
*   **3.1:** Implement `SqliteManager`. Auto-generate `CREATE TABLE` statements from the Config.
*   **3.2:** Implement the `Upsert` Logic (The Implicit Creation).
*   **3.3:** Test: Parse a YAML file with a new "Client", assert that a new Row appears in the SQLite database.

### PHASE 4: The Sky & API
*   **4.1:** Integrate `DuckDB` for analytics views.
*   **4.2:** Build the `Axum` server with GraphQL.
*   **4.3:** Test: Send a GraphQL query for a calculated view (e.g., "Total Revenue"), assert correct JSON response.

### PHASE 5: The Oracle (AI Agents)
*   **5.1:** Implement `ToolGenerator`. Convert Schema -> OpenAI Function JSON.
*   **5.2:** Test: Verify the JSON output matches OpenAI specifications.

---

## 5. INITIATION
**Current Status:** Green field (Empty Folder).
**Instructions:**
1.  Acknowledge this detailed architecture.
2.  Confirm you understand the "Implicit Creation" and "Context Engine" algorithms.
3.  Begin **Phase 1.1**. Provide the `Cargo.toml` with necessary dependencies (`tokio`, `serde`, `serde_yaml`, `notify`, `rusqlite`, `axum`, `async-graphql`, `ignore`, `anyhow`, `tracing`) and the basic `main.rs` scaffolding.

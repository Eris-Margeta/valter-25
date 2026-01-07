# PLAN 3: THE TAURI MIGRATION & UNIFIED INTERFACE
**Objective:** Transition Valter from a disjointed Client-Server model to a unified, cross-platform **Native Application** using Tauri v2.
**Goal:** A single codebase that runs as a macOS/Windows/Linux app OR a headless web server, with centralized types and a streamlined Developer Experience (DX).

---

## 1. ARCHITECTURAL SHIFT

### Current Architecture (Decoupled)
*   **Backend:** `core` (Rust binary) runs HTTP Server + Watcher.
*   **Frontend:** `dashboard` (React) connects via HTTP (`localhost:port`).
*   **Friction:** Requires running two terminals. No native OS integration (menu bars, native file dialogs).

### Target Architecture (Converged)
*   **The Core Library:** We refactor `core` from a *binary* into a *library crate*.
*   **The Runner (Tauri):** Tauri acts as the "host". It initializes the `core` library in a background thread.
*   **The UI:** The React Dashboard runs inside the Tauri WebView.
*   **Data Transport:** 
    *   **Desktop Mode:** UI talks to Core via Tauri IPC (Inter-Process Communication) or local-loopback GraphQL.
    *   **Browser Mode:** UI talks to Core via standard HTTP GraphQL (for remote access).

---

## 2. EXECUTION ROADMAP

### PHASE 3.1: Core Refactor (Bin to Lib)
**Goal:** Make the backend logic consumable by Tauri without launching a separate process.
*   [ ] **Refactor `Cargo.toml`:** Change `core` to define a `[lib]` entry point.
*   [ ] **Extract `main.rs`:** Move the bootstrapping logic (Watcher setup, DB init, API start) into a public function `valter_core::init()`.
*   [ ] **State Management:** Ensure `SqliteManager` and `EventProcessor` can be shared safely across threads (Arc/Mutex patterns are already in place, verify they hold up).

### PHASE 3.2: Tauri Integration
**Goal:** Initialize the Tauri shell in the Monorepo.
*   [ ] **Initialize Tauri:** Run `pnpm tauri init` in the root (or specific folder).
*   [ ] **Directory Structure:**
    *   Move `dashboard` contents into `apps/desktop/src` (or similar structure).
    *   Configure Tauri to use `core` as a dependency.
*   [ ] **Lifecycle Management:** Update Tauri's `main.rs` to spawn the `valter_core` logic on application start.
*   [ ] **Config Handover:** Ensure Tauri passes the correct paths (Dev vs Prod) to the internal `core` logic.

### PHASE 3.3: Frontend Routing & Dynamic Views
**Goal:** Move from a single-page dashboard to a routed application with dedicated entity pages.
*   [ ] **Install React Router:** Switch from conditional rendering to `react-router-dom`.
*   [ ] **Dynamic Routes:** Implement generic routes:
    *   `/entity/:cloud_name/:id` (e.g., `/entity/operator/123`)
    *   `/island/:island_name` (e.g., `/island/Project_Phoenix`)
*   [ ] **The "Universal Form":** Create a generic React component `<EntityForm definition={...} data={...} />`.
    *   It reads `valter.config` fields.
    *   It generates Inputs/Selects based on types.
    *   It calls `update_island_field` on save.

### PHASE 3.4: Native Capabilities
**Goal:** Utilize OS features.
*   [ ] **Native Menus:** Add "Rescan", "Open Config", "Quit" to the system menu bar.
*   [ ] **File Dialogs:** When creating a new project, use the OS file picker to select the root folder (optional, if we want to support multiple roots easily).

### PHASE 3.5: Build & CI/CD
**Goal:** produce `.app` and `.exe` files.
*   [ ] **Update `Justfile`:** Add `just build-app` commands.
*   [ ] **GitHub Actions:** Configure runners for macOS and Windows to build Tauri artifacts.

---

## 3. IMPLEMENTATION DETAILS & RULES

### A. Directory Restructuring (Monorepo)
We will move towards a clearer structure:
```text
/
├── core/                 # Rust Logic (Library)
├── app/                  # The Tauri Wrapper (Rust + React)
│   ├── src-tauri/        # Rust entry point
│   ├── src/              # React Frontend (Moved from dashboard)
├── valter.config         # Prod Config
└── Justfile              # Orchestrator
```

### B. The API Bridge Strategy
To support both **Desktop** and **Web-Only** modes without duplicating code:
1.  We will keep the **GraphQL Server (Axum)** running inside the Tauri app on `localhost:9090` (or random port).
2.  The Frontend will continue to use `graphqlRequest`.
3.  **Why?** Rewriting 20+ GraphQL resolvers to Tauri Commands is time-consuming and breaks the "Headless Server" capability.
4.  **Security:** We will bind the internal server to `127.0.0.1` and secure it with a token generated at runtime if necessary.

### C. Refactoring Protocol (Strict)
1.  **Do NOT delete `dashboard` immediately.** We will copy it to `app/src`, verify it works, then remove the old folder.
2.  **Kebab-Case Enforcement:** All new React files (e.g., `entity-view.tsx`, `side-bar.tsx`) must use kebab-case.
3.  **Clean Imports:** When moving React components, ensure we strip out unused imports (`React`, legacy CSS) and switch to Named Exports.

---

## 4. IMMEDIATE NEXT STEPS (Session 1)

1.  **Backup:** Commit current state.
2.  **Scaffold:** Create the Tauri structure.
3.  **Lib-ify Core:** Modify `core/src/lib.rs` to expose the startup logic.
4.  **Connect:** Make Tauri start the Core.

[/FILE_CONTENT_END]Here is the detailed **PLAN-3.md**. It outlines the strategic migration to Tauri while preserving the core "Filesystem-first" architecture and ensuring the system remains robust.

[FILE_CONTENT_START ID="NEW_FILE" PATH="PLAN-3.md"]
# PLAN 3: THE TAURI MIGRATION & UNIFIED INTERFACE
**Objective:** Transition Valter from a disjointed Client-Server model to a unified, cross-platform **Native Application** using Tauri v2.
**Goal:** A single codebase that runs as a macOS/Windows/Linux app OR a headless web server, with centralized types and a streamlined Developer Experience (DX).

---

## 1. ARCHITECTURAL SHIFT

### Current Architecture (Decoupled)
*   **Backend:** `core` (Rust binary) runs HTTP Server + Watcher.
*   **Frontend:** `dashboard` (React) connects via HTTP (`localhost:port`).
*   **Friction:** Requires running two terminals. No native OS integration (menu bars, native file dialogs).

### Target Architecture (Converged)
*   **The Core Library:** We refactor `core` from a *binary* into a *library crate*.
*   **The Runner (Tauri):** Tauri acts as the "host". It initializes the `core` library in a background thread.
*   **The UI:** The React Dashboard runs inside the Tauri WebView.
*   **Data Transport:** 
    *   **Desktop Mode:** UI talks to Core via Tauri IPC (Inter-Process Communication) or local-loopback GraphQL.
    *   **Browser Mode:** UI talks to Core via standard HTTP GraphQL (for remote access).

---

## 2. EXECUTION ROADMAP

### PHASE 3.1: Core Refactor (Bin to Lib)
**Goal:** Make the backend logic consumable by Tauri without launching a separate process.
*   [ ] **Refactor `Cargo.toml`:** Change `core` to define a `[lib]` entry point.
*   [ ] **Extract `main.rs`:** Move the bootstrapping logic (Watcher setup, DB init, API start) into a public function `valter_core::init()`.
*   [ ] **State Management:** Ensure `SqliteManager` and `EventProcessor` can be shared safely across threads (Arc/Mutex patterns are already in place, verify they hold up).

### PHASE 3.2: Tauri Integration
**Goal:** Initialize the Tauri shell in the Monorepo.
*   [ ] **Initialize Tauri:** Run `pnpm tauri init` in the root (or specific folder).
*   [ ] **Directory Structure:**
    *   Move `dashboard` contents into `apps/desktop/src` (or similar structure).
    *   Configure Tauri to use `core` as a dependency.
*   [ ] **Lifecycle Management:** Update Tauri's `main.rs` to spawn the `valter_core` logic on application start.
*   [ ] **Config Handover:** Ensure Tauri passes the correct paths (Dev vs Prod) to the internal `core` logic.

### PHASE 3.3: Frontend Routing & Dynamic Views
**Goal:** Move from a single-page dashboard to a routed application with dedicated entity pages.
*   [ ] **Install React Router:** Switch from conditional rendering to `react-router-dom`.
*   [ ] **Dynamic Routes:** Implement generic routes:
    *   `/entity/:cloud_name/:id` (e.g., `/entity/operator/123`)
    *   `/island/:island_name` (e.g., `/island/Project_Phoenix`)
*   [ ] **The "Universal Form":** Create a generic React component `<EntityForm definition={...} data={...} />`.
    *   It reads `valter.config` fields.
    *   It generates Inputs/Selects based on types.
    *   It calls `update_island_field` on save.

### PHASE 3.4: Native Capabilities
**Goal:** Utilize OS features.
*   [ ] **Native Menus:** Add "Rescan", "Open Config", "Quit" to the system menu bar.
*   [ ] **File Dialogs:** When creating a new project, use the OS file picker to select the root folder (optional, if we want to support multiple roots easily).

### PHASE 3.5: Build & CI/CD
**Goal:** produce `.app` and `.exe` files.
*   [ ] **Update `Justfile`:** Add `just build-app` commands.
*   [ ] **GitHub Actions:** Configure runners for macOS and Windows to build Tauri artifacts.

---

## 3. IMPLEMENTATION DETAILS & RULES

### A. Directory Restructuring (Monorepo)
We will move towards a clearer structure:
```text
/
├── core/                 # Rust Logic (Library)
├── app/                  # The Tauri Wrapper (Rust + React)
│   ├── src-tauri/        # Rust entry point
│   ├── src/              # React Frontend (Moved from dashboard)
├── valter.config         # Prod Config
└── Justfile              # Orchestrator
```

### B. The API Bridge Strategy
To support both **Desktop** and **Web-Only** modes without duplicating code:
1.  We will keep the **GraphQL Server (Axum)** running inside the Tauri app on `localhost:9090` (or random port).
2.  The Frontend will continue to use `graphqlRequest`.
3.  **Why?** Rewriting 20+ GraphQL resolvers to Tauri Commands is time-consuming and breaks the "Headless Server" capability.
4.  **Security:** We will bind the internal server to `127.0.0.1` and secure it with a token generated at runtime if necessary.

### C. Refactoring Protocol (Strict)
1.  **Do NOT delete `dashboard` immediately.** We will copy it to `app/src`, verify it works, then remove the old folder.
2.  **Kebab-Case Enforcement:** All new React files (e.g., `entity-view.tsx`, `side-bar.tsx`) must use kebab-case.
3.  **Clean Imports:** When moving React components, ensure we strip out unused imports (`React`, legacy CSS) and switch to Named Exports.

---

## 4. IMMEDIATE NEXT STEPS (Session 1)

1.  **Backup:** Commit current state.
2.  **Scaffold:** Create the Tauri structure.
3.  **Lib-ify Core:** Modify `core/src/lib.rs` to expose the startup logic.
4.  **Connect:** Make Tauri start the Core.


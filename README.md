# STRATA ENGINE

**A Local-First, AI-Native, Hyper-Converged Data Operating System.**

Strata turns your filesystem into a structured, queryable, and AI-ready database. It watches your folders, indexes your metadata, aggregates your finances, and provides a generic GraphQL API + AI Oracle to interact with your digital empire.

---

## 🚀 Quick Start

### Prerequisites
*   **Rust** (latest stable)
*   **Node.js** (v18+) & **pnpm**
*   **Gemini API Key** (for the Oracle)

### Installation
1.  **Clone the Repository**
2.  **Configure Environment**
    Export your API key in your shell:
    ```bash
    export GEMINI_API_KEY="your_api_key_here"
    ```
3.  **Launch the System**
    We provide a unified startup script that handles cleanup, installation, and logging:
    ```bash
    ./run.sh
    ```

This will launch:
*   **Strata Daemon** (Backend) at `http://localhost:8000`
*   **Strata Dashboard** (Frontend) at `http://localhost:5173`

---

## 🧠 Core Concepts

### 1. The Source of Truth (`strata.config`)
This file defines your universe (Whitelabel). It tells Strata what "Things" exist (Clients, Operators, Assets) and how to calculate metrics (Revenue, Hours).

### 2. Islands (Your Projects)
Islands are folders in your `./DEV` directory. To add data to Strata, you **do not** write SQL. You simply create a folder.
*   **Deep Scan:** Strata automatically looks into sub-folders (e.g., `INTERNAL/Finances`) to sum up invoices and hours based on your config.

### 3. Safety Valve (Conflict Resolution)
Strata protects your data quality.
*   If you type `Client: Mircosoft` (typo) in a file, Strata **will not** pollute your database.
*   It pauses and alerts you in the **Action Center**.
*   You can choose to **Create New**, **Ignore**, or **Fix File** (Merge) directly from the UI.

### 4. Bi-Directional Sync
*   **Disk -> DB:** Changing a file updates the dashboard instantly.
*   **DB -> Disk:** Editing a status in the Dashboard updates the `meta.yaml` file on your hard drive.

---

## 🛠️ Tech Stack

*   **Core:** Rust (Tokio, Axum, Rusqlite, Notify, Async-GraphQL, StrSim)
*   **Frontend:** React, Vite, Tailwind CSS, Lucide
*   **AI:** Google Gemini 1.5 Flash via REST API

## 📂 Project Structure

*   `strata.config`: The Whitelabel Definition file.
*   `/src`: Rust Backend
    *   `processor.rs`: Event handling & Deep Aggregation logic.
    *   `cloud.rs`: SQLite manager & Safety Valve logic.
    *   `fs_writer.rs`: Safe filesystem modification.
    *   `api.rs`: Generic GraphQL Resolvers.
*   `/web`: React Frontend
    *   `components/ActionCenter.tsx`: Interface for resolving data conflicts.
    *   `components/DynamicTable.tsx`: Generic data grid with inline editing.



# STRATA ENGINE

**A Local-First, AI-Native Hyper-Converged Database System.**

Strata turns your filesystem into a structured, queryable, and AI-ready database. It watches your folders, indexes your metadata, and provides a GraphQL API + AI Oracle to interact with your digital empire.

---

## 🚀 Quick Start

### Prerequisites
*   **Rust** (latest stable)
*   **Node.js** (v18+) & **npm**
*   **Gemini API Key** (for the Oracle)

### Installation
1.  **Clone the Repository**
2.  **Configure Environment**
    Export your API key in your shell:
    ```bash
    export GEMINI_API_KEY="your_api_key_here"
    ```
3.  **Launch the System**
    We provide a unified startup script:
    ```bash
    ./run.sh
    ```

This will launch:
*   **Strata Daemon** (Backend) at `http://localhost:8000`
*   **Strata Dashboard** (Frontend) at `http://localhost:5173`

---

## 🧠 Core Concepts

### 1. The Source of Truth (`strata.config`)
This file defines your universe. It tells Strata what "Things" exist.
```yaml
DEFINITIONS:
  - CLOUD: Client
    fields: [name, email]
  - ISLAND: Project
    path: "./DEV/*"
```

### 2. Islands (Your Files)
Islands are folders in your `./DEV` directory. To add data to Strata, you **do not** write SQL. You simply create a folder.
*   **Create:** `mkdir DEV/MyNewProject`
*   **Define:** Create `DEV/MyNewProject/meta.yaml`
    ```yaml
    name: "My New Project"
    client: "Acme Corp"
    operator: "Alice"
    ```
*   **Result:** Strata detects this file. It automatically creates "Acme Corp" in the `Client` table and links it to the project.

### 3. Clouds (The Database)
Strata maintains a hidden SQLite database (`strata.db`) that mirrors your files. This allows for instant querying and relational integrity.

### 4. The Oracle (AI)
The system includes an AI agent aware of your database schema.
*   **Ask:** "What projects is Alice working on?"
*   **Answer:** The Oracle queries the live database and answers with context.

---

## 🛠️ Tech Stack

*   **Core:** Rust (Tokio, Axum, Rusqlite, Notify, Async-GraphQL)
*   **Frontend:** React, Vite, Tailwind CSS, Lucide
*   **AI:** Google Gemini 1.5 Flash via REST API

## 📂 Project Structure

*   `/src`: Rust Backend Source
    *   `main.rs`: Entry point & startup logic.
    *   `watcher.rs`: Filesystem event monitor.
    *   `processor.rs`: Business logic for ingesting files.
    *   `cloud.rs`: SQLite abstraction layer.
    *   `api.rs`: GraphQL Schema & Resolvers.
    *   `oracle.rs`: AI Tool Generation.
*   `/web`: React Frontend
*   `strata.config`: Schema Definition.

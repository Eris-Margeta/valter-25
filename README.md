# VALTER ERP

**A Local-First, AI-Native, Hyper-Converged Data Operating System.**

Valter turns your filesystem into a structured, queryable, and AI-ready database. It watches your folders, indexes your metadata, aggregates your finances, and provides a generic GraphQL API + AI Oracle to interact with your digital empire.

![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen)

---

## 📋 Prerequisites

Before running Valter, ensure your system has the following installed:

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | `stable` (1.75+) | Compiling the Core Daemon. |
| **Node.js** | `v18+` | Running the Dashboard & Website. |
| **pnpm** | `latest` | Package manager for frontend. |
| **Just** | `latest` | Command runner (replaces Make/Shell scripts). |

### OS Specific Setup
*   **macOS:** `brew install rustup node pnpm just`
*   **Linux:** Use your package manager (apt/dnf/pacman) or `curl` installers.
*   **Windows:** WSL2 is recommended, or Native with PowerShell. You must have C++ Build Tools installed for Rust.

---

## 🚀 Getting Started (Development Mode)

This mode is for **contributors** or for **testing** the system using the included demo data. It runs entirely inside the repository folder (Monorepo).

1.  **Clone the Repository**
    ```bash
    git clone https://github.com/yourusername/valter.git
    cd valter
    ```

2.  **Configure Environment**
    Export your AI API key (required for the Oracle features):
    ```bash
    export GEMINI_API_KEY="your_api_key_here"
    ```

3.  **Launch via Just**
    We use `just` to orchestrate the backend and frontend simultaneously.
    ```bash
    just dev
    ```

**What happens next?**
*   **Backend** starts on `http://localhost:8000`. It uses `valter.dev.config` and watches the `./dev-projects-folder` inside the repo.
*   **Frontend** starts on `http://localhost:5173`.
*   You will see demo data (Project Phoenix) immediately.

---

## 📦 Installation (Production Mode)

This mode installs Valter as a **System Utility**. Use this if you want to use Valter to manage your *real* personal or business data.

### 1. Install
```bash
just install
```
> **Warning:** This command creates `~/.valter/` and installs the binary to `~/.local/bin/`. If you had a previous installation, the configuration file will be overwritten, but your database will be preserved.

### 2. Configure
Go to your home directory and edit the configuration to point to your real work folder:
```yaml
# ~/.valter/valter.config
ISLANDS:
  - name: "Project"
    root_path: "/Users/me/MyActualBusiness/*" # <--- UPDATE THIS
```

### 3. Run
You can now run Valter from anywhere in your terminal:
```bash
valter
```

### 4. Updating
To update the software without losing your data or configuration:
```bash
just update
```
*Valter automatically handles SQL schema migrations if the new version introduces new fields.*

---

## 🧠 Core Concepts

### 1. The Source of Truth (`valter.config`)
This file defines your universe. It tells Valter what "Things" exist (Clients, Operators, Assets) and how to calculate metrics (Revenue, Hours).
*   **Dev Mode:** Uses `valter.dev.config` (Repo root).
*   **Prod Mode:** Uses `~/.valter/valter.config` (User Home).

### 2. Islands (The Filesystem)
Islands are folders on your disk. To add data to Valter, you **do not** write SQL. You simply create a folder or edit a file.
*   **Deep Scan:** Valter automatically looks into sub-folders (e.g., `INTERNAL/Finances/*.md`) to sum up invoices and hours based on your config rules.

### 3. Safety Valve (Conflict Resolution)
Valter protects your data quality.
*   If you type `Client: Mircosoft` (typo) in a file, Valter **will not** pollute your SQL database.
*   It pauses processing for that file and creates a **Pending Action**.
*   You resolve this in the **Dashboard**: choose to **Merge** (fix the file on disk) or **Create New** (if it's actually a new client).

### 4. Bi-Directional Sync
*   **Disk -> DB:** Changing a file updates the dashboard instantly via OS-level watchers.
*   **DB -> Disk:** Editing a status in the Dashboard performs an atomic write to the `meta.yaml` file on your hard drive.

---

## 🛠️ Tech Stack

*   **Core (Backend):** Rust (Tokio, Axum, Rusqlite, Notify, Async-GraphQL).
*   **Dashboard (Frontend):** React 19, Vite, Tailwind CSS, Lucide.
*   **Docs (Website):** Astro Framework.
*   **AI:** Google Gemini 1.5 Flash via REST API.

## 📂 Project Structure

```text
VALTER-ERP/
├── core/                   # Rust Backend Daemon
│   ├── src/main.rs         # Entry point (Smart Config Detection)
│   └── src/cloud.rs        # SQLite & Auto-Migration Logic
├── dashboard/              # React Frontend
├── website/                # Astro Documentation Site
├── dev-projects-folder/    # Demo Data for 'just dev'
├── valter.dev.config       # Config for Monorepo Mode
└── Justfile                # Command Runner Definitions

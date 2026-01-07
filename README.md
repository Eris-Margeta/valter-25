# VALTER ERP

**A Local-First, AI-Native, Hyper-Converged Data Operating System.**

Valter turns your filesystem into a structured, queryable, and AI-ready database. It watches your folders, indexes your metadata, aggregates your finances, and provides a unified interface to interact with your digital empire.

![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen)

---

## 📋 Prerequisites

Before running Valter, ensure your system has the following installed:

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | `stable` (1.75+) | Compiling the Core. |
| **Node.js** | `v24+` | Running the Frontend. |
| **pnpm** | `latest` | Package manager. |
| **Just** | `latest` | Command runner. |

### OS Specific Setup
*   **macOS:** `brew install rustup node pnpm just`
*   **Linux:** Use your package manager (apt/dnf/pacman).
*   **Windows:** WSL2 or Native with PowerShell + C++ Build Tools.

---

## 🚀 Getting Started (Development)

This runs the **Native Desktop App** in development mode using the included demo data.

1.  **Clone the Repository**
    ```bash
    git clone https://github.com/Eris-Margeta/valter-25.git
    cd valter
    ```

2.  **Configure Environment**
    Export your AI API key (optional, for Oracle features):
    ```bash
    export GEMINI_API_KEY="your_api_key_here"
    ```

3.  **Launch via Just**
    ```bash
    just dev
    ```
    *This starts the Tauri window with hot-reloading.*

---

## 📦 Installation (Production)

You can install Valter as a **System Daemon** (Headless) or build the **Desktop App**.

### Option A: System Daemon (CLI)
Installs the headless binary to `~/.local/bin/valter`.
```bash
just install
```
> **Warning:** This overwrites `~/.valter/valter.config` (but preserves DB).

### Option B: Desktop App
Builds the standalone application for your OS (DMG/EXE).
```bash
just build-app
```
*Find the installer in `app/src-tauri/target/release/bundle/`.*

---

## 🧠 Core Concepts

### 1. The Source of Truth (`valter.config`)
This file defines your universe. It tells Valter what "Things" exist (Clients, Operators) and how to calculate metrics.
*   **Dev Mode:** Uses `valter.dev.config` (Repo root).
*   **Prod Mode:** Uses `~/.valter/valter.config` (User Home).

### 2. Islands (The Filesystem)
Islands are folders on your disk. To add data, you simply create a folder or edit a file. Valter watches these changes in real-time.

### 3. Safety Valve (Conflict Resolution)
Valter protects data quality. If you introduce a typo (e.g., `Client: Mircosoft`), Valter pauses processing and creates a **Pending Action**. You resolve this in the App by fixing the file (Merge) or creating a new entity.

---

## 🛠️ Tech Stack

*   **Core:** Rust (Tokio, Axum, Rusqlite, Notify, Async-GraphQL).
*   **App Shell:** Tauri v2.
*   **Frontend:** React 19, Vite, Tailwind CSS, React Router.
*   **Docs:** Astro Framework.
*   **AI:** Google Gemini 2.5 Flash via REST API.

## 📂 Project Structure

```text
VALTER-ERP/
├── core/                   # Rust Backend Library
├── app/                    # The Unified Application
│   ├── src-tauri/          # Tauri Host (Rust)
│   └── src/                # React Frontend
├── website/                # Documentation Site
├── dev-projects-folder/    # Demo Data
├── valter.dev.config       # Dev Configuration
└── Justfile                # Build Scriptsa


Cargo commands:
cargo check

Other commands:
(
INSTALL:
cargo install cargo-edit

USE:
cargo upgrade
)

(
INSTALL:
rustup toolchain install nightly 
cargo install cargo-udeps --locked

USE:
cargo +nightly udeps
)






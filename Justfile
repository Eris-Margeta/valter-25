# VALTER ERP - JUSTFILE
# Commands for Development, Installation, Maintenance, and Release.

default:
    @just --list

# --- DEVELOPMENT ---

# Run the Tauri Desktop App (Unified) in Dev Mode
dev:
    @echo "🚀 Starting Valter Desktop App..."
    cd app && pnpm tauri dev

# Run the backend logic independently (Headless Mode)
dev-core:
    @echo "⚙️ Starting Valter Core (Headless)..."
    cargo run --manifest-path core/Cargo.toml -- run

# Run only the Web Frontend (Browser Mode)
dev-web:
    @echo "🌐 Starting Valter Web Dashboard..."
    cd app && pnpm dev

# Clean artifacts
clean:
    @echo "🧹 Cleaning up..."
    rm -rf target core/target app/src-tauri/target
    rm -rf app/dist app/.vite app/node_modules
    rm -f valter.db valter.log valter.pid valter.db-shm valter.db-wal
    rm -f core/src/fs_writer.rs.bk

# --- BUILD & RELEASE ---

# Build the Desktop App (DMG/EXE)
build-app:
    @echo "📦 Building Desktop App..."
    cd app && pnpm tauri build

# Build the Headless Server Binary (Legacy/Server Mode)
build-server:
    @echo "🏗️  Building Dashboard Assets..."
    cd app && pnpm install && pnpm build
    @echo "📦 Building Headless Core..."
    cargo build --release --manifest-path core/Cargo.toml

# Release workflow
release version:
    @echo "🚀 Preparing release {{version}}..."
    @if [ -z "{{version}}" ]; then echo "❌ Error: Version required. Usage: just release v0.1.0"; exit 1; fi
    @if [ -n "$(git status --porcelain)" ]; then echo "❌ Error: Git is dirty. Commit changes first."; exit 1; fi
    @echo "📦 Building Assets..."
    cd app && pnpm install && pnpm build
    @echo "🏷️  Tagging & Pushing..."
    git tag -a {{version}} -m "Release {{version}}"
    git push origin {{version}}
    @echo "✅ Done! GitHub Actions will now build and publish the release."

# --- INSTALLATION (SYSTEM WIDE / HEADLESS) ---

install:
    @echo "⚠️  WARNING: This will overwrite ~/.valter configuration and binary."
    @echo "   Press Ctrl+C to cancel or Enter to proceed."
    @read _
    
    @echo "🏗️  Building Dashboard..."
    cd app && pnpm install && pnpm build
    
    @echo "📦 Building Core Binary..."
    cargo build --release --manifest-path core/Cargo.toml
    
    @echo "📂 Creating System Directories (~/.valter)..."
    mkdir -p ~/.valter
    mkdir -p ~/.local/bin
    
    @echo "🚚 Installing Binary..."
    cp target/release/valter ~/.local/bin/valter
    
    @# MACOS SIGNING FIX
    @if [ "$(uname)" = "Darwin" ]; then \
        echo "🍎 macOS detected: Signing binary..."; \
        codesign -s - --force ~/.local/bin/valter; \
    fi
    
    @echo "📝 Installing Default Config..."
    cp valter.config.example ~/.valter/valter.config
    
    @echo "✅ Installation Complete!"
    @echo "   To start: 'valter start'"
    @echo "   Then open: http://localhost:9090 (or configured port)"

# Update the binary without touching config
update:
    @echo "🔄 Updating Valter Binary..."
    @echo "🏗️  Rebuilding Dashboard..."
    cd app && pnpm install && pnpm build
    @echo "📦 Rebuilding Core..."
    cargo build --release --manifest-path core/Cargo.toml
    
    @echo "🚚 Copying binary..."
    cp target/release/valter ~/.local/bin/valter
    
    @if [ "$(uname)" = "Darwin" ]; then \
        echo "🍎 macOS detected: Signing binary..."; \
        codesign -s - --force ~/.local/bin/valter; \
    fi
    
    @echo "✅ Updated. Restart daemon with 'valter stop' then 'valter start'."


# =========================================================================
# AUTOMATIZIRANO TESTIRANJE KONFIGURACIJE
# =========================================================================
test-config:
    @./scripts/test-env-config.sh

# Testira frontend aplikaciju (lint & build) i daje jasan status
test-app:
    @echo "🧪 Pokrećem testiranje aplikacije (lint & build)... Detaljan log se sprema u app-test.log"
    @# Pokrećemo skriptu i preusmjeravamo sav izlaz u log datoteku.
    @# Nakon toga, provjeravamo izlazni kod skripte.
    @# Ako je bio 0 (uspjeh), ispisujemo poruku o uspjehu.
    @# Ako nije bio 0 (neuspjeh), ispisujemo poruku o grešci.
    @if ./scripts/test-app.sh > app-test.log 2>&1; then \
        echo "\n✅ \033[1;32mTESTIRANJE USPJEŠNO ZAVRŠENO!\033[0m"; \
    else \
        echo "\n❌ \033[1;31mTESTIRANJE NIJE USPJELO.\033[0m Provjerite 'app-test.log' za detalje."; \
        exit 1; \
    fi.sh > app-test.log 2>&1

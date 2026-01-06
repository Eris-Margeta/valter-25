# VALTER ERP - JUSTFILE
# Commands for Development, Installation, Maintenance, and Release.

default:
    @just --list

# --- DEVELOPMENT ---

# Run the full stack in Dev Mode with AUTO-CLEANUP
dev:
    @echo "🧹 Pre-flight: Killing zombies on ports 8000 & 5173..."
    @-lsof -ti:8000 | xargs kill -9 2>/dev/null || true
    @-lsof -ti:5173 | xargs kill -9 2>/dev/null || true
    
    @# Ensure dist folder exists so rust-embed doesn't crash compilation
    @mkdir -p dashboard/dist && touch dashboard/dist/index.html
    
    @echo "🚀 Starting VALTER DEV Environment..."
    @echo "   Backend API: http://localhost:8000/graphql"
    @echo "   Frontend UI: http://localhost:5173"
    
    @# Trap SIGINT (Ctrl+C) to run cleanup
    @# CHANGE: We run cargo via --manifest-path to keep CWD at repo root
    @trap 'echo "\n🛑 Shutting down..."; lsof -ti:8000 | xargs kill -9 2>/dev/null; lsof -ti:5173 | xargs kill -9 2>/dev/null; exit 0' SIGINT; \
    (cargo run --manifest-path core/Cargo.toml -- run) & \
    (cd dashboard && pnpm install && pnpm dev) & \
    wait

clean:
    @echo "🧹 Cleaning up..."
    rm -rf target core/target
    rm -rf dashboard/dist dashboard/.vite dashboard/node_modules
    rm -f valter.db valter.log valter.pid valter.db-shm valter.db-wal

# --- RELEASE ---

release version:
    @echo "🚀 Preparing release {{version}}..."
    @if [ -z "{{version}}" ]; then echo "❌ Error: Version required. Usage: just release v0.1.0"; exit 1; fi
    @if [ -n "$(git status --porcelain)" ]; then echo "❌ Error: Git is dirty. Commit changes first."; exit 1; fi
    @echo "📦 Building Frontend for Release..."
    cd dashboard && pnpm install && pnpm build
    @echo "🏷️  Tagging & Pushing..."
    git tag -a {{version}} -m "Release {{version}}"
    git push origin {{version}}
    @echo "✅ Done! GitHub Actions will now build and publish the release."

# --- INSTALLATION (SYSTEM WIDE) ---

install:
    @echo "⚠️  WARNING: This will overwrite ~/.valter configuration and binary."
    @echo "   Press Ctrl+C to cancel or Enter to proceed."
    @read _
    
    @echo "🏗️  Building Dashboard (React)..."
    cd dashboard && pnpm install && pnpm build
    
    @echo "📦 Building Core Binary (Embedding Dashboard)..."
    cargo build --release
    
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

# --- MAINTENANCE ---

update:
    @echo "🔄 Updating Valter Binary..."
    @echo "🏗️  Rebuilding Dashboard..."
    cd dashboard && pnpm install && pnpm build
    @echo "📦 Rebuilding Core..."
    cargo build --release
    cp target/release/valter ~/.local/bin/valter
    
    @if [ "$(uname)" = "Darwin" ]; then \
        echo "🍎 macOS detected: Signing binary..."; \
        codesign -s - --force ~/.local/bin/valter; \
    fi
    
    @echo "✅ Updated. Restart daemon with 'valter stop' then 'valter start'."

check-migrations:
    @echo "🔍 Checking migrations..."
    @echo "Note: Currently Valter applies migrations automatically on startup."


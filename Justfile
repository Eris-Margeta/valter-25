# VALTER ERP - JUSTFILE

default:
    @just --list

# --- DEVELOPMENT ---

dev:
    @echo "🧹 Pre-flight: Killing zombies..."
    @-lsof -ti:8000 | xargs kill -9 2>/dev/null || true
    @-lsof -ti:5173 | xargs kill -9 2>/dev/null || true
    
    @# Ensure dist folder exists so rust-embed doesn't crash compilation
    @mkdir -p dashboard/dist && touch dashboard/dist/index.html
    
    @echo "🚀 Starting VALTER DEV Environment..."
    @echo "   Backend: http://localhost:8000"
    @echo "   Frontend: http://localhost:5173 (Hot Reload)"
    @trap 'echo "\n🛑 Shutting down..."; lsof -ti:8000 | xargs kill -9 2>/dev/null; lsof -ti:5173 | xargs kill -9 2>/dev/null; exit 0' SIGINT; \
    (cd core && cargo run -- run) & \
    (cd dashboard && pnpm dev) & \
    wait

clean:
    @echo "🧹 Cleaning up..."
    rm -rf target core/target
    rm -rf dashboard/dist dashboard/.vite dashboard/node_modules
    rm -f valter.db valter.log valter.pid

# --- RELEASE ---

release version:
    @echo "🚀 Preparing release {{version}}..."
    @if [ -z "{{version}}" ]; then echo "❌ Error: Version required."; exit 1; fi
    @echo "📦 Building Frontend for Release..."
    cd dashboard && pnpm install && pnpm build
    
    @echo "🏷️  Tagging & Pushing..."
    git tag -a {{version}} -m "Release {{version}}"
    git push origin {{version}}

# --- INSTALLATION (SYSTEM WIDE) ---

install:
    @echo "⚠️  WARNING: This will overwrite ~/.valter configuration and binary."
    @read _
    
    @echo "🏗️  Building Dashboard (React)..."
    cd dashboard && pnpm install && pnpm build
    
    @echo "📦 Building Core Binary (Embedding Dashboard)..."
    # Cargo build will now pick up the files in dashboard/dist
    cargo build --release
    
    @echo "📂 Setting up ~/.valter..."
    mkdir -p ~/.valter ~/.local/bin
    
    @echo "🚚 Installing Binary..."
    cp target/release/valter ~/.local/bin/valter
    
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
    @echo "✅ Updated. Restart daemon with 'valter stop' then 'valter start'."

